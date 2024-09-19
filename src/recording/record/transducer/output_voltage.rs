use std::time::Duration;

use autd3_driver::defined::{ULTRASOUND_FREQ, ULTRASOUND_PERIOD, ULTRASOUND_PERIOD_COUNT};
use polars::{df, frame::DataFrame};

use crate::error::EmulatorError;

use super::TransducerRecord;

impl<'a> TransducerRecord<'a> {
    pub(crate) const TS: f32 = 1. / (ULTRASOUND_FREQ.hz() as f32 * ULTRASOUND_PERIOD_COUNT as f32);
    pub(crate) const V: f32 = 12.0;

    #[inline(always)]
    pub(crate) fn output_times(&self, start: usize, n: usize) -> Vec<f32> {
        (start..)
            .take(n)
            .map(|i| i as u32 * ULTRASOUND_PERIOD)
            .flat_map(|t| (0..=255u8).map(move |i| t.as_secs_f32() + i as f32 * Self::TS))
            .collect::<Vec<_>>()
    }

    #[inline(always)]
    fn extend(buf: &[u8], start: usize, n: usize) -> impl Iterator<Item = u8> + '_ {
        let last = *buf.last().unwrap();
        buf.iter()
            .skip(start)
            .copied()
            .chain(std::iter::repeat(last))
            .take(n)
    }

    #[inline(always)]
    pub(crate) fn _output_voltage_within(&self, start: usize, n: usize) -> Vec<f32> {
        Self::extend(&self.pulse_width, start, n)
            .zip(Self::extend(&self.phase, start, n))
            .flat_map(|(pw, phase)| {
                let rise = phase.wrapping_sub(pw / 2);
                let fall = phase.wrapping_add(pw / 2 + (pw & 0x01));
                (0..=255u8).map(move |i| {
                    #[allow(clippy::collapsible_else_if)]
                    if rise <= fall {
                        if (rise <= i) && (i < fall) {
                            Self::V
                        } else {
                            -Self::V
                        }
                    } else {
                        if (i < fall) || (rise <= i) {
                            Self::V
                        } else {
                            -Self::V
                        }
                    }
                })
            })
            .collect::<Vec<_>>()
    }

    pub fn output_voltage_within(
        &self,
        start: Duration,
        end: Duration,
    ) -> Result<DataFrame, EmulatorError> {
        if start.as_nanos() % ULTRASOUND_PERIOD.as_nanos() != 0
            || end.as_nanos() % ULTRASOUND_PERIOD.as_nanos() != 0
        {
            return Err(EmulatorError::InvalidDuration);
        }
        let n = ((end - start).as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as usize;
        let start = (start.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as usize;
        let time = self.output_times(start, n);
        let output_voltage = self._output_voltage_within(start, n);
        Ok(df!(
            "time[s]" => &time,
            "voltage[V]" => &output_voltage
        )
        .unwrap())
    }

    pub fn output_voltage(&self) -> DataFrame {
        self.output_voltage_within(
            Duration::ZERO,
            self.pulse_width.len() as u32 * ULTRASOUND_PERIOD,
        )
        .unwrap()
    }
}
