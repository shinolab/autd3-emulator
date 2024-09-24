mod output_ultrasound;
mod output_voltage;
mod sound_field;

use polars::prelude::*;

use super::TransducerRecord;

use derive_more::Deref;

#[derive(Deref, Debug)]
pub struct DeviceRecord<'a> {
    #[deref]
    pub(crate) records: Vec<TransducerRecord<'a>>,
    pub(crate) aabb: bvh::aabb::Aabb<f32, 3>,
}

impl<'a> DeviceRecord<'a> {
    pub fn drive(&self) -> DataFrame {
        let mut df =
            df!("time[s]" => &TransducerRecord::time(0, self.records[0].pulse_width.len()))
                .unwrap();
        let series = self
            .iter()
            .enumerate()
            .flat_map(|(i, tr)| {
                [
                    Series::new(format!("phase_{}", i).into(), &tr.phase),
                    Series::new(format!("pulsewidth_{}", i).into(), &tr.pulse_width),
                ]
            })
            .collect::<Vec<_>>();
        df.hstack_mut(&series).unwrap();
        df
    }
}
