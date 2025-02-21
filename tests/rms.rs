use std::time::Duration;

use autd3::{driver::defined::ultrasound_period, prelude::*};
use autd3_emulator::*;

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
        autd.tick(100 * ultrasound_period())?;
        Ok(())
    })?;

    let point = Vector3::new(0., 0., 300. * mm);
    let mut rms = record.sound_field(
        RangeXY {
            x: point.x - 100.0..=point.x + 100.0,
            y: point.y - 100.0..=point.y + 100.0,
            z: point.z,
            resolution: 100.,
        },
        RmsRecordOption {
            #[cfg(feature = "gpu")]
            gpu,
            ..Default::default()
        },
    )?;

    let df = rms.observe_points();
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
    let _df = rms.next(100 * ultrasound_period())?;

    assert!(
        record
            .sound_field(
                RangeXY {
                    x: point.x - 1.0..=point.x + 1.0,
                    y: point.y - 1.0..=point.y + 1.0,
                    z: point.z,
                    resolution: 1.,
                },
                RmsRecordOption {
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
#[case(false)]
#[cfg_attr(feature = "gpu", case(true))]
#[test]
fn record_rms_resume(
    #[allow(unused_variables)]
    #[case]
    gpu: bool,
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
        autd.tick(10 * ultrasound_period())?;
        Ok(())
    })?;

    let point = Vector3::new(0., 0., 1. * mm);
    let range = RangeXY {
        x: point.x - 50.0..=point.x + 50.0,
        y: point.y - 50.0..=point.y + 50.0,
        z: point.z,
        resolution: 1.,
    };

    let mut rms = record.sound_field(
        range.clone(),
        RmsRecordOption {
            #[cfg(feature = "gpu")]
            gpu,
            ..Default::default()
        },
    )?;
    assert_eq!(
        record
            .sound_field(
                range,
                RmsRecordOption {
                    #[cfg(feature = "gpu")]
                    gpu,
                    ..Default::default()
                },
            )?
            .next(10 * ultrasound_period())?,
        polars::functions::concat_df_horizontal(
            &[
                rms.next(5 * ultrasound_period())?,
                rms.next(5 * ultrasound_period())?,
            ],
            false,
        )?
    );

    Ok(())
}

#[test]
fn record_rms_skip() -> anyhow::Result<()> {
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
        autd.tick(10 * ultrasound_period())?;
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
            RmsRecordOption::default(),
        )?;
        sf.next(5 * ultrasound_period())?;
        sf.next(5 * ultrasound_period())?
    };

    let mut rms = record.sound_field(
        RangeXY {
            x: point.x - 9.0..=point.x + 9.0,
            y: point.y - 50.0..=point.y + 50.0,
            z: point.z,
            resolution: 1.,
        },
        RmsRecordOption::default(),
    )?;
    let v = rms
        .skip(5 * ultrasound_period())?
        .next(5 * ultrasound_period())?;

    assert_eq!(expect, v);

    Ok(())
}

#[cfg(feature = "gpu")]
#[test]
fn record_rms_gpu_eq_cpu() -> anyhow::Result<()> {
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
        autd.tick(10 * ultrasound_period())?;
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
            RmsRecordOption::default(),
        )?
        .next(10 * ultrasound_period())?;

    let mut rms = record.sound_field(
        RangeXY {
            x: point.x - 9.0..=point.x + 9.0,
            y: point.y - 50.0..=point.y + 50.0,
            z: point.z,
            resolution: 1.,
        },
        RmsRecordOption {
            gpu: true,
            ..Default::default()
        },
    )?;
    let gpu = rms.next(10 * ultrasound_period())?;

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
        autd.tick(ultrasound_period())?;
        Ok(())
    })?;

    let point = Vector3::new(0., 0., 300. * mm);
    let mut rms = record.sound_field(
        RangeXY {
            x: point.x - 100.0..=point.x + 100.0,
            y: point.y - 100.0..=point.y + 100.0,
            z: point.z,
            resolution: 100.,
        },
        RmsRecordOption::default(),
    )?;

    assert!(rms.next(2 * ultrasound_period()).is_err());

    Ok(())
}
