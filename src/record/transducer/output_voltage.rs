use autd3::driver::common::ULTRASOUND_FREQ;

use crate::record::ULTRASOUND_PERIOD_COUNT;

use super::TransducerRecord;

impl TransducerRecord {
    pub(crate) const TS: f32 = 1. / (ULTRASOUND_FREQ.hz() as f32 * ULTRASOUND_PERIOD_COUNT as f32);
    pub(crate) const V: f32 = 12.0;

    pub(crate) fn _output_voltage_within_inplace(&self, start: usize, n: usize, v: &mut [f32]) {
        const T: u16 = ULTRASOUND_PERIOD_COUNT as u16;
        self.pulse_width[start..]
            .iter()
            .zip(self.phase[start..].iter())
            .take(n)
            .flat_map(|(pw, phase)| {
                let rise = ((T + (*phase as u16 * 2)) - pw / 2) % T;
                let fall = (*phase as u16 * 2 + pw / 2 + (pw & 0x01)) % T;
                (0..T).map(move |i| {
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
            .zip(v.iter_mut())
            .for_each(|(src, dst)| *dst = src);
    }

    pub(crate) fn _output_voltage_within(&self, start: usize, n: usize) -> Option<Vec<f32>> {
        if start + n > self.pulse_width.len() {
            return None;
        }
        let mut v = vec![0.0; n * ULTRASOUND_PERIOD_COUNT];
        self._output_voltage_within_inplace(start, n, &mut v);
        Some(v)
    }
}

#[cfg(test)]
mod tests {
    use autd3::prelude::Point3;

    use super::*;

    #[test]
    fn test_output_voltage_within() {
        let record = TransducerRecord {
            pulse_width: vec![],
            phase: vec![],
            tr: autd3::driver::geometry::Transducer::new(Point3::origin()),
        };
        assert!(record._output_voltage_within(0, 1).is_none());
    }
}
