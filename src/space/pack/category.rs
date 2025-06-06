use {
    super::{attributes::MarkerAttributes, taco_safe_name, Pack, PartialItem},
    indexmap::IndexMap,
    std::sync::Arc,
};

pub struct Category {
    pub id: String,
    pub full_id: String,
    pub display_name: String,
    pub is_separator: bool,
    pub is_hidden: bool,
    pub default_toggle: bool,
    // Map of local to global name.
    pub sub_categories: Arc<IndexMap<String, String>>,
    /// Attributes for markers attached to this category.
    pub marker_attributes: Arc<MarkerAttributes>,
}

impl Category {
    pub fn from_xml(
        pack: &mut Pack,
        parse_stack: &[PartialItem],
        attrs: Vec<xml::attribute::OwnedAttribute>,
    ) -> anyhow::Result<Category> {
        let mut marker_attributes = MarkerAttributes::default();

        let mut id = String::new();
        let mut display_name = None;
        let mut is_separator = false;
        let mut is_hidden = false;
        let mut default_toggle = true;

        for attr in attrs {
            let attr_name = attr.name.local_name.trim_start_matches("bh-");
            if attr_name.eq_ignore_ascii_case("name") {
                id = taco_safe_name(&attr.value, false);
            } else if attr_name.eq_ignore_ascii_case("displayname") {
                display_name = Some(attr.value);
            } else if attr_name.eq_ignore_ascii_case("isseparator") {
                if let Ok(val) = attr.value.parse() {
                    is_separator = val;
                }
            } else if attr_name.eq_ignore_ascii_case("ishidden") {
                if let Ok(val) = attr.value.parse() {
                    is_hidden = val;
                }
            } else if attr_name.eq_ignore_ascii_case("defaulttoggle") {
                if let Ok(val) = attr.value.parse() {
                    default_toggle = val;
                }
            } else if !marker_attributes.try_add(pack, &attr) {
                log::warn!(
                    "Unknown MarkerCategory attribute '{}'",
                    attr.name.local_name
                );
            }
        }

        let full_id = if let Some(PartialItem::MarkerCategory(cat)) = parse_stack.last() {
            format!("{}.{id}", cat.full_id)
        } else {
            id.clone()
        };

        let marker_attributes = Arc::new(marker_attributes);

        Ok(Category {
            display_name: display_name.unwrap_or(id.clone()),
            id,
            full_id,
            is_separator,
            is_hidden,
            default_toggle,
            sub_categories: Default::default(),
            marker_attributes,
        })
    }

    pub fn merge(&mut self, mut new: Category) {
        if self.id != new.id || self.full_id != new.full_id {
            log::error!(
                "Invalid category state. Attempted to merge {} onto {}",
                new.full_id,
                self.full_id
            );
            return;
        }
        // This should not result in a clone because nobody else should own the Arc.
        if Arc::strong_count(&new.marker_attributes) > 1 {
            log::warn!("Multiple owners for category attributes.");
        }
        Arc::make_mut(&mut new.marker_attributes).merge(&self.marker_attributes);
        self.marker_attributes = new.marker_attributes;
        let self_subs = Arc::make_mut(&mut self.sub_categories);
        for (local_id, full_id) in Arc::make_mut(&mut new.sub_categories).drain(..) {
            if !self_subs.contains_key(&local_id) {
                self_subs.insert(local_id, full_id);
            }
        }
    }
}
