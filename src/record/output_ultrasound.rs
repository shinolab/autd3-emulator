use autd3::driver::defined::ULTRASOUND_PERIOD_COUNT;
use polars::{frame::DataFrame, prelude::Column};

use super::Record;

impl Record {
    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    #[doc(hidden)]
    fn output_ultrasound_inplace(&self, mut v: impl Iterator<Item = *mut f32>) {
        let cols = self.drive_cols();
        let rows = self.drive_rows();

        let mut output_ultrasounds = self
            .records
            .iter()
            .map(|tr| tr.output_ultrasound())
            .collect::<Vec<_>>();
        let mut buf = vec![vec![0.0; ULTRASOUND_PERIOD_COUNT]; rows];
        (0..cols).for_each(|_| {
            (0..rows).for_each(|row| {
                output_ultrasounds[row]._next_inplace(1, &mut buf[row]);
            });
            (0..ULTRASOUND_PERIOD_COUNT).for_each(|i| {
                let dst = v.next().unwrap();
                (0..rows).for_each(|row| unsafe {
                    *dst.add(row) = buf[row][i];
                });
            });
        })
    }

    /// Returns the time series data of the emitted ultrasound for each transducer.
    pub fn output_ultrasound(&self) -> DataFrame {
        let mut v = vec![vec![0.; self.drive_rows()]; self.output_cols()];
        self.output_ultrasound_inplace(v.iter_mut().map(|v| v.as_mut_ptr()));
        DataFrame::new(
            (0..self.output_cols())
                .map(|i| i as u64)
                .zip(v.iter())
                .map(|(t, v)| Column::new(format!("p[a.u.]@{}[25us/256]", t).into(), &v))
                .collect(),
        )
        .unwrap()
    }
}
