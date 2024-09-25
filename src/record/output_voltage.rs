use polars::{df, frame::DataFrame, prelude::NamedFrom, series::Series};

use super::Record;

impl Record {
    pub fn output_voltage(&self) -> DataFrame {
        let n = self.records[0].records[0].pulse_width.len();
        let t = self.records[0].records[0].output_times(0, n);
        let series = self
            .records
            .iter()
            .enumerate()
            .flat_map(|(dev_idx, dev)| {
                dev.records.iter().enumerate().map(move |(tr_idx, tr)| {
                    let v = tr._output_voltage_within(0, n).unwrap();
                    Series::new(format!("voltage_{}_{}[V]", dev_idx, tr_idx).into(), &v)
                })
            })
            .collect::<Vec<_>>();
        let mut df = df!("time[s]" => &t).unwrap();
        df.hstack_mut(&series).unwrap();
        df
    }
}
