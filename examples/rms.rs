use std::time::Duration;

use autd3::prelude::*;
use autd3_emulator::*;
use polars::{io::SerWriter, prelude::CsvWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("INFO: rms does not take into account propagation delay nor transducer response.");

    let emulator = Emulator::new([AUTD3 {
        pos: Point3::origin(),
        rot: UnitQuaternion::identity(),
    }]);

    let focus = emulator.center() + Vector3::new(0., 0., 150. * mm);

    // plot sound field around focus
    {
        let record = emulator.record(|autd| {
            autd.send(Silencer::disable())?;
            autd.send((
                Static { intensity: 0xFF },
                Focus {
                    pos: focus,
                    option: Default::default(),
                },
            ))?;
            autd.tick(Duration::from_micros(25))?;
            Ok(())
        })?;

        println!("Calculating rms around focus...");
        let mut sound_field = record.sound_field(
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
        )?;

        let mut df = polars::functions::concat_df_horizontal(
            &[
                sound_field.observe_points(),
                sound_field.next(Duration::from_micros(25))?,
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
        let record = emulator.record(|autd| {
            autd.send(Silencer::default())?;
            autd.send((
                Static { intensity: 0xFF },
                FociSTM {
                    foci: (0..4)
                        .map(|i| {
                            let theta = 2. * PI * i as f32 / 4.;
                            focus + Vector3::new(theta.cos(), theta.sin(), 0.) * 20. * mm
                        })
                        .collect::<Vec<_>>(),
                    config: SamplingConfig::new(1. * kHz),
                },
            ))?;
            autd.tick(Duration::from_millis(5))?;
            Ok(())
        })?;

        println!("Calculating rms with STM...");
        let mut sound_field = record.sound_field(
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
        )?;

        let mut df = polars::functions::concat_df_horizontal(
            &[
                sound_field.observe_points(),
                sound_field.next(Duration::from_millis(5))?,
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
