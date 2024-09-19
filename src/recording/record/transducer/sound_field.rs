use autd3_driver::geometry::Vector3;

use polars::prelude::*;

use crate::recording::{Range, RecordOption};

use super::TransducerRecord;

impl<'a> TransducerRecord<'a> {
    #[inline(always)]
    pub(crate) fn _sound_field_at(
        &self,
        dist: f32,
        t: f32,
        sound_speed: f32,
        output_ultrasound: &[f32],
    ) -> f32 {
        const P0: f32 =
            autd3_driver::defined::T4010A1_AMPLITUDE * 1.41421356237 / (4. * std::f32::consts::PI);

        let t_out = t - dist / sound_speed;
        let idx = t_out / Self::TS;
        let a = idx.floor() as isize;
        // TODO: more precise interpolation
        P0 / dist
            * match a {
                a if a < 0 => 0.,
                a if a == output_ultrasound.len() as isize - 1 => output_ultrasound[a as usize],
                a if a > output_ultrasound.len() as isize - 1 => 0.,
                a => {
                    let alpha = idx - a as f32;
                    output_ultrasound[a as usize] * (1. - alpha)
                        + output_ultrasound[(a + 1) as usize] * alpha
                }
            }
    }

    pub fn sound_field_at(&self, point: Vector3, option: RecordOption) -> DataFrame {
        let times = option
            .time
            .map(|t| t.times().collect())
            .unwrap_or(self.output_times());
        let output_ultrasound = self._output_ultrasound();
        let tp = self.tr.position();
        let dist = (point - tp).norm();
        let p = times
            .iter()
            .map(|&t| {
                self._sound_field_at(dist, t, option.sound_speed, output_ultrasound.as_slice())
            })
            .collect::<Vec<_>>();
        df!(
            "time[s]" => &times,
            &format!("p[Pa]@({},{},{})", point.x, point.y, point.z) => &p
        )
        .unwrap()
    }

    pub(crate) fn _sound_field(&self, dist: &[f32], t: f32, sound_speed: f32) -> Vec<f32> {
        let output_ultrasound = self._output_ultrasound();
        dist.iter()
            .map(|&d| self._sound_field_at(d, t, sound_speed, output_ultrasound.as_slice()))
            .collect()
    }

    pub fn sound_field(&self, range: Range, option: RecordOption) -> DataFrame {
        let (x, y, z) = range.points();
        let mut df = df!(
                "x[mm]" => &x,
                "y[mm]" => &y,
                "z[mm]" => &z)
        .unwrap();
        let p = itertools::izip!(x, y, z)
            .map(|(x, y, z)| (Vector3::new(x, y, z) - self.tr.position()).norm())
            .collect::<Vec<_>>();
        let times = option
            .time
            .map(|t| t.times().collect())
            .unwrap_or(self.output_times());
        times.into_iter().for_each(|t| {
            let p = self._sound_field(&p, t, option.sound_speed);
            df.hstack_mut(&[Series::new(format!("p[Pa]@{}", t).into(), &p)])
                .unwrap();
        });
        df
    }
}
