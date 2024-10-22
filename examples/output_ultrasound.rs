use std::time::Duration;

use anyhow::Result;

use autd3::prelude::*;
use autd3_emulator::{Emulator, RecorderControllerExt};

use textplots::{Chart, Plot, Shape};

#[tokio::main]
async fn main() -> Result<()> {
    let emulator = Emulator::new([AUTD3::new(Vector3::zeros())]);

    // output voltage
    {
        let record = emulator
            .record(|mut autd| async {
                autd.send(Silencer::disable()).await?;
                autd.send((
                    Static::with_intensity(0xFF),
                    Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))),
                ))
                .await?;
                autd.tick(Duration::from_millis(1))?;
                Ok(autd)
            })
            .await?;

        let df = record.output_voltage();
        let t = df["time[s]"].f32()?;
        let v = df["voltage_0_0[V]"].f32()?; // voltage_<device idx>_<transducer idx>
        println!("output voltage");
        Chart::new(300, 40, 0.0, 1.0)
            .lineplot(&Shape::Lines(
                &t.into_no_null_iter()
                    .zip(v.into_no_null_iter())
                    .map(|(t, v)| (t * 1000., v))
                    .collect::<Vec<_>>(),
            ))
            .display();
    };

    // output ultrasound
    {
        let record = emulator
            .record(|mut autd| async {
                autd.send(Silencer::disable()).await?;
                autd.send((
                    Static::with_intensity(0xFF),
                    Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))),
                ))
                .await?;
                autd.tick(Duration::from_millis(1))?;
                Ok(autd)
            })
            .await?;

        let df = record.output_ultrasound();
        let t = df["time[s]"].f32()?;
        let v = df["p_0_0[a.u.]"].f32()?;
        println!("output ultrasound");
        Chart::new(300, 40, 0.0, 1.0)
            .lineplot(&Shape::Lines(
                &t.into_no_null_iter()
                    .zip(v.into_no_null_iter())
                    .map(|(t, v)| (t * 1000., v))
                    .collect::<Vec<_>>(),
            ))
            .display();
    };

    Ok(())
}
