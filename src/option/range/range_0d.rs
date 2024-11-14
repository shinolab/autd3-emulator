use autd3::{driver::geometry::Point3, prelude::Vector3};
use bvh::aabb::Aabb;

use super::Range;

impl Range for Vector3 {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        std::iter::once((self.x, self.y, self.z))
    }

    fn aabb(&self) -> Aabb<f32, 3> {
        Aabb {
            min: Point3::from(*self),
            max: Point3::from(*self),
        }
    }
}
