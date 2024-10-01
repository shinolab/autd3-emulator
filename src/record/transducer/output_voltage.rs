use autd3::driver::defined::{ULTRASOUND_FREQ, ULTRASOUND_PERIOD, ULTRASOUND_PERIOD_COUNT};

use super::TransducerRecord;

impl TransducerRecord {
    pub(crate) const TS: f32 = 1. / (ULTRASOUND_FREQ.hz() as f32 * ULTRASOUND_PERIOD_COUNT as f32);
    pub(crate) const V: f32 = 12.0;

    pub(crate) fn output_times_inplace(&self, start: usize, n: usize, time: &mut [f32]) {
        (start..)
            .take(n)
            .map(|i| i as u32 * ULTRASOUND_PERIOD)
            .flat_map(|t| {
                (0..ULTRASOUND_PERIOD_COUNT).map(move |i| t.as_secs_f32() + i as f32 * Self::TS)
            })
            .zip(time.iter_mut())
            .for_each(|(src, dst)| *dst = src);
    }

    pub(crate) fn output_times(&self, start: usize, n: usize) -> Vec<f32> {
        let mut time = vec![0.; n * ULTRASOUND_PERIOD_COUNT];
        self.output_times_inplace(start, n, &mut time);
        time
    }

    pub(crate) fn _output_voltage_within_inplace(&self, start: usize, n: usize, v: &mut [f32]) {
        self.pulse_width
            .iter()
            .zip(self.phase.iter())
            .skip(start)
            .take(n)
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
