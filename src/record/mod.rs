mod output_ultrasound;
mod output_voltage;
mod sound_field;
mod transducer;

use autd3::prelude::DcSysTime;
use derive_more::Debug;
use getset::CopyGetters;
use polars::{frame::DataFrame, prelude::Column};

pub use sound_field::{
    instant::{Instant, InstantRecordOption},
    rms::{Rms, RmsRecordOption},
};
pub(crate) use transducer::TransducerRecord;

/// A record of the ultrasound data.
#[derive(CopyGetters, Debug)]
pub struct Record {
    pub(crate) records: Vec<TransducerRecord>,
    #[getset(get_copy = "pub")]
    /// The start time of the record.
    pub(crate) start: DcSysTime,
    #[getset(get_copy = "pub")]
    /// The end time of the record.
    pub(crate) end: DcSysTime,
    pub(crate) aabb: bvh::aabb::Aabb<f32, 3>,
}

impl Record {
    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    #[doc(hidden)]
    fn drive_rows(&self) -> usize {
        self.records.len()
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    #[doc(hidden)]
    fn drive_cols(&self) -> usize {
        self.records[0].pulse_width.len()
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    #[doc(hidden)]
    fn phase_inplace(&self, time: &mut [u64], mut v: impl Iterator<Item = *mut u8>) {
        let cols = self.drive_cols();
        let rows = self.drive_rows();
        (0..cols).for_each(|col| {
            time[col] = TransducerRecord::time(col);
            let dst = v.next().unwrap();
            (0..rows).for_each(|row| unsafe {
                *dst.add(row) = self.records[row].phase[col];
            });
        })
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    #[doc(hidden)]
    fn pulse_width_inplace(&self, time: &mut [u64], mut v: impl Iterator<Item = *mut u8>) {
        let cols = self.drive_cols();
        let rows = self.drive_rows();
        (0..cols).for_each(|col| {
            time[col] = TransducerRecord::time(col);
            let dst = v.next().unwrap();
            (0..rows).for_each(|row| unsafe {
                *dst.add(row) = self.records[row].pulse_width[col];
            });
        })
    }

    /// Returns the time series data of the phase parameter for each transducer.
    pub fn phase(&self) -> DataFrame {
        let mut time = vec![0; self.drive_cols()];
        let mut phase = vec![vec![0; self.drive_rows()]; self.drive_cols()];
        self.phase_inplace(&mut time, phase.iter_mut().map(|v| v.as_mut_ptr()));
        DataFrame::new(
            time.iter()
                .zip(phase.iter())
                .map(|(t, p)| Column::new(format!("phase@{}[ns]", t).into(), &p))
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    /// Returns the time series data of the pulse width for each transducer.
    pub fn pulse_width(&self) -> DataFrame {
        let mut time = vec![0; self.drive_cols()];
        let mut pulse_width = vec![vec![0; self.drive_rows()]; self.drive_cols()];
        self.pulse_width_inplace(&mut time, pulse_width.iter_mut().map(|v| v.as_mut_ptr()));
        DataFrame::new(
            time.iter()
                .zip(pulse_width.iter())
                .map(|(t, p)| Column::new(format!("pulse_width@{}[ns]", t).into(), &p))
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }
}
