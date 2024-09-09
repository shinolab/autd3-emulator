use autd3_driver::{defined::ULTRASOUND_PERIOD, error::AUTDInternalError};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CalcError {
    #[error("Recording is already started")]
    RecordingAlreadyStarted,
    #[error("Recording is not started")]
    RecodingNotStarted,
    #[error("Tick must be multiple of {:?}", ULTRASOUND_PERIOD)]
    InvalidTick,
    #[error("Invalid operation when recording")]
    InvalidOperationWhenRecording,
}

impl From<CalcError> for AUTDInternalError {
    fn from(value: CalcError) -> Self {
        AUTDInternalError::LinkError(value.to_string())
    }
}
