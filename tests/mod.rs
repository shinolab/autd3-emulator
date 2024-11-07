mod drive;
mod output_ultrasound;
mod output_voltage;
mod rms;
mod sound_field;

use autd3::prelude::*;
use autd3_emulator::*;

use std::time::Duration;

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

#[tokio::test]
async fn not_recorded() -> anyhow::Result<()> {
    let emulator = Controller::builder([AUTD3::new(Vector3::zeros())]).into_emulator();

    let record = emulator
        .record(|mut autd| async {
            autd.send(Silencer::disable()).await?;
            autd.send(Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(ULTRASOUND_PERIOD)?;
            Ok(autd)
        })
        .await?;

    let point = Vector3::new(0., 0., 300. * mm);
    let mut sound_field = record
        .sound_field(
            Range {
                x: point.x - 100.0..=point.x + 100.0,
                y: point.y - 100.0..=point.y + 100.0,
                z: point.z..=point.z,
                resolution: 100.,
            },
            RecordOption {
                time_step: Duration::from_micros(1),
                ..Default::default()
            },
        )
        .await?;

    assert!(sound_field.next(2 * ULTRASOUND_PERIOD).await.is_err());

    Ok(())
}
