use crate::utils::aabb::Aabb;

use autd3::driver::geometry::Vector3;

use super::Range;

/// A range of 1D space along the x axis.
#[derive(Clone, Debug)]
pub struct RangeX {
    /// The range of the x axis.
    pub x: std::ops::RangeInclusive<f32>,
    /// The coordinate of the y axis.
    pub y: f32,
    /// The coordinate of the z axis.
    pub z: f32,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeX {
    fn nx(&self) -> usize {
        ((self.x.end() - self.x.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeX {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let nx = self.nx();
        let x_start = *self.x.start();
        let y = self.y;
        let z = self.z;
        let resolution = self.resolution;
        (0..nx).map(move |ix| {
            let px = x_start + resolution * ix as f32;
            (px, y, z)
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(*self.x.start(), self.y, self.z).into();
        let max = Vector3::new(*self.x.end(), self.y, self.z).into();
        Aabb { min, max }
    }
}

/// A range of 1D space along the y axis.
#[derive(Clone, Debug)]
pub struct RangeY {
    /// The coordinate of the x axis.
    pub x: f32,
    /// The range of the y axis.
    pub y: std::ops::RangeInclusive<f32>,
    /// The coordinate of the z axis.
    pub z: f32,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeY {
    fn ny(&self) -> usize {
        ((self.y.end() - self.y.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeY {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let ny = self.ny();
        let x = self.x;
        let y_start = *self.y.start();
        let z = self.z;
        let resolution = self.resolution;

        (0..ny).map(move |iy| {
            let py = y_start + resolution * iy as f32;
            (x, py, z)
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(self.x, *self.y.start(), self.z).into();
        let max = Vector3::new(self.x, *self.y.end(), self.z).into();
        Aabb { min, max }
    }
}

/// A range of 1D space along the z axis.
#[derive(Clone, Debug)]
pub struct RangeZ {
    /// The coordinate of the x axis.
    pub x: f32,
    /// The coordinate of the y axis.
    pub y: f32,
    /// The range of the z axis.
    pub z: std::ops::RangeInclusive<f32>,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeZ {
    fn nz(&self) -> usize {
        ((self.z.end() - self.z.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeZ {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let nz = self.nz();
        let x = self.x;
        let y = self.y;
        let z_start = *self.z.start();
        let resolution = self.resolution;

        (0..nz).map(move |iz| {
            let pz = z_start + resolution * iz as f32;
            (x, y, pz)
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(self.x, self.y, *self.z.start()).into();
        let max = Vector3::new(self.x, self.y, *self.z.end()).into();
        Aabb { min, max }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(1, 0., 0., 0.1)]
    #[case(11, 0., 1., 0.1)]
    #[case(11, 10., 20., 1.)]
    fn test_nx(#[case] n: usize, #[case] start: f32, #[case] end: f32, #[case] resolution: f32) {
        let range = RangeX {
            x: start..=end,
            y: 0.0,
            z: 0.0,
            resolution,
        };
        assert_eq!(n, range.nx());
    }

    #[rstest::rstest]
    #[test]
    #[case(1, 0., 0., 0.1)]
    #[case(11, 0., 1., 0.1)]
    #[case(11, 10., 20., 1.)]
    fn test_ny(#[case] n: usize, #[case] start: f32, #[case] end: f32, #[case] resolution: f32) {
        let range = RangeY {
            x: 0.0,
            y: start..=end,
            z: 0.0,
            resolution,
        };
        assert_eq!(n, range.ny());
    }

    #[rstest::rstest]
    #[test]
    #[case(1, 0., 0., 0.1)]
    #[case(11, 0., 1., 0.1)]
    #[case(11, 10., 20., 1.)]
    fn test_nz(#[case] n: usize, #[case] start: f32, #[case] end: f32, #[case] resolution: f32) {
        let range = RangeZ {
            x: 0.0,
            y: 0.0,
            z: start..=end,
            resolution,
        };
        assert_eq!(n, range.nz());
    }

    #[rstest::rstest]
    #[test]
    #[case((vec![0., 1.], vec![0., 0.], vec![0., 0.]), RangeX { x:0.0..=1., y:0.0, z:0.0, resolution:1. })]
    #[case((vec![0., 0.], vec![0., 1.], vec![0., 0.]), RangeY { x:0.0, y:0.0..=1., z:0.0, resolution:1. })]
    #[case((vec![0., 0.], vec![0., 0.], vec![0., 1.]), RangeZ { x:0.0, y:0.0, z:0.0..=1., resolution:1. })]
    #[case((vec![0., 1., 2., 3.], vec![0., 0., 0., 0.], vec![0., 0., 0., 0.]), RangeX { x:0.0..=3., y:0.0, z:0.0, resolution:1. })]
    #[case((vec![0., 0., 0., 0.], vec![0., 1., 2., 3.], vec![0., 0., 0., 0.]), RangeY { x:0.0, y:0.0..=3., z:0.0, resolution:1. })]
    #[case((vec![0., 0., 0., 0.], vec![0., 0., 0., 0.], vec![0., 1., 2., 3.]), RangeZ { x:0.0, y:0.0, z:0.0..=3., resolution:1. })]
    fn test_points(#[case] expected: (Vec<f32>, Vec<f32>, Vec<f32>), #[case] range: impl Range) {
        assert_eq!(expected, range.points().collect());
    }

    #[test]
    fn test_aabb_x() {
        let range = RangeX {
            x: 0.0..=10.,
            y: 5.0,
            z: 3.0,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 0.0);
        assert_eq!(aabb.min.y, 5.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 10.0);
        assert_eq!(aabb.max.y, 5.0);
        assert_eq!(aabb.max.z, 3.0);
    }

    #[test]
    fn test_aabb_y() {
        let range = RangeY {
            x: 5.0,
            y: 0.0..=10.,
            z: 3.0,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 5.0);
        assert_eq!(aabb.min.y, 0.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 5.0);
        assert_eq!(aabb.max.y, 10.0);
        assert_eq!(aabb.max.z, 3.0);
    }

    #[test]
    fn test_aabb_z() {
        let range = RangeZ {
            x: 5.0,
            y: 3.0,
            z: 0.0..=10.,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 5.0);
        assert_eq!(aabb.min.y, 3.0);
        assert_eq!(aabb.min.z, 0.0);
        assert_eq!(aabb.max.x, 5.0);
        assert_eq!(aabb.max.y, 3.0);
        assert_eq!(aabb.max.z, 10.0);
    }

    #[test]
    fn test_range_y_iterator() {
        let range = RangeY {
            x: 1.0,
            y: 0.0..=2.0,
            z: 3.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((1.0, 0.0, 3.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 3.0)));
        assert_eq!(iter.next(), Some((1.0, 2.0, 3.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_range_z_iterator() {
        let range = RangeZ {
            x: 1.0,
            y: 2.0,
            z: 0.0..=2.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((1.0, 2.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 2.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 2.0, 2.0)));
        assert_eq!(iter.next(), None);
    }
}
