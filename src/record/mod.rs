mod output_ultrasound;
mod output_voltage;
mod sound_field;
mod transducer;
mod rms;

use autd3::{derive::Builder, prelude::DcSysTime};
use derive_more::Debug;
use polars::{df, frame::DataFrame, prelude::Column};

pub use sound_field::SoundField;
pub(crate) use transducer::TransducerRecord;

#[derive(Debug)]
pub struct DeviceRecord {
    pub(crate) records: Vec<TransducerRecord>,
    pub(crate) aabb: bvh::aabb::Aabb<f32, 3>,
}

#[derive(Builder, Debug)]
pub struct Record {
    pub(crate) records: Vec<DeviceRecord>,
    #[get]
    pub(crate) start: DcSysTime,
    #[get]
    pub(crate) end: DcSysTime,
}

impl Record {
    #[cfg(feature = "inplace")]
    pub fn drive_time_len(&self) -> usize {
        self.records[0].records[0].pulse_width.len()
    }

    #[cfg(feature = "inplace")]
    pub fn drive_time_inplace(&self, time: &mut [u64]) {
        TransducerRecord::time_inplace(0, self.drive_time_len(), time)
    }

    #[cfg(feature = "inplace")]
    pub fn drive_phase_inplace(&self, dev_idx: usize, tr_idx: usize, phase: &mut [u8]) {
        phase.copy_from_slice(&self.records[dev_idx].records[tr_idx].phase);
    }

    #[cfg(feature = "inplace")]
    pub fn drive_pulsewidth_inplace(&self, dev_idx: usize, tr_idx: usize, pulsewidth: &mut [u8]) {
        pulsewidth.copy_from_slice(&self.records[dev_idx].records[tr_idx].pulse_width);
    }

    #[cfg(feature = "inplace")]
    pub fn num_devices(&self) -> usize {
        self.records.len()
    }

    #[cfg(feature = "inplace")]
    pub fn num_transducers(&self, dev_idx: usize) -> usize {
        self.records[dev_idx].records.len()
    }

    pub fn drive(&self) -> DataFrame {
        let mut df =
            df!("time[ns]" => &TransducerRecord::time(0, self.records[0].records[0].pulse_width.len()))
                .unwrap();
        let series = self
            .records
            .iter()
            .enumerate()
            .flat_map(|(dev_idx, dev)| {
                dev.records
                    .iter()
                    .enumerate()
                    .flat_map(move |(tr_idx, tr)| {
                        [
                            Column::new(format!("phase_{}_{}", dev_idx, tr_idx).into(), &tr.phase),
                            Column::new(
                                format!("pulsewidth_{}_{}", dev_idx, tr_idx).into(),
                                &tr.pulse_width,
                            ),
                        ]
                    })
            })
            .collect::<Vec<_>>();
        df.hstack_mut(&series).unwrap();
        df
    }
}
