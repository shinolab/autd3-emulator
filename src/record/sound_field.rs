use std::{collections::VecDeque, time::Duration};

use autd3::{
    driver::defined::ULTRASOUND_PERIOD_COUNT,
    prelude::{Vector3, ULTRASOUND_PERIOD},
};
use bvh::aabb::Aabb;
use polars::{df, frame::DataFrame, prelude::NamedFrom, series::Series};
use rayon::prelude::*;

use super::{transducer, Record, TransducerRecord};
use crate::{EmulatorError, Range, RecordOption};

#[derive(Debug)]
pub struct SoundField<'a> {
    pub(crate) cursor: isize,
    option: RecordOption,
    last_frame: usize,
    rem_frame: usize,
    max_frame: usize,
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
    pub(crate) const P0: f32 = autd3::driver::defined::T4010A1_AMPLITUDE * std::f32::consts::SQRT_2
        / (4. * std::f32::consts::PI);

    pub fn next(&mut self, duration: Duration) -> Result<DataFrame, EmulatorError> {
        if duration.as_nanos() % ULTRASOUND_PERIOD.as_nanos() != 0 {
            return Err(EmulatorError::InvalidDuration);
        }
        let num_frames = (duration.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as usize;

        if self.last_frame + num_frames > self.max_frame {
            return Err(EmulatorError::NotRecorded);
        }

        if self.output_ultrasound_cache.is_empty() {
            self.output_ultrasound_cache = self
                .output_ultrasound
                .par_iter_mut()
                .map(|ut| {
                    (0..self.cache_size)
                        .flat_map(|i| {
                            if self.cursor + i >= 0 {
                                ut._next(1)
                                    .unwrap_or_else(|| vec![0.; ULTRASOUND_PERIOD_COUNT])
                            } else {
                                vec![0.; ULTRASOUND_PERIOD_COUNT]
                            }
                        })
                        .collect()
                })
                .collect();
            self.cursor += self.cache_size;
            self.rem_frame = self.frame_window_size;
        }

        let time_step = self.option.time_step;
        let sound_speed = self.option.sound_speed;

        let mut cur_frame = self.last_frame;
        let pb = self.option.pb(num_frames * self.num_points_in_frame);

        let dists = itertools::izip!(self.x.iter(), self.y.iter(), self.z.iter())
            .map(|(&x, &y, &z)| Vector3::new(x, y, z))
            .map(|p| {
                self.transducer_positions
                    .iter()
                    .map(|tp| (p - tp).norm())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let mut columns = Vec::new();

        loop {
            if cur_frame == self.last_frame + num_frames {
                break;
            }

            let end_frame = if self.rem_frame == 0 {
                (0..self.frame_window_size).try_for_each(|_| -> Result<(), EmulatorError> {
                    self.output_ultrasound_cache
                        .iter_mut()
                        .zip(self.output_ultrasound.iter_mut())
                        .try_for_each(
                            |(cache, output_ultrasound)| -> Result<(), EmulatorError> {
                                drop(cache.drain(0..ULTRASOUND_PERIOD_COUNT));
                                cache.extend(if self.cursor >= 0 {
                                    output_ultrasound
                                        ._next(1)
                                        .unwrap_or_else(|| vec![0.; ULTRASOUND_PERIOD_COUNT])
                                } else {
                                    vec![0.; ULTRASOUND_PERIOD_COUNT]
                                });
                                Ok(())
                            },
                        )?;
                    self.cursor += 1;
                    Ok(())
                })?;
                cur_frame + self.frame_window_size
            } else {
                cur_frame + self.rem_frame
            };
            let end_frame = if end_frame > self.last_frame + num_frames {
                self.rem_frame = end_frame - (self.last_frame + num_frames);
                self.last_frame + num_frames
            } else {
                self.rem_frame = 0;
                end_frame
            };
            let num_frames = end_frame - cur_frame;

            let offset = (self.cursor - self.cache_size) * ULTRASOUND_PERIOD_COUNT as isize;
            let mut cache = vec![vec![0.0f32; dists.len()]; self.num_points_in_frame];
            (0..num_frames)
                .map(|i| ((cur_frame + i) as u32 * ULTRASOUND_PERIOD).as_secs_f32())
                .for_each(|start_time| {
                    (0..self.num_points_in_frame)
                        .into_par_iter()
                        .map(|i| start_time + (i as u32 * time_step).as_secs_f32())
                        .map(|t| {
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
                            p
                        })
                        .collect_into_vec(&mut cache);
                    (0..self.num_points_in_frame).for_each(|i| {
                        columns.push(Series::new(
                            format!(
                                "p[Pa]@{}",
                                start_time + (i as u32 * time_step).as_secs_f32()
                            )
                            .into(),
                            &cache[i],
                        ));
                    });
                });

            cur_frame = end_frame;
        }

        let mut df = df!(
            "x[mm]" => &self.x,
            "y[mm]" => &self.y,
            "z[mm]" => &self.z,
        )
        .unwrap();
        df.hstack_mut(&columns).unwrap();

        self.last_frame = cur_frame;

        Ok(df)
    }
}

impl Record {
    pub fn sound_field(
        &self,
        range: Range,
        option: RecordOption,
    ) -> Result<SoundField<'_>, EmulatorError> {
        if ULTRASOUND_PERIOD.as_nanos() % option.time_step.as_nanos() != 0 {
            return Err(EmulatorError::InvalidTimeStep);
        }

        let max_frame = self.records[0].records[0].pulse_width.len();

        let num_points_in_frame =
            (ULTRASOUND_PERIOD.as_nanos() / option.time_step.as_nanos()) as usize;

        let (x, y, z) = range.points();

        let aabb = self
            .records
            .iter()
            .fold(Aabb::empty(), |acc, dev| acc.join(&dev.aabb));

        let min_dist = crate::utils::aabb::aabb_min_dist(&aabb, &range.aabb());
        let max_dist = crate::utils::aabb::aabb_max_dist(&aabb, &range.aabb());

        let required_frame_size = (max_dist / option.sound_speed / ULTRASOUND_PERIOD.as_secs_f32())
            .ceil() as usize
            - (min_dist / option.sound_speed / ULTRASOUND_PERIOD.as_secs_f32()).floor() as usize;

        let frame_window_size = {
            let num_transducers = self.records.iter().map(|r| r.records.len()).sum::<usize>();

            let mem_usage = x.len() * size_of::<f32>()
                + y.len() * size_of::<f32>()
                + z.len() * size_of::<f32>();

            let mem_usage = mem_usage + x.len() * num_transducers * size_of::<f32>();

            let memory_limits = option.memory_limits_hint_mb.saturating_mul(1024 * 1024);

            let frame_window_size_mem = ((memory_limits.saturating_sub(mem_usage))
                / (ULTRASOUND_PERIOD_COUNT * num_transducers))
                .saturating_sub(required_frame_size)
                .max(1);

            let frame_window_size_time =
                ((Duration::from_nanos(self.end.sys_time() - self.start.sys_time()).as_nanos()
                    / ULTRASOUND_PERIOD.as_nanos()) as usize)
                    .max(1);

            frame_window_size_mem.min(frame_window_size_time)
        };

        dbg!(frame_window_size);

        let cursor =
            -((max_dist / option.sound_speed / ULTRASOUND_PERIOD.as_secs_f32()).ceil() as isize);

        let output_ultrasound = self
            .records
            .iter()
            .flat_map(|dev| dev.records.iter().map(|tr| tr.output_ultrasound()))
            .collect::<Vec<_>>();
        let cache_size = (required_frame_size + frame_window_size) as isize;

        Ok(SoundField {
            cursor,
            last_frame: 0,
            rem_frame: 0,
            max_frame,
            x,
            y,
            z,
            frame_window_size,
            cache_size,
            num_points_in_frame,
            output_ultrasound,
            output_ultrasound_cache: Vec::new(),
            transducer_positions: self
                .records
                .iter()
                .flat_map(|dev| dev.records.iter().map(|tr| *tr.tr.position()))
                .collect(),
            option,
        })
    }
}
