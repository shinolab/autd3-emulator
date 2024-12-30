use autd3::{
    driver::{defined::ULTRASOUND_PERIOD, error::AUTDDriverError},
    error::AUTDError,
};
use thiserror::Error;

/// An interface for error handling in autd3-emulator.
#[derive(Error, Debug)]
pub enum EmulatorError {
    /// Error when the tick is not a multiple of the ultrasound period.
    #[error("Tick must be multiple of {:?}", ULTRASOUND_PERIOD)]
    InvalidTick,
    /// Error when the duration is not a multiple of the ultrasound period.
    #[error("Duration must be multiple of {:?}", ULTRASOUND_PERIOD)]
    InvalidDuration,
    /// Error when the time step is not a divisor of the ultrasound period.
    #[error("Time step must divide {:?}", ULTRASOUND_PERIOD)]
    InvalidTimeStep,
    /// Error when requesting data outside the recorded range.
    #[error("Not recorded")]
    NotRecorded,
    #[allow(missing_docs)]
    #[error("{0}")]
    Driver(#[from] AUTDDriverError),
    #[allow(missing_docs)]
    #[error("{0}")]
    AUTD(#[from] AUTDError),
    /// Error when the suitable GPU adapter is not found.
    #[cfg(feature = "gpu")]
    #[error("No suitable adapter found")]
    NoSuitableAdapterFound,
    #[allow(missing_docs)]
    #[cfg(feature = "gpu")]
    #[error("{0}")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
    #[allow(missing_docs)]
    #[cfg(feature = "gpu")]
    #[error("{0}")]
    RecvError(#[from] flume::RecvError),
    #[allow(missing_docs)]
    #[cfg(feature = "gpu")]
    #[error("{0}")]
    BufferAsyncError(#[from] wgpu::BufferAsyncError),
}
