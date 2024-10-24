use autd3::{derive::Datagram, gain, prelude::*};
use autd3_emulator::{Emulator, Range, RecordOption, RecorderControllerExt};

use polars::prelude::{df, NamedFrom, Series};
use std::time::Duration;

#[rstest::rstest]
#[case(Silencer::disable())]
#[case(Silencer::disable().with_target(SilencerTarget::PulseWidth))]
#[tokio::test]
async fn record_drive(#[case] silencer: impl Datagram) -> anyhow::Result<()> {
    let emulator = Emulator::new([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())]);

    let to_pulse_width = |a, b| {
        let i = (a as usize * b as usize) / 255;
        ((((i as f32) / 255.).asin() / PI) * 256.).round() as u8
    };
    let mut expect = df!(
        "time[ns]" => [0u64, 25000, 50000],
    )
    .unwrap();
    let series = emulator
        .geometry()
        .iter()
        .flat_map(|dev| {
            dev.iter().flat_map(|tr| {
                let dev_idx = tr.dev_idx() as u8;
                let tr_idx = tr.idx() as u8;
                let pulse_width = Series::new(
                    format!("pulsewidth_{}_{}", dev_idx, tr_idx).into(),
                    &[
                        to_pulse_width(100, tr_idx),
                        to_pulse_width(200, tr_idx),
                        to_pulse_width(200, tr_idx),
                    ],
                );
                let phase = Series::new(
                    format!("phase_{}_{}", dev_idx, tr_idx).into(),
                    &[dev_idx as u8, (0x01 + dev_idx) as _, (0x01 + dev_idx) as _]
                        .iter()
                        .copied()
                        .collect::<Vec<_>>(),
                );
                [phase, pulse_width]
            })
        })
        .collect::<Vec<_>>();
    expect.hstack_mut(&series).unwrap();

    let record = emulator
        .record(|mut autd| async {
            autd.send(silencer).await?;
            autd.send((
                Static::with_intensity(100),
                gain::Custom::new(|_| {
                    |tr| {
                        (
                            Phase::new(tr.dev_idx() as _),
                            EmitIntensity::new(tr.idx() as _),
                        )
                    }
                }),
            ))
            .await?;
            autd.tick(ULTRASOUND_PERIOD)?;
            autd.send((
                Static::with_intensity(200),
                gain::Custom::new(|_| {
                    |tr| {
                        (
                            Phase::new(0x01) + Phase::new(tr.dev_idx() as _),
                            EmitIntensity::new(tr.idx() as _),
                        )
                    }
                }),
            ))
            .await?;
            autd.tick(2 * ULTRASOUND_PERIOD)?;

            Ok(autd)
        })
        .await?;

    assert_eq!(expect, record.drive());

    Ok(())
}

#[tokio::test]
async fn record_output_voltage() -> anyhow::Result<()> {
    let emulator = Emulator::new([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())]);

    let record = emulator
        .record(|mut autd| async {
            autd.send(Silencer::disable()).await?;
            autd.send(PulseWidthEncoder::new(|_dev| {
                |i| match i {
                    0x80 => 64,
                    0xFF => 128,
                    _ => 0,
                }
            }))
            .await?;
            autd.send(Uniform::new((Phase::new(0x00), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(ULTRASOUND_PERIOD)?;
            autd.send(Uniform::new((Phase::new(0x80), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(ULTRASOUND_PERIOD)?;
            autd.send(Uniform::new((Phase::new(0x80), EmitIntensity::new(0x80))))
                .await?;
            autd.tick(ULTRASOUND_PERIOD)?;
            autd.send(Uniform::new((Phase::new(0x00), EmitIntensity::new(0x00))))
                .await?;
            autd.tick(ULTRASOUND_PERIOD)?;
            Ok(autd)
        })
        .await?;

    let v = record.output_voltage();
    v["time[25us/256]"]
        .u64()?
        .into_no_null_iter()
        .enumerate()
        .for_each(|(i, t)| assert_eq!(i as u64, t));
    let expect_1 = [vec![12.; 64], vec![-12.; 128], vec![12.; 64]].concat();
    let expect_2 = [vec![-12.; 64], vec![12.; 128], vec![-12.; 64]].concat();
    let expect_3 = [vec![-12.; 96], vec![12.; 64], vec![-12.; 96]].concat();
    let expect_4 = vec![-12.; 256];
    let expect = [expect_1, expect_2, expect_3, expect_4].concat();
    emulator.geometry().iter().for_each(|dev| {
        dev.iter().for_each(|tr| {
            assert_eq!(
                expect,
                v[format!("voltage_{}_{}[V]", tr.dev_idx(), tr.idx()).as_str()]
                    .f32()
                    .unwrap()
                    .into_no_null_iter()
                    .collect::<Vec<_>>()
            );
        });
    });

    Ok(())
}

#[tokio::test]
async fn record_output_ultrasound() -> anyhow::Result<()> {
    let emulator = Emulator::new([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())]);

    let record = emulator
        .record(|mut autd| async {
            autd.send(Silencer::disable()).await?;
            autd.send(Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(30 * ULTRASOUND_PERIOD)?;
            Ok(autd)
        })
        .await?;

    let df = record.output_ultrasound();
    df["time[25us/256]"]
        .u64()?
        .into_no_null_iter()
        .enumerate()
        .for_each(|(i, t)| assert_eq!(i as u64, t));
    emulator.geometry().iter().for_each(|dev| {
        dev.iter().for_each(|tr| {
            // TODO: check the value
            assert_eq!(
                30 * 256,
                df[format!("p_{}_{}[a.u.]", tr.dev_idx(), tr.idx()).as_str()]
                    .f32()
                    .unwrap()
                    .iter()
                    .count()
            );
        });
    });

    Ok(())
}

#[rstest::rstest]
#[case(false)]
#[cfg_attr(feature = "gpu", case(true))]
#[tokio::test]
async fn record_sound_field(
    #[allow(unused_variables)]
    #[case]
    gpu: bool,
) -> anyhow::Result<()> {
    let emulator = Emulator::new([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())]);

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
            RecordOption {
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
            RecordOption {
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
            RecordOption {
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
    let emulator = Emulator::new([AUTD3::new(Vector3::zeros())]);

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
            RecordOption {
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
            RecordOption {
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
    let emulator = Emulator::new([AUTD3::new(Vector3::zeros())]);

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
                RecordOption {
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
            RecordOption {
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
    let emulator = Emulator::new([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())]);

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
            RecordOption {
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
    let emulator = Emulator::new([AUTD3::new(Vector3::zeros())]);

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
            RecordOption {
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
            RecordOption {
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
async fn invalid_tick() -> anyhow::Result<()> {
    let emulator = Emulator::new([AUTD3::new(Vector3::zeros())]);

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
    let emulator = Emulator::new([AUTD3::new(Vector3::zeros())]);

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
