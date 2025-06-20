use std::time::Duration;

use anyhow::Result;

use autd3::prelude::*;
use autd3_emulator::*;

use polars::prelude::AnyValue;
use textplots::{Chart, Plot, Shape};

fn main() -> Result<()> {
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

    // pulse width under 200Hz sine modulation with silencer
    {
        let record = emulator.record(|autd| {
            autd.send(Silencer::default())?;
            autd.send((
                Sine {
                    freq: 200 * Hz,
                    option: Default::default(),
                },
                Uniform {
                    intensity: Intensity(0xFF),
                    phase: Phase::ZERO,
                },
            ))?;
            autd.tick(Duration::from_millis(10))?;
            Ok(())
        })?;

        let df = record.pulse_width();

        let t = df.get_column_names().into_iter().map(|n| {
            n.as_str()
                .replace("pulse_width@", "")
                .replace("[ns]", "")
                .parse::<f32>()
                .unwrap()
        });
        let pulse_width = df.get_row(0)?.0.into_iter().map(|v| match v {
            AnyValue::UInt16(v) => v,
            _ => panic!(),
        });
        println!("pulse width under 200Hz sine modulation with silencer");
        dbg!(&df);
        Chart::new(180, 40, 5.0, 10.0)
            .lineplot(&Shape::Lines(
                &t.zip(pulse_width)
                    .map(|(t, v)| (t / 1_000_000., v as f32))
                    .collect::<Vec<_>>(),
            ))
            .display();
    }

    // pulse width under 200Hz sine modulation without silencer
    {
        let record = emulator.record(|autd| {
            autd.send(Silencer::disable())?;
            autd.send((
                Sine {
                    freq: 200 * Hz,
                    option: Default::default(),
                },
                Uniform {
                    intensity: Intensity(0xFF),
                    phase: Phase::ZERO,
                },
            ))?;
            autd.tick(Duration::from_millis(10))?;
            Ok(())
        })?;

        let df = record.pulse_width();

        let t = df.get_column_names().into_iter().map(|n| {
            n.as_str()
                .replace("pulse_width@", "")
                .replace("[ns]", "")
                .parse::<f32>()
                .unwrap()
        });
        let pulse_width = df.get_row(0)?.0.into_iter().map(|v| match v {
            AnyValue::UInt16(v) => v,
            _ => panic!(),
        });
        println!("pulse width under 200Hz sine modulation without silencer");
        dbg!(&df);
        Chart::new(180, 40, 5.0, 10.0)
            .lineplot(&Shape::Lines(
                &t.zip(pulse_width)
                    .map(|(t, v)| (t / 1_000_000., v as f32))
                    .collect::<Vec<_>>(),
            ))
            .display();
    }

    // plot sound pressure at focus under 200Hz sin modulation with silencer
    {
        let focus = emulator.center() + Vector3::new(0., 0., 150. * mm);

        let record = emulator.record(|autd| {
            autd.send(Silencer::default())?;
            autd.send((
                Sine {
                    freq: 200 * Hz,
                    option: Default::default(),
                },
                Focus {
                    pos: focus,
                    option: Default::default(),
                },
            ))?;
            autd.tick(Duration::from_millis(20))?;
            Ok(())
        })?;

        println!("Calculating sound pressure at focus under 200Hz sin modulation with silencer...");
        let mut sound_field = record.sound_field(
            focus,
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                print_progress: true,
                ..Default::default()
            },
        )?;

        let df = sound_field.next(Duration::from_millis(20))?;

        let t = df.get_column_names().into_iter().map(|n| {
            n.as_str()
                .replace("p[Pa]@", "")
                .replace("[ns]", "")
                .parse::<u64>()
                .unwrap()
        });
        let p = df.get_row(0)?.0.into_iter().map(|v| match v {
            AnyValue::Float32(v) => v,
            _ => panic!(),
        });
        println!("sound pressure at focus under 200Hz sin modulation with silencer");
        dbg!(&df);
        Chart::new(180, 40, 0.0, 20.0)
            .lineplot(&Shape::Lines(
                &t.zip(p)
                    .map(|(t, p)| (t as f32 / 1_000_000., p))
                    .collect::<Vec<_>>(),
            ))
            .display();
    }

    Ok(())
}
