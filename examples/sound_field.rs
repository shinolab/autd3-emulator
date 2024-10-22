use std::time::Duration;

use anyhow::Result;

use autd3::prelude::*;
use autd3_emulator::{Emulator, Range, RecordOption, RecorderControllerExt};
use polars::{io::SerWriter, prelude::CsvWriter};

#[tokio::main]
async fn main() -> Result<()> {
    let emulator = Emulator::new([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())]);

    let focus = emulator.geometry().center() + Vector3::new(0., 0., 150. * mm);

    // plot sound field around focus
    {
        let record = emulator
            .record(|mut autd| async {
                autd.send(Silencer::disable()).await?;
                autd.send((Static::with_intensity(0xFF), Focus::new(focus)))
                    .await?;
                autd.tick(Duration::from_millis(1))?;
                Ok(autd)
            })
            .await?;

        println!("Calculating sound field around focus...");
        let mut sound_field = record
            .sound_field(
                Range {
                    x: focus.x - 20.0..=focus.x + 20.0,
                    y: focus.y - 20.0..=focus.y + 20.0,
                    z: focus.z..=focus.z,
                    resolution: 1.,
                },
                RecordOption {
                    time_step: Duration::from_micros(1),
                    print_progress: true,
                    #[cfg(feature = "gpu")]
                    gpu: true,
                    ..Default::default()
                },
            )
            .await?;
        let mut df = sound_field.next(Duration::from_millis(1)).await?;

        CsvWriter::new(std::fs::File::create("sound_field_around_focus.csv")?)
            .include_header(true)
            .finish(&mut df)?;
        println!("Focus sound field data is saved as sound_field_around_focus.csv");
    }

    // plot STM
    {
        let record = emulator
            .record(|mut autd| async {
                autd.send(Silencer::default()).await?;
                autd.send((
                    Static::with_intensity(0xFF),
                    FociSTM::new(
                        SamplingConfig::new(1. * kHz)?,
                        (0..4).map(|i| {
                            let theta = 2. * PI * i as f32 / 4.;
                            focus + Vector3::new(theta.cos(), theta.sin(), 0.) * 20. * mm
                        }),
                    )?,
                ))
                .await?;
                autd.tick(Duration::from_millis(5))?;
                Ok(autd)
            })
            .await?;

        println!("Calculating sound field with STM...");
        let mut sound_field = record
            .sound_field(
                Range {
                    x: focus.x - 30.0..=focus.x + 30.0,
                    y: focus.y - 30.0..=focus.y + 30.0,
                    z: focus.z..=focus.z,
                    resolution: 1.,
                },
                RecordOption {
                    time_step: ULTRASOUND_PERIOD / 10,
                    print_progress: true,
                    #[cfg(feature = "gpu")]
                    gpu: true,
                    ..Default::default()
                },
            )
            .await?;
        let mut df = sound_field.next(Duration::from_millis(5)).await?;
        CsvWriter::new(std::fs::File::create("sound_field_stm.csv")?)
            .include_header(true)
            .finish(&mut df)?;
        println!("STM sound field data is saved as sound_field_stm.csv");
    }

    println!("See plot_field.py for visualization.");

    Ok(())
}
