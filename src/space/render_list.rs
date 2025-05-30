use {
    super::dx11::PerspectiveInputData,
    crate::marker::atomic::MapSpace,
    bvh::{
        aabb::{Bounded, IntersectsAabb},
        bounding_hierarchy::{BHShape, BoundingHierarchy},
        bvh::Bvh,
    },
    glamour::vec4,
    std::collections::BinaryHeap,
};

pub struct RenderEntity {
    pub bounds: glamour::Box3<MapSpace>,
    pub position: glamour::Vector3<MapSpace>,
    pub translucent: bool,
    // todo: stuff to actually draw it.
}

pub struct RenderList {
    entities: Vec<RenderEntity>,
    spatial_map: SpatialMap,
    draw_order_heap: BinaryHeap<HeapEntity>,
}

impl RenderList {
    pub fn build(entities: Vec<RenderEntity>) -> RenderList {
        let spatial_map = SpatialMap::build(&entities);
        RenderList {
            entities,
            spatial_map,
            draw_order_heap: BinaryHeap::with_capacity(4096),
        }
    }

    /// Gets visible entities in the correct draw order.
    pub fn get_entities_for_drawing<'rs>(
        &'rs mut self,
        cam_origin: glamour::Vector3<MapSpace>,
        cam_dir: glamour::Vector3<MapSpace>,
        frustum: &'rs MapFrustum,
    ) -> impl Iterator<Item = &'rs RenderEntity> + 'rs {
        self.draw_order_heap.clear();
        RenderOrderBuilder {
            entities: &self.entities,
            bvh_iter: self.spatial_map.select_visible_entities(frustum),
            draw_order_heap: &mut self.draw_order_heap,
            cam_origin,
            cam_dir,
        }
    }
}

struct RenderOrderBuilder<'rs, BvhIter> {
    entities: &'rs [RenderEntity],
    bvh_iter: BvhIter,
    draw_order_heap: &'rs mut BinaryHeap<HeapEntity>,
    cam_origin: glamour::Vector3<MapSpace>,
    cam_dir: glamour::Vector3<MapSpace>,
}

impl<'rs, BvhIter> Iterator for RenderOrderBuilder<'rs, BvhIter>
where
    BvhIter: Iterator<Item = usize>,
{
    type Item = &'rs RenderEntity;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.bvh_iter.next() {
            let entity = &self.entities[next];
            if !entity.translucent {
                return Some(entity);
            } else {
                let cam_dist = (entity.position - self.cam_origin).dot(self.cam_dir);
                let cam_dist = f32::to_bits(cam_dist) as i32;
                let cam_dist = cam_dist ^ ((cam_dist >> 30) as u32 >> 1) as i32;
                self.draw_order_heap.push(HeapEntity {
                    cam_dist,
                    idx: next,
                });
            }
        }

        self.draw_order_heap.pop().map(|he| &self.entities[he.idx])
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct HeapEntity {
    cam_dist: i32,
    idx: usize,
}

struct RenderEntityShape {
    bounds: bvh::aabb::Aabb<f32, 3>,
    entity_idx: usize,
    bh_idx: usize,
}

impl RenderEntityShape {
    fn new((entity_idx, entity): (usize, &RenderEntity)) -> Self {
        RenderEntityShape {
            bounds: bvh::aabb::Aabb {
                min: [
                    entity.bounds.min.x,
                    entity.bounds.min.y,
                    entity.bounds.min.z,
                ]
                .into(),
                max: [
                    entity.bounds.max.x,
                    entity.bounds.max.y,
                    entity.bounds.max.z,
                ]
                .into(),
            },
            entity_idx,
            bh_idx: 0,
        }
    }
}

impl Bounded<f32, 3> for RenderEntityShape {
    fn aabb(&self) -> bvh::aabb::Aabb<f32, 3> {
        self.bounds
    }
}

impl BHShape<f32, 3> for RenderEntityShape {
    fn set_bh_node_index(&mut self, bh_idx: usize) {
        self.bh_idx = bh_idx;
    }

    fn bh_node_index(&self) -> usize {
        self.bh_idx
    }
}

struct SpatialMap {
    shapes: Vec<RenderEntityShape>,
    bvh: Bvh<f32, 3>,
}

impl SpatialMap {
    fn build(entities: &[RenderEntity]) -> SpatialMap {
        let mut shapes: Vec<_> = entities
            .iter()
            .enumerate()
            .map(RenderEntityShape::new)
            .collect();
        let bvh = Bvh::build_par(&mut shapes);
        SpatialMap { shapes, bvh }
    }

    pub fn select_visible_entities<'a>(
        &'a self,
        frustum: &'a MapFrustum,
    ) -> impl Iterator<Item = usize> + 'a {
        self.bvh
            .traverse_iterator(frustum, &self.shapes)
            .map(|shape| shape.entity_idx)
    }
}

#[derive(Copy, Clone)]
pub struct MapFrustum(pub [glamour::Vector4<MapSpace>; 6]);

impl MapFrustum {
    pub fn from_camera_data(
        data: &PerspectiveInputData,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> MapFrustum {
        // TODO: Compute frustum planes.
        MapFrustum([Default::default(); 6])
    }
}

impl IntersectsAabb<f32, 3> for MapFrustum {
    fn intersects_aabb(&self, aabb: &bvh::aabb::Aabb<f32, 3>) -> bool {
        for plane in self.0 {
            if plane.dot(vec4!(aabb.min.x, aabb.min.y, aabb.min.z, 1.0)) < 0.0
                && plane.dot(vec4!(aabb.max.x, aabb.min.y, aabb.min.z, 1.0)) < 0.0
                && plane.dot(vec4!(aabb.min.x, aabb.max.y, aabb.min.z, 1.0)) < 0.0
                && plane.dot(vec4!(aabb.max.x, aabb.max.y, aabb.min.z, 1.0)) < 0.0
                && plane.dot(vec4!(aabb.min.x, aabb.min.y, aabb.max.z, 1.0)) < 0.0
                && plane.dot(vec4!(aabb.max.x, aabb.min.y, aabb.max.z, 1.0)) < 0.0
                && plane.dot(vec4!(aabb.min.x, aabb.max.y, aabb.max.z, 1.0)) < 0.0
                && plane.dot(vec4!(aabb.max.x, aabb.max.y, aabb.max.z, 1.0)) < 0.0
            {
                return false;
            }
        }
        true
    }
}
