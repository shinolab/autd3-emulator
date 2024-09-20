mod output_ultrasound;
mod output_voltage;
mod sound_field;
mod sound_pressure;

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
        self.iter().enumerate().for_each(|(i, tr)| {
            df.hstack_mut(&[
                Series::new(format!("phase_{}", i).into(), &tr.phase),
                Series::new(format!("pulsewidth_{}", i).into(), &tr.pulse_width),
            ])
            .unwrap();
        });
        df
    }
}
