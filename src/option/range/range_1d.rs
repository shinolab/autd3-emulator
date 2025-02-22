use autd3::driver::geometry::Vector3;

use bvh::aabb::Aabb;

use paste::paste;

use super::Range;

macro_rules! make_range_1d {
    ($first:ident, $second:ident, $third:ident) => {
        paste! {
            #[doc = concat!("A range of 1D space along the ", stringify!($first), " axis.")]
            #[derive(Clone, Debug)]
            pub struct [<Range $first:upper>] {
                #[doc = concat!("The range of the ", stringify!($first), " axis.")]
                pub $first: std::ops::RangeInclusive<f32>,
                #[doc = concat!("The coordinate of the ", stringify!($second), " axis.")]
                pub $second: f32,
                #[doc = concat!("The coordinate of the ", stringify!($third), " axis.")]
                pub $third: f32,
                /// The resolution of the range.
                pub resolution: f32,
            }

            impl [<Range $first:upper>] {
                fn n(range: &std::ops::RangeInclusive<f32>, resolution: f32) -> usize {
                    ((range.end() - range.start()) / resolution).floor() as usize + 1
                }

                fn [<n $first>](&self) -> usize {
                    Self::n(&self.$first, self.resolution)
                }

                fn _points(n: usize, start: f32, resolution: f32) -> impl Iterator<Item = f32> + Clone {
                    (0..n).map(move |i| start + resolution * i as f32)
                }

                fn [<points_ $first>](&self) -> impl Iterator<Item = f32> + Clone {
                    Self::_points(self.[<n $first>](), *self.$first.start(), self.resolution)
                }

                fn [<points_ $second>](&self) -> impl Iterator<Item = f32> + Clone {
                    std::iter::once(self.$second)
                }

                fn [<points_ $third>](&self) -> impl Iterator<Item = f32> + Clone {
                    std::iter::once(self.$third)
                }

                fn [<min_ $first>](&self) -> f32 {
                    *self.$first.start()
                }

                fn [<max_ $first>](&self) -> f32 {
                    *self.$first.end()
                }

                fn [<min_ $second>](&self) -> f32 {
                    self.$second
                }

                fn [<max_ $second>](&self) -> f32 {
                    self.$second
                }

                fn [<min_ $third>](&self) -> f32 {
                    self.$third
                }

                fn [<max_ $third>](&self) -> f32 {
                    self.$third
                }
            }

            impl Range for [<Range $first:upper>] {
                fn points(&self) -> impl Iterator<Item = (f32, f32, f32)> {
                    itertools::iproduct!(self.[<points_ $third>](), self.[<points_ $second>](), self.[<points_ $first>]())
                        .map(|([<p $third>], [<p $second>], [<p $first>])| ([<p x>], [<p y>], [<p z>]))
                }

                fn aabb(&self) -> Aabb<f32, 3> {
                    let min = Vector3::new(self.min_x(), self.min_y(), self.min_z()).into();
                    let max = Vector3::new(self.max_x(), self.max_y(), self.max_z()).into();
                    Aabb { min, max }
                }
            }
        }
    };
}

make_range_1d!(x, y, z);
make_range_1d!(y, z, x);
make_range_1d!(z, x, y);

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
    fn test_points(#[case] expected: (Vec<f32>, Vec<f32>, Vec<f32>), #[case] range: impl Range) {
        assert_eq!(expected, range.points().collect());
    }
}
