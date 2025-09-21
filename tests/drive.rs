use autd3::{driver::common::ULTRASOUND_PERIOD, gain, prelude::*};
use autd3_emulator::*;

use polars::{frame::DataFrame, prelude::Column};

#[rstest::rstest]
#[test]
fn record_phase() -> Result<(), EmulatorError> {
    let silencer = Silencer {
        config: Silencer::disable().config,
    };

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

    let expect = DataFrame::new(
        [0, 25000, 50000]
            .iter()
            .zip([0, 1, 1])
            .map(|(t, o)| {
                Column::new(
                    format!("pulse_width@{t}[ns]").into(),
                    emulator
                        .iter()
                        .flat_map(|dev| {
                            dev.iter().map(|tr| (Phase(o) + Phase(tr.dev_idx() as _)).0)
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect(),
    )
    .unwrap();

    let record = emulator.record(|autd| {
        autd.send(silencer)?;
        autd.send((
            Static { intensity: 100 },
            gain::Custom::new(|_| {
                |tr| Drive {
                    phase: Phase(tr.dev_idx() as _),
                    intensity: Intensity::MAX,
                }
            }),
        ))?;
        autd.tick(ULTRASOUND_PERIOD)?;
        autd.send((
            Static { intensity: 200 },
            gain::Custom::new(|_| {
                |tr| Drive {
                    phase: Phase(0x01) + Phase(tr.dev_idx() as _),
                    intensity: Intensity::MAX,
                }
            }),
        ))?;
        autd.tick(2 * ULTRASOUND_PERIOD)?;

        Ok(())
    })?;

    assert_eq!(expect, record.phase());

    Ok(())
}

#[test]
fn record_pulse_width() -> Result<(), EmulatorError> {
    use polars::frame::DataFrame;

    let silencer = Silencer {
        config: Silencer::disable().config,
    };

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

    let to_pulse_width = |a, b| {
        let i = (a as usize * b) / 255;
        ((((i as f32) / 255.).asin() / PI) * 512.).round() as u8
    };

    let expect = DataFrame::new(
        [0, 25000, 50000]
            .iter()
            .zip([100, 200, 200])
            .map(|(t, i)| {
                Column::new(
                    format!("pulse_width@{t}[ns]").into(),
                    emulator
                        .iter()
                        .flat_map(|dev| dev.iter().map(|tr| to_pulse_width(i, tr.idx())))
                        .collect::<Vec<_>>(),
                )
            })
            .collect(),
    )
    .unwrap();

    let record = emulator.record(|autd| {
        autd.send(silencer)?;
        autd.send((
            Static { intensity: 100 },
            gain::Custom::new(|_| {
                |tr| Drive {
                    phase: Phase::ZERO,
                    intensity: Intensity(tr.idx() as _),
                }
            }),
        ))?;
        autd.tick(ULTRASOUND_PERIOD)?;
        autd.send((
            Static { intensity: 200 },
            gain::Custom::new(|_| {
                |tr| Drive {
                    phase: Phase::ZERO,
                    intensity: Intensity(tr.idx() as _),
                }
            }),
        ))?;
        autd.tick(2 * ULTRASOUND_PERIOD)?;

        Ok(())
    })?;

    assert_eq!(expect, record.pulse_width());

    Ok(())
}
