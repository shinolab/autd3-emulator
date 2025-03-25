use autd3::{driver::defined::ULTRASOUND_PERIOD, prelude::*};
use autd3_emulator::*;

use std::time::Duration;

#[rstest::rstest]
#[case(false)]
#[cfg_attr(feature = "gpu", case(true))]
#[test]
fn record_sound_field(
    #[allow(unused_variables)]
    #[case]
    gpu: bool,
) -> anyhow::Result<()> {
    let emulator = Emulator::new([
        AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        },
        AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        },
    ]);

    let record = emulator.record(|autd| {
        autd.send(Silencer::disable())?;
        autd.send(Uniform {
            phase: Phase(0x40),
            intensity: EmitIntensity(0xFF),
        })?;
        autd.tick(100 * ULTRASOUND_PERIOD)?;
        Ok(())
    })?;

    let point = Vector3::new(0., 0., 300. * mm);
    let mut sound_field = record.sound_field(
        RangeXY {
            x: point.x - 100.0..=point.x + 100.0,
            y: point.y - 100.0..=point.y + 100.0,
            z: point.z,
            resolution: 100.,
        },
        InstantRecordOption {
            time_step: Duration::from_micros(1),
            #[cfg(feature = "gpu")]
            gpu,
            ..Default::default()
        },
    )?;

    let df = sound_field.observe_points();
    assert_eq!(
        vec![-100.0, 0.0, 100.0, -100.0, 0.0, 100.0, -100.0, 0.0, 100.0],
        df["x[mm]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    );
    assert_eq!(
        vec![-100.0, -100.0, -100.0, 0.0, 0.0, 0.0, 100.0, 100.0, 100.0],
        df["y[mm]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    );
    assert_eq!(
        vec![
            300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0
        ],
        df["z[mm]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    );

    // TODO: check the value
    let _df = sound_field.next(100 * ULTRASOUND_PERIOD)?;

    assert!(
        record
            .sound_field(
                RangeXY {
                    x: point.x - 1.0..=point.x + 1.0,
                    y: point.y - 1.0..=point.y + 1.0,
                    z: point.z,
                    resolution: 1.,
                },
                InstantRecordOption {
                    time_step: Duration::from_micros(2),
                    #[cfg(feature = "gpu")]
                    gpu,
                    ..Default::default()
                },
            )
            .is_err()
    );
    assert!(
        record
            .sound_field(
                RangeXY {
                    x: point.x - 1.0..=point.x + 1.0,
                    y: point.y - 1.0..=point.y + 1.0,
                    z: point.z,
                    resolution: 1.,
                },
                InstantRecordOption {
                    time_step: Duration::from_micros(1),
                    #[cfg(feature = "gpu")]
                    gpu,
                    ..Default::default()
                },
            )?
            .next(Duration::from_micros(1))
            .is_err()
    );

    Ok(())
}

#[rstest::rstest]
#[case(false, 20)]
#[cfg_attr(feature = "gpu", case(true, 10))]
#[test]
fn record_sound_field_resume(
    #[allow(unused_variables)]
    #[case]
    gpu: bool,
    #[case] memory_limits_hint_mb: usize,
) -> anyhow::Result<()> {
    let emulator = Emulator::new([AUTD3 {
        pos: Point3::origin(),
        rot: UnitQuaternion::identity(),
    }]);

    let record = emulator.record(|autd| {
        autd.send(Silencer::disable())?;
        autd.send(Uniform {
            phase: Phase(0x40),
            intensity: EmitIntensity(0xFF),
        })?;
        autd.tick(10 * ULTRASOUND_PERIOD)?;
        Ok(())
    })?;

    let point = Vector3::new(0., 0., 1. * mm);
    let range = RangeXY {
        x: point.x - 50.0..=point.x + 50.0,
        y: point.y - 50.0..=point.y + 50.0,
        z: point.z,
        resolution: 1.,
    };

    let mut sound_field = record.sound_field(
        range.clone(),
        InstantRecordOption {
            time_step: Duration::from_micros(1),
            memory_limits_hint_mb,
            #[cfg(feature = "gpu")]
            gpu,
            ..Default::default()
        },
    )?;
    assert_eq!(
        record
            .sound_field(
                range,
                InstantRecordOption {
                    time_step: Duration::from_micros(1),
                    #[cfg(feature = "gpu")]
                    gpu,
                    ..Default::default()
                },
            )?
            .next(10 * ULTRASOUND_PERIOD)?,
        polars::functions::concat_df_horizontal(
            &[
                sound_field.next(5 * ULTRASOUND_PERIOD)?,
                sound_field.next(5 * ULTRASOUND_PERIOD)?,
            ],
            false,
        )?
    );

    Ok(())
}

#[test]
fn record_sound_field_skip() -> anyhow::Result<()> {
    let emulator = Emulator::new([AUTD3 {
        pos: Point3::origin(),
        rot: UnitQuaternion::identity(),
    }]);

    let record = emulator.record(|autd| {
        autd.send(Silencer::disable())?;
        autd.send(Uniform {
            phase: Phase(0x40),
            intensity: EmitIntensity(0xFF),
        })?;
        autd.tick(10 * ULTRASOUND_PERIOD)?;
        Ok(())
    })?;

    let point = Vector3::new(0., 0., 1. * mm);
    let expect = {
        let mut sf = record.sound_field(
            RangeXY {
                x: point.x - 9.0..=point.x + 9.0,
                y: point.y - 50.0..=point.y + 50.0,
                z: point.z,
                resolution: 1.,
            },
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                ..Default::default()
            },
        )?;
        sf.next(5 * ULTRASOUND_PERIOD)?;
        sf.next(5 * ULTRASOUND_PERIOD)?
    };

    let mut sound_field = record.sound_field(
        RangeXY {
            x: point.x - 9.0..=point.x + 9.0,
            y: point.y - 50.0..=point.y + 50.0,
            z: point.z,
            resolution: 1.,
        },
        InstantRecordOption {
            time_step: Duration::from_micros(1),
            memory_limits_hint_mb: 1,
            ..Default::default()
        },
    )?;
    let v = sound_field
        .skip(5 * ULTRASOUND_PERIOD)?
        .next(5 * ULTRASOUND_PERIOD)?;

    assert_eq!(expect, v);

    Ok(())
}

#[rstest::rstest]
#[case(false, 0)]
#[case(false, 1)]
#[case(false, usize::MAX)]
#[cfg_attr(feature = "gpu", case(true, 0))]
#[cfg_attr(feature = "gpu", case(true, 1))]
#[test]
fn record_sound_field_with_limit(
    #[allow(unused_variables)]
    #[case]
    gpu: bool,
    #[case] memory_limits_hint_mb: usize,
) -> anyhow::Result<()> {
    let emulator = Emulator::new([
        AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        },
        AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        },
    ]);

    let record = emulator.record(|autd| {
        autd.send(Silencer::disable())?;
        autd.send(Uniform {
            phase: Phase(0x40),
            intensity: EmitIntensity(0xFF),
        })?;
        autd.tick(100 * ULTRASOUND_PERIOD)?;
        Ok(())
    })?;

    let point = Vector3::new(0., 0., 300. * mm);
    let mut sound_field = record.sound_field(
        RangeXY {
            x: point.x - 100.0..=point.x + 100.0,
            y: point.y - 100.0..=point.y + 100.0,
            z: point.z,
            resolution: 100.,
        },
        InstantRecordOption {
            time_step: Duration::from_micros(1),
            memory_limits_hint_mb,
            #[cfg(feature = "gpu")]
            gpu,
            ..Default::default()
        },
    )?;

    let df = sound_field.observe_points();
    assert_eq!(
        vec![-100.0, 0.0, 100.0, -100.0, 0.0, 100.0, -100.0, 0.0, 100.0],
        df["x[mm]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    );
    assert_eq!(
        vec![-100.0, -100.0, -100.0, 0.0, 0.0, 0.0, 100.0, 100.0, 100.0],
        df["y[mm]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    );
    assert_eq!(
        vec![
            300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0, 300.0
        ],
        df["z[mm]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    );

    // TODO: check the value
    let _df = sound_field.next(100 * ULTRASOUND_PERIOD)?;

    Ok(())
}

#[cfg(feature = "gpu")]
#[test]
fn record_sound_field_gpu_eq_cpu() -> anyhow::Result<()> {
    let emulator = Emulator::new([AUTD3 {
        pos: Point3::origin(),
        rot: UnitQuaternion::identity(),
    }]);

    let record = emulator.record(|autd| {
        autd.send(Silencer::disable())?;
        autd.send(Uniform {
            phase: Phase(0x40),
            intensity: EmitIntensity(0xFF),
        })?;
        autd.tick(10 * ULTRASOUND_PERIOD)?;
        Ok(())
    })?;

    let point = Vector3::new(0., 0., 1. * mm);
    let cpu = record
        .sound_field(
            RangeXY {
                x: point.x - 9.0..=point.x + 9.0,
                y: point.y - 50.0..=point.y + 50.0,
                z: point.z,
                resolution: 1.,
            },
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                ..Default::default()
            },
        )?
        .next(10 * ULTRASOUND_PERIOD)?;

    let mut sound_field = record.sound_field(
        RangeXY {
            x: point.x - 9.0..=point.x + 9.0,
            y: point.y - 50.0..=point.y + 50.0,
            z: point.z,
            resolution: 1.,
        },
        InstantRecordOption {
            time_step: Duration::from_micros(1),
            gpu: true,
            ..Default::default()
        },
    )?;
    let gpu = sound_field.next(10 * ULTRASOUND_PERIOD)?;

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

#[test]
fn not_recorded() -> anyhow::Result<()> {
    let emulator = Emulator::new([AUTD3 {
        pos: Point3::origin(),
        rot: UnitQuaternion::identity(),
    }]);

    let record = emulator.record(|autd| {
        autd.send(Silencer::disable())?;
        autd.send(Uniform {
            phase: Phase(0x40),
            intensity: EmitIntensity(0xFF),
        })?;
        autd.tick(ULTRASOUND_PERIOD)?;
        Ok(())
    })?;

    let point = Vector3::new(0., 0., 300. * mm);
    let mut sound_field = record.sound_field(
        RangeXY {
            x: point.x - 100.0..=point.x + 100.0,
            y: point.y - 100.0..=point.y + 100.0,
            z: point.z,
            resolution: 100.,
        },
        InstantRecordOption {
            time_step: Duration::from_micros(1),
            ..Default::default()
        },
    )?;

    assert!(sound_field.next(2 * ULTRASOUND_PERIOD).is_err());

    Ok(())
}
