use {
    arc_atomic::AtomicArc,
    glam::Vec3,
    std::sync::{Arc, OnceLock},
};

pub static PERSPECTIVEINPUTDATA: OnceLock<Arc<AtomicArc<PerspectiveInputData>>> = OnceLock::new();

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PerspectiveInputData {
    pub front: Vec3,
    pub pos: Vec3,
    pub fov: f32,
    pub playpos: Vec3,
}

impl PerspectiveInputData {
    pub fn create() {
        let aarc = Arc::new(AtomicArc::new(Arc::new(Self::default())));
        let _ = PERSPECTIVEINPUTDATA.set(aarc);
    }

    pub fn read() -> Option<Arc<Self>> {
        Some(PERSPECTIVEINPUTDATA.get()?.load())
    }

    pub fn swap_camera(front: Vec3, pos: Vec3, playpos: Vec3) {
        if let Some(data) = PERSPECTIVEINPUTDATA.get() {
            let pdata = data.load();
            data.store(Arc::new(PerspectiveInputData {
                playpos,
                fov: pdata.fov,
                front,
                pos,
            }))
        }
    }

    pub fn swap_fov(fov: f32) {
        if let Some(data) = PERSPECTIVEINPUTDATA.get() {
            let pdata = data.load();
            data.store(Arc::new(PerspectiveInputData {
                fov,
                playpos: pdata.playpos,
                front: pdata.front,
                pos: pdata.pos,
            }))
        }
    }
}
