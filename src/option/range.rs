use autd3::driver::geometry::Vector3;
use unzip3::Unzip3;

use bvh::aabb::Aabb;

#[derive(Clone, Debug)]
pub struct Range {
    pub x: std::ops::RangeInclusive<f32>,
    pub y: std::ops::RangeInclusive<f32>,
    pub z: std::ops::RangeInclusive<f32>,
    pub resolution: f32,
}

impl Range {
    fn n(range: &std::ops::RangeInclusive<f32>, resolution: f32) -> usize {
        ((range.end() - range.start()) / resolution).floor() as usize + 1
    }

    pub fn nx(&self) -> usize {
        Self::n(&self.x, self.resolution)
    }

    pub fn ny(&self) -> usize {
        Self::n(&self.y, self.resolution)
    }

    pub fn nz(&self) -> usize {
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

    pub fn points(&self) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        match (self.nx(), self.ny(), self.nz()) {
            (_, 1, 1) => {
                let x: Vec<_> = self.points_x().collect();
                let len = x.len();
                (x, vec![*self.y.start(); len], vec![*self.z.start(); len])
            }
            (1, _, 1) => {
                let y: Vec<_> = self.points_y().collect();
                let len = y.len();
                (vec![*self.x.start(); len], y, vec![*self.z.start(); len])
            }
            (1, 1, _) => {
                let z: Vec<_> = self.points_z().collect();
                let len = z.len();
                (vec![*self.x.start(); len], vec![*self.y.start(); len], z)
            }
            (_, _, 1) => {
                let (y, x): (Vec<_>, Vec<_>) =
                    itertools::iproduct!(self.points_y(), self.points_x()).unzip();
                let len = x.len();
                (x, y, vec![*self.z.start(); len])
            }
            (_, 1, _) => {
                let (z, x): (Vec<_>, Vec<_>) =
                    itertools::iproduct!(self.points_z(), self.points_x()).unzip();
                let len = x.len();
                (x, vec![*self.y.start(); len], z)
            }
            (1, _, _) => {
                let (z, y): (Vec<_>, Vec<_>) =
                    itertools::iproduct!(self.points_z(), self.points_y()).unzip();
                let len = y.len();
                (vec![*self.x.start(); len], y, z)
            }
            (_, _, _) => {
                let (z, y, x) =
                    itertools::iproduct!(self.points_z(), self.points_y(), self.points_x())
                        .unzip3();
                (x, y, z)
            }
        }
    }

    pub(crate) fn aabb(&self) -> Aabb<f32, 3> {
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
        let range = Range {
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
        let range = Range {
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
        let range = Range {
            x: 0.0..=1.,
            y: 0.0..=1.,
            z: start..=end,
            resolution,
        };
        assert_eq!(n, range.nz());
    }

    #[rstest::rstest]
    #[test]
    #[case((vec![0., 1., 2.], vec![0.; 3], vec![0.; 3]), 0.0..=2., 0.0..=0., 0.0..=0., 1.)]
    #[case((vec![0.; 3], vec![0., 1., 2.], vec![0.; 3]), 0.0..=0., 0.0..=2., 0.0..=0., 1.)]
    #[case((vec![0.; 3], vec![0.; 3], vec![0., 1., 2.]), 0.0..=0., 0.0..=0., 0.0..=2., 1.)]
    #[case((vec![0., 1., 0., 1.], vec![0., 0., 1., 1.], vec![0., 0., 0., 0.]), 0.0..=1., 0.0..=1., 0.0..=0., 1.)]
    #[case((vec![0., 1., 0., 1.], vec![0., 0., 0., 0.], vec![0., 0., 1., 1.]), 0.0..=1., 0.0..=0., 0.0..=1., 1.)]
    #[case((vec![0., 0., 0., 0.], vec![0., 1., 0., 1.], vec![0., 0., 1., 1.]), 0.0..=0., 0.0..=1., 0.0..=1., 1.)]
    #[case((vec![0., 1., 0., 1., 0., 1., 0., 1.], vec![0., 0., 1., 1., 0., 0., 1., 1.], vec![0., 0., 0., 0., 1., 1., 1., 1.]), 0.0..=1., 0.0..=1., 0.0..=1., 1.)]
    fn test_points(
        #[case] expected: (Vec<f32>, Vec<f32>, Vec<f32>),
        #[case] x: std::ops::RangeInclusive<f32>,
        #[case] y: std::ops::RangeInclusive<f32>,
        #[case] z: std::ops::RangeInclusive<f32>,
        #[case] resolution: f32,
    ) {
        assert_eq!(
            expected,
            Range {
                x,
                y,
                z,
                resolution,
            }
            .points()
        );
    }
}
