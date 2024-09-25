use polars::{df, frame::DataFrame, prelude::NamedFrom, series::Series};

use super::Record;

impl Record {
    pub fn output_ultrasound(&self) -> DataFrame {
        let n = self.records[0].records[0].pulse_width.len();

        let time = self.records[0].records[0].output_times(0, n);

        let series = self
            .records
            .iter()
            .enumerate()
            .flat_map(|(dev_idx, dev)| {
                dev.records.iter().enumerate().map(move |(tr_idx, tr)| {
                    let o = tr.output_ultrasound()._next(n).unwrap();
                    Series::new(format!("p_{}_{}[a.u.]", dev_idx, tr_idx).into(), &o)
                })
            })
            .collect::<Vec<_>>();

        let mut df = df!("time[s]" => &time).unwrap();
        df.hstack_mut(&series).unwrap();
        df
    }
}
