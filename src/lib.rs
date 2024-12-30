#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides a emulator for autd3 that calculates sound field, emulates of firmware, etc.

mod error;
mod option;
mod record;
mod utils;

use autd3::controller::timer::TimerStrategy;
use autd3::controller::ControllerBuilder;
use bvh::aabb::Aabb;
pub use error::EmulatorError;
pub use option::*;
use polars::{df, frame::DataFrame};
use record::TransducerRecord;
pub use record::{Instant, InstantRecordOption, Record, Rms, RmsRecordOption};

use std::time::Duration;

use derive_more::{Deref, DerefMut};

use autd3::{
    driver::{
        defined::ULTRASOUND_PERIOD,
        derive::{Builder, *},
        ethercat::DcSysTime,
        firmware::{
            cpu::{RxMessage, TxMessage},
            fpga::{EmitIntensity, Phase, SilencerTarget},
        },
        link::{Link, LinkBuilder},
    },
    Controller,
};
use autd3_firmware_emulator::{
    cpu::params::{TAG_CLEAR, TAG_SILENCER},
    fpga::emulator::SilencerEmulator,
    CPUEmulator,
};

use crate::utils::device::clone_device;

pub(crate) struct RawTransducerRecord {
    pub pulse_width: Vec<u8>,
    pub phase: Vec<u8>,
    pub silencer_phase: SilencerEmulator<Phase>,
    pub silencer_intensity: SilencerEmulator<EmitIntensity>,
    pub silencer_target: SilencerTarget,
}

pub(crate) struct RawDeviceRecord {
    pub records: Vec<RawTransducerRecord>,
}

pub(crate) struct RawRecord {
    pub records: Vec<RawDeviceRecord>,
    pub start: DcSysTime,
    pub current: DcSysTime,
}

/// A recorder for the sound field.
pub struct Recorder {
    is_open: bool,
    emulators: Vec<CPUEmulator>,
    geometry: Geometry,
    record: RawRecord,
}

/// A builder for the recorder.
#[derive(Builder)]
pub struct RecorderBuilder {
    start_time: DcSysTime,
}

#[cfg_attr(feature = "async-trait", autd3::driver::async_trait)]
impl LinkBuilder for RecorderBuilder {
    type L = Recorder;

    async fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDDriverError> {
        let emulators = geometry
            .iter()
            .enumerate()
            .filter(|(_, dev)| dev.enable)
            .map(|(i, dev)| CPUEmulator::new(i, dev.num_transducers()))
            .collect::<Vec<_>>();
        let record = RawRecord {
            records: emulators
                .iter()
                .map(|cpu| RawDeviceRecord {
                    records: geometry[cpu.idx()]
                        .iter()
                        .map(|_| RawTransducerRecord {
                            pulse_width: Vec::new(),
                            phase: Vec::new(),
                            silencer_phase: cpu.fpga().silencer_emulator_phase(0),
                            silencer_intensity: cpu.fpga().silencer_emulator_intensity(0),
                            silencer_target: cpu.fpga().silencer_target(),
                        })
                        .collect(),
                })
                .collect(),
            current: self.start_time,
            start: self.start_time,
        };
        Ok(Recorder {
            is_open: true,
            emulators,
            geometry: Geometry::new(
                geometry.devices().map(clone_device).collect(),
                geometry.default_parallel_threshold(),
            ),
            record,
        })
    }
}

#[cfg_attr(feature = "async-trait", autd3::driver::async_trait)]
impl Link for Recorder {
    async fn close(&mut self) -> Result<(), AUTDDriverError> {
        self.is_open = false;
        Ok(())
    }

    async fn send(&mut self, tx: &[TxMessage]) -> Result<bool, AUTDDriverError> {
        self.emulators
            .iter_mut()
            .zip(self.record.records.iter_mut())
            .for_each(|(cpu, r)| {
                cpu.send(tx);

                let should_update_silencer =
                    |tag: u8| -> bool { matches!(tag, TAG_SILENCER | TAG_CLEAR) };
                let update_silencer = should_update_silencer(tx[cpu.idx()].payload()[0]);
                let slot_2_offset = tx[cpu.idx()].header().slot_2_offset as usize;
                let update_silencer = if slot_2_offset != 0 {
                    update_silencer
                        || should_update_silencer(tx[cpu.idx()].payload()[slot_2_offset])
                } else {
                    update_silencer
                };
                if update_silencer {
                    r.records.iter_mut().for_each(|tr| {
                        tr.silencer_target = cpu.fpga().silencer_target();
                        tr.silencer_phase = cpu
                            .fpga()
                            .silencer_emulator_phase_continue_with(tr.silencer_phase);
                        tr.silencer_intensity = cpu
                            .fpga()
                            .silencer_emulator_intensity_continue_with(tr.silencer_intensity);
                    });
                }
            });

        Ok(true)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDDriverError> {
        self.emulators.iter_mut().for_each(|cpu| {
            cpu.update_with_sys_time(self.record.current);
            rx[cpu.idx()] = cpu.rx();
        });

        Ok(true)
    }

    fn is_open(&self) -> bool {
        self.is_open
    }
}

impl Recorder {
    /// Progresses by the specified time.
    pub fn tick(&mut self, tick: Duration) -> Result<(), EmulatorError> {
        // This function must be public for capi.
        if tick.is_zero() || tick.as_nanos() % ULTRASOUND_PERIOD.as_nanos() != 0 {
            return Err(EmulatorError::InvalidTick);
        }
        let mut t = self.record.current;
        let end = t + tick;
        loop {
            self.emulators
                .iter_mut()
                .zip(self.geometry.iter())
                .for_each(|(cpu, dev)| {
                    cpu.update_with_sys_time(t);
                    let m = cpu.fpga().modulation();
                    let d = cpu.fpga().drives();
                    dev.iter().zip(d).for_each(|(tr, d)| {
                        let tr_record = &mut self.record.records[tr.dev_idx()].records[tr.idx()];
                        tr_record.pulse_width.push(match tr_record.silencer_target {
                            SilencerTarget::Intensity => cpu.fpga().pulse_width_encoder_table_at(
                                tr_record
                                    .silencer_intensity
                                    .apply((d.intensity().value() as u16 * m as u16 / 255) as u8)
                                    as _,
                            ),
                            SilencerTarget::PulseWidth => tr_record
                                .silencer_intensity
                                .apply(cpu.fpga().to_pulse_width(d.intensity(), m)),
                        });
                        tr_record
                            .phase
                            .push(tr_record.silencer_phase.apply(d.phase().value()))
                    });
                });
            t = t + ULTRASOUND_PERIOD;
            if t == end {
                break;
            }
        }
        self.record.current = end;
        Ok(())
    }
}

/// A emulator for the AUTD devices.
#[derive(Builder, Deref, DerefMut)]
pub struct Emulator {
    #[get(ref, ref_mut, no_doc)]
    #[deref]
    #[deref_mut]
    geometry: Geometry,
    default_timeout: Duration,
    send_interval: Duration,
    receive_interval: Duration,
    timer_strategy: TimerStrategy,
}

impl Emulator {
    /// Records the sound field.
    ///
    /// # Example
    ///
    /// ```
    /// # use autd3::prelude::*;
    /// # use autd3_emulator::*;
    /// # use std::time::Duration;
    /// # async fn example() -> Result<(), EmulatorError> {
    /// let emulator = Controller::builder([AUTD3::new(Point3::origin())]).into_emulator();
    /// let record = emulator
    ///      .record(|mut autd| async {
    ///          autd.send(Silencer::default()).await?;
    ///          autd.send((Sine::new(200. * Hz), Uniform::new(EmitIntensity::new(0xFF)))).await?;
    ///          autd.tick(Duration::from_millis(10))?;
    ///          Ok(autd)
    ///      })
    ///      .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn record<F>(
        &self,
        f: impl FnOnce(Controller<Recorder>) -> F,
    ) -> Result<Record, EmulatorError>
    where
        F: std::future::Future<Output = Result<Controller<Recorder>, EmulatorError>>,
    {
        self.record_from(DcSysTime::ZERO, f).await
    }

    /// Records the sound field from the specified time.
    pub async fn record_from<F>(
        &self,
        start_time: DcSysTime,
        f: impl FnOnce(Controller<Recorder>) -> F,
    ) -> Result<Record, EmulatorError>
    where
        F: std::future::Future<Output = Result<Controller<Recorder>, EmulatorError>>,
    {
        let builder = Controller::builder(self.geometry.iter().map(clone_device))
            .with_default_parallel_threshold(self.geometry.default_parallel_threshold())
            .with_default_timeout(self.default_timeout)
            .with_receive_interval(self.receive_interval)
            .with_send_interval(self.send_interval)
            .with_timer_strategy(self.timer_strategy);

        let recorder = builder.open(RecorderBuilder { start_time }).await?;

        let mut recorder = f(recorder).await?;

        let start = recorder.link().record.start;
        let end = recorder.link().record.current;
        let devices = {
            let mut tmp: Vec<Device> = Vec::new();
            std::mem::swap(&mut tmp, recorder.geometry_mut());
            tmp
        };

        let aabb = devices
            .iter()
            .filter(|dev| dev.enable)
            .fold(Aabb::empty(), |aabb, dev| aabb.join(dev.aabb()));

        let records = recorder
            .link_mut()
            .record
            .records
            .drain(..)
            .zip(devices.into_iter().filter(|dev| dev.enable))
            .flat_map(|(rd, dev)| {
                rd.records
                    .into_iter()
                    .zip(dev.into_iter())
                    .map(|(r, tr)| TransducerRecord {
                        pulse_width: r.pulse_width,
                        phase: r.phase,
                        tr,
                    })
            })
            .collect();

        recorder.close().await?;

        Ok(Record {
            records,
            start,
            end,
            aabb,
        })
    }

    fn transducers(&self) -> impl Iterator<Item = &Transducer> {
        self.geometry.devices().flat_map(|dev| dev.iter())
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    #[doc(hidden)]
    fn transducer_table_rows(&self) -> usize {
        self.geometry.num_transducers()
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    #[doc(hidden)]
    fn dev_indices_inplace(&self, dev_indices: &mut [u16]) {
        self.transducers()
            .zip(dev_indices.iter_mut())
            .for_each(|(tr, dst)| *dst = tr.dev_idx() as u16);
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    #[doc(hidden)]
    fn tr_indices_inplace(&self, tr_indices: &mut [u8]) {
        self.transducers()
            .zip(tr_indices.iter_mut())
            .for_each(|(tr, dst)| *dst = tr.idx() as u8);
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    #[doc(hidden)]
    fn tr_positions_inplace(&self, x: &mut [f32], y: &mut [f32], z: &mut [f32]) {
        self.transducers()
            .zip(x.iter_mut())
            .zip(y.iter_mut())
            .zip(z.iter_mut())
            .for_each(|(((tr, x), y), z)| {
                *x = tr.position().x;
                *y = tr.position().y;
                *z = tr.position().z;
            });
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    #[doc(hidden)]
    fn tr_dir_inplace(&self, x: &mut [f32], y: &mut [f32], z: &mut [f32]) {
        self.transducers()
            .zip(x.iter_mut())
            .zip(y.iter_mut())
            .zip(z.iter_mut())
            .for_each(|(((tr, x), y), z)| {
                *x = self.geometry[tr.dev_idx()].axial_direction().x;
                *y = self.geometry[tr.dev_idx()].axial_direction().y;
                *z = self.geometry[tr.dev_idx()].axial_direction().z;
            });
    }

    /// Returns properties of transducers.
    pub fn transducer_table(&self) -> DataFrame {
        let n = self.transducer_table_rows();
        let mut dev_indices = vec![0; n];
        let mut tr_indices = vec![0; n];
        let mut x = vec![0.0; n];
        let mut y = vec![0.0; n];
        let mut z = vec![0.0; n];
        let mut nx = vec![0.0; n];
        let mut ny = vec![0.0; n];
        let mut nz = vec![0.0; n];
        self.dev_indices_inplace(&mut dev_indices);
        self.tr_indices_inplace(&mut tr_indices);
        self.tr_positions_inplace(&mut x, &mut y, &mut z);
        self.tr_dir_inplace(&mut nx, &mut ny, &mut nz);
        df!(
            "dev_idx" => &dev_indices,
            "tr_idx" => &tr_indices,
            "x[mm]" => &x,
            "y[mm]" => &y,
            "z[mm]" => &z,
            "nx" => &nx,
            "ny" => &ny,
            "nz" => &nz,
        )
        .unwrap()
    }
}

/// A trait to convert [`ControllerBuilder`] into [`Emulator`].
pub trait ControllerBuilderIntoEmulatorExt {
    /// Converts [`ControllerBuilder`] into [`Emulator`].
    fn into_emulator(self) -> Emulator;
}

impl ControllerBuilderIntoEmulatorExt for ControllerBuilder {
    fn into_emulator(self) -> Emulator {
        let default_parallel_threshold = self.default_parallel_threshold();
        let default_timeout = self.default_timeout();
        let send_interval = self.send_interval();
        let receive_interval = self.receive_interval();
        let timer_strategy = *self.timer_strategy();
        Emulator {
            geometry: Geometry::new(self.devices(), default_parallel_threshold),
            default_timeout,
            send_interval,
            receive_interval,
            timer_strategy,
        }
    }
}

/// A extension trait for `Controller<Recorder>`.
pub trait RecorderControllerExt {
    /// Progresses by the specified time.
    fn tick(&mut self, tick: Duration) -> Result<(), EmulatorError>;
}

impl RecorderControllerExt for Controller<Recorder> {
    fn tick(&mut self, tick: Duration) -> Result<(), EmulatorError> {
        self.link_mut().tick(tick)
    }
}
