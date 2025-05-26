use std::{collections::VecDeque, time::Duration};

use autd3::{driver::common::ULTRASOUND_PERIOD_COUNT, prelude::Point3};
use indicatif::ProgressBar;

use crate::record::{TransducerRecord, transducer::output_ultrasound::OutputUltrasound};

use rayon::prelude::*;

#[derive(Debug)]
pub(crate) struct Cpu<'a> {
    output_ultrasound: Vec<OutputUltrasound<'a>>,
    output_ultrasound_cache: Vec<VecDeque<f32>>,
    dists: Vec<Vec<f32>>,
    cache: Vec<Vec<f32>>,
    frame_window_size: usize,
}

impl<'a> Cpu<'a> {
    pub(crate) const P0: f32 = autd3::driver::common::T4010A1_AMPLITUDE * std::f32::consts::SQRT_2
        / (4. * std::f32::consts::PI);

    pub(crate) fn new(
        x: &[f32],
        y: &[f32],
        z: &[f32],
        transducer_positions: impl Iterator<Item = Point3>,
        output_ultrasound: Vec<OutputUltrasound<'a>>,
        frame_window_size: usize,
        num_points_in_frame: usize,
    ) -> Self {
        let transducer_positions = transducer_positions.collect::<Vec<_>>();
        let dists = itertools::izip!(x.iter(), y.iter(), z.iter())
            .map(|(&x, &y, &z)| Point3::new(x, y, z))
            .map(|p| {
                transducer_positions
                    .iter()
                    .map(|tp| (p - tp).norm())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        Self {
            output_ultrasound,
            output_ultrasound_cache: Vec::new(),
            cache: vec![vec![0.0f32; dists.len()]; num_points_in_frame],
            dists,
            frame_window_size,
        }
    }

    pub(crate) fn init(&mut self, cache_size: isize, cursor: &mut isize, rem_frame: &mut usize) {
        if self.output_ultrasound_cache.is_empty() {
            self.output_ultrasound_cache = self
                .output_ultrasound
                .par_iter_mut()
                .map(|ut| {
                    (0..cache_size)
                        .flat_map(|i| {
                            if *cursor + i >= 0 {
                                ut._next(1)
                                    .unwrap_or_else(|| vec![0.; ULTRASOUND_PERIOD_COUNT])
                            } else {
                                vec![0.; ULTRASOUND_PERIOD_COUNT]
                            }
                        })
                        .collect()
                })
                .collect();
            *cursor += cache_size;
            *rem_frame = self.frame_window_size;
        }
    }

    pub(crate) fn progress(&mut self, cursor: &mut isize) {
        let n = match *cursor {
            c if (c + self.frame_window_size as isize) < 0 => 0,
            c if c >= 0 => self.frame_window_size,
            c => (c + self.frame_window_size as isize) as usize,
        };
        self.output_ultrasound_cache
            .iter_mut()
            .zip(self.output_ultrasound.iter_mut())
            .par_bridge()
            .for_each(|(cache, output_ultrasound)| {
                drop(cache.drain(0..ULTRASOUND_PERIOD_COUNT * n));
                (0..n).for_each(|_| {
                    cache.extend(
                        output_ultrasound
                            ._next(1)
                            .unwrap_or_else(|| vec![0.; ULTRASOUND_PERIOD_COUNT]),
                    );
                })
            });
        *cursor += self.frame_window_size as isize;
    }

    pub(crate) fn compute(
        &mut self,
        start_time: Duration,
        time_step: Duration,
        num_points_in_frame: usize,
        sound_speed: f32,
        offset: isize,
        pb: &ProgressBar,
    ) -> &Vec<Vec<f32>> {
        (0..num_points_in_frame)
            .into_par_iter()
            .map(|i| (start_time + i as u32 * time_step).as_secs_f32())
            .map(|t| {
                let p = self
                    .dists
                    .iter()
                    .map(|d| {
                        Self::P0
                            * d.iter()
                                .zip(self.output_ultrasound_cache.iter())
                                .map(|(dist, output_ultrasound)| {
                                    let t_out = t - dist / sound_speed;
                                    let a = t_out / TransducerRecord::TS;
                                    let idx = a.floor() as isize;
                                    let alpha = a - idx as f32;
                                    let idx = (idx - offset) as usize;
                                    (output_ultrasound[idx] * (1. - alpha)
                                        + output_ultrasound[idx + 1] * alpha)
                                        / dist
                                })
                                .sum::<f32>()
                    })
                    .collect::<Vec<_>>();
                pb.inc(1);
                p
            })
            .collect_into_vec(&mut self.cache);
        &self.cache
    }
}
