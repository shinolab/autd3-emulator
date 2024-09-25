pub(crate) mod output_ultrasound;
mod output_voltage;

use polars::prelude::*;

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
    pub(crate) fn time(start: usize, n: usize) -> Series {
        (start..)
            .take(n)
            .map(|i| (i as u32 * ULTRASOUND_PERIOD).as_secs_f32())
            .collect()
    }
}
