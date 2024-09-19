use autd3_driver::geometry::Vector3;
use polars::prelude::*;

use crate::recording::{Range, RecordOption};

use super::TransducerRecord;

use derive_more::Deref;

#[derive(Deref, Debug)]
pub struct DeviceRecord<'a> {
    #[deref]
    pub(crate) records: Vec<TransducerRecord<'a>>,
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

    // pub fn output_voltage(&self) -> DataFrame {
    //     let mut df = self[0].output_voltage();
    //     df.rename("voltage[V]", "voltage_0[V]".into()).unwrap();
    //     self.iter().enumerate().skip(1).for_each(|(i, tr)| {
    //         let mut d = tr.output_voltage();
    //         d.rename("voltage[V]", format!("voltage_{}[V]", i).into())
    //             .unwrap();
    //         let mut d = d.take_columns();
    //         let voltage = d.pop().unwrap();
    //         df.hstack_mut(&[voltage]).unwrap();
    //     });
    //     df
    // }

    // pub fn output_ultrasound(&self) -> DataFrame {
    //     let mut df = self[0].output_ultrasound();
    //     df.rename("p[a.u.]", "p_0[a.u.]".into()).unwrap();
    //     self.iter().enumerate().skip(1).for_each(|(i, tr)| {
    //         let mut d = tr.output_ultrasound();
    //         d.rename("p[a.u.]", format!("p_{}[a.u.]", i).into())
    //             .unwrap();
    //         let mut d = d.take_columns();
    //         let v = d.pop().unwrap();
    //         df.hstack_mut(&[v]).unwrap();
    //     });
    //     df
    // }

    // pub fn sound_field_at(&self, point: &Vector3, option: RecordOption) -> DataFrame {
    //     let times = option
    //         .time
    //         .as_ref()
    //         .map(|t| t.times().collect())
    //         .unwrap_or(self[0].output_times());

    //     let pb = option.pb(times.len());

    //     let p = times
    //         .iter()
    //         .map(|&t| {
    //             let p = self.iter().skip(1).fold(
    //                 {
    //                     let tp = self[0].tr.position();
    //                     let dist = (point - tp).norm();
    //                     let output_ultrasound = self[0]._output_ultrasound();
    //                     self[0]._sound_field_at(
    //                         dist,
    //                         t,
    //                         option.sound_speed,
    //                         output_ultrasound.as_slice(),
    //                     )
    //                 },
    //                 |acc, tr| {
    //                     let tp = tr.tr.position();
    //                     let dist = (point - tp).norm();
    //                     let output_ultrasound = tr._output_ultrasound();
    //                     acc + tr._sound_field_at(
    //                         dist,
    //                         t,
    //                         option.sound_speed,
    //                         output_ultrasound.as_slice(),
    //                     )
    //                 },
    //             );
    //             pb.inc(1);
    //             p
    //         })
    //         .collect::<Vec<_>>();
    //     df!(
    //         "time[s]" => &times,
    //         &format!("p[Pa]@({},{},{})", point.x,point. y, point.z) => &p
    //     )
    //     .unwrap()
    // }

    // pub fn sound_field(&self, range: Range, option: RecordOption) -> DataFrame {
    //     let (x, y, z) = range.points();
    //     let mut df = df!(
    //             "x[mm]" => &x,
    //             "y[mm]" => &y,
    //             "z[mm]" => &z)
    //     .unwrap();

    //     let dists = self
    //         .iter()
    //         .map(|tr| {
    //             itertools::izip!(&x, &y, &z)
    //                 .map(|(&x, &y, &z)| (Vector3::new(x, y, z) - tr.tr.position()).norm())
    //                 .collect::<Vec<_>>()
    //         })
    //         .collect::<Vec<_>>();

    //     let times = option
    //         .time
    //         .as_ref()
    //         .map(|t| t.times().collect())
    //         .unwrap_or(self[0].output_times());

    //     let pb = option.pb(times.len());
    //     times.into_iter().for_each(|t| {
    //         let p = self.iter().skip(1).fold(
    //             self[0]._sound_field(&dists[0], t, option.sound_speed),
    //             |acc, tr| {
    //                 let p = tr._sound_field(&dists[tr.tr.idx()], t, option.sound_speed);
    //                 acc.into_iter()
    //                     .zip(p.into_iter())
    //                     .map(|(a, b)| a + b)
    //                     .collect()
    //             },
    //         );
    //         df.hstack_mut(&[Series::new(format!("p[Pa]@{}", t).into(), &p)])
    //             .unwrap();
    //         pb.inc(1);
    //     });
    //     df
    // }
}
