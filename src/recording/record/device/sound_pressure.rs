use std::time::Duration;

use autd3_driver::geometry::Vector3;
use polars::prelude::{df, DataFrame};

use crate::{
    error::EmulatorError,
    recording::{RecordOption, TransducerRecord},
};

use super::DeviceRecord;

impl<'a> DeviceRecord<'a> {
    pub fn sound_pressure(
        &self,
        point: &Vector3,
        time: std::ops::Range<Duration>,
        option: RecordOption,
    ) -> Result<DataFrame, EmulatorError> {
        let duration = time.end.saturating_sub(time.start);
        let time = TransducerRecord::_time(time, option.time_step);

        let pb = option.pb(self.len());
        let p = self.iter().skip(1).fold(
            self[0]._sound_pressure(point, &time, duration, option.sound_speed)?,
            |acc, tr| {
                pb.inc(1);
                let p = tr
                    ._sound_pressure(point, &time, duration, option.sound_speed)
                    .unwrap();
                acc.into_iter().zip(p).map(|(a, b)| a + b).collect()
            },
        );
        pb.inc(1);
        Ok(df!(
            "time[s]" => &time,
            &format!("p[Pa]@({},{},{})", point.x,point. y, point.z) => &p
        )
        .unwrap())
    }
}
