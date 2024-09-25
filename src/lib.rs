mod error;
mod option;
mod record;
mod utils;

pub use error::EmulatorError;
pub use option::{Range, RecordOption};
use record::{DeviceRecord, Record, TransducerRecord};

use std::time::Duration;

use autd3::{
    driver::{
        defined::ULTRASOUND_PERIOD,
        derive::{Builder, *},
        ethercat::DcSysTime,
        firmware::{
            cpu::{RxMessage, TxDatagram},
            fpga::{EmitIntensity, Phase, SilencerTarget},
        },
        geometry::IntoDevice,
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

pub struct Recorder {
    last_geometry_version: usize,
    is_open: bool,
    emulators: Vec<CPUEmulator>,
    geometry: Geometry,
    timeout: std::time::Duration,
    record: RawRecord,
    recording: bool,
}

#[derive(Builder)]
pub struct RecorderBuilder {
    start_time: DcSysTime,
}

#[cfg_attr(feature = "async-trait", autd3::driver::async_trait)]
impl LinkBuilder for RecorderBuilder {
    type L = Recorder;

    async fn open(self, geometry: &Geometry) -> Result<Self::L, AUTDInternalError> {
        let emulators = geometry
            .iter()
            .enumerate()
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
            last_geometry_version: geometry.version(),
            is_open: true,
            emulators,
            geometry: Geometry::new(geometry.iter().map(|dev| clone_device(dev)).collect()),
            timeout: DEFAULT_TIMEOUT,
            record,
            recording: false,
        })
    }
}

#[cfg_attr(feature = "async-trait", autd3::driver::async_trait)]
impl Link for Recorder {
    async fn close(&mut self) -> Result<(), AUTDInternalError> {
        self.is_open = false;
        Ok(())
    }

    async fn send(&mut self, tx: &TxDatagram) -> Result<bool, AUTDInternalError> {
        self.emulators
            .iter_mut()
            .zip(self.record.records.iter_mut())
            .for_each(|(cpu, r)| {
                cpu.send(tx);

                let should_update_silencer =
                    |tag: u8| -> bool { matches!(tag, TAG_SILENCER | TAG_CLEAR) };
                let update_silencer = should_update_silencer(tx[cpu.idx()].payload[0]);
                let slot_2_offset = tx[cpu.idx()].header.slot_2_offset as usize;
                let update_silencer = if slot_2_offset != 0 {
                    update_silencer || should_update_silencer(tx[cpu.idx()].payload[slot_2_offset])
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

        Ok(true)
    }

    async fn receive(&mut self, rx: &mut [RxMessage]) -> Result<bool, AUTDInternalError> {
        self.emulators.iter_mut().for_each(|cpu| {
            cpu.update_with_sys_time(self.record.current);
            rx[cpu.idx()] = cpu.rx();
        });

        Ok(true)
    }

    async fn update(&mut self, geometry: &Geometry) -> Result<(), AUTDInternalError> {
        if self.last_geometry_version == geometry.version() {
            return Ok(());
        }
        self.last_geometry_version = geometry.version();

        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn timeout(&self) -> std::time::Duration {
        self.timeout
    }
}

impl Recorder {
    pub fn tick(&mut self, tick: Duration) -> Result<(), EmulatorError> {
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

#[derive(Builder)]
pub struct Emulator {
    #[get(ref, ref_mut)]
    geometry: Geometry,
    #[get]
    #[set]
    parallel_threshold: usize,
    #[get]
    #[set]
    send_interval: Duration,
    #[get]
    #[set]
    receive_interval: Duration,
    #[cfg(target_os = "windows")]
    #[get]
    #[set]
    timer_resolution: Option<std::num::NonZeroU32>,
}

impl Emulator {
    #[must_use]
    pub fn new<D: IntoDevice, F: IntoIterator<Item = D>>(iter: F) -> Emulator {
        Self {
            geometry: Geometry::new(
                iter.into_iter()
                    .enumerate()
                    .map(|(i, d)| d.into_device(i as _))
                    .collect(),
            ),
            parallel_threshold: 4,
            send_interval: Duration::from_millis(1),
            receive_interval: Duration::from_millis(1),
            #[cfg(target_os = "windows")]
            timer_resolution: Some(std::num::NonZeroU32::MIN),
        }
    }

    pub async fn record<F>(
        &self,
        f: impl FnOnce(Controller<Recorder>) -> F,
    ) -> Result<Record, EmulatorError>
    where
        F: std::future::Future<Output = Result<Controller<Recorder>, EmulatorError>>,
    {
        self.record_from(DcSysTime::ZERO, f).await
    }

    pub async fn record_from<F>(
        &self,
        start_time: DcSysTime,
        f: impl FnOnce(Controller<Recorder>) -> F,
    ) -> Result<Record, EmulatorError>
    where
        F: std::future::Future<Output = Result<Controller<Recorder>, EmulatorError>>,
    {
        let mut recorder = Controller::builder(self.geometry.iter().map(|dev| clone_device(dev)))
            .with_parallel_threshold(self.parallel_threshold)
            .with_receive_interval(self.receive_interval)
            .with_send_interval(self.send_interval)
            .with_timer_resolution(self.timer_resolution)
            .open(RecorderBuilder { start_time })
            .await?;

        recorder.recording = true;
        let mut recorder = f(recorder).await?;
        recorder.is_open = false;

        let devices = recorder.geometry_mut().drain(..).collect::<Vec<_>>();
        let records = recorder.record.records.drain(..).collect::<Vec<_>>();
        let start = recorder.record.start;
        let end = recorder.record.current;

        recorder.close().await?;

        Ok(Record {
            records: records
                .into_iter()
                .zip(devices.into_iter())
                .map(|(rd, dev)| DeviceRecord {
                    aabb: *dev.aabb(),
                    records: rd
                        .records
                        .into_iter()
                        .zip(dev.into_iter())
                        .map(|(r, tr)| TransducerRecord {
                            pulse_width: r.pulse_width,
                            phase: r.phase,
                            tr,
                        })
                        .collect(),
                })
                .collect(),
            start,
            end,
        })
    }
}
