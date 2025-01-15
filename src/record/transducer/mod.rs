pub(crate) mod output_ultrasound;
mod output_voltage;

use autd3::driver::defined::ultrasound_period;

use derive_more::Debug;

#[derive(Debug)]
pub(crate) struct TransducerRecord {
    pub(crate) pulse_width: Vec<u8>,
    pub(crate) phase: Vec<u8>,
    #[debug(skip)]
    pub(crate) tr: autd3::driver::geometry::Transducer,
}

impl TransducerRecord {
    pub(crate) fn time(idx: usize) -> u64 {
        (idx as u32 * ultrasound_period()).as_nanos() as _
    }
}
