use autd3::{
    driver::{defined::ULTRASOUND_PERIOD, error::AUTDInternalError},
    error::AUTDError,
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
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
}
