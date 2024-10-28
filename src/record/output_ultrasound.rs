use autd3::driver::defined::ULTRASOUND_PERIOD_COUNT;
use polars::{df, frame::DataFrame, prelude::Column};

use super::Record;

impl Record {
    #[cfg(feature = "inplace")]
    pub fn output_ultrasound_inplace(&self, dev_idx: usize, tr_idx: usize, v: &mut [f32]) {
        let n = self.records[dev_idx].records[tr_idx].pulse_width.len();
        self.records[dev_idx].records[tr_idx]
            .output_ultrasound()
            ._next_inplace(n, v);
    }

    pub fn output_ultrasound(&self) -> DataFrame {
        let n = self.records[0].records[0].pulse_width.len();

        let time = self.records[0].records[0].output_times(0, n);

        let mut v = vec![0.0; n * ULTRASOUND_PERIOD_COUNT];
        let series = (0..self.records.len())
            .flat_map(|dev_idx| {
                (0..self.records[dev_idx].records.len()).map(move |tr_idx| (dev_idx, tr_idx))
            })
            .map(|(dev_idx, tr_idx)| {
                self.records[dev_idx].records[tr_idx]
                    .output_ultrasound()
                    ._next_inplace(n, &mut v);
                Column::new(format!("p_{}_{}[a.u.]", dev_idx, tr_idx).into(), &v)
            })
            .collect::<Vec<_>>();

        let mut df = df!("time[25us/256]" => &time).unwrap();
        df.hstack_mut(&series).unwrap();
        df
    }
}
