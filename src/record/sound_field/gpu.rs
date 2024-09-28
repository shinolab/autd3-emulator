use std::time::Duration;

use indicatif::ProgressBar;

use crate::EmulatorError;

#[derive(Debug)]
pub(crate) struct Gpu {}

impl Gpu {
    pub(crate) fn init(&mut self, _cache_size: isize, _cursor: &mut isize, _rem_frame: &mut usize) {
        todo!()
    }

    pub(crate) fn progress(&mut self, _cursor: &mut isize) -> Result<(), EmulatorError> {
        todo!()
    }

    pub(crate) fn compute(
        &mut self,
        _start_time: f32,
        _time_step: Duration,
        _num_points_in_frame: usize,
        _sound_speed: f32,
        _offset: isize,
        _pb: &ProgressBar,
    ) -> &Vec<Vec<f32>> {
        todo!()
    }
}
