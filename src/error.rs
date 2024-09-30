use autd3::{
    driver::{defined::ULTRASOUND_PERIOD, error::AUTDInternalError},
    error::AUTDError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmulatorError {
    #[error("Tick must be multiple of {:?}", ULTRASOUND_PERIOD)]
    InvalidTick,
    #[error("Duration must be multiple of {:?}", ULTRASOUND_PERIOD)]
    InvalidDuration,
    #[error("Time step must divide {:?}", ULTRASOUND_PERIOD)]
    InvalidTimeStep,
    #[error("Not recorded")]
    NotRecorded,
    #[error("{0}")]
    Internal(#[from] AUTDInternalError),
    #[error("{0}")]
    AUTD(#[from] AUTDError),
    #[cfg(feature = "gpu")]
    #[error("No suitable adapter found")]
    NoSuitableAdapterFound,
    #[cfg(feature = "gpu")]
    #[error("{0}")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
    #[cfg(feature = "gpu")]
    #[error("{0}")]
    RecvError(#[from] flume::RecvError),
    #[cfg(feature = "gpu")]
    #[error("{0}")]
    BufferAsyncError(#[from] wgpu::BufferAsyncError),
}
