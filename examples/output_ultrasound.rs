use std::time::Duration;

use anyhow::Result;

use autd3::prelude::*;
use autd3_emulator::*;

use polars::prelude::AnyValue;
use textplots::{Chart, Plot, Shape};

fn main() -> Result<()> {
    let emulator = Emulator::new([AUTD3 {
        pos: Point3::origin(),
        rot: UnitQuaternion::identity(),
    }]);

    // output voltage
    {
        let record = emulator.record(|autd| {
            autd.send(Silencer::disable())?;
            autd.send((
                Static { intensity: 0xFF },
                Uniform {
                    phase: Phase(0x40),
                    intensity: EmitIntensity(0xFF),
                },
            ))?;
            autd.tick(Duration::from_millis(1))?;
            Ok(())
        })?;

        let df = record.output_voltage();

        let t = df.get_column_names().into_iter().map(|n| {
            n.as_str()
                .replace("voltage[V]@", "")
                .replace("[25us/256]", "")
                .parse::<f32>()
                .unwrap()
                * 0.025
                / 256.
        });
        let v = df.get_row(0)?.0.into_iter().map(|v| match v {
            AnyValue::Float32(v) => v,
            _ => panic!(),
        });
        println!("output voltage");
        dbg!(&df);
        Chart::new(300, 40, 0.0, 1.0)
            .lineplot(&Shape::Lines(&t.zip(v).collect::<Vec<_>>()))
            .display();
    };

    // output ultrasound
    {
        let record = emulator.record(|autd| {
            autd.send(Silencer::disable())?;
            autd.send((
                Static { intensity: 0xFF },
                Uniform {
                    phase: Phase(0x40),
                    intensity: EmitIntensity(0xFF),
                },
            ))?;
            autd.tick(Duration::from_millis(1))?;
            Ok(())
        })?;

        let df = record.output_ultrasound();

        let t = df.get_column_names().into_iter().map(|n| {
            n.as_str()
                .replace("p[a.u.]@", "")
                .replace("[25us/256]", "")
                .parse::<f32>()
                .unwrap()
                * 0.025
                / 256.
        });
        let v = df.get_row(0)?.0.into_iter().map(|v| match v {
            AnyValue::Float32(v) => v,
            _ => panic!(),
        });
        println!("output ultrasound");
        dbg!(&df);
        Chart::new(300, 40, 0.0, 1.0)
            .lineplot(&Shape::Lines(&t.zip(v).collect::<Vec<_>>()))
            .display();
    };

    Ok(())
}
