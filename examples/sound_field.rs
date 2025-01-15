use std::time::Duration;

use anyhow::Result;

use autd3::prelude::*;
use autd3_emulator::*;
use polars::{io::SerWriter, prelude::CsvWriter};

fn main() -> Result<()> {
    let emulator = Controller::builder([AUTD3::new(Point3::origin())]).into_emulator();

    let focus = emulator.center() + Vector3::new(0., 0., 150. * mm);

    // plot sound field around focus
    {
        let record = emulator.record(|autd| {
            autd.send(Silencer::disable())?;
            autd.send((Static::with_intensity(0xFF), Focus::new(focus)))?;
            autd.tick(Duration::from_millis(1))?;
            Ok(())
        })?;

        println!("Calculating sound field around focus...");
        let mut sound_field = record.sound_field(
            RangeXY {
                x: focus.x - 20.0..=focus.x + 20.0,
                y: focus.y - 20.0..=focus.y + 20.0,
                z: focus.z,
                resolution: 1.,
            },
            InstantRecordOption {
                time_step: Duration::from_micros(1),
                print_progress: true,
                #[cfg(feature = "gpu")]
                gpu: true,
                ..Default::default()
            },
        )?;

        let mut df = polars::functions::concat_df_horizontal(
            &[
                sound_field.observe_points(),
                sound_field.next(Duration::from_millis(1))?,
            ],
            false,
        )?;
        CsvWriter::new(std::fs::File::create("sound_field_around_focus.csv")?)
            .include_header(true)
            .finish(&mut df)?;
        println!("Focus sound field data is saved as sound_field_around_focus.csv");
    }

    // plot STM
    {
        let record = emulator.record(|autd| {
            autd.send(Silencer::default())?;
            autd.send((
                Static::with_intensity(0xFF),
                FociSTM::new(
                    SamplingConfig::new(1. * kHz)?,
                    (0..4).map(|i| {
                        let theta = 2. * PI * i as f32 / 4.;
                        focus + Vector3::new(theta.cos(), theta.sin(), 0.) * 20. * mm
                    }),
                )?,
            ))?;
            autd.tick(Duration::from_millis(5))?;
            Ok(())
        })?;

        println!("Calculating sound field with STM...");
        let mut sound_field = record.sound_field(
            RangeXY {
                x: focus.x - 30.0..=focus.x + 30.0,
                y: focus.y - 30.0..=focus.y + 30.0,
                z: focus.z,
                resolution: 1.,
            },
            InstantRecordOption {
                time_step: Duration::from_micros(25) / 10,
                print_progress: true,
                #[cfg(feature = "gpu")]
                gpu: true,
                ..Default::default()
            },
        )?;

        let mut df = polars::functions::concat_df_horizontal(
            &[
                sound_field.observe_points(),
                sound_field.next(Duration::from_millis(5))?,
            ],
            false,
        )?;
        CsvWriter::new(std::fs::File::create("sound_field_stm.csv")?)
            .include_header(true)
            .finish(&mut df)?;
        println!("STM sound field data is saved as sound_field_stm.csv");
    }

    println!("See plot_field.py for visualization.");

    Ok(())
}
