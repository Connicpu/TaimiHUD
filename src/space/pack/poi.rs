use {
    super::{loader::PackLoaderContext, taco_safe_name, Pack},
    crate::marker::atomic::MapSpace,
    anyhow::Context,
    glamour::Vector3,
};

pub struct Poi {
    pub id: String,
    pub map_id: i32,
    pub position: Vector3<MapSpace>,
}

impl Poi {
    pub fn from_xml(
        pack: &mut Pack,
        ctx: &impl PackLoaderContext,
        attrs: Vec<xml::attribute::OwnedAttribute>,
    ) -> anyhow::Result<Poi> {
        let mut id = String::new();
        let mut map_id = None;
        let mut pos_x = None;
        let mut pos_y = None;
        let mut pos_z = None;

        for attr in attrs {
            if attr.name.local_name.eq_ignore_ascii_case("type") {
                id = taco_safe_name(&attr.value, true);
            } else if attr.name.local_name.eq_ignore_ascii_case("MapID") {
                map_id = Some(attr.value.parse().context("Parse POI MapID")?);
            } else if attr.name.local_name.eq_ignore_ascii_case("xpos") {
                pos_x = Some(attr.value.parse().context("Parse POI xpos")?);
            } else if attr.name.local_name.eq_ignore_ascii_case("ypos") {
                pos_y = Some(attr.value.parse().context("Parse POI ypos")?);
            } else if attr.name.local_name.eq_ignore_ascii_case("zpos") {
                pos_z = Some(attr.value.parse().context("Parse POI zpos")?);
            } else {
                log::warn!("Unknown POI attribute '{}'", attr.name.local_name);
            }
        }

        let Some(map_id) = map_id else {
            anyhow::bail!("POI must have MapID");
        };

        let (Some(pos_x), Some(pos_y), Some(pos_z)) = (pos_x, pos_y, pos_z) else {
            anyhow::bail!("POI must have xpos, ypos, and zpos");
        };
        let position = glamour::vec3!(pos_x, pos_y, pos_z);

        Ok(Poi {
            id,
            map_id,
            position,
        })
    }
}
