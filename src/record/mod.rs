mod output_ultrasound;
mod output_voltage;
mod sound_field;
mod transducer;

use autd3::{derive::Builder, prelude::DcSysTime};
use derive_more::Debug;
use polars::{df, frame::DataFrame, prelude::NamedFrom, series::Series};

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
    pub fn drive(&self) -> DataFrame {
        let mut df =
            df!("time[s]" => &TransducerRecord::time(0, self.records[0].records[0].pulse_width.len()))
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
                            Series::new(format!("phase_{}_{}", dev_idx, tr_idx).into(), &tr.phase),
                            Series::new(
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