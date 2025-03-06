use autd3::driver::geometry::{Point3, Vector3};
use bvh::aabb::Aabb;

fn corners(aabb: &Aabb<f32, 3>) -> Vec<Point3> {
    itertools::iproduct!(
        [aabb.min.x, aabb.max.x],
        [aabb.min.y, aabb.max.y],
        [aabb.min.z, aabb.max.z]
    )
    .map(|(x, y, z)| Point3::new(x, y, z))
    .collect()
}

pub(crate) fn aabb_max_dist(a: &Aabb<f32, 3>, b: &Aabb<f32, 3>) -> f32 {
    itertools::iproduct!(corners(a), corners(b))
        .map(|(a, b)| (a - b).norm())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
}

pub(crate) fn aabb_min_dist(a: &Aabb<f32, 3>, b: &Aabb<f32, 3>) -> f32 {
    let min = Vector3::from_iterator(a.min.iter().zip(b.min.iter()).map(|(a, b)| a.max(*b)));
    let max = Vector3::from_iterator(a.max.iter().zip(b.max.iter()).map(|(a, b)| a.min(*b)));
    min.iter()
        .zip(max.iter())
        .filter(|(min, max)| min > max)
        .map(|(min, max)| (min - max).powi(2))
        .sum::<f32>()
        .sqrt()
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use autd3::{
        core::geometry::Geometry,
        prelude::{AUTD3, EulerAngle, UnitQuaternion, rad},
    };
    use rand::Rng;

    use crate::option::*;

    use super::*;

    fn aabb_max_dist_naive(geo: &Geometry, range: &impl Range) -> f32 {
        let points = range
            .points()
            .map(|(x, y, z)| Point3::new(x, y, z))
            .collect::<Vec<_>>();
        itertools::iproduct!(
            geo.iter()
                .flat_map(|dev| dev.iter())
                .map(|tr| tr.position()),
            points
        )
        .map(|(tp, p)| (p - tp).norm())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
    }

    fn aabb_min_dist_naive(geo: &Geometry, range: &impl Range) -> f32 {
        let points = range
            .points()
            .map(|(x, y, z)| Point3::new(x, y, z))
            .collect::<Vec<_>>();
        itertools::iproduct!(
            geo.iter()
                .flat_map(|dev| dev.iter())
                .map(|tr| tr.position()),
            points
        )
        .map(|(tp, p)| (p - tp).norm())
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
    }

    #[rstest::rstest]
    #[test]
    #[case::x_include(RangeX{ x: -10.0..=200.0, y: 0.0, z: 0.0, resolution: 1.0 })]
    #[case::x_separate(RangeX{ x: 200.0..=400.0, y: 0.0, z: 0.0, resolution: 1.0 })]
    #[case::y_include(RangeY{ x: 0.0, y: -10.0..=200.0, z: 0.0, resolution: 1.0 })]
    #[case::y_separate(RangeY{ x: 0.0, y: 200.0..=400.0, z: 0.0, resolution: 1.0 })]
    #[case::z_include(RangeZ{ x: 0.0, y: 0.0, z: -10.0..=100.0, resolution: 1.0 })]
    #[case::z_separate(RangeZ{ x: 0.0, y: 0.0, z: 100.0..=200.0, resolution: 1.0 })]
    #[case::include(RangeXYZ{ x: -10.0..=200.0, y: -10.0..=150.0, z: -10.0..=60.0, resolution: 10.0 })]
    #[case::separate(RangeXYZ{ x: -10.0..=200.0, y: -10.0..=150.0, z: 150.0..=150.0, resolution: 10.0 })]
    fn test_aabb_max_dist(#[case] range: impl Range) {
        let geo = Geometry::new(vec![
            AUTD3 {
                pos: Point3::origin(),
                rot: UnitQuaternion::identity(),
            }
            .into(),
            AUTD3 {
                pos: Point3::new(0., 0., 50.),
                rot: UnitQuaternion::identity(),
            }
            .into(),
        ]);
        approx::assert_relative_eq!(
            aabb_max_dist_naive(&geo, &range),
            aabb_max_dist(&geo.aabb(), &range.aabb())
        );
    }

    #[rstest::rstest]
    #[test]
    #[case::x_include(RangeXYZ{ x: -10.0..=200.0, y: 0.0..=0.0, z: 0.0..=0.0, resolution: 1.0 })]
    #[case::x_separate(RangeXYZ{ x: 200.0..=400.0, y: 0.0..=0.0, z: 0.0..=0.0, resolution: 1.0 })]
    #[case::y_include(RangeXYZ{ x: 0.0..=0.0, y: -10.0..=200.0, z: 0.0..=0.0, resolution: 1.0 })]
    #[case::y_separate(RangeXYZ{ x: 0.0..=0.0, y: 200.0..=400.0, z: 0.0..=0.0, resolution: 1.0 })]
    #[case::z_include(RangeXYZ{ x: 0.0..=0.0, y: 0.0..=0.0, z: -10.0..=100.0, resolution: 1.0 })]
    #[case::z_separate(RangeXYZ{ x: 0.0..=0.0, y: 0.0..=0.0, z: 100.0..=200.0, resolution: 1.0 })]
    #[case::include(RangeXYZ{ x: -10.0..=200.0, y: -10.0..=150.0, z: -10.0..=60.0, resolution: 10.0 })]
    #[case::separate(RangeXYZ{ x: -10.0..=200.0, y: -10.0..=150.0, z: 150.0..=150.0, resolution: 10.0 })]
    fn test_aabb_min_dist(#[case] range: impl Range) {
        let geo = Geometry::new(vec![
            AUTD3 {
                pos: Point3::origin(),
                rot: UnitQuaternion::identity(),
            }
            .into(),
            AUTD3 {
                pos: Point3::new(0., 0., 50.),
                rot: UnitQuaternion::identity(),
            }
            .into(),
        ]);
        approx::assert_relative_eq!(
            aabb_min_dist_naive(&geo, &range),
            aabb_min_dist(&geo.aabb(), &range.aabb())
        );
    }

    #[test]
    fn test_aabb_max_dist_rand() {
        let mut rng = rand::rng();
        let range = RangeXYZ {
            x: -100.0..=100.0,
            y: -100.0..=100.0,
            z: 100.0..=100.0,
            resolution: 10.0,
        };
        for _ in 0..10 {
            let geo = Geometry::new(vec![
                AUTD3 {
                    pos: Point3::new(
                        rng.random_range(-300.0..300.0),
                        rng.random_range(-300.0..300.0),
                        rng.random_range(-300.0..300.0),
                    ),
                    rot: EulerAngle::ZYZ(
                        [0.0 * rad, PI / 2. * rad, PI * rad, PI * 3. / 2. * rad]
                            [rng.random_range(0..4)],
                        [0.0 * rad, PI / 2. * rad, PI * rad, PI * 3. / 2. * rad]
                            [rng.random_range(0..4)],
                        [0.0 * rad, PI / 2. * rad, PI * rad, PI * 3. / 2. * rad]
                            [rng.random_range(0..4)],
                    ),
                }
                .into(),
            ]);
            approx::assert_abs_diff_eq!(
                aabb_max_dist_naive(&geo, &range),
                aabb_max_dist(&geo.aabb(), &range.aabb()),
                epsilon = 1e-3
            );
        }
    }
}
