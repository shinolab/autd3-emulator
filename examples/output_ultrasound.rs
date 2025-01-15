use std::time::Duration;

use anyhow::Result;

use autd3::prelude::*;
use autd3_emulator::*;

use polars::prelude::AnyValue;
use textplots::{Chart, Plot, Shape};

fn main() -> Result<()> {
    let emulator = Controller::builder([AUTD3::new(Point3::origin())]).into_emulator();

    // output voltage
    {
        let record = emulator.record(|autd| {
            autd.send(Silencer::disable())?;
            autd.send((
                Static::with_intensity(0xFF),
                Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))),
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
                Static::with_intensity(0xFF),
                Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))),
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
