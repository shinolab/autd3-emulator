pub(crate) mod output_ultrasound;
mod output_voltage;

use autd3::driver::{defined::ULTRASOUND_PERIOD, derive::Builder};

use derive_more::Debug;

#[derive(Builder, Debug)]
pub(crate) struct TransducerRecord {
    pub(crate) pulse_width: Vec<u8>,
    pub(crate) phase: Vec<u8>,
    #[debug(skip)]
    pub(crate) tr: autd3::driver::geometry::Transducer,
}

impl TransducerRecord {
    pub(crate) fn time_inplace(start: usize, n: usize, time: &mut [f32]) {
        (start..)
            .take(n)
            .zip(time.iter_mut())
            .for_each(|(i, dst)| *dst = (i as u32 * ULTRASOUND_PERIOD).as_secs_f32());
    }

    pub(crate) fn time(start: usize, n: usize) -> Vec<f32> {
        let mut time = vec![0.; n];
        Self::time_inplace(start, n, &mut time);
        time
    }
}
