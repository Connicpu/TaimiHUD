use {
    anyhow::Context,
    indexmap::IndexMap,
    loader::PackLoaderContext,
    std::{collections::HashMap, io::BufReader, path::PathBuf},
    xml::{common::Position, reader::XmlEvent},
};

pub mod loader;
pub mod poi;
pub mod trail;

#[derive(Default)]
pub struct Pack {
    pub texture_list: HashMap<String, usize>,
    pub textures: Vec<()>, // TODO: datatype.
    pub pois: Vec<poi::Poi>,
    pub trails: Vec<trail::Trail>,
    pub categories: IndexMap<String, Category>,
}

impl Pack {
    pub fn load(loader: impl PackLoaderContext) -> anyhow::Result<Pack> {
        let mut pack = Pack::default();

        let pack_defs = loader.all_files_with_ext("xml")?;
        for def in pack_defs {
            parse_pack_def(&mut pack, &loader, &def)?;
        }

        Ok(pack)
    }

    fn get_or_load_texture(&mut self, asset: &str) -> anyhow::Result<usize> {
        if let Some(&id) = self.texture_list.get(asset) {
            return Ok(id);
        }

        // TODO: Load texture somehow.

        let id = self.textures.len();
        self.textures.push(() /*TODO: data*/);
        Ok(id)
    }
}

pub struct Category {
    pub id: String,
    pub full_id: String,
    pub display_name: String,
    pub is_separator: bool,
    pub is_hidden: bool,
    pub default_toggle: bool,
    pub sub_categories: IndexMap<String, Category>,
}

impl Category {
    pub fn from_xml(
        parse_stack: &[PartialItem],
        attrs: Vec<xml::attribute::OwnedAttribute>,
    ) -> anyhow::Result<Category> {
        let mut id = String::new();
        let mut display_name = None;
        let mut is_separator = false;
        let mut is_hidden = false;
        let mut default_toggle = true;

        for attr in attrs {
            if attr.name.local_name.eq_ignore_ascii_case("name") {
                id = taco_safe_name(&attr.value, false);
            } else if attr.name.local_name.eq_ignore_ascii_case("DisplayName") {
                display_name = Some(attr.value);
            } else {
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
        Ok(Category {
            display_name: display_name.unwrap_or(id.clone()),
            id,
            full_id,
            is_separator,
            is_hidden,
            default_toggle,
            sub_categories: Default::default(),
        })
    }
}

fn taco_safe_name(value: &str, is_full: bool) -> String {
    let mut result = String::with_capacity(value.len());
    for c in value.chars() {
        if c.is_ascii_alphanumeric() || (is_full && c == '.') {
            result.push(c);
        } else {
            result.push('_');
        }
    }
    result
}

pub fn parse_pack_def(
    pack: &mut Pack,
    ctx: &impl PackLoaderContext,
    asset: &str,
) -> anyhow::Result<()> {
    let mut parser = xml::EventReader::new(BufReader::new(ctx.load_asset(asset)?));

    match inner_parse_pack_def(pack, ctx, &mut parser) {
        Ok(()) => Ok(()),
        Err(e) => Err(e).context(format!("Parsing pack def at {asset}:{}", parser.position())),
    }
}

fn inner_parse_pack_def(
    pack: &mut Pack,
    ctx: &impl PackLoaderContext,
    parser: &mut xml::EventReader<impl std::io::Read>,
) -> anyhow::Result<()> {
    let mut parse_stack: Vec<PartialItem> = Vec::with_capacity(16);

    loop {
        match parser.next()? {
            XmlEvent::StartElement {
                name, attributes, ..
            } if valid_elem_start(parse_stack.last(), &name) => {
                match name.local_name.to_ascii_lowercase().as_str() {
                    "overlaydata" => {
                        parse_stack.push(PartialItem::OverlayData);
                    }
                    "markercategory" => {
                        let category = Category::from_xml(&parse_stack, attributes)?;
                        parse_stack.push(PartialItem::MarkerCategory(category));
                    }
                    "pois" => {
                        parse_stack.push(PartialItem::PoiGroup);
                    }
                    "poi" => {
                        let poi = poi::Poi::from_xml(pack, ctx, attributes)?;
                        parse_stack.push(PartialItem::Poi(poi));
                    }
                    "trail" => {
                        let trail = trail::Trail::from_xml(pack, ctx, attributes)?;
                        parse_stack.push(PartialItem::Trail(trail));
                    }
                    _ => anyhow::bail!("Unexpected <{name}>"),
                }
            }
            XmlEvent::StartElement { name, .. } => anyhow::bail!("Unexpected <{name}>"),
            XmlEvent::EndElement { name } if valid_elem_end(parse_stack.last(), &name) => {
                match name.local_name.to_ascii_lowercase().as_str() {
                    "overlaydata" | "pois" => {
                        parse_stack.pop();
                    }
                    "markercategory" => {
                        let Some(PartialItem::MarkerCategory(category)) = parse_stack.pop() else {
                            anyhow::bail!("Inconsistent internal state");
                        };

                        match parse_stack.last_mut() {
                            Some(PartialItem::OverlayData) => {
                                pack.categories.insert(category.id.clone(), category);
                            }
                            Some(PartialItem::MarkerCategory(parent)) => {
                                parent.sub_categories.insert(category.id.clone(), category);
                            }
                            _ => anyhow::bail!("Inconsistent internal state"),
                        }
                    }
                    "poi" => {
                        let Some(PartialItem::Poi(poi)) = parse_stack.pop() else {
                            anyhow::bail!("Inconsistent internal state");
                        };

                        pack.pois.push(poi);
                    }
                    "trail" => {
                        let Some(PartialItem::Trail(trail)) = parse_stack.pop() else {
                            anyhow::bail!("Inconsistent internal state");
                        };

                        pack.trails.push(trail);
                    }
                    _ => anyhow::bail!("Unexpected </{name}>"),
                }
            }
            XmlEvent::EndElement { name } => {
                anyhow::bail!("Unexpected </{name}>")
            }
            XmlEvent::StartDocument { .. } => {}
            XmlEvent::EndDocument => {
                if !parse_stack.is_empty() {
                    anyhow::bail!("Unexpected end of document");
                }
                break;
            }
            XmlEvent::ProcessingInstruction { .. } => {}
            XmlEvent::CData(_) => {}
            XmlEvent::Comment(_) => {}
            XmlEvent::Characters(_) => {}
            XmlEvent::Whitespace(_) => {}
        }
    }
    Ok(())
}

enum PartialItem {
    OverlayData,
    MarkerCategory(Category),
    PoiGroup,
    Poi(poi::Poi),
    Trail(trail::Trail),
}

fn valid_elem_start(stack_top: Option<&PartialItem>, name: &xml::name::OwnedName) -> bool {
    match (name.local_name.to_ascii_lowercase().as_str(), stack_top) {
        ("overlaydata", None) => true,
        ("markercategory", Some(PartialItem::OverlayData | PartialItem::MarkerCategory(_))) => true,
        ("pois", Some(PartialItem::OverlayData)) => true,
        ("poi", Some(PartialItem::PoiGroup)) => true,
        ("trail", Some(PartialItem::PoiGroup)) => true,
        _ => false,
    }
}

fn valid_elem_end(stack_top: Option<&PartialItem>, name: &xml::name::OwnedName) -> bool {
    match (name.local_name.to_ascii_lowercase().as_str(), stack_top) {
        ("overlaydata", Some(PartialItem::OverlayData)) => true,
        ("markercategory", Some(PartialItem::MarkerCategory(_))) => true,
        ("pois", Some(PartialItem::PoiGroup)) => true,
        ("poi", Some(PartialItem::Poi(_))) => true,
        ("trail", Some(PartialItem::Trail(_))) => true,
        _ => false,
    }
}
