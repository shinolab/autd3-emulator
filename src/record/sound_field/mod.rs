mod cpu;
#[cfg(feature = "gpu")]
mod gpu;

use std::time::Duration;

use autd3::{driver::defined::ULTRASOUND_PERIOD_COUNT, prelude::ULTRASOUND_PERIOD};
use bvh::aabb::Aabb;
use indicatif::ProgressBar;
use polars::{df, frame::DataFrame, prelude::NamedFrom, series::Series};

use super::Record;
use crate::{EmulatorError, Range, RecordOption};

#[derive(Debug)]
enum ComputeDevice<'a> {
    CPU(cpu::Cpu<'a>),
    #[cfg(feature = "gpu")]
    GPU(gpu::Gpu<'a>),
}

impl<'a> ComputeDevice<'a> {
    fn init(&mut self, cache_size: isize, cursor: &mut isize, rem_frame: &mut usize) {
        match self {
            Self::CPU(cpu) => cpu.init(cache_size, cursor, rem_frame),
            #[cfg(feature = "gpu")]
            Self::GPU(gpu) => gpu.init(cache_size, cursor, rem_frame),
        }
    }

    fn progress(&mut self, cursor: &mut isize) -> Result<(), EmulatorError> {
        match self {
            Self::CPU(cpu) => cpu.progress(cursor),
            #[cfg(feature = "gpu")]
            Self::GPU(gpu) => gpu.progress(cursor),
        }
    }

    async fn compute(
        &mut self,
        start_time: f32,
        time_step: Duration,
        num_points_in_frame: usize,
        sound_speed: f32,
        offset: isize,
        pb: &ProgressBar,
    ) -> Result<&Vec<Vec<f32>>, EmulatorError> {
        match self {
            Self::CPU(cpu) => Ok(cpu.compute(
                start_time,
                time_step,
                num_points_in_frame,
                sound_speed,
                offset,
                pb,
            )),
            #[cfg(feature = "gpu")]
            Self::GPU(gpu) => {
                gpu.compute(
                    start_time,
                    time_step,
                    num_points_in_frame,
                    sound_speed,
                    offset,
                    pb,
                )
                .await
            }
        }
    }
}

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
    compute_device: ComputeDevice<'a>,
}

impl<'a> SoundField<'a> {
    pub async fn next(&mut self, duration: Duration) -> Result<DataFrame, EmulatorError> {
        self._next(duration, false).await.map(Option::unwrap)
    }

    pub async fn skip(&mut self, duration: Duration) -> Result<&mut Self, EmulatorError> {
        self._next(duration, true).await.map(|_| self)
    }

    async fn _next(
        &mut self,
        duration: Duration,
        skip: bool,
    ) -> Result<Option<DataFrame>, EmulatorError> {
        if duration.as_nanos() % ULTRASOUND_PERIOD.as_nanos() != 0 {
            return Err(EmulatorError::InvalidDuration);
        }
        let num_frames = (duration.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as usize;

        if self.last_frame + num_frames > self.max_frame {
            return Err(EmulatorError::NotRecorded);
        }

        self.compute_device
            .init(self.cache_size, &mut self.cursor, &mut self.rem_frame);

        let time_step = self.option.time_step;
        let sound_speed = self.option.sound_speed;

        let mut cur_frame = self.last_frame;
        let pb = self.option.pb(num_frames * self.num_points_in_frame);

        let mut columns = Vec::new();

        loop {
            if cur_frame == self.last_frame + num_frames {
                break;
            }

            let end_frame = if self.rem_frame == 0 {
                self.compute_device.progress(&mut self.cursor)?;
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

            if !skip {
                let offset = (self.cursor - self.cache_size) * ULTRASOUND_PERIOD_COUNT as isize;
                for i in 0..num_frames {
                    let start_time = ((cur_frame + i) as u32 * ULTRASOUND_PERIOD).as_secs_f32();
                    let r = self
                        .compute_device
                        .compute(
                            start_time,
                            time_step,
                            self.num_points_in_frame,
                            sound_speed,
                            offset,
                            &pb,
                        )
                        .await?;
                    (0..r.len()).for_each(|i| {
                        columns.push(Series::new(
                            format!(
                                "p[Pa]@{}",
                                start_time + (i as u32 * time_step).as_secs_f32()
                            )
                            .into(),
                            &r[i],
                        ));
                    });
                }
            }
            cur_frame = end_frame;
        }
        self.last_frame = cur_frame;

        if skip {
            return Ok(None);
        }

        let mut df = df!(
            "x[mm]" => &self.x,
            "y[mm]" => &self.y,
            "z[mm]" => &self.z,
        )
        .unwrap();
        df.hstack_mut(&columns).unwrap();

        Ok(Some(df))
    }
}

impl Record {
    pub async fn sound_field(
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

            #[cfg(feature = "gpu")]
            let mem_usage = if option.gpu {
                mem_usage
            } else {
                mem_usage + x.len() * num_transducers * size_of::<f32>()
            };
            #[cfg(not(feature = "gpu"))]
            let mem_usage = mem_usage + x.len() * num_transducers * size_of::<f32>();

            let memory_limits = option.memory_limits_hint_mb.saturating_mul(1024 * 1024);

            let frame_window_size_mem = ((memory_limits.saturating_sub(mem_usage))
                / (ULTRASOUND_PERIOD_COUNT * num_transducers * size_of::<f32>()))
            .saturating_sub(required_frame_size)
            .max(1);

            let frame_window_size_time =
                ((Duration::from_nanos(self.end.sys_time() - self.start.sys_time()).as_nanos()
                    / ULTRASOUND_PERIOD.as_nanos()) as usize)
                    .max(1);

            frame_window_size_mem.min(frame_window_size_time)
        };

        let cursor =
            -((max_dist / option.sound_speed / ULTRASOUND_PERIOD.as_secs_f32()).ceil() as isize);

        let output_ultrasound = self
            .records
            .iter()
            .flat_map(|dev| dev.records.iter().map(|tr| tr.output_ultrasound()))
            .collect::<Vec<_>>();
        let cache_size = (required_frame_size + frame_window_size) as isize;

        #[cfg(feature = "gpu")]
        let compute_device = if option.gpu {
            ComputeDevice::GPU(
                gpu::Gpu::new(
                    &x,
                    &y,
                    &z,
                    self.records
                        .iter()
                        .flat_map(|dev| dev.records.iter().map(|tr| *tr.tr.position())),
                    output_ultrasound,
                    frame_window_size,
                    num_points_in_frame,
                    cache_size,
                    option.memory_limits_hint_mb,
                )
                .await?,
            )
        } else {
            ComputeDevice::CPU(cpu::Cpu::new(
                &x,
                &y,
                &z,
                self.records
                    .iter()
                    .flat_map(|dev| dev.records.iter().map(|tr| *tr.tr.position())),
                output_ultrasound,
                frame_window_size,
                num_points_in_frame,
            ))
        };
        #[cfg(not(feature = "gpu"))]
        let compute_device = ComputeDevice::CPU(cpu::Cpu::new(
            &x,
            &y,
            &z,
            self.records
                .iter()
                .flat_map(|dev| dev.records.iter().map(|tr| *tr.tr.position())),
            output_ultrasound,
            frame_window_size,
            num_points_in_frame,
        ));

        Ok(SoundField {
            compute_device,
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
            option,
        })
    }
}
