use std::time::Duration;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_emulator::{
    recording::{Range, RecordOption},
    Emulator,
};

use polars::prelude::AnyValue;
use textplots::{Chart, Plot, Shape};

#[tokio::main]
async fn main() -> Result<()> {
    let mut autd =
        Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
            .open(Emulator::builder())
            .await?;

    // raw modulation buffer
    {
        autd.send(Sine::new(200. * Hz)).await?;

        let df = autd[0].modulation();
        let t = df["time[s]"].f32()?;
        let modulation = df["modulation"].u8()?;
        println!("200Hz sine raw modulation buffer");
        Chart::new(180, 40, 0.0, 5.0)
            .lineplot(&Shape::Lines(
                &t.into_no_null_iter()
                    .zip(modulation.into_no_null_iter())
                    .map(|(t, v)| (t * 1000., v as f32))
                    .collect::<Vec<_>>(),
            ))
            .display();
    }

    // pulse width under 200Hz sine modulation with silencer
    {
        autd.send(Silencer::default()).await?;
        autd.start_recording()?;
        autd.send((Sine::new(200. * Hz), Uniform::new(EmitIntensity::new(0xFF))))
            .await?;
        autd.tick(Duration::from_millis(10))?;
        let record = autd.finish_recording()?;

        let df = record[0][0].drive();
        let t = df["time[s]"].f32()?;
        let pulse_width = df["pulsewidth"].u8()?;
        println!("pulse width under 200Hz sine modulation with silencer");
        Chart::new(180, 40, 5.0, 10.0)
            .lineplot(&Shape::Lines(
                &t.into_no_null_iter()
                    .zip(pulse_width.into_no_null_iter())
                    .map(|(t, v)| (t * 1000., v as f32))
                    .collect::<Vec<_>>(),
            ))
            .display();
    };

    // pulse width under 200Hz sine modulation without silencer
    {
        autd.send(Silencer::disable()).await?;
        autd.start_recording()?;
        autd.send((Sine::new(200. * Hz), Uniform::new(EmitIntensity::new(0xFF))))
            .await?;
        autd.tick(Duration::from_millis(10))?;
        let record = autd.finish_recording()?;

        let df = record[0][0].drive();
        let t = df["time[s]"].f32()?;
        let pulse_width = df["pulsewidth"].u8()?;
        println!("pulse width under 200Hz sine modulation without silencer");
        Chart::new(180, 40, 5.0, 10.0)
            .lineplot(&Shape::Lines(
                &t.into_no_null_iter()
                    .zip(pulse_width.into_no_null_iter())
                    .map(|(t, v)| (t * 1000., v as f32))
                    .collect::<Vec<_>>(),
            ))
            .display();
    };

    // plot sound pressure at focus under 200Hz sin modulation with silencer
    {
        let focus = autd.geometry().center() + Vector3::new(0., 0., 150. * mm);

        autd.send(Silencer::default()).await?;
        autd.start_recording()?;
        autd.send((Sine::new(200. * Hz), Focus::new(focus))).await?;
        autd.tick(Duration::from_millis(20))?;
        let record = autd.finish_recording()?;

        println!("Calculating sound pressure at focus under 200Hz sin modulation with silencer...");
        let mut sound_field = record[0].sound_field(
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
        )?;

        let df = sound_field.next(Duration::from_millis(20))?;

        let t = df
            .get_column_names()
            .into_iter()
            .skip(3)
            .map(|n| n.as_str().replace("p[Pa]@", "").parse::<f32>().unwrap());
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
                &t.zip(p).map(|(t, p)| (t * 1000., p)).collect::<Vec<_>>(),
            ))
            .display();
    }

    autd.close().await?;

    Ok(())
}
