use autd3::driver::geometry::{Point3, Vector3};

use crate::utils::aabb::Aabb;

use super::Range;

impl Range for Vec<Vector3> {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        self.iter().map(|v| (v.x, v.y, v.z))
    }

    fn aabb(&self) -> Aabb {
        self.iter()
            .fold(Aabb::empty(), |aabb, v| aabb.grow(Point3::from(*v)))
    }
}

impl Range for Vec<Point3> {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        self.iter().map(|v| (v.x, v.y, v.z))
    }

    fn aabb(&self) -> Aabb {
        self.iter().fold(Aabb::empty(), |aabb, v| aabb.grow(*v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec3_points() {
        assert_eq!(
            (vec![0., 3.], vec![1., 4.], vec![2., 5.]),
            vec![Vector3::new(0., 1., 2.), Vector3::new(3., 4., 5.)]
                .points()
                .collect()
        );
    }

    #[test]
    fn vec3_aabb() {
        let aabb = vec![Vector3::new(0., 1., 2.), Vector3::new(3., 4., 5.)].aabb();
        assert_eq!(Point3::new(0., 1., 2.), aabb.min);
        assert_eq!(Point3::new(3., 4., 5.), aabb.max);
    }

    #[test]
    fn point3_points() {
        assert_eq!(
            (vec![0., 3.], vec![1., 4.], vec![2., 5.]),
            vec![Point3::new(0., 1., 2.), Point3::new(3., 4., 5.)]
                .points()
                .collect()
        );
    }

    #[test]
    fn point3_aabb() {
        let aabb = vec![Point3::new(0., 1., 2.), Point3::new(3., 4., 5.)].aabb();
        assert_eq!(Point3::new(0., 1., 2.), aabb.min);
        assert_eq!(Point3::new(3., 4., 5.), aabb.max);
    }
}
