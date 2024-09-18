use std::time::Duration;

pub struct TimeRange {
    pub duration: std::ops::RangeInclusive<Duration>,
    pub time_step_s: f32,
}

impl TimeRange {
    fn n(&self) -> usize {
        ((self.duration.end().as_secs_f32() - self.duration.start().as_secs_f32())
            / self.time_step_s)
            .floor() as usize
            + 1
    }

    pub fn times(&self) -> impl Iterator<Item = f32> {
        let start = self.duration.start().as_secs_f32();
        let step = self.time_step_s;
        (0..self.n()).map(move |i| start + step * i as f32)
    }
}

#[cfg(test)]
mod tests {
    use autd3::prelude::ULTRASOUND_PERIOD;

    use super::*;

    #[rstest::rstest]
    #[test]
    #[case(vec![0.], Duration::ZERO..=Duration::ZERO, ULTRASOUND_PERIOD.as_secs_f32())]
    #[case(vec![0., ULTRASOUND_PERIOD.as_secs_f32(), (2 * ULTRASOUND_PERIOD).as_secs_f32()], Duration::ZERO..=2 * ULTRASOUND_PERIOD, ULTRASOUND_PERIOD.as_secs_f32())]
    fn test_times(
        #[case] expect: Vec<f32>,
        #[case] duration: std::ops::RangeInclusive<Duration>,
        #[case] time_step_s: f32,
    ) {
        assert_eq!(
            expect,
            TimeRange {
                duration,
                time_step_s,
            }
            .times()
            .collect::<Vec<_>>()
        );
    }
}
