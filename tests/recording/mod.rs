mod device;
mod transducer;

use std::time::Duration;

use autd3::{prelude::*, Controller};
use autd3_link_calc::{error::CalcError, Calc};

#[tokio::test]
async fn recording_not_started() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    autd.send(Silencer::disable()).await?;
    assert_eq!(
        Err(CalcError::RecodingNotStarted),
        autd.tick(ULTRASOUND_PERIOD)
    );
    autd.start_recording()?;
    let _ = autd.finish_recording()?;
    assert_eq!(
        CalcError::RecodingNotStarted,
        autd.finish_recording().unwrap_err()
    );

    autd.close().await?;

    Ok(())
}

#[tokio::test]
async fn recording_invalid_tick() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    autd.send(Silencer::disable()).await?;
    autd.start_recording()?;
    autd.tick(ULTRASOUND_PERIOD)?;
    assert_eq!(Err(CalcError::InvalidTick), autd.tick(Duration::ZERO));
    assert_eq!(
        Err(CalcError::InvalidTick),
        autd.tick(ULTRASOUND_PERIOD / 2)
    );
    let _ = autd.finish_recording()?;

    autd.close().await?;

    Ok(())
}

#[tokio::test]
async fn recording_invalid_op() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    autd.send(Silencer::default()).await?;
    autd.start_recording()?;
    autd.tick(ULTRASOUND_PERIOD)?;
    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::from(
            CalcError::InvalidOperationWhenRecording,
        ))),
        autd.send(Silencer::default()).await,
    );
    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::from(
            CalcError::InvalidOperationWhenRecording,
        ))),
        autd.send(PulseWidthEncoder::default()).await,
    );
    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::from(
            CalcError::InvalidOperationWhenRecording,
        ))),
        autd.send(Clear::new()).await,
    );
    assert_eq!(
        Err(AUTDError::Internal(AUTDInternalError::from(
            CalcError::InvalidOperationWhenRecording,
        ))),
        autd.send((Null::new(), Clear::new())).await,
    );
    let _ = autd.finish_recording()?;

    autd.close().await?;

    Ok(())
}
