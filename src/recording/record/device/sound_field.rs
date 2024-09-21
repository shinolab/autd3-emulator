use std::{collections::VecDeque, time::Duration};

use autd3_driver::{
    defined::{ULTRASOUND_PERIOD, ULTRASOUND_PERIOD_COUNT},
    geometry::Vector3,
};
use polars::prelude::{df, DataFrame, NamedFrom, Series};

use crate::{
    error::EmulatorError,
    recording::{record::transducer, Range, RecordOption, TransducerRecord},
};

use super::DeviceRecord;

#[derive(Debug)]
pub struct SoundField<'a> {
    pub(crate) cursor: isize,
    option: RecordOption,
    last_frame: usize,
    rem_frame: usize,
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    frame_window_size: usize,
    cache_size: isize,
    num_points_in_frame: usize,
    output_ultrasound: Vec<transducer::output_ultrasound::OutputUltrasound<'a>>,
    output_ultrasound_cache: Vec<VecDeque<f32>>,
    transducer_positions: Vec<Vector3>,
}

impl<'a> SoundField<'a> {
    pub(crate) const P0: f32 = autd3_driver::defined::T4010A1_AMPLITUDE * std::f32::consts::SQRT_2
        / (4. * std::f32::consts::PI);

    pub fn next(&mut self, duration: Duration) -> Result<DataFrame, EmulatorError> {
        if duration.as_nanos() % ULTRASOUND_PERIOD.as_nanos() != 0 {
            return Err(EmulatorError::InvalidDuration);
        }

        let time_step = self.option.time_step;
        let sound_speed = self.option.sound_speed;
        let num_frames = (duration.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as usize;

        let mut df = df!(
            "x[mm]" => &self.x,
            "y[mm]" => &self.y,
            "z[mm]" => &self.z,
        )
        .unwrap();

        let mut cur_frame = self.last_frame;
        let pb = self.option.pb(num_frames * self.num_points_in_frame);

        loop {
            if cur_frame == self.last_frame + num_frames {
                break;
            }

            let end_frame = if self.rem_frame != 0 {
                cur_frame + self.rem_frame
            } else {
                cur_frame + self.frame_window_size
            };
            let end_frame = if end_frame > self.last_frame + num_frames {
                self.rem_frame = end_frame - (self.last_frame + num_frames);
                self.last_frame + num_frames
            } else {
                self.rem_frame = 0;
                end_frame
            };
            let num_frames = end_frame - cur_frame;

            let dists = itertools::izip!(self.x.iter(), self.y.iter(), self.z.iter())
                .map(|(&x, &y, &z)| Vector3::new(x, y, z))
                .map(|p| {
                    self.transducer_positions
                        .iter()
                        .map(|tp| (p - tp).norm())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();

            let offset = (self.cursor - self.cache_size) * ULTRASOUND_PERIOD_COUNT as isize;
            (0..num_frames)
                .map(|i| ((cur_frame + i) as u32 * ULTRASOUND_PERIOD).as_secs_f32())
                .for_each(|start_time| {
                    (0..self.num_points_in_frame)
                        .map(|i| start_time + (i as u32 * time_step).as_secs_f32())
                        .for_each(|t| {
                            let p = dists
                                .iter()
                                .map(|d| {
                                    d.iter()
                                        .zip(self.output_ultrasound_cache.iter())
                                        .map(|(dist, output_ultrasound)| {
                                            let t_out = t - dist / sound_speed;
                                            let idx = t_out / TransducerRecord::TS;
                                            let a = idx.floor() as isize;
                                            let alpha = idx - a as f32;
                                            let a = (a - offset) as usize;
                                            Self::P0 / dist
                                                * (output_ultrasound[a] * (1. - alpha)
                                                    + output_ultrasound[a + 1] * alpha)
                                        })
                                        .sum::<f32>()
                                })
                                .collect::<Vec<_>>();
                            pb.inc(1);
                            df.hstack_mut(&[Series::new(format!("p[Pa]@{}", t).into(), &p)])
                                .unwrap();
                        });
                });

            if self.rem_frame == 0 {
                (0..self.frame_window_size).for_each(|_| {
                    self.output_ultrasound_cache
                        .iter_mut()
                        .zip(self.output_ultrasound.iter_mut())
                        .for_each(|(cache, output_ultrasound)| {
                            drop(cache.drain(0..ULTRASOUND_PERIOD_COUNT));
                            cache.extend(if self.cursor >= 0 {
                                output_ultrasound._next(1)
                            } else {
                                vec![0.; ULTRASOUND_PERIOD_COUNT]
                            });
                        });
                    self.cursor += 1;
                });
            }

            cur_frame = end_frame;
        }

        self.last_frame = cur_frame;

        Ok(df)
    }
}

impl<'a> DeviceRecord<'a> {
    pub fn sound_field(
        &'a self,
        range: Range,
        option: RecordOption,
    ) -> Result<SoundField<'a>, EmulatorError> {
        if ULTRASOUND_PERIOD.as_nanos() % option.time_step.as_nanos() != 0 {
            return Err(EmulatorError::InvalidTimeStep);
        }

        let (x, y, z) = range.points();

        let min_dist = crate::utils::aabb::aabb_min_dist(&self.aabb, &range.aabb());
        let max_dist = crate::utils::aabb::aabb_max_dist(&self.aabb, &range.aabb());

        let cursor =
            (max_dist / option.sound_speed / ULTRASOUND_PERIOD.as_secs_f32()).ceil() as usize;
        let frame_window_size = 32;
        let num_points_in_frame =
            (ULTRASOUND_PERIOD.as_nanos() / option.time_step.as_nanos()) as usize;

        let mut output_ultrasound = self
            .iter()
            .map(|tr| tr.output_ultrasound())
            .collect::<Vec<_>>();
        let cache_size = (cursor
            - (min_dist / option.sound_speed / ULTRASOUND_PERIOD.as_secs_f32()).floor() as usize
            + frame_window_size) as isize;
        let cursor = -(cursor as isize);
        let output_ultrasound_cache = output_ultrasound
            .iter_mut()
            .map(|ut| {
                (0..cache_size)
                    .flat_map(|i| {
                        if cursor + i >= 0 {
                            ut._next(1)
                        } else {
                            vec![0.; ULTRASOUND_PERIOD_COUNT]
                        }
                    })
                    .collect()
            })
            .collect();
        let cursor = cursor + cache_size;

        Ok(SoundField {
            cursor,
            last_frame: 0,
            rem_frame: 0,
            x,
            y,
            z,
            frame_window_size,
            cache_size,
            num_points_in_frame,
            output_ultrasound,
            output_ultrasound_cache,
            transducer_positions: self.iter().map(|tr| *tr.tr.position()).collect(),
            option,
        })
    }
}
