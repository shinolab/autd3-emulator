pub(crate) mod output_ultrasound;
mod output_voltage;

use autd3::driver::common::ULTRASOUND_PERIOD;

use derive_more::Debug;

#[derive(Debug)]
pub(crate) struct TransducerRecord {
    pub(crate) pulse_width: Vec<u16>,
    pub(crate) phase: Vec<u8>,
    #[debug(skip)]
    pub(crate) tr: autd3::driver::geometry::Transducer,
}

impl TransducerRecord {
    pub(crate) fn time(idx: usize) -> u64 {
        (idx as u32 * ULTRASOUND_PERIOD).as_nanos() as _
    }
}
