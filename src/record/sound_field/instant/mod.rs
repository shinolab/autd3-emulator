mod cpu;
#[cfg(feature = "gpu")]
mod gpu;
mod option;

use std::time::Duration;

use autd3::{driver::defined::ULTRASOUND_PERIOD_COUNT, prelude::ULTRASOUND_PERIOD};
use indicatif::ProgressBar;
use polars::{df, frame::DataFrame, prelude::Column};
use unzip3::Unzip3;

use super::{super::Record, SoundFieldOption};
use crate::{EmulatorError, Range};

pub use option::InstantRecordOption;

#[derive(Debug)]
enum ComputeDevice<'a> {
    Cpu(cpu::Cpu<'a>),
    #[cfg(feature = "gpu")]
    Gpu(gpu::Gpu<'a>),
}

impl<'a> ComputeDevice<'a> {
    fn init(&mut self, cache_size: isize, cursor: &mut isize, rem_frame: &mut usize) {
        match self {
            Self::Cpu(cpu) => cpu.init(cache_size, cursor, rem_frame),
            #[cfg(feature = "gpu")]
            Self::Gpu(gpu) => gpu.init(cache_size, cursor, rem_frame),
        }
    }

    fn progress(&mut self, cursor: &mut isize) {
        match self {
            Self::Cpu(cpu) => cpu.progress(cursor),
            #[cfg(feature = "gpu")]
            Self::Gpu(gpu) => gpu.progress(cursor),
        }
    }

    async fn compute(
        &mut self,
        start_time: Duration,
        time_step: Duration,
        num_points_in_frame: usize,
        sound_speed: f32,
        offset: isize,
        pb: &ProgressBar,
    ) -> Result<&Vec<Vec<f32>>, EmulatorError> {
        match self {
            Self::Cpu(cpu) => Ok(cpu.compute(
                start_time,
                time_step,
                num_points_in_frame,
                sound_speed,
                offset,
                pb,
            )),
            #[cfg(feature = "gpu")]
            Self::Gpu(gpu) => {
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
pub struct Instant<'a> {
    pub(crate) cursor: isize,
    option: InstantRecordOption,
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

impl<'a> Instant<'a> {
    pub async fn next(&mut self, duration: Duration) -> Result<DataFrame, EmulatorError> {
        let n = self.next_time_len(duration);
        let mut time = vec![0; n];
        let mut v = vec![vec![0.0; self.next_points_len()]; n];
        self.next_inplace(
            duration,
            false,
            &mut time,
            v.iter_mut().map(|v| v.as_mut_ptr()),
        )
        .await?;

        let columns = time
            .iter()
            .zip(v.iter())
            .map(|(t, v)| Column::new(format!("p[Pa]@{}[ns]", t).into(), v))
            .collect::<Vec<_>>();

        let mut df = df!(
            "x[mm]" => &self.x,
            "y[mm]" => &self.y,
            "z[mm]" => &self.z,
        )
        .unwrap();
        df.hstack_mut(&columns).unwrap();

        Ok(df)
    }

    pub async fn skip(&mut self, duration: Duration) -> Result<&mut Self, EmulatorError> {
        self.next_inplace(duration, true, &mut [], std::iter::empty())
            .await?;
        Ok(self)
    }

    #[cfg(feature = "inplace")]
    pub fn x_inplace(&self, x: &mut [f32]) {
        x.copy_from_slice(&self.x);
    }

    #[cfg(feature = "inplace")]
    pub fn y_inplace(&self, y: &mut [f32]) {
        y.copy_from_slice(&self.y);
    }

    #[cfg(feature = "inplace")]
    pub fn z_inplace(&self, z: &mut [f32]) {
        z.copy_from_slice(&self.z);
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    fn next_time_len(&self, duration: Duration) -> usize {
        let num_frames = (duration.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as usize;
        num_frames * self.num_points_in_frame
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    fn next_points_len(&self) -> usize {
        self.x.len()
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    async fn next_inplace(
        &mut self,
        duration: Duration,
        skip: bool,
        time: &mut [u64],
        mut v: impl Iterator<Item = *mut f32>,
    ) -> Result<(), EmulatorError> {
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

        let mut idx = 0;
        loop {
            if cur_frame == self.last_frame + num_frames {
                break;
            }

            let end_frame = if self.rem_frame == 0 {
                self.compute_device.progress(&mut self.cursor);
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
                    let start_time = (cur_frame + i) as u32 * ULTRASOUND_PERIOD;
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
                        time[idx] = (start_time + (i as u32 * time_step)).as_nanos() as u64;
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                r[i].as_ptr(),
                                v.next().unwrap(),
                                r[i].len(),
                            );
                        }
                        idx += 1;
                    });
                }
            }
            cur_frame = end_frame;
        }
        self.last_frame = cur_frame;

        Ok(())
    }
}

impl Record {
    async fn sound_field_instant(
        &self,
        range: impl Range,
        option: InstantRecordOption,
    ) -> Result<Instant, EmulatorError> {
        if ULTRASOUND_PERIOD.as_nanos() % option.time_step.as_nanos() != 0 {
            return Err(EmulatorError::InvalidTimeStep);
        }

        let max_frame = self.records[0].pulse_width.len();

        let num_points_in_frame =
            (ULTRASOUND_PERIOD.as_nanos() / option.time_step.as_nanos()) as usize;

        let (x, y, z): (Vec<_>, Vec<_>, Vec<_>) = range.points().unzip3();

        let min_dist = crate::utils::aabb::aabb_min_dist(&self.aabb, &range.aabb());
        let max_dist = crate::utils::aabb::aabb_max_dist(&self.aabb, &range.aabb());

        let required_frame_size = (max_dist / option.sound_speed / ULTRASOUND_PERIOD.as_secs_f32())
            .ceil() as usize
            - (min_dist / option.sound_speed / ULTRASOUND_PERIOD.as_secs_f32()).floor() as usize;

        let frame_window_size = {
            let num_transducers = self.records.len();

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
            .map(|tr| tr.output_ultrasound())
            .collect::<Vec<_>>();
        let cache_size = (required_frame_size + frame_window_size) as isize;

        #[cfg(feature = "gpu")]
        let compute_device = if option.gpu {
            ComputeDevice::Gpu(
                gpu::Gpu::new(
                    &x,
                    &y,
                    &z,
                    self.records.iter().map(|tr| *tr.tr.position()),
                    output_ultrasound,
                    frame_window_size,
                    num_points_in_frame,
                    cache_size,
                )
                .await?,
            )
        } else {
            ComputeDevice::Cpu(cpu::Cpu::new(
                &x,
                &y,
                &z,
                self.records.iter().map(|tr| *tr.tr.position()),
                output_ultrasound,
                frame_window_size,
                num_points_in_frame,
            ))
        };
        #[cfg(not(feature = "gpu"))]
        let compute_device = ComputeDevice::Cpu(cpu::Cpu::new(
            &x,
            &y,
            &z,
            self.records.iter().map(|tr| *tr.tr.position()),
            output_ultrasound,
            frame_window_size,
            num_points_in_frame,
        ));

        Ok(Instant {
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

impl<'a> SoundFieldOption<'a> for InstantRecordOption {
    type Output = Instant<'a>;

    async fn sound_field(
        self,
        record: &'a Record,
        range: impl Range,
    ) -> Result<Self::Output, EmulatorError> {
        record.sound_field_instant(range, self).await
    }
}
