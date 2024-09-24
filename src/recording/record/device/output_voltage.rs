use polars::prelude::DataFrame;

use super::DeviceRecord;

impl<'a> DeviceRecord<'a> {
    pub fn output_voltage(&self) -> DataFrame {
        let mut df = self[0].output_voltage();
        df.rename("voltage[V]", "voltage_0[V]".into()).unwrap();
        let series = self
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, tr)| {
                let mut d = tr.output_voltage();
                d.rename("voltage[V]", format!("voltage_{}[V]", i).into())
                    .unwrap();
                let mut d = d.take_columns();
                d.pop().unwrap()
            })
            .collect::<Vec<_>>();
        df.hstack_mut(&series).unwrap();
        df
    }
}
