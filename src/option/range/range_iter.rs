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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_points() {
        assert_eq!(
            (vec![0., 3.], vec![1., 4.], vec![2., 5.]),
            vec![Vector3::new(0., 1., 2.), Vector3::new(3., 4., 5.)]
                .points()
                .collect()
        );
    }

    #[test]
    fn test_aabb() {
        let aabb = vec![Vector3::new(0., 1., 2.), Vector3::new(3., 4., 5.)].aabb();
        assert_eq!(Point3::new(0., 1., 2.), aabb.min);
        assert_eq!(Point3::new(3., 4., 5.), aabb.max);
    }
}
