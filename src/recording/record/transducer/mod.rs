// mod output_ultrasound;
mod output_voltage;
// mod sound_field;

use std::cell::RefCell;

use polars::prelude::*;

use autd3_driver::{defined::ULTRASOUND_PERIOD, derive::Builder};

use derive_more::Debug;

#[derive(Builder, Debug)]
pub struct TransducerRecord<'a> {
    pub(crate) pulse_width: Vec<u8>,
    pub(crate) phase: Vec<u8>,
    #[debug(skip)]
    pub(crate) output_ultrasound_cache: RefCell<Vec<f32>>,
    #[debug(skip)]
    pub(crate) tr: &'a autd3_driver::geometry::Transducer,
}

impl<'a> TransducerRecord<'a> {
    pub(crate) fn time(start: usize, n: usize) -> Series {
        (start..)
            .take(n)
            .map(|i| (i as u32 * ULTRASOUND_PERIOD).as_secs_f32())
            .collect()
    }

    pub fn drive(&self) -> DataFrame {
        let time = Self::time(0, self.pulse_width.len());
        df!(
            "time[s]" => &time,
            "phase" => &self.phase,
            "pulsewidth" => &self.pulse_width
        )
        .unwrap()
    }
}
