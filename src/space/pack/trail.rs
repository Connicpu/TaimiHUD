use {
    super::{loader::PackLoaderContext, taco_safe_name, Pack},
    crate::marker::atomic::MapSpace,
    anyhow::Context,
    glamour::Vector3,
    std::io::BufReader,
};

pub struct Trail {
    pub id: String,
    pub data: TrailData,
}

impl Trail {
    pub fn from_xml(
        pack: &mut Pack,
        ctx: &impl PackLoaderContext,
        attrs: Vec<xml::attribute::OwnedAttribute>,
    ) -> anyhow::Result<Trail> {
        let mut id = String::new();
        let mut trail_path = None;
        for attr in attrs {
            if attr.name.local_name.eq_ignore_ascii_case("trailData") {
                id = taco_safe_name(&attr.value, true);
            } else if attr.name.local_name.eq_ignore_ascii_case("trailData") {
                trail_path = Some(attr.value);
            } else {
                log::warn!("Unknown Trail attribute '{}'", attr.name.local_name);
            }
        }

        if id.is_empty() {
            anyhow::bail!("No 'type' specified for Trail");
        }

        let Some(trail_path) = trail_path else {
            anyhow::bail!("No 'trailData' specified for Trail '{id}'");
        };

        let data = read_trl_file(BufReader::new(ctx.load_asset(&trail_path)?))?;

        Ok(Trail { id, data })
    }
}

pub struct TrailData {
    pub map_id: i32,
    pub sections: Vec<TrailSection>,
}

pub struct TrailSection {
    pub points: Vec<Vector3<MapSpace>>,
}

pub fn read_trl_file(mut reader: impl std::io::Read) -> anyhow::Result<TrailData> {
    let mut buf32 = [0u8; 4];
    reader
        .read_exact(&mut buf32)
        .context("Reading trail version")?;
    if i32::from_le_bytes(buf32) != 0 {
        anyhow::bail!("Trl version '0' is the only known valid format version");
    }

    reader
        .read_exact(&mut buf32)
        .context("Reading trail map_id")?;
    let map_id = i32::from_le_bytes(buf32);

    let mut sections = vec![];
    let mut current_section = vec![];

    let mut read_more = true;
    while read_more {
        let point_data = match read_point(&mut reader) {
            Ok(point_data) => point_data,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                read_more = false;
                EMPTY_POINT
            }
            Err(e) => return Err(e).context("Reading trail sections"),
        };

        if point_data == EMPTY_POINT {
            if !current_section.is_empty() {
                sections.push(TrailSection {
                    points: std::mem::take(&mut current_section),
                });
            }
        } else {
            let x = f32::from_le_bytes(point_data[0]);
            let y = f32::from_le_bytes(point_data[1]);
            let z = f32::from_le_bytes(point_data[2]);
            let point = glamour::vec3!(x, y, z);
            current_section.push(point);
        }
    }

    Ok(TrailData {
        map_id,
        sections: vec![],
    })
}

const EMPTY_POINT: [[u8; 4]; 3] = [[0; 4]; 3];

fn read_point(reader: &mut impl std::io::Read) -> std::io::Result<[[u8; 4]; 3]> {
    let mut point_data = [[0; 4]; 3];
    reader.read_exact(&mut point_data[0])?;
    reader.read_exact(&mut point_data[1])?;
    reader.read_exact(&mut point_data[2])?;
    Ok(point_data)
}
