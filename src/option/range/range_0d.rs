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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_points() {
        use unzip3::Unzip3;
        assert_eq!(
            (vec![0.], vec![1.], vec![2.]),
            Vector3::new(0., 1., 2.).points().unzip3()
        );
    }

    #[test]
    fn test_aabb() {
        assert_eq!(Point3::new(0., 1., 2.), Vector3::new(0., 1., 2.).aabb().min);
        assert_eq!(Point3::new(0., 1., 2.), Vector3::new(0., 1., 2.).aabb().max);
    }
}