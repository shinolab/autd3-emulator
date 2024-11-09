use autd3::prelude::*;
use autd3_emulator::*;

use std::time::Duration;

#[rstest::rstest]
#[case(false)]
#[cfg_attr(feature = "gpu", case(true))]
#[tokio::test]
async fn record_sound_field(
    #[allow(unused_variables)]
    #[case]
    gpu: bool,
) -> anyhow::Result<()> {
    let emulator =
        Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
            .into_emulator();

    let record = emulator
        .record(|mut autd| async {
            autd.send(Silencer::disable()).await?;
            autd.send(Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(100 * ULTRASOUND_PERIOD)?;
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
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                #[cfg(feature = "gpu")]
                gpu,
                ..Default::default()
            },
        )
        .await?;

    let df = sound_field.next(100 * ULTRASOUND_PERIOD).await?;

    assert_eq!(
        vec![-100.0, 0.0, 100.0, -100.0, 0.0, 100.0, -100.0, 0.0, 100.0],
        df["x[mm]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    );
    assert_eq!(
        vec![-100.0, -100.0, -100.0, 0.0, 0.0, 0.0, 100.0, 100.0, 100.0],
        df["y[mm]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    );
    assert_eq!(
        vec![300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0],
        df["z[mm]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    );
    // TODO: check the value

    assert!(record
        .sound_field(
            Range {
                x: point.x - 1.0..=point.x + 1.0,
                y: point.y - 1.0..=point.y + 1.0,
                z: point.z..=point.z,
                resolution: 1.,
            },
            InstantRecordOption {
                time_step: Duration::from_micros(2),
                #[cfg(feature = "gpu")]
                gpu,
                ..Default::default()
            },
        )
        .await
        .is_err());
    assert!(record
        .sound_field(
            Range {
                x: point.x - 1.0..=point.x + 1.0,
                y: point.y - 1.0..=point.y + 1.0,
                z: point.z..=point.z,
                resolution: 1.,
            },
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                #[cfg(feature = "gpu")]
                gpu,
                ..Default::default()
            },
        )
        .await?
        .next(Duration::from_micros(1))
        .await
        .is_err());

    Ok(())
}

#[rstest::rstest]
#[case(false, 20)]
#[cfg_attr(feature = "gpu", case(true, 10))]
#[tokio::test]
async fn record_sound_field_resume(
    #[allow(unused_variables)]
    #[case]
    gpu: bool,
    #[case] memory_limits_hint_mb: usize,
) -> anyhow::Result<()> {
    let emulator = Controller::builder([AUTD3::new(Vector3::zeros())]).into_emulator();

    let record = emulator
        .record(|mut autd| async {
            autd.send(Silencer::disable()).await?;
            autd.send(Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(10 * ULTRASOUND_PERIOD)?;
            Ok(autd)
        })
        .await?;

    let point = Vector3::new(0., 0., 1. * mm);
    let range = Range {
        x: point.x - 50.0..=point.x + 50.0,
        y: point.y - 50.0..=point.y + 50.0,
        z: point.z..=point.z,
        resolution: 1.,
    };
    let expect = record
        .sound_field(
            range.clone(),
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                #[cfg(feature = "gpu")]
                gpu,
                ..Default::default()
            },
        )
        .await?
        .next(10 * ULTRASOUND_PERIOD)
        .await?;

    let mut sound_field = record
        .sound_field(
            range,
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                memory_limits_hint_mb,
                #[cfg(feature = "gpu")]
                gpu,
                ..Default::default()
            },
        )
        .await?;
    let mut v1 = sound_field.next(5 * ULTRASOUND_PERIOD).await?;
    let v2 = sound_field.next(5 * ULTRASOUND_PERIOD).await?;
    let columns = v2.get_columns();
    v1.hstack_mut(&columns[3..]).unwrap();

    assert_eq!(expect, v1);

    Ok(())
}

#[tokio::test]
async fn record_sound_field_skip() -> anyhow::Result<()> {
    let emulator = Controller::builder([AUTD3::new(Vector3::zeros())]).into_emulator();

    let record = emulator
        .record(|mut autd| async {
            autd.send(Silencer::disable()).await?;
            autd.send(Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(10 * ULTRASOUND_PERIOD)?;
            Ok(autd)
        })
        .await?;

    let point = Vector3::new(0., 0., 1. * mm);
    let expect = {
        let mut sf = record
            .sound_field(
                Range {
                    x: point.x - 9.0..=point.x + 9.0,
                    y: point.y - 50.0..=point.y + 50.0,
                    z: point.z..=point.z,
                    resolution: 1.,
                },
                InstantRecordOption {
                    time_step: Duration::from_micros(1),
                    ..Default::default()
                },
            )
            .await?;
        sf.next(5 * ULTRASOUND_PERIOD).await?;
        sf.next(5 * ULTRASOUND_PERIOD).await?
    };

    let mut sound_field = record
        .sound_field(
            Range {
                x: point.x - 9.0..=point.x + 9.0,
                y: point.y - 50.0..=point.y + 50.0,
                z: point.z..=point.z,
                resolution: 1.,
            },
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                memory_limits_hint_mb: 1,
                ..Default::default()
            },
        )
        .await?;
    let v = sound_field
        .skip(5 * ULTRASOUND_PERIOD)
        .await?
        .next(5 * ULTRASOUND_PERIOD)
        .await?;

    assert_eq!(expect, v);

    Ok(())
}

#[rstest::rstest]
#[case(false, 0)]
#[case(false, 1)]
#[case(false, usize::MAX)]
#[cfg_attr(feature = "gpu", case(true, 0))]
#[cfg_attr(feature = "gpu", case(true, 1))]
#[tokio::test]
async fn record_sound_field_with_limit(
    #[allow(unused_variables)]
    #[case]
    gpu: bool,
    #[case] memory_limits_hint_mb: usize,
) -> anyhow::Result<()> {
    let emulator =
        Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
            .into_emulator();

    let record = emulator
        .record(|mut autd| async {
            autd.send(Silencer::disable()).await?;
            autd.send(Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(100 * ULTRASOUND_PERIOD)?;
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
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                memory_limits_hint_mb,
                #[cfg(feature = "gpu")]
                gpu,
                ..Default::default()
            },
        )
        .await?;

    let df = sound_field.next(100 * ULTRASOUND_PERIOD).await?;

    assert_eq!(
        vec![-100.0, 0.0, 100.0, -100.0, 0.0, 100.0, -100.0, 0.0, 100.0],
        df["x[mm]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    );
    assert_eq!(
        vec![-100.0, -100.0, -100.0, 0.0, 0.0, 0.0, 100.0, 100.0, 100.0],
        df["y[mm]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    );
    assert_eq!(
        vec![300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0],
        df["z[mm]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    );
    // TODO: check the value

    Ok(())
}

#[cfg(feature = "gpu")]
#[tokio::test]
async fn record_sound_field_gpu_eq_cpu() -> anyhow::Result<()> {
    let emulator = Controller::builder([AUTD3::new(Vector3::zeros())]).into_emulator();

    let record = emulator
        .record(|mut autd| async {
            autd.send(Silencer::disable()).await?;
            autd.send(Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(10 * ULTRASOUND_PERIOD)?;
            Ok(autd)
        })
        .await?;

    let point = Vector3::new(0., 0., 1. * mm);
    let cpu = record
        .sound_field(
            Range {
                x: point.x - 9.0..=point.x + 9.0,
                y: point.y - 50.0..=point.y + 50.0,
                z: point.z..=point.z,
                resolution: 1.,
            },
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                ..Default::default()
            },
        )
        .await?
        .next(10 * ULTRASOUND_PERIOD)
        .await?;

    let mut sound_field = record
        .sound_field(
            Range {
                x: point.x - 9.0..=point.x + 9.0,
                y: point.y - 50.0..=point.y + 50.0,
                z: point.z..=point.z,
                resolution: 1.,
            },
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                gpu: true,
                ..Default::default()
            },
        )
        .await?;
    let gpu = sound_field.next(10 * ULTRASOUND_PERIOD).await?;

    assert_eq!(cpu.shape(), gpu.shape());
    cpu.get_columns()
        .iter()
        .zip(gpu.get_columns())
        .try_for_each(|(cpu, gpu)| -> anyhow::Result<()> {
            cpu.f32()?
                .into_no_null_iter()
                .zip(gpu.f32()?.into_no_null_iter())
                .for_each(|(cpu, gpu)| {
                    approx::assert_abs_diff_eq!(cpu, gpu, epsilon = 0.1);
                });
            Ok(())
        })?;

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
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                ..Default::default()
            },
        )
        .await?;

    assert!(sound_field.next(2 * ULTRASOUND_PERIOD).await.is_err());

    Ok(())
}
