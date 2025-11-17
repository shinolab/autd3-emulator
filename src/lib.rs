#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::unescaped_backticks)]

//! This crate provides a emulator for autd3 that calculates sound field, emulates of firmware, etc.

mod error;
mod option;
mod record;
mod utils;

pub use error::EmulatorError;
pub use option::*;
#[cfg(feature = "polars")]
use polars::{df, frame::DataFrame};
use record::TransducerRecord;
pub use record::{Instant, InstantRecordOption, Record, Rms, RmsRecordOption};

use std::time::Duration;

use autd3::{
    controller::{Controller, SenderOption},
    driver::{common::ULTRASOUND_PERIOD, ethercat::DcSysTime},
};
use autd3_core::{
    firmware::{Drive, Intensity, Phase},
    geometry::{Device, Geometry, Transducer},
    link::{Link, LinkError, RxMessage, TxBufferPoolSync, TxMessage},
};
use autd3_firmware_emulator::{
    CPUEmulator,
    cpu::params::{TAG_CLEAR, TAG_SILENCER},
    fpga::emulator::SilencerEmulator,
};

use crate::utils::{aabb::Aabb, device::clone_device};

pub(crate) struct RawTransducerRecord {
    pub pulse_width: Vec<u16>,
    pub phase: Vec<u8>,
    pub silencer_phase: SilencerEmulator<Phase>,
    pub silencer_intensity: SilencerEmulator<Intensity>,
}

pub(crate) struct RawDeviceRecord {
    pub records: Vec<RawTransducerRecord>,
}

pub(crate) struct RawRecord {
    pub records: Vec<RawDeviceRecord>,
    pub start: DcSysTime,
    pub current: DcSysTime,
}

struct NopSleeper;

impl autd3_core::sleep::Sleeper for NopSleeper {
    fn sleep(&self, _duration: Duration) {} // GRCOV_EXCL_LINE
}

/// A recorder for the sound field.
pub struct Recorder {
    start_time: DcSysTime,
    is_open: bool,
    emulators: Vec<CPUEmulator>,
    geometry: Geometry,
    record: RawRecord,
    buffer_pool: TxBufferPoolSync,
    drives_buffer: Vec<Vec<Drive>>,
    phases_buffer: Vec<Vec<Phase>>,
    output_mask_buffer: Vec<Vec<bool>>,
}

impl Recorder {
    fn new(start_time: DcSysTime) -> Self {
        Self {
            start_time,
            is_open: false,
            emulators: Vec::new(),
            geometry: Geometry::new(Vec::new()),
            record: RawRecord {
                records: Vec::new(),
                start: DcSysTime::ZERO,
                current: DcSysTime::ZERO,
            },
            buffer_pool: TxBufferPoolSync::new(),
            drives_buffer: Vec::new(),
            phases_buffer: Vec::new(),
            output_mask_buffer: Vec::new(),
        }
    }
}

impl Link for Recorder {
    // GRCOV_EXCL_START
    fn close(&mut self) -> Result<(), LinkError> {
        self.is_open = false;
        Ok(())
    }
    // GRCOV_EXCL_STOP

    fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        self.emulators
            .iter_mut()
            .zip(self.record.records.iter_mut())
            .for_each(|(cpu, r)| {
                cpu.send(&tx);

                let should_update_silencer =
                    |tag: u8| -> bool { matches!(tag, TAG_SILENCER | TAG_CLEAR) };
                let update_silencer = should_update_silencer(tx[cpu.idx()].payload()[0]);
                let slot_2_offset = tx[cpu.idx()].header.slot_2_offset as usize;
                let update_silencer = if slot_2_offset != 0 {
                    update_silencer
                        || should_update_silencer(tx[cpu.idx()].payload()[slot_2_offset])
                } else {
                    update_silencer
                };
                if update_silencer {
                    r.records.iter_mut().for_each(|tr| {
                        tr.silencer_phase = cpu
                            .fpga()
                            .silencer_emulator_phase_continue_with(tr.silencer_phase);
                        tr.silencer_intensity = cpu
                            .fpga()
                            .silencer_emulator_intensity_continue_with(tr.silencer_intensity);
                    });
                }
            });
        self.buffer_pool.return_buffer(tx);

        Ok(())
    }

    fn receive(&mut self, rx: &mut [RxMessage]) -> Result<(), LinkError> {
        self.emulators.iter_mut().for_each(|cpu| {
            cpu.update_with_sys_time(self.record.current);
            rx[cpu.idx()] = cpu.rx();
        });

        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn open(&mut self, geometry: &Geometry) -> Result<(), LinkError> {
        self.is_open = true;
        self.buffer_pool.init(geometry);
        self.emulators = geometry
            .iter()
            .enumerate()
            .map(|(i, dev)| CPUEmulator::new(i, dev.num_transducers()))
            .collect::<Vec<_>>();
        self.record = RawRecord {
            records: self
                .emulators
                .iter()
                .map(|cpu| RawDeviceRecord {
                    records: geometry[cpu.idx()]
                        .iter()
                        .map(|_| RawTransducerRecord {
                            pulse_width: Vec::new(),
                            phase: Vec::new(),
                            silencer_phase: cpu.fpga().silencer_emulator_phase(0),
                            silencer_intensity: cpu.fpga().silencer_emulator_intensity(0),
                        })
                        .collect(),
                })
                .collect(),
            current: self.start_time,
            start: self.start_time,
        };
        self.geometry = Geometry::new(geometry.iter().map(clone_device).collect());
        self.drives_buffer = self
            .emulators
            .iter()
            .map(|cpu| vec![Drive::NULL; cpu.num_transducers()])
            .collect();
        self.phases_buffer = self
            .emulators
            .iter()
            .map(|cpu| vec![Phase::ZERO; cpu.num_transducers()])
            .collect();
        self.output_mask_buffer = self
            .emulators
            .iter()
            .map(|cpu| vec![true; cpu.num_transducers()])
            .collect();

        Ok(())
    }

    fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        Ok(self.buffer_pool.borrow())
    }
}

impl Recorder {
    /// Progresses by the specified time.
    pub fn tick(&mut self, tick: Duration) -> Result<(), EmulatorError> {
        // This function must be public for capi.
        if tick.is_zero() || !tick.as_nanos().is_multiple_of(ULTRASOUND_PERIOD.as_nanos()) {
            return Err(EmulatorError::InvalidTick);
        }
        let mut t = self.record.current;
        let end = t + tick;
        loop {
            self.emulators
                .iter_mut()
                .zip(self.geometry.iter())
                .zip(self.drives_buffer.iter_mut())
                .zip(self.phases_buffer.iter_mut())
                .zip(self.output_mask_buffer.iter_mut())
                .for_each(|((((cpu, dev), drives_buf), phase_buf), output_mask_buf)| {
                    cpu.update_with_sys_time(t);
                    let m = cpu.fpga().modulation();
                    let cur_seg = cpu.fpga().current_stm_segment();
                    let cur_idx = cpu.fpga().current_stm_idx();
                    unsafe {
                        cpu.fpga().drives_at_inplace(
                            cur_seg,
                            cur_idx,
                            phase_buf.as_mut_ptr(),
                            output_mask_buf.as_mut_ptr(),
                            drives_buf.as_mut_ptr(),
                        )
                    };
                    dev.iter().zip(drives_buf).for_each(|(tr, d)| {
                        let tr_record = &mut self.record.records[tr.dev_idx()].records[tr.idx()];
                        tr_record.pulse_width.push(
                            cpu.fpga()
                                .pulse_width_encoder_table_at(
                                    tr_record
                                        .silencer_intensity
                                        .apply((d.intensity.0 as u16 * m as u16 / 255) as u8)
                                        as _,
                                )
                                .pulse_width()
                                .unwrap(),
                        );
                        tr_record
                            .phase
                            .push(tr_record.silencer_phase.apply(d.phase.0))
                    });
                });
            t += ULTRASOUND_PERIOD;
            if t == end {
                break;
            }
        }
        self.record.current = end;
        Ok(())
    }
}

/// A emulator for the AUTD devices.
pub struct Emulator {
    /// The geometry of the devices.
    geometry: Geometry,
}

impl std::ops::Deref for Emulator {
    type Target = Geometry;

    fn deref(&self) -> &Self::Target {
        &self.geometry
    }
}

impl std::ops::DerefMut for Emulator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.geometry
    }
}

impl Emulator {
    /// Creates a new emulator.
    pub fn new<D: Into<Device>, F: IntoIterator<Item = D>>(devices: F) -> Self {
        Self {
            geometry: Geometry::new(devices.into_iter().map(|dev| dev.into()).collect()),
        }
    }

    #[doc(hidden)]
    pub const fn geometry(&self) -> &Geometry {
        &self.geometry
    }

    #[doc(hidden)]
    pub fn geometry_mut(&mut self) -> &mut Geometry {
        &mut self.geometry
    }

    /// Records the sound field.
    ///
    /// # Example
    ///
    /// ```
    /// # use autd3::prelude::*;
    /// # use autd3_emulator::*;
    /// # use std::time::Duration;
    /// # fn example() -> Result<(), EmulatorError> {
    /// let emulator = Emulator::new([AUTD3 {
    ///        pos: Point3::origin(),
    ///        rot: UnitQuaternion::identity(),
    ///    }]);
    /// let record = emulator
    ///      .record(|autd| {
    ///          autd.send(Silencer::default())?;
    ///          autd.send((Sine { freq: 200 * Hz, option: Default::default() }, Uniform { intensity: Intensity(0xFF), phase: Phase::ZERO }))?;
    ///          autd.tick(Duration::from_millis(10))?;
    ///          Ok(())
    ///      })
    ///      ?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn record(
        &self,
        f: impl FnOnce(&mut Controller<Recorder>) -> Result<(), EmulatorError>,
    ) -> Result<Record, EmulatorError> {
        self.record_from(DcSysTime::ZERO, f)
    }

    fn collect_record(mut recorder: Controller<Recorder>) -> Result<Record, EmulatorError> {
        let start = recorder.link().record.start;
        let end = recorder.link().record.current;

        // Here, we take the geometry from the recorder and clear it.
        // So, calling `Controller::send` cause `failed to confirm response` error after here.
        let mut geometry = {
            let mut tmp = Geometry::new(Vec::new());
            std::mem::swap(&mut tmp, &mut recorder);
            tmp
        };
        recorder.link_mut().is_open = false;

        let aabb = Aabb::from_geometry(&geometry);
        let records = recorder
            .link_mut()
            .record
            .records
            .drain(..)
            .zip(geometry.drain(..))
            .flat_map(|(rd, dev)| {
                rd.records
                    .into_iter()
                    .zip(dev)
                    .map(|(r, tr)| TransducerRecord {
                        pulse_width: r.pulse_width,
                        phase: r.phase,
                        tr,
                    })
            })
            .collect();

        drop(recorder);

        Ok(Record {
            records,
            start,
            end,
            aabb,
        })
    }

    /// Records the sound field from the specified time.
    pub fn record_from(
        &self,
        start_time: DcSysTime,
        f: impl FnOnce(&mut Controller<Recorder>) -> Result<(), EmulatorError>,
    ) -> Result<Record, EmulatorError> {
        let mut recorder = Controller::open_with(
            self.geometry.iter().map(clone_device),
            Recorder::new(start_time),
            SenderOption {
                send_interval: None,
                receive_interval: None,
                ..Default::default()
            },
            NopSleeper,
        )?;
        f(&mut recorder)?;
        Self::collect_record(recorder)
    }

    // GRCOV_EXCL_START
    #[doc(hidden)]
    // This function is used in capi.
    pub fn record_from_take(
        &self,
        start_time: DcSysTime,
        f: impl FnOnce(Controller<Recorder>) -> Result<Controller<Recorder>, EmulatorError>,
    ) -> Result<Record, EmulatorError> {
        let recorder = Controller::open_with(
            self.geometry.iter().map(clone_device),
            Recorder::new(start_time),
            SenderOption {
                send_interval: None,
                receive_interval: None,
                ..Default::default()
            },
            NopSleeper,
        )?;
        let recorder = f(recorder)?;
        Self::collect_record(recorder)
    }
    // GRCOV_EXCL_STOP

    fn transducers(&self) -> impl Iterator<Item = &Transducer> {
        self.geometry.iter().flat_map(|dev| dev.iter())
    }

    #[doc(hidden)]
    pub fn transducer_table_rows(&self) -> usize {
        self.geometry.num_transducers()
    }

    #[doc(hidden)]
    pub fn dev_indices_inplace(&self, dev_indices: &mut [u16]) {
        self.transducers()
            .zip(dev_indices.iter_mut())
            .for_each(|(tr, dst)| *dst = tr.dev_idx() as u16);
    }

    #[doc(hidden)]
    pub fn tr_indices_inplace(&self, tr_indices: &mut [u8]) {
        self.transducers()
            .zip(tr_indices.iter_mut())
            .for_each(|(tr, dst)| *dst = tr.idx() as u8);
    }

    #[doc(hidden)]
    pub fn tr_positions_inplace(&self, x: &mut [f32], y: &mut [f32], z: &mut [f32]) {
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

    #[doc(hidden)]
    pub fn tr_dir_inplace(&self, x: &mut [f32], y: &mut [f32], z: &mut [f32]) {
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

    #[cfg(feature = "polars")]
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

#[cfg(test)]
mod tests {
    use super::*;

    use autd3::prelude::*;

    #[test]
    fn deref_mut() -> Result<(), Box<dyn std::error::Error>> {
        let mut autd = Emulator::new([AUTD3::default()]);
        assert_eq!(1, autd.len());
        autd.reconfigure(|dev| dev);
        Ok(())
    }

    #[test]
    fn geometry() -> Result<(), Box<dyn std::error::Error>> {
        let mut autd = Emulator::new([AUTD3::default()]);
        assert_eq!(1, autd.geometry().len());
        autd.geometry_mut().reconfigure(|dev| dev);
        Ok(())
    }
}
