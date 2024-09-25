use autd3::driver::defined::{ULTRASOUND_FREQ, ULTRASOUND_PERIOD, ULTRASOUND_PERIOD_COUNT};

use super::TransducerRecord;

impl TransducerRecord {
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
    pub(crate) fn _output_voltage_within(&self, start: usize, n: usize) -> Option<Vec<f32>> {
        if start + n > self.pulse_width.len() {
            return None;
        }
        Some(
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
                .collect::<Vec<_>>(),
        )
    }
}
