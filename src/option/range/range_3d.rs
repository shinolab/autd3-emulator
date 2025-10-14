use crate::utils::aabb::Aabb;

use autd3::driver::geometry::Vector3;

use super::Range;

/// A range of 3D space iterating in the order of x-y-z.
#[derive(Clone, Debug)]
pub struct RangeXYZ {
    /// The range of the x axis.
    pub x: std::ops::RangeInclusive<f32>,
    /// The range of the y axis.
    pub y: std::ops::RangeInclusive<f32>,
    /// The range of the z axis.
    pub z: std::ops::RangeInclusive<f32>,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeXYZ {
    fn nx(&self) -> usize {
        ((self.x.end() - self.x.start()) / self.resolution).floor() as usize + 1
    }

    fn ny(&self) -> usize {
        ((self.y.end() - self.y.start()) / self.resolution).floor() as usize + 1
    }

    fn nz(&self) -> usize {
        ((self.z.end() - self.z.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeXYZ {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let nx = self.nx();
        let ny = self.ny();
        let nz = self.nz();
        let x_start = *self.x.start();
        let y_start = *self.y.start();
        let z_start = *self.z.start();
        let resolution = self.resolution;

        (0..nz).flat_map(move |iz| {
            let pz = z_start + resolution * iz as f32;
            (0..ny).flat_map(move |iy| {
                let py = y_start + resolution * iy as f32;
                (0..nx).map(move |ix| {
                    let px = x_start + resolution * ix as f32;
                    (px, py, pz)
                })
            })
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(*self.x.start(), *self.y.start(), *self.z.start()).into();
        let max = Vector3::new(*self.x.end(), *self.y.end(), *self.z.end()).into();
        Aabb { min, max }
    }
}

/// A range of 3D space iterating in the order of x-z-y.
#[derive(Clone, Debug)]
pub struct RangeXZY {
    /// The range of the x axis.
    pub x: std::ops::RangeInclusive<f32>,
    /// The range of the y axis.
    pub y: std::ops::RangeInclusive<f32>,
    /// The range of the z axis.
    pub z: std::ops::RangeInclusive<f32>,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeXZY {
    fn nx(&self) -> usize {
        ((self.x.end() - self.x.start()) / self.resolution).floor() as usize + 1
    }

    fn ny(&self) -> usize {
        ((self.y.end() - self.y.start()) / self.resolution).floor() as usize + 1
    }

    fn nz(&self) -> usize {
        ((self.z.end() - self.z.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeXZY {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let nx = self.nx();
        let ny = self.ny();
        let nz = self.nz();
        let x_start = *self.x.start();
        let y_start = *self.y.start();
        let z_start = *self.z.start();
        let resolution = self.resolution;

        (0..ny).flat_map(move |iy| {
            let py = y_start + resolution * iy as f32;
            (0..nz).flat_map(move |iz| {
                let pz = z_start + resolution * iz as f32;
                (0..nx).map(move |ix| {
                    let px = x_start + resolution * ix as f32;
                    (px, py, pz)
                })
            })
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(*self.x.start(), *self.y.start(), *self.z.start()).into();
        let max = Vector3::new(*self.x.end(), *self.y.end(), *self.z.end()).into();
        Aabb { min, max }
    }
}

/// A range of 3D space iterating in the order of y-x-z.
#[derive(Clone, Debug)]
pub struct RangeYXZ {
    /// The range of the x axis.
    pub x: std::ops::RangeInclusive<f32>,
    /// The range of the y axis.
    pub y: std::ops::RangeInclusive<f32>,
    /// The range of the z axis.
    pub z: std::ops::RangeInclusive<f32>,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeYXZ {
    fn nx(&self) -> usize {
        ((self.x.end() - self.x.start()) / self.resolution).floor() as usize + 1
    }

    fn ny(&self) -> usize {
        ((self.y.end() - self.y.start()) / self.resolution).floor() as usize + 1
    }

    fn nz(&self) -> usize {
        ((self.z.end() - self.z.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeYXZ {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let nx = self.nx();
        let ny = self.ny();
        let nz = self.nz();
        let x_start = *self.x.start();
        let y_start = *self.y.start();
        let z_start = *self.z.start();
        let resolution = self.resolution;

        (0..nz).flat_map(move |iz| {
            let pz = z_start + resolution * iz as f32;
            (0..nx).flat_map(move |ix| {
                let px = x_start + resolution * ix as f32;
                (0..ny).map(move |iy| {
                    let py = y_start + resolution * iy as f32;
                    (px, py, pz)
                })
            })
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(*self.x.start(), *self.y.start(), *self.z.start()).into();
        let max = Vector3::new(*self.x.end(), *self.y.end(), *self.z.end()).into();
        Aabb { min, max }
    }
}

/// A range of 3D space iterating in the order of y-z-x.
#[derive(Clone, Debug)]
pub struct RangeYZX {
    /// The range of the x axis.
    pub x: std::ops::RangeInclusive<f32>,
    /// The range of the y axis.
    pub y: std::ops::RangeInclusive<f32>,
    /// The range of the z axis.
    pub z: std::ops::RangeInclusive<f32>,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeYZX {
    fn nx(&self) -> usize {
        ((self.x.end() - self.x.start()) / self.resolution).floor() as usize + 1
    }

    fn ny(&self) -> usize {
        ((self.y.end() - self.y.start()) / self.resolution).floor() as usize + 1
    }

    fn nz(&self) -> usize {
        ((self.z.end() - self.z.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeYZX {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let nx = self.nx();
        let ny = self.ny();
        let nz = self.nz();
        let x_start = *self.x.start();
        let y_start = *self.y.start();
        let z_start = *self.z.start();
        let resolution = self.resolution;

        (0..nx).flat_map(move |ix| {
            let px = x_start + resolution * ix as f32;
            (0..nz).flat_map(move |iz| {
                let pz = z_start + resolution * iz as f32;
                (0..ny).map(move |iy| {
                    let py = y_start + resolution * iy as f32;
                    (px, py, pz)
                })
            })
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(*self.x.start(), *self.y.start(), *self.z.start()).into();
        let max = Vector3::new(*self.x.end(), *self.y.end(), *self.z.end()).into();
        Aabb { min, max }
    }
}

/// A range of 3D space iterating in the order of z-x-y.
#[derive(Clone, Debug)]
pub struct RangeZXY {
    /// The range of the x axis.
    pub x: std::ops::RangeInclusive<f32>,
    /// The range of the y axis.
    pub y: std::ops::RangeInclusive<f32>,
    /// The range of the z axis.
    pub z: std::ops::RangeInclusive<f32>,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeZXY {
    fn nx(&self) -> usize {
        ((self.x.end() - self.x.start()) / self.resolution).floor() as usize + 1
    }

    fn ny(&self) -> usize {
        ((self.y.end() - self.y.start()) / self.resolution).floor() as usize + 1
    }

    fn nz(&self) -> usize {
        ((self.z.end() - self.z.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeZXY {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let nx = self.nx();
        let ny = self.ny();
        let nz = self.nz();
        let x_start = *self.x.start();
        let y_start = *self.y.start();
        let z_start = *self.z.start();
        let resolution = self.resolution;

        (0..ny).flat_map(move |iy| {
            let py = y_start + resolution * iy as f32;
            (0..nx).flat_map(move |ix| {
                let px = x_start + resolution * ix as f32;
                (0..nz).map(move |iz| {
                    let pz = z_start + resolution * iz as f32;
                    (px, py, pz)
                })
            })
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(*self.x.start(), *self.y.start(), *self.z.start()).into();
        let max = Vector3::new(*self.x.end(), *self.y.end(), *self.z.end()).into();
        Aabb { min, max }
    }
}

/// A range of 3D space iterating in the order of z-y-x.
#[derive(Clone, Debug)]
pub struct RangeZYX {
    /// The range of the x axis.
    pub x: std::ops::RangeInclusive<f32>,
    /// The range of the y axis.
    pub y: std::ops::RangeInclusive<f32>,
    /// The range of the z axis.
    pub z: std::ops::RangeInclusive<f32>,
    /// The resolution of the range.
    pub resolution: f32,
}

impl RangeZYX {
    fn nx(&self) -> usize {
        ((self.x.end() - self.x.start()) / self.resolution).floor() as usize + 1
    }

    fn ny(&self) -> usize {
        ((self.y.end() - self.y.start()) / self.resolution).floor() as usize + 1
    }

    fn nz(&self) -> usize {
        ((self.z.end() - self.z.start()) / self.resolution).floor() as usize + 1
    }
}

impl Range for RangeZYX {
    fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
        let nx = self.nx();
        let ny = self.ny();
        let nz = self.nz();
        let x_start = *self.x.start();
        let y_start = *self.y.start();
        let z_start = *self.z.start();
        let resolution = self.resolution;

        (0..nx).flat_map(move |ix| {
            let px = x_start + resolution * ix as f32;
            (0..ny).flat_map(move |iy| {
                let py = y_start + resolution * iy as f32;
                (0..nz).map(move |iz| {
                    let pz = z_start + resolution * iz as f32;
                    (px, py, pz)
                })
            })
        })
    }

    fn aabb(&self) -> Aabb {
        let min = Vector3::new(*self.x.start(), *self.y.start(), *self.z.start()).into();
        let max = Vector3::new(*self.x.end(), *self.y.end(), *self.z.end()).into();
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
        let range = RangeXYZ {
            x: start..=end,
            y: 0.0..=1.,
            z: 0.0..=1.,
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
        let range = RangeXYZ {
            x: 0.0..=1.,
            y: start..=end,
            z: 0.0..=1.,
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
        let range = RangeXYZ {
            x: 0.0..=1.,
            y: 0.0..=1.,
            z: start..=end,
            resolution,
        };
        assert_eq!(n, range.nz());
    }

    #[rstest::rstest]
    #[test]
    #[case((vec![0., 1., 2.], vec![0.; 3], vec![0.; 3]), RangeXYZ { x:0.0..=2., y:0.0..=0., z:0.0..=0., resolution:1. })]
    #[case((vec![0.; 3], vec![0., 1., 2.], vec![0.; 3]), RangeXYZ { x:0.0..=0., y:0.0..=2., z:0.0..=0., resolution:1. })]
    #[case((vec![0.; 3], vec![0.; 3], vec![0., 1., 2.]), RangeXYZ { x:0.0..=0., y:0.0..=0., z:0.0..=2., resolution:1. })]
    #[case((vec![0., 1., 0., 1.], vec![0., 0., 1., 1.], vec![0., 0., 0., 0.]), RangeXYZ { x:0.0..=1., y:0.0..=1., z:0.0..=0., resolution:1. })]
    #[case((vec![0., 1., 0., 1.], vec![0., 0., 0., 0.], vec![0., 0., 1., 1.]), RangeXYZ { x:0.0..=1., y:0.0..=0., z:0.0..=1., resolution:1. })]
    #[case((vec![0., 0., 0., 0.], vec![0., 1., 0., 1.], vec![0., 0., 1., 1.]), RangeXYZ { x:0.0..=0., y:0.0..=1., z:0.0..=1., resolution:1. })]
    #[case((vec![0., 1., 0., 1., 0., 1., 0., 1.], vec![0., 0., 1., 1., 0., 0., 1., 1.], vec![0., 0., 0., 0., 1., 1., 1., 1.]), RangeXYZ { x:0.0..=1., y:0.0..=1., z:0.0..=1., resolution:1. })]
    #[case((vec![0., 1., 0., 1., 0., 1., 0., 1.], vec![0., 0., 0., 0., 1., 1., 1., 1.], vec![0., 0., 1., 1., 0., 0., 1., 1.]), RangeXZY { x:0.0..=1., y:0.0..=1., z:0.0..=1., resolution:1. })]
    #[case((vec![0., 0., 1., 1., 0., 0., 1., 1.], vec![0., 1., 0., 1., 0., 1., 0., 1.], vec![0., 0., 0., 0., 1., 1., 1., 1.]), RangeYXZ { x:0.0..=1., y:0.0..=1., z:0.0..=1., resolution:1. })]
    #[case((vec![0., 0., 0., 0., 1., 1., 1., 1.], vec![0., 1., 0., 1., 0., 1., 0., 1.], vec![0., 0., 1., 1., 0., 0., 1., 1.]), RangeYZX { x:0.0..=1., y:0.0..=1., z:0.0..=1., resolution:1. })]
    #[case((vec![0., 0., 1., 1., 0., 0., 1., 1.], vec![0., 0., 0., 0., 1., 1., 1., 1.], vec![0., 1., 0., 1., 0., 1., 0., 1.]), RangeZXY { x:0.0..=1., y:0.0..=1., z:0.0..=1., resolution:1. })]
    #[case((vec![0., 0., 0., 0., 1., 1., 1., 1.], vec![0., 0., 1., 1., 0., 0., 1., 1.], vec![0., 1., 0., 1., 0., 1., 0., 1.]), RangeZYX { x:0.0..=1., y:0.0..=1., z:0.0..=1., resolution:1. })]
    fn test_points(#[case] expected: (Vec<f32>, Vec<f32>, Vec<f32>), #[case] range: impl Range) {
        assert_eq!(expected, range.points().collect());
    }

    #[test]
    fn test_aabb() {
        let range = RangeXYZ {
            x: 0.0..=10.,
            y: 5.0..=15.,
            z: 3.0..=8.,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 0.0);
        assert_eq!(aabb.min.y, 5.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 10.0);
        assert_eq!(aabb.max.y, 15.0);
        assert_eq!(aabb.max.z, 8.0);
    }

    #[test]
    fn test_aabb_xzy() {
        let range = RangeXZY {
            x: 0.0..=10.,
            y: 5.0..=15.,
            z: 3.0..=8.,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 0.0);
        assert_eq!(aabb.min.y, 5.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 10.0);
        assert_eq!(aabb.max.y, 15.0);
        assert_eq!(aabb.max.z, 8.0);
    }

    #[test]
    fn test_aabb_yxz() {
        let range = RangeYXZ {
            x: 0.0..=10.,
            y: 5.0..=15.,
            z: 3.0..=8.,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 0.0);
        assert_eq!(aabb.min.y, 5.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 10.0);
        assert_eq!(aabb.max.y, 15.0);
        assert_eq!(aabb.max.z, 8.0);
    }

    #[test]
    fn test_aabb_yzx() {
        let range = RangeYZX {
            x: 0.0..=10.,
            y: 5.0..=15.,
            z: 3.0..=8.,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 0.0);
        assert_eq!(aabb.min.y, 5.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 10.0);
        assert_eq!(aabb.max.y, 15.0);
        assert_eq!(aabb.max.z, 8.0);
    }

    #[test]
    fn test_aabb_zxy() {
        let range = RangeZXY {
            x: 0.0..=10.,
            y: 5.0..=15.,
            z: 3.0..=8.,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 0.0);
        assert_eq!(aabb.min.y, 5.0);
        assert_eq!(aabb.min.z, 3.0);
        assert_eq!(aabb.max.x, 10.0);
        assert_eq!(aabb.max.y, 15.0);
        assert_eq!(aabb.max.z, 8.0);
    }

    #[test]
    fn test_aabb_zyx() {
        let range = RangeZYX {
            x: 0.0..=10.,
            y: 3.0..=8.,
            z: 5.0..=7.,
            resolution: 1.,
        };
        let aabb = range.aabb();
        assert_eq!(aabb.min.x, 0.0);
        assert_eq!(aabb.min.y, 3.0);
        assert_eq!(aabb.min.z, 5.0);
        assert_eq!(aabb.max.x, 10.0);
        assert_eq!(aabb.max.y, 8.0);
        assert_eq!(aabb.max.z, 7.0);
    }

    #[test]
    fn test_range_xyz_iterator() {
        let range = RangeXYZ {
            x: 0.0..=1.0,
            y: 0.0..=1.0,
            z: 0.0..=1.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((0.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 1.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_range_xzy_iterator() {
        let range = RangeXZY {
            x: 0.0..=1.0,
            y: 0.0..=1.0,
            z: 0.0..=1.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((0.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 1.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_range_yxz_iterator() {
        let range = RangeYXZ {
            x: 0.0..=1.0,
            y: 0.0..=1.0,
            z: 0.0..=1.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((0.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 1.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_range_yzx_iterator() {
        let range = RangeYZX {
            x: 0.0..=1.0,
            y: 0.0..=1.0,
            z: 0.0..=1.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((0.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 1.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_range_zxy_iterator() {
        let range = RangeZXY {
            x: 0.0..=1.0,
            y: 0.0..=1.0,
            z: 0.0..=1.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((0.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 1.0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_range_zyx_iterator() {
        let range = RangeZYX {
            x: 0.0..=1.0,
            y: 0.0..=1.0,
            z: 0.0..=1.0,
            resolution: 1.0,
        };
        let mut iter = range.points();
        assert_eq!(iter.next(), Some((0.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((0.0, 1.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 0.0, 1.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 0.0)));
        assert_eq!(iter.next(), Some((1.0, 1.0, 1.0)));
        assert_eq!(iter.next(), None);
    }
}
