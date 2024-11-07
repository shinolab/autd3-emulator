mod drive;
mod output_ultrasound;
mod output_voltage;
mod rms;
mod sound_field;

use autd3::prelude::*;
use autd3_emulator::*;

#[tokio::test]
async fn invalid_tick() -> anyhow::Result<()> {
    let emulator = Controller::builder([AUTD3::new(Vector3::zeros())]).into_emulator();

    let record = emulator
        .record(|mut autd| async {
            autd.send(Silencer::disable()).await?;
            autd.tick(ULTRASOUND_PERIOD / 2)?;
            Ok(autd)
        })
        .await;

    assert!(record.is_err());

    Ok(())
}
