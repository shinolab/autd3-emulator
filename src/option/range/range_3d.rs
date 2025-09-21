use crate::utils::aabb::Aabb;

use autd3::driver::geometry::Vector3;

use pastey::paste;

use super::Range;

macro_rules! make_range_3d {
    ($first:ident, $second:ident, $third:ident) => {
        paste! {
            #[doc = concat!("A range of 3D space iterating in the order of ", stringify!($first), "-", stringify!($second), "-", stringify!($third), ".")]
            #[derive(Clone, Debug)]
            pub struct [<Range $first:upper $second:upper $third:upper>] {
                #[doc = concat!("The range of the ", stringify!($first), " axis.")]
                pub $first: std::ops::RangeInclusive<f32>,
                #[doc = concat!("The range of the ", stringify!($second), " axis.")]
                pub $second: std::ops::RangeInclusive<f32>,
                #[doc = concat!("The range of the ", stringify!($third), " axis.")]
                pub $third: std::ops::RangeInclusive<f32>,
                /// The resolution of the range.
                pub resolution: f32,
            }

            impl [<Range $first:upper $second:upper $third:upper>] {
                fn n(range: &std::ops::RangeInclusive<f32>, resolution: f32) -> usize {
                    ((range.end() - range.start()) / resolution).floor() as usize + 1
                }

                fn nx(&self) -> usize {
                    Self::n(&self.x, self.resolution)
                }

                fn ny(&self) -> usize {
                    Self::n(&self.y, self.resolution)
                }

                fn nz(&self) -> usize {
                    Self::n(&self.z, self.resolution)
                }

                fn _points(n: usize, start: f32, resolution: f32) -> impl Iterator<Item = f32> + Clone {
                    (0..n).map(move |i| start + resolution * i as f32)
                }

                fn points_x(&self) -> impl Iterator<Item = f32> + Clone {
                    Self::_points(self.nx(), *self.x.start(), self.resolution)
                }

                fn points_y(&self) -> impl Iterator<Item = f32> + Clone {
                    Self::_points(self.ny(), *self.y.start(), self.resolution)
                }

                fn points_z(&self) -> impl Iterator<Item = f32> + Clone {
                    Self::_points(self.nz(), *self.z.start(), self.resolution)
                }
            }

            impl Range for [<Range $first:upper $second:upper $third:upper>] {
                fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
                    itertools::iproduct!(self.[<points_ $third>](), self.[<points_ $second>](), self.[<points_ $first>]())
                        .map(|([<p $third>], [<p $second>], [<p $first>])| ([<p x>], [<p y>], [<p z>]))
                }

                fn aabb(&self) -> Aabb {
                    let min = Vector3::new(*self.x.start(), *self.y.start(), *self.z.start()).into();
                    let max = Vector3::new(*self.x.end(), *self.y.end(), *self.z.end()).into();
                    Aabb { min, max }
                }
            }
        }
    };
}

make_range_3d!(x, y, z);
make_range_3d!(x, z, y);
make_range_3d!(y, x, z);
make_range_3d!(y, z, x);
make_range_3d!(z, x, y);
make_range_3d!(z, y, x);

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
}
