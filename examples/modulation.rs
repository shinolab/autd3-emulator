use std::time::Duration;

use anyhow::Result;

use autd3::prelude::*;
use autd3_emulator::*;

use polars::prelude::AnyValue;
use textplots::{Chart, Plot, Shape};

#[tokio::main]
async fn main() -> Result<()> {
    let emulator =
        Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
            .into_emulator();

    // pulse width under 200Hz sine modulation with silencer
    {
        let record = emulator
            .record(|mut autd| async {
                autd.send(Silencer::default()).await?;
                autd.send((Sine::new(200. * Hz), Uniform::new(EmitIntensity::new(0xFF))))
                    .await?;
                autd.tick(Duration::from_millis(10))?;
                Ok(autd)
            })
            .await?;

        let df = record.drive();
        let t = df["time[ns]"].u64()?;
        let pulse_width = df["pulsewidth_0_0"].u8()?; // pulsewidth_<device idx>_<transducer idx>
        println!("pulse width under 200Hz sine modulation with silencer");
        Chart::new(180, 40, 5.0, 10.0)
            .lineplot(&Shape::Lines(
                &t.into_no_null_iter()
                    .zip(pulse_width.into_no_null_iter())
                    .map(|(t, v)| (t as f32 / 1000_000., v as f32))
                    .collect::<Vec<_>>(),
            ))
            .display();
    };

    // pulse width under 200Hz sine modulation without silencer
    {
        let record = emulator
            .record(|mut autd| async {
                autd.send(Silencer::disable()).await?;
                autd.send((Sine::new(200. * Hz), Uniform::new(EmitIntensity::new(0xFF))))
                    .await?;
                autd.tick(Duration::from_millis(10))?;
                Ok(autd)
            })
            .await?;

        let df = record.drive();
        let t = df["time[ns]"].u64()?;
        let pulse_width = df["pulsewidth_0_0"].u8()?; // pulsewidth_<device idx>_<transducer idx>
        println!("pulse width under 200Hz sine modulation without silencer");
        Chart::new(180, 40, 5.0, 10.0)
            .lineplot(&Shape::Lines(
                &t.into_no_null_iter()
                    .zip(pulse_width.into_no_null_iter())
                    .map(|(t, v)| (t as f32 / 1000_000., v as f32))
                    .collect::<Vec<_>>(),
            ))
            .display();
    };

    // plot sound pressure at focus under 200Hz sin modulation with silencer
    {
        let focus = emulator.center() + Vector3::new(0., 0., 150. * mm);

        let record = emulator
            .record(|mut autd| async {
                autd.send(Silencer::default()).await?;
                autd.send((Sine::new(200. * Hz), Focus::new(focus))).await?;
                autd.tick(Duration::from_millis(20))?;
                Ok(autd)
            })
            .await?;

        println!("Calculating sound pressure at focus under 200Hz sin modulation with silencer...");
        let mut sound_field = record
            .sound_field(
                Range {
                    x: focus.x..=focus.x,
                    y: focus.y..=focus.y,
                    z: focus.z..=focus.z,
                    resolution: 1.0 * mm,
                },
                RecordOption {
                    time_step: Duration::from_micros(1),
                    print_progress: true,
                    ..Default::default()
                },
            )
            .await?;

        let df = sound_field.next(Duration::from_millis(20)).await?;

        let t = df.get_column_names().into_iter().skip(3).map(|n| {
            n.as_str()
                .replace("p[Pa]@", "")
                .replace("[ns]", "")
                .parse::<u64>()
                .unwrap()
        });
        let p = df
            .get_row(0)?
            .0
            .into_iter()
            .skip(3)
            .map(|v| match v.into() {
                AnyValue::Float32(v) => v,
                _ => panic!(),
            });
        println!("sound pressure at focus under 200Hz sin modulation with silencer");
        Chart::new(180, 40, 0.0, 20.0)
            .lineplot(&Shape::Lines(
                &t.zip(p)
                    .map(|(t, p)| (t as f32 / 1000_000., p))
                    .collect::<Vec<_>>(),
            ))
            .display();
    }

    Ok(())
}
