mod device;
mod transducer;

pub use device::DeviceRecord;
pub use transducer::TransducerRecord;

use autd3_driver::{derive::Builder, ethercat::DcSysTime};

use derive_more::{Debug, Deref};

#[derive(Deref, Builder, Debug)]
pub struct Record<'a> {
    #[deref]
    pub(crate) records: Vec<DeviceRecord<'a>>,
    #[get]
    pub(crate) start: DcSysTime,
    #[get]
    pub(crate) end: DcSysTime,
}
