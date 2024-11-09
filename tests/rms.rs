use std::time::Duration;

use autd3::prelude::*;
use autd3_emulator::*;

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
    let mut rms = record
        .sound_field(
            Range {
                x: point.x - 100.0..=point.x + 100.0,
                y: point.y - 100.0..=point.y + 100.0,
                z: point.z..=point.z,
                resolution: 100.,
            },
            RmsRecordOption {
                #[cfg(feature = "gpu")]
                gpu,
                ..Default::default()
            },
        )
        .await?;

    let df = rms.next(100 * ULTRASOUND_PERIOD).await?;

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
            RmsRecordOption {
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
#[case(false)]
#[cfg_attr(feature = "gpu", case(true))]
#[tokio::test]
async fn record_rms_resume(
    #[allow(unused_variables)]
    #[case]
    gpu: bool,
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
            RmsRecordOption {
                #[cfg(feature = "gpu")]
                gpu,
                ..Default::default()
            },
        )
        .await?
        .next(10 * ULTRASOUND_PERIOD)
        .await?;

    let mut rms = record
        .sound_field(
            range,
            RmsRecordOption {
                #[cfg(feature = "gpu")]
                gpu,
                ..Default::default()
            },
        )
        .await?;
    let mut v1 = rms.next(5 * ULTRASOUND_PERIOD).await?;
    let v2 = rms.next(5 * ULTRASOUND_PERIOD).await?;
    let columns = v2.get_columns();
    v1.hstack_mut(&columns[3..]).unwrap();

    assert_eq!(expect, v1);

    Ok(())
}

#[tokio::test]
async fn record_rms_skip() -> anyhow::Result<()> {
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
                RmsRecordOption::default(),
            )
            .await?;
        sf.next(5 * ULTRASOUND_PERIOD).await?;
        sf.next(5 * ULTRASOUND_PERIOD).await?
    };

    let mut rms = record
        .sound_field(
            Range {
                x: point.x - 9.0..=point.x + 9.0,
                y: point.y - 50.0..=point.y + 50.0,
                z: point.z..=point.z,
                resolution: 1.,
            },
            RmsRecordOption::default(),
        )
        .await?;
    let v = rms
        .skip(5 * ULTRASOUND_PERIOD)
        .await?
        .next(5 * ULTRASOUND_PERIOD)
        .await?;

    assert_eq!(expect, v);

    Ok(())
}

#[cfg(feature = "gpu")]
#[tokio::test]
async fn record_rms_gpu_eq_cpu() -> anyhow::Result<()> {
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
            RmsRecordOption::default(),
        )
        .await?
        .next(10 * ULTRASOUND_PERIOD)
        .await?;

    let mut rms = record
        .sound_field(
            Range {
                x: point.x - 9.0..=point.x + 9.0,
                y: point.y - 50.0..=point.y + 50.0,
                z: point.z..=point.z,
                resolution: 1.,
            },
            RmsRecordOption {
                gpu: true,
                ..Default::default()
            },
        )
        .await?;
    let gpu = rms.next(10 * ULTRASOUND_PERIOD).await?;

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
    let mut rms = record
        .sound_field(
            Range {
                x: point.x - 100.0..=point.x + 100.0,
                y: point.y - 100.0..=point.y + 100.0,
                z: point.z..=point.z,
                resolution: 100.,
            },
            RmsRecordOption::default(),
        )
        .await?;

    assert!(rms.next(2 * ULTRASOUND_PERIOD).await.is_err());

    Ok(())
}
