mod field;
mod record;

use std::time::Duration;

use autd3_driver::{
    defined::ULTRASOUND_PERIOD,
    ethercat::DcSysTime,
    firmware::fpga::{EmitIntensity, Phase, SilencerTarget},
};
use autd3_firmware_emulator::fpga::emulator::SilencerEmulator;
pub use field::{Range, RecordOption};
pub use record::{DeviceRecord, Record, TransducerRecord};

use crate::{error::EmulatorError, Emulator};

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

impl Emulator {
    pub fn start_recording(&mut self) -> Result<(), EmulatorError> {
        self.start_record_at(DcSysTime::ZERO)
    }

    pub fn start_record_at(&mut self, start_time: DcSysTime) -> Result<(), EmulatorError> {
        if self.record.is_some() {
            return Err(EmulatorError::RecordingAlreadyStarted);
        }
        self.record = Some(RawRecord {
            records: self
                .sub_devices
                .iter()
                .map(|sd| RawDeviceRecord {
                    records: sd
                        .device
                        .iter()
                        .map(|_| RawTransducerRecord {
                            pulse_width: Vec::new(),
                            phase: Vec::new(),
                            silencer_phase: sd.cpu.fpga().silencer_emulator_phase(0),
                            silencer_intensity: sd.cpu.fpga().silencer_emulator_intensity(0),
                            silencer_target: sd.cpu.fpga().silencer_target(),
                        })
                        .collect(),
                })
                .collect(),
            current: start_time,
            start: start_time,
        });
        Ok(())
    }

    pub fn finish_recording(&mut self) -> Result<Record, EmulatorError> {
        if self.record.is_none() {
            return Err(EmulatorError::RecodingNotStarted);
        }
        let RawRecord {
            records,
            start,
            current: end,
        } = self.record.take().unwrap();
        Ok(Record {
            records: records
                .into_iter()
                .zip(self.sub_devices.iter())
                .map(|(rd, sd)| DeviceRecord {
                    records: rd
                        .records
                        .into_iter()
                        .zip(sd.device.iter())
                        .map(|(r, tr)| TransducerRecord {
                            pulse_width: r.pulse_width,
                            phase: r.phase,
                            tr,
                        })
                        .collect(),
                    aabb: *sd.device.aabb(),
                })
                .collect(),
            start,
            end,
        })
    }

    pub fn tick(&mut self, tick: Duration) -> Result<(), EmulatorError> {
        if let Some(record) = &mut self.record {
            if tick.is_zero() || tick.as_nanos() % ULTRASOUND_PERIOD.as_nanos() != 0 {
                return Err(EmulatorError::InvalidTick);
            }
            let mut t = record.current;
            let end = t + tick;
            loop {
                self.sub_devices.iter_mut().for_each(|sd| {
                    sd.cpu.update_with_sys_time(t);
                    let m = sd.cpu.fpga().modulation();
                    let d = sd.cpu.fpga().drives();
                    sd.device.iter().zip(d).for_each(|(tr, d)| {
                        let tr_record = &mut record.records[tr.dev_idx()].records[tr.idx()];
                        tr_record.pulse_width.push(match tr_record.silencer_target {
                            SilencerTarget::Intensity => {
                                sd.cpu.fpga().pulse_width_encoder_table_at(
                                    tr_record.silencer_intensity.apply(
                                        (d.intensity().value() as u16 * m as u16 / 255) as u8,
                                    ) as _,
                                )
                            }
                            SilencerTarget::PulseWidth => tr_record
                                .silencer_intensity
                                .apply(sd.cpu.fpga().to_pulse_width(d.intensity(), m)),
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
            record.current = end;
            Ok(())
        } else {
            Err(EmulatorError::RecodingNotStarted)
        }
    }
}
