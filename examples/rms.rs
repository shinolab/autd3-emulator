use std::time::Duration;

use anyhow::Result;

use autd3::prelude::*;
use autd3_emulator::*;
use polars::{io::SerWriter, prelude::CsvWriter};

#[tokio::main]
async fn main() -> Result<()> {
    println!("INFO: rms does not take into account propagation delay nor transducer response.");

    let emulator = Controller::builder([AUTD3::new(Vector3::zeros())]).into_emulator();

    let focus = emulator.center() + Vector3::new(0., 0., 150. * mm);

    // plot sound field around focus
    {
        let record = emulator
            .record(|mut autd| async {
                autd.send(Silencer::disable()).await?;
                autd.send((Static::with_intensity(0xFF), Focus::new(focus)))
                    .await?;
                autd.tick(ULTRASOUND_PERIOD)?;
                Ok(autd)
            })
            .await?;

        println!("Calculating rms around focus...");
        let mut sound_field = record
            .sound_field(
                RangeXY {
                    x: focus.x - 20.0..=focus.x + 20.0,
                    y: focus.y - 20.0..=focus.y + 20.0,
                    z: focus.z,
                    resolution: 1.,
                },
                RmsRecordOption {
                    #[cfg(feature = "gpu")]
                    gpu: true,
                    ..Default::default()
                },
            )
            .await?;

        let mut df = polars::functions::concat_df_horizontal(
            &[
                sound_field.observe_points(),
                sound_field.next(ULTRASOUND_PERIOD).await?,
            ],
            false,
        )?;
        CsvWriter::new(std::fs::File::create("rms_focus.csv")?)
            .include_header(true)
            .finish(&mut df)?;
        println!("Focus sound field data is saved as rms_focus.csv");
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

        println!("Calculating rms with STM...");
        let mut sound_field = record
            .sound_field(
                RangeXY {
                    x: focus.x - 30.0..=focus.x + 30.0,
                    y: focus.y - 30.0..=focus.y + 30.0,
                    z: focus.z,
                    resolution: 1.,
                },
                RmsRecordOption {
                    #[cfg(feature = "gpu")]
                    gpu: true,
                    ..Default::default()
                },
            )
            .await?;

        let mut df = polars::functions::concat_df_horizontal(
            &[
                sound_field.observe_points(),
                sound_field.next(Duration::from_millis(5)).await?,
            ],
            false,
        )?;
        CsvWriter::new(std::fs::File::create("rms_stm.csv")?)
            .include_header(true)
            .finish(&mut df)?;
        println!("STM sound field data is saved as rms_stm.csv");
    }

    println!("See plot_rms.py for visualization.");

    Ok(())
}
