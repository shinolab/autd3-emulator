mod output_ultrasound;
mod output_voltage;
mod sound_field;
mod transducer;

use autd3::{derive::Builder, prelude::DcSysTime};
use derive_more::Debug;
use polars::{df, frame::DataFrame, prelude::Column};

pub use sound_field::{
    instant::{Instant, InstantRecordOption},
    rms::{Rms, RmsRecordOption},
};
pub(crate) use transducer::TransducerRecord;

#[derive(Builder, Debug)]
pub struct Record {
    pub(crate) records: Vec<TransducerRecord>,
    #[get]
    pub(crate) start: DcSysTime,
    #[get]
    pub(crate) end: DcSysTime,
    pub(crate) aabb: bvh::aabb::Aabb<f32, 3>,
}

impl Record {
    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    fn drive_rows(&self) -> usize {
        self.records.len()
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    fn drive_cols(&self) -> usize {
        self.records[0].pulse_width.len()
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    fn dev_indices_inplace(&self, dev_indices: &mut [u16]) {
        self.records
            .iter()
            .zip(dev_indices.iter_mut())
            .for_each(|(src, dst)| *dst = src.tr.dev_idx() as u16);
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    fn tr_indices_inplace(&self, tr_indices: &mut [u8]) {
        self.records
            .iter()
            .zip(tr_indices.iter_mut())
            .for_each(|(src, dst)| *dst = src.tr.idx() as u8);
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
    fn tr_positions_inplace(&self, x: &mut [f32], y: &mut [f32], z: &mut [f32]) {
        self.records
            .iter()
            .zip(x.iter_mut())
            .zip(y.iter_mut())
            .zip(z.iter_mut())
            .for_each(|(((src, x), y), z)| {
                *x = src.tr.position().x;
                *y = src.tr.position().y;
                *z = src.tr.position().z;
            });
    }

    #[cfg_attr(feature = "inplace", visibility::make(pub))]
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

    fn drive_df(&self) -> DataFrame {
        let n = self.drive_rows();
        let mut dev_indices = vec![0; n];
        let mut tr_indices = vec![0; n];
        let mut x = vec![0.0; n];
        let mut y = vec![0.0; n];
        let mut z = vec![0.0; n];
        self.dev_indices_inplace(&mut dev_indices);
        self.tr_indices_inplace(&mut tr_indices);
        self.tr_positions_inplace(&mut x, &mut y, &mut z);
        df!(
            "dev_idx" => &dev_indices,
            "tr_idx" => &tr_indices,
            "x[mm]" => &x,
            "y[mm]" => &y,
            "z[mm]" => &z,
        )
        .unwrap()
    }

    pub fn phase(&self) -> DataFrame {
        let mut df = self.drive_df();
        let mut time = vec![0; self.drive_cols()];
        let mut phase = vec![vec![0; self.drive_rows()]; self.drive_cols()];
        self.phase_inplace(&mut time, phase.iter_mut().map(|v| v.as_mut_ptr()));
        let colmuns = time
            .iter()
            .zip(phase.iter())
            .map(|(t, p)| Column::new(format!("phase@{}[ns]", t).into(), &p))
            .collect::<Vec<_>>();
        df.hstack_mut(&colmuns).unwrap();
        df
    }

    pub fn pulse_width(&self) -> DataFrame {
        let mut df = self.drive_df();
        let mut time = vec![0; self.drive_cols()];
        let mut pulse_width = vec![vec![0; self.drive_rows()]; self.drive_cols()];
        self.pulse_width_inplace(&mut time, pulse_width.iter_mut().map(|v| v.as_mut_ptr()));
        let colmuns = time
            .iter()
            .zip(pulse_width.iter())
            .map(|(t, p)| Column::new(format!("pulse_width@{}[ns]", t).into(), &p))
            .collect::<Vec<_>>();
        df.hstack_mut(&colmuns).unwrap();
        df
    }
}
