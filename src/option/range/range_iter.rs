use autd3::{driver::geometry::Point3, prelude::Vector3};

use super::Range;

impl Range for Vec<Vector3> {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        self.iter().map(|v| (v.x, v.y, v.z))
    }

    fn aabb(&self) -> bvh::aabb::Aabb<f32, 3> {
        self.iter().fold(bvh::aabb::Aabb::empty(), |aabb, v| {
            aabb.grow(&Point3::from(*v))
        })
    }
}
