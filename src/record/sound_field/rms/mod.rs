mod cpu;
#[cfg(feature = "gpu")]
mod gpu;
mod option;

use std::{
    f32::consts::{PI, SQRT_2},
    time::Duration,
};

use autd3::{
    derive::Phase,
    driver::defined::ULTRASOUND_PERIOD_COUNT,
    prelude::{ULTRASOUND_FREQ, ULTRASOUND_PERIOD},
};
use polars::{df, frame::DataFrame, prelude::Column};
use unzip3::Unzip3;

use super::{super::Record, SoundFieldOption};
use crate::{EmulatorError, Range};

pub use option::RmsRecordOption;

#[derive(Debug)]
struct RmsTransducerRecord {
    pub(crate) amp: Vec<f32>,
    pub(crate) phase: Vec<f32>,
}

#[derive(Debug)]
enum ComputeDevice {
    Cpu(cpu::Cpu),
    #[cfg(feature = "gpu")]
    Gpu(gpu::Gpu),
}

impl ComputeDevice {
    async fn compute(&mut self, idx: usize, sound_speed: f32) -> Result<&Vec<f32>, EmulatorError> {
        match self {
            Self::Cpu(cpu) => Ok(cpu.compute(idx, sound_speed)),
            #[cfg(feature = "gpu")]
            Self::Gpu(gpu) => gpu.compute(idx, sound_speed).await,
        }
    }
}

#[derive(Debug)]
pub struct Rms {
    option: RmsRecordOption,
    cursor: usize,
    max_frame: usize,
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
    compute_device: ComputeDevice,
}

impl Rms {
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
            .map(|(t, v)| Column::new(format!("rms[Pa]@{}[ns]", t).into(), v))
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
        (duration.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as usize
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

        if self.cursor + num_frames > self.max_frame {
            return Err(EmulatorError::NotRecorded);
        }

        if !skip {
            let wavenumber = 2. * PI * ULTRASOUND_FREQ.hz() as f32 / self.option.sound_speed;
            let pb = self.option.pb(num_frames);
            let mut i = 0;
            while i < num_frames {
                let cur_frame = self.cursor + i;
                let r = self.compute_device.compute(cur_frame, wavenumber).await?;
                time[i] = (cur_frame as u32 * ULTRASOUND_PERIOD).as_nanos() as u64;
                unsafe {
                    std::ptr::copy_nonoverlapping(r.as_ptr(), v.next().unwrap(), r.len());
                }
                i += 1;
                pb.inc(1);
            }
        }

        self.cursor += num_frames;

        Ok(())
    }
}

impl Record {
    pub(crate) const P0: f32 = autd3::driver::defined::T4010A1_AMPLITUDE / (4. * PI) / SQRT_2;

    async fn sound_field_rms(
        &self,
        range: impl Range,
        option: RmsRecordOption,
    ) -> Result<Rms, EmulatorError> {
        let max_frame = self.records[0].records[0].pulse_width.len();

        let (x, y, z): (Vec<_>, Vec<_>, Vec<_>) = range.points().unzip3();

        let records = self
            .records
            .iter()
            .flat_map(|r| {
                r.records.iter().map(|tr| RmsTransducerRecord {
                    amp: (tr
                        .pulse_width
                        .iter()
                        .map(|&w| Self::P0 * (PI * w as f32 / ULTRASOUND_PERIOD_COUNT as f32).sin())
                        .collect()),
                    phase: tr.phase.iter().map(|&p| Phase::new(p).radian()).collect(),
                })
            })
            .collect();

        #[cfg(feature = "gpu")]
        let compute_device = if option.gpu {
            ComputeDevice::Gpu(
                gpu::Gpu::new(
                    &x,
                    &y,
                    &z,
                    self.records
                        .iter()
                        .flat_map(|dev| dev.records.iter().map(|tr| *tr.tr.position())),
                    records,
                )
                .await?,
            )
        } else {
            ComputeDevice::Cpu(cpu::Cpu::new(
                &x,
                &y,
                &z,
                self.records
                    .iter()
                    .flat_map(|dev| dev.records.iter().map(|tr| *tr.tr.position())),
                records,
            ))
        };
        #[cfg(not(feature = "gpu"))]
        let compute_device = ComputeDevice::Cpu(cpu::Cpu::new(
            &x,
            &y,
            &z,
            self.records
                .iter()
                .flat_map(|dev| dev.records.iter().map(|tr| *tr.tr.position())),
            records,
        ));

        Ok(Rms {
            compute_device,
            cursor: 0,
            max_frame,
            x,
            y,
            z,
            option,
        })
    }
}

impl<'a> SoundFieldOption<'a> for RmsRecordOption {
    type Output = Rms;

    async fn sound_field(
        self,
        record: &'a Record,
        range: impl Range,
    ) -> Result<Self::Output, EmulatorError> {
        record.sound_field_rms(range, self).await
    }
}