use crate::utils::aabb::Aabb;

use autd3::driver::geometry::Vector3;

use super::Range;

/// A range of 2D space iterating in the order of x-y.
#[derive(Clone, Debug)]
pub struct RangeXY {
    /// The range of the x axis.
    pub x: std::ops::RangeInclusive<f32>,
    /// The range of the y axis.
    pub y: std::ops::RangeInclusive<f32>,
    /// The coordinate of the z axis.
    pub z: f32,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeXY {
    fn nx(&self) -> usize {
        ((self.x.end() - self.x.start()) / self.resolution).floor() as usize + 1
    }

    fn ny(&self) -> usize {
        ((self.y.end() - self.y.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeXY {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let nx = self.nx();
        let ny = self.ny();
        let x_start = *self.x.start();
        let y_start = *self.y.start();
        let z = self.z;
        let resolution = self.resolution;

        (0..ny).flat_map(move |iy| {
            let py = y_start + resolution * iy as f32;
            (0..nx).map(move |ix| {
                let px = x_start + resolution * ix as f32;
                (px, py, z)
            })
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(*self.x.start(), *self.y.start(), self.z).into();
        let max = Vector3::new(*self.x.end(), *self.y.end(), self.z).into();
        Aabb { min, max }
    }
}

/// A range of 2D space iterating in the order of x-z.
#[derive(Clone, Debug)]
pub struct RangeXZ {
    /// The range of the x axis.
    pub x: std::ops::RangeInclusive<f32>,
    /// The coordinate of the y axis.
    pub y: f32,
    /// The range of the z axis.
    pub z: std::ops::RangeInclusive<f32>,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeXZ {
    fn nx(&self) -> usize {
        ((self.x.end() - self.x.start()) / self.resolution).floor() as usize + 1
    }

    fn nz(&self) -> usize {
        ((self.z.end() - self.z.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeXZ {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let nx = self.nx();
        let nz = self.nz();
        let x_start = *self.x.start();
        let y = self.y;
        let z_start = *self.z.start();
        let resolution = self.resolution;

        (0..nz).flat_map(move |iz| {
            let pz = z_start + resolution * iz as f32;
            (0..nx).map(move |ix| {
                let px = x_start + resolution * ix as f32;
                (px, y, pz)
            })
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(*self.x.start(), self.y, *self.z.start()).into();
        let max = Vector3::new(*self.x.end(), self.y, *self.z.end()).into();
        Aabb { min, max }
    }
}

/// A range of 2D space iterating in the order of y-x.
#[derive(Clone, Debug)]
pub struct RangeYX {
    /// The range of the x axis.
    pub x: std::ops::RangeInclusive<f32>,
    /// The range of the y axis.
    pub y: std::ops::RangeInclusive<f32>,
    /// The coordinate of the z axis.
    pub z: f32,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeYX {
    fn nx(&self) -> usize {
        ((self.x.end() - self.x.start()) / self.resolution).floor() as usize + 1
    }

    fn ny(&self) -> usize {
        ((self.y.end() - self.y.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeYX {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let nx = self.nx();
        let ny = self.ny();
        let x_start = *self.x.start();
        let y_start = *self.y.start();
        let z = self.z;
        let resolution = self.resolution;

        (0..nx).flat_map(move |ix| {
            let px = x_start + resolution * ix as f32;
            (0..ny).map(move |iy| {
                let py = y_start + resolution * iy as f32;
                (px, py, z)
            })
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(*self.x.start(), *self.y.start(), self.z).into();
        let max = Vector3::new(*self.x.end(), *self.y.end(), self.z).into();
        Aabb { min, max }
    }
}

/// A range of 2D space iterating in the order of y-z.
#[derive(Clone, Debug)]
pub struct RangeYZ {
    /// The coordinate of the x axis.
    pub x: f32,
    /// The range of the y axis.
    pub y: std::ops::RangeInclusive<f32>,
    /// The range of the z axis.
    pub z: std::ops::RangeInclusive<f32>,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeYZ {
    fn ny(&self) -> usize {
        ((self.y.end() - self.y.start()) / self.resolution).floor() as usize + 1
    }

    fn nz(&self) -> usize {
        ((self.z.end() - self.z.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeYZ {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let ny = self.ny();
        let nz = self.nz();
        let x = self.x;
        let y_start = *self.y.start();
        let z_start = *self.z.start();
        let resolution = self.resolution;

        (0..nz).flat_map(move |iz| {
            let pz = z_start + resolution * iz as f32;
            (0..ny).map(move |iy| {
                let py = y_start + resolution * iy as f32;
                (x, py, pz)
            })
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(self.x, *self.y.start(), *self.z.start()).into();
        let max = Vector3::new(self.x, *self.y.end(), *self.z.end()).into();
        Aabb { min, max }
    }
}

/// A range of 2D space iterating in the order of z-x.
#[derive(Clone, Debug)]
pub struct RangeZX {
    /// The range of the x axis.
    pub x: std::ops::RangeInclusive<f32>,
    /// The coordinate of the y axis.
    pub y: f32,
    /// The range of the z axis.
    pub z: std::ops::RangeInclusive<f32>,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeZX {
    fn nx(&self) -> usize {
        ((self.x.end() - self.x.start()) / self.resolution).floor() as usize + 1
    }

    fn nz(&self) -> usize {
        ((self.z.end() - self.z.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeZX {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let nx = self.nx();
        let nz = self.nz();
        let x_start = *self.x.start();
        let y = self.y;
        let z_start = *self.z.start();
        let resolution = self.resolution;

        (0..nx).flat_map(move |ix| {
            let px = x_start + resolution * ix as f32;
            (0..nz).map(move |iz| {
                let pz = z_start + resolution * iz as f32;
                (px, y, pz)
            })
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(*self.x.start(), self.y, *self.z.start()).into();
        let max = Vector3::new(*self.x.end(), self.y, *self.z.end()).into();
        Aabb { min, max }
    }
}

/// A range of 2D space iterating in the order of z-y.
#[derive(Clone, Debug)]
pub struct RangeZY {
    /// The coordinate of the x axis.
    pub x: f32,
    /// The range of the y axis.
    pub y: std::ops::RangeInclusive<f32>,
    /// The range of the z axis.
    pub z: std::ops::RangeInclusive<f32>,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeZY {
    fn ny(&self) -> usize {
        ((self.y.end() - self.y.start()) / self.resolution).floor() as usize + 1
    }

    fn nz(&self) -> usize {
        ((self.z.end() - self.z.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeZY {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let ny = self.ny();
        let nz = self.nz();
        let x = self.x;
        let y_start = *self.y.start();
        let z_start = *self.z.start();
        let resolution = self.resolution;

        (0..ny).flat_map(move |iy| {
            let py = y_start + resolution * iy as f32;
            (0..nz).map(move |iz| {
                let pz = z_start + resolution * iz as f32;
                (x, py, pz)
            })
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(self.x, *self.y.start(), *self.z.start()).into();
        let max = Vector3::new(self.x, *self.y.end(), *self.z.end()).into();
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
        let range = RangeXY {
            x: start..=end,
            y: 0.0..=0.,
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
        let range = RangeXY {
            x: 0.0..=0.,
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
        let range = RangeZX {
            x: 0.0..=0.,
            y: 0.0,
            z: start..=end,
            resolution,
        };
        assert_eq!(n, range.nz());
    }

    #[rstest::rstest]
    #[test]
    #[case(1, 0., 0., 0.1)]
    #[case(11, 0., 1., 0.1)]
    #[case(11, 10., 20., 1.)]
    fn test_nx_xz(#[case] n: usize, #[case] start: f32, #[case] end: f32, #[case] resolution: f32) {
        let range = RangeXZ {
            x: start..=end,
            y: 0.0,
            z: 0.0..=0.,
            resolution,
        };
        assert_eq!(n, range.nx());
    }

    #[rstest::rstest]
    #[test]
    #[case(1, 0., 0., 0.1)]
    #[case(11, 0., 1., 0.1)]
    #[case(11, 10., 20., 1.)]
    fn test_nz_xz(#[case] n: usize, #[case] start: f32, #[case] end: f32, #[case] resolution: f32) {
        let range = RangeXZ {
            x: 0.0..=0.,
            y: 0.0,
            z: start..=end,
            resolution,
        };
        assert_eq!(n, range.nz());
    }

    #[rstest::rstest]
    #[test]
    #[case(1, 0., 0., 0.1)]
    #[case(11, 0., 1., 0.1)]
    #[case(11, 10., 20., 1.)]
    fn test_nx_yx(#[case] n: usize, #[case] start: f32, #[case] end: f32, #[case] resolution: f32) {
        let range = RangeYX {
            x: start..=end,
            y: 0.0..=0.,
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
    fn test_ny_yx(#[case] n: usize, #[case] start: f32, #[case] end: f32, #[case] resolution: f32) {
        let range = RangeYX {
            x: 0.0..=0.,
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
    fn test_ny_yz(#[case] n: usize, #[case] start: f32, #[case] end: f32, #[case] resolution: f32) {
        let range = RangeYZ {
            x: 0.0,
            y: start..=end,
            z: 0.0..=0.,
            resolution,
        };
        assert_eq!(n, range.ny());
    }

    #[rstest::rstest]
    #[test]
    #[case(1, 0., 0., 0.1)]
    #[case(11, 0., 1., 0.1)]
    #[case(11, 10., 20., 1.)]
    fn test_nz_yz(#[case] n: usize, #[case] start: f32, #[case] end: f32, #[case] resolution: f32) {
        let range = RangeYZ {
            x: 0.0,
            y: 0.0..=0.,
            z: start..=end,
            resolution,
        };
        assert_eq!(n, range.nz());
    }

    #[rstest::rstest]
    #[test]
    #[case(1, 0., 0., 0.1)]
    #[case(11, 0., 1., 0.1)]
    #[case(11, 10., 20., 1.)]
    fn test_nx_zx(#[case] n: usize, #[case] start: f32, #[case] end: f32, #[case] resolution: f32) {
        let range = RangeZX {
            x: start..=end,
            y: 0.0,
            z: 0.0..=0.,
            resolution,
        };
        assert_eq!(n, range.nx());
    }

    #[rstest::rstest]
    #[test]
    #[case(1, 0., 0., 0.1)]
    #[case(11, 0., 1., 0.1)]
    #[case(11, 10., 20., 1.)]
    fn test_ny_zy(#[case] n: usize, #[case] start: f32, #[case] end: f32, #[case] resolution: f32) {
        let range = RangeZY {
            x: 0.0,
            y: start..=end,
            z: 0.0..=0.,
            resolution,
        };
        assert_eq!(n, range.ny());
    }

    #[rstest::rstest]
    #[test]
    #[case(1, 0., 0., 0.1)]
    #[case(11, 0., 1., 0.1)]
    #[case(11, 10., 20., 1.)]
    fn test_nz_zy(#[case] n: usize, #[case] start: f32, #[case] end: f32, #[case] resolution: f32) {
        let range = RangeZY {
            x: 0.0,
            y: 0.0..=0.,
            z: start..=end,
            resolution,
        };
        assert_eq!(n, range.nz());
    }

    #[rstest::rstest]
    #[test]
    #[case((vec![0., 1., 2.], vec![0.; 3], vec![0.; 3]), RangeXY { x:0.0..=2., y:0.0..=0., z:0.0, resolution:1. })]
    #[case((vec![0.; 3], vec![0., 1., 2.], vec![0.; 3]), RangeXY { x:0.0..=0., y:0.0..=2., z:0.0, resolution:1. })]
    #[case((vec![0., 1., 0., 1.], vec![0., 0., 1., 1.], vec![0., 0., 0., 0.]), RangeXY { x:0.0..=1., y:0.0..=1., z:0.0, resolution:1. })]
    #[case((vec![0., 1., 0., 1.], vec![0., 0., 0., 0.], vec![0., 0., 1., 1.]), RangeXZ { x:0.0..=1., y:0.0, z:0.0..=1., resolution:1. })]
    #[case((vec![0., 0., 1., 1.], vec![0., 1., 0., 1.], vec![0., 0., 0., 0.]), RangeYX { x:0.0..=1., y:0.0..=1., z:0.0, resolution:1. })]
    #[case((vec![0., 0., 0., 0.], vec![0., 1., 0., 1.], vec![0., 0., 1., 1.]), RangeYZ { x:0.0, y:0.0..=1., z:0.0..=1., resolution:1. })]
    #[case((vec![0., 0., 1., 1.], vec![0., 0., 0., 0.], vec![0., 1., 0., 1.]), RangeZX { x:0.0..=1., y:0.0, z:0.0..=1., resolution:1. })]
    #[case((vec![0., 0., 0., 0.], vec![0., 0., 1., 1.], vec![0., 1., 0., 1.]), RangeZY { x:0.0, y:0.0..=1., z:0.0..=1., resolution:1. })]
    #[case((vec![0., 1., 2., 0., 1., 2., 0., 1., 2.], vec![0., 0., 0., 1., 1., 1., 2., 2., 2.], vec![0.; 9]), RangeXY { x:0.0..=2., y:0.0..=2., z:0.0, resolution:1. })]
    #[case((vec![0., 1., 2., 0., 1., 2., 0., 1., 2.], vec![0.; 9], vec![0., 0., 0., 1., 1., 1., 2., 2., 2.]), RangeXZ { x:0.0..=2., y:0.0, z:0.0..=2., resolution:1. })]
    #[case((vec![0., 0., 0., 1., 1., 1., 2., 2., 2.], vec![0., 1., 2., 0., 1., 2., 0., 1., 2.], vec![0.; 9]), RangeYX { x:0.0..=2., y:0.0..=2., z:0.0, resolution:1. })]
    #[case((vec![0.; 9], vec![0., 1., 2., 0., 1., 2., 0., 1., 2.], vec![0., 0., 0., 1., 1., 1., 2., 2., 2.]), RangeYZ { x:0.0, y:0.0..=2., z:0.0..=2., resolution:1. })]
    #[case((vec![0., 0., 0., 1., 1., 1., 2., 2., 2.], vec![0.; 9], vec![0., 1., 2., 0., 1., 2., 0., 1., 2.]), RangeZX { x:0.0..=2., y:0.0, z:0.0..=2., resolution:1. })]
    #[case((vec![0.; 9], vec![0., 0., 0., 1., 1., 1., 2., 2., 2.], vec![0., 1., 2., 0., 1., 2., 0., 1., 2.]), RangeZY { x:0.0, y:0.0..=2., z:0.0..=2., resolution:1. })]
    fn test_points(#[case] expected: (Vec<f32>, Vec<f32>, Vec<f32>), #[case] range: impl Range) {
        assert_eq!(expected, range.points().collect());
    }

    #[test]
    fn test_aabb_xy() {
        let range = RangeXY {
            x: 0.0..=10.,
            y: 5.0..=15.,
            z: 3.0,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 0.0);
        assert_eq!(aabb.min.y, 5.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 10.0);
        assert_eq!(aabb.max.y, 15.0);
        assert_eq!(aabb.max.z, 3.0);
    }

    #[test]
    fn test_aabb_xz() {
        let range = RangeXZ {
            x: 0.0..=10.,
            y: 5.0,
            z: 3.0..=8.,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 0.0);
        assert_eq!(aabb.min.y, 5.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 10.0);
        assert_eq!(aabb.max.y, 5.0);
        assert_eq!(aabb.max.z, 8.0);
    }

    #[test]
    fn test_aabb_yx() {
        let range = RangeYX {
            x: 0.0..=10.,
            y: 5.0..=15.,
            z: 3.0,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 0.0);
        assert_eq!(aabb.min.y, 5.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 10.0);
        assert_eq!(aabb.max.y, 15.0);
        assert_eq!(aabb.max.z, 3.0);
    }

    #[test]
    fn test_aabb_yz() {
        let range = RangeYZ {
            x: 5.0,
            y: 0.0..=10.,
            z: 3.0..=8.,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 5.0);
        assert_eq!(aabb.min.y, 0.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 5.0);
        assert_eq!(aabb.max.y, 10.0);
        assert_eq!(aabb.max.z, 8.0);
    }

    #[test]
    fn test_aabb_zx() {
        let range = RangeZX {
            x: 0.0..=10.,
            y: 5.0,
            z: 3.0..=8.,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 0.0);
        assert_eq!(aabb.min.y, 5.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 10.0);
        assert_eq!(aabb.max.y, 5.0);
        assert_eq!(aabb.max.z, 8.0);
    }

    #[test]
    fn test_aabb_zy() {
        let range = RangeZY {
            x: 5.0,
            y: 0.0..=10.,
            z: 3.0..=8.,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 5.0);
        assert_eq!(aabb.min.y, 0.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 5.0);
        assert_eq!(aabb.max.y, 10.0);
        assert_eq!(aabb.max.z, 8.0);
    }

    #[test]
    fn test_range_xz_iterator() {
        let range = RangeXZ {
            x: 0.0..=1.0,
            y: 5.0,
            z: 0.0..=1.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((0.0, 5.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 5.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 5.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 5.0, 1.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_range_yx_iterator() {
        let range = RangeYX {
            x: 0.0..=1.0,
            y: 0.0..=1.0,
            z: 5.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((0.0, 0.0, 5.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 5.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 5.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 5.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_range_yz_iterator() {
        let range = RangeYZ {
            x: 5.0,
            y: 0.0..=1.0,
            z: 0.0..=1.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((5.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((5.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((5.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((5.0, 1.0, 1.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_range_zx_iterator() {
        let range = RangeZX {
            x: 0.0..=1.0,
            y: 5.0,
            z: 0.0..=1.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((0.0, 5.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 5.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 5.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 5.0, 1.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_range_zy_iterator() {
        let range = RangeZY {
            x: 5.0,
            y: 0.0..=1.0,
            z: 0.0..=1.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((5.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((5.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((5.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((5.0, 1.0, 1.0)));
        assert_eq!(iter.next(), None);
    }
}
