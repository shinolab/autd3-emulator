use autd3::driver::{common::ULTRASOUND_PERIOD, error::AUTDDriverError};
use autd3_core::firmware::SamplingConfigError;

// GRCOV_EXCL_START

/// An interface for error handling in autd3-emulator.
#[derive(Debug)]
pub enum EmulatorError {
    /// Error when the tick is not a multiple of the ultrasound period.
    InvalidTick,
    /// Error when the duration is not a multiple of the ultrasound period.
    InvalidDuration,
    /// Error when the time step is not a divisor of the ultrasound period.
    InvalidTimeStep,
    /// Error when requesting data outside the recorded range.
    NotRecorded,
    #[allow(missing_docs)]
    SamplingConfig(SamplingConfigError),
    #[allow(missing_docs)]
    Driver(AUTDDriverError),
    #[allow(missing_docs)]
    #[cfg(feature = "gpu")]
    RequestDeviceError(wgpu::RequestDeviceError),
    #[allow(missing_docs)]
    #[cfg(feature = "gpu")]
    BufferAsyncError(wgpu::BufferAsyncError),
    #[allow(missing_docs)]
    #[cfg(feature = "gpu")]
    RequestAdapterError(wgpu::RequestAdapterError),
    #[allow(missing_docs)]
    #[cfg(feature = "gpu")]
    PollError(wgpu::PollError),
    #[allow(missing_docs)]
    #[cfg(feature = "polars")]
    Polars(polars::error::PolarsError),
}

impl std::fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmulatorError::InvalidTick => {
                write!(f, "Tick must be multiple of {:?}", ULTRASOUND_PERIOD)
            }
            EmulatorError::InvalidDuration => {
                write!(f, "Duration must be multiple of {:?}", ULTRASOUND_PERIOD)
            }
            EmulatorError::InvalidTimeStep => {
                write!(f, "Time step must divide {:?}", ULTRASOUND_PERIOD)
            }
            EmulatorError::NotRecorded => write!(f, "Not recorded"),
            EmulatorError::SamplingConfig(e) => write!(f, "{}", e),
            EmulatorError::Driver(e) => write!(f, "{}", e),
            #[cfg(feature = "gpu")]
            EmulatorError::RequestDeviceError(e) => write!(f, "{}", e),
            #[cfg(feature = "gpu")]
            EmulatorError::BufferAsyncError(e) => write!(f, "{}", e),
            #[cfg(feature = "gpu")]
            EmulatorError::RequestAdapterError(e) => write!(f, "{}", e),
            #[cfg(feature = "gpu")]
            EmulatorError::PollError(e) => write!(f, "{}", e),
            #[cfg(feature = "polars")]
            EmulatorError::Polars(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for EmulatorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EmulatorError::SamplingConfig(e) => Some(e),
            EmulatorError::Driver(e) => Some(e),
            #[cfg(feature = "gpu")]
            EmulatorError::RequestDeviceError(e) => Some(e),
            #[cfg(feature = "gpu")]
            EmulatorError::BufferAsyncError(e) => Some(e),
            #[cfg(feature = "gpu")]
            EmulatorError::RequestAdapterError(e) => Some(e),
            #[cfg(feature = "gpu")]
            EmulatorError::PollError(e) => Some(e),
            #[cfg(feature = "polars")]
            EmulatorError::Polars(e) => Some(e),
            _ => None,
        }
    }
}

impl From<SamplingConfigError> for EmulatorError {
    fn from(e: SamplingConfigError) -> Self {
        EmulatorError::SamplingConfig(e)
    }
}

impl From<AUTDDriverError> for EmulatorError {
    fn from(e: AUTDDriverError) -> Self {
        EmulatorError::Driver(e)
    }
}

#[cfg(feature = "gpu")]
impl From<wgpu::RequestDeviceError> for EmulatorError {
    fn from(e: wgpu::RequestDeviceError) -> Self {
        EmulatorError::RequestDeviceError(e)
    }
}

#[cfg(feature = "gpu")]
impl From<wgpu::BufferAsyncError> for EmulatorError {
    fn from(e: wgpu::BufferAsyncError) -> Self {
        EmulatorError::BufferAsyncError(e)
    }
}

#[cfg(feature = "gpu")]
impl From<wgpu::RequestAdapterError> for EmulatorError {
    fn from(e: wgpu::RequestAdapterError) -> Self {
        EmulatorError::RequestAdapterError(e)
    }
}

#[cfg(feature = "gpu")]
impl From<wgpu::PollError> for EmulatorError {
    fn from(e: wgpu::PollError) -> Self {
        EmulatorError::PollError(e)
    }
}

#[cfg(feature = "polars")]
impl From<polars::error::PolarsError> for EmulatorError {
    fn from(e: polars::error::PolarsError) -> Self {
        EmulatorError::Polars(e)
    }
}

// GRCOV_EXCL_STOP
