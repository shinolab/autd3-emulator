mod field;
mod record;

use std::{cell::RefCell, time::Duration};

pub use field::{Range, RecordOption, TimeRange};
pub use record::{DeviceRecord, Record, TransducerRecord};

use autd3_driver::{defined::ULTRASOUND_PERIOD, ethercat::DcSysTime, firmware::fpga::Drive};

use crate::{error::EmulatorError, Emulator};

pub(crate) struct RawTransducerRecord {
    pub drive: Vec<Drive>,
    pub modulation: Vec<u8>,
}

pub(crate) struct RawDeviceRecord {
    pub(crate) records: Vec<RawTransducerRecord>,
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
                            drive: Vec::new(),
                            modulation: Vec::new(),
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
                            drive: r.drive,
                            modulation: r.modulation,
                            fpga: sd.cpu.fpga(),
                            tr,
                            output_ultrasound_cache: RefCell::new(Vec::new()),
                        })
                        .collect(),
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
                    sd.device.iter().for_each(|tr| {
                        record.records[tr.dev_idx()].records[tr.idx()]
                            .drive
                            .push(d[tr.idx()]);
                        record.records[tr.dev_idx()].records[tr.idx()]
                            .modulation
                            .push(m);
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
