use std::time::Duration;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_emulator::{
    recording::{Range, RecordOption},
    Emulator,
};
use polars::{io::SerWriter, prelude::CsvWriter};

#[tokio::main]
async fn main() -> Result<()> {
    let mut autd =
        Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
            .open(Emulator::builder())
            .await?;

    let focus = autd.geometry().center() + Vector3::new(0., 0., 150. * mm);

    // // plot sound field around focus
    // {
    //     autd.send(Silencer::disable()).await?;
    //     autd.start_recording()?;
    //     autd.send((Static::with_intensity(0xFF), Focus::new(focus)))
    //         .await?;
    //     autd.tick(Duration::from_millis(1))?;
    //     let record = autd.finish_recording()?;

    //     println!("Calculating sound field around focus...");
    //     let mut df = record[0].sound_field(
    //         Range {
    //             x: focus.x - 20.0..=focus.x + 20.0,
    //             y: focus.y - 20.0..=focus.y + 20.0,
    //             z: focus.z..=focus.z,
    //             resolution: 1.,
    //         },
    //         Duration::ZERO..=Duration::from_millis(1),
    //         RecordOption {
    //             time_step: Duration::from_micros(1),
    //             print_progress: true,
    //             ..Default::default()
    //         },
    //     );
    //     CsvWriter::new(std::fs::File::create("sound_field_around_focus.csv")?)
    //         .include_header(true)
    //         .finish(&mut df)?;
    //     println!("Focus sound field data is saved as sound_field_around_focus.csv");
    // }

    // plot focus under 200Hz sin modulation with silencer
    {
        autd.send(Silencer::default()).await?;
        autd.start_recording()?;
        autd.send((Sine::new(200. * Hz), Focus::new(focus))).await?;
        autd.tick(Duration::from_millis(20))?;
        let record = autd.finish_recording()?;

        println!("Calculating sound pressure at focus under 200Hz sin modulation with silencer...");
        let mut df = record[0].sound_pressure(
            &focus,
            Duration::ZERO..=Duration::from_millis(20),
            RecordOption {
                time_step: Duration::from_micros(1),
                print_progress: true,
                ..Default::default()
            },
        )?;
        CsvWriter::new(std::fs::File::create(
            "sound_pressure_at_focus_with_am.csv",
        )?)
        .include_header(true)
        .finish(&mut df)?;
        println!("Focus sound field data is saved as sound_pressure_at_focus_with_am.csv");
    }

    // // plot STM
    // {
    //     autd.send(Silencer::default()).await?;
    //     autd.start_recording()?;
    //     autd.send((
    //         Static::with_intensity(0xFF),
    //         FociSTM::new(
    //             SamplingConfig::new(1. * kHz)?,
    //             (0..10).map(|i| {
    //                 let theta = 2. * PI * i as f32 / 10.;
    //                 focus + Vector3::new(theta.cos(), theta.sin(), 0.) * 20. * mm
    //             }),
    //         )?,
    //     ))
    //     .await?;
    //     autd.tick(Duration::from_millis(20))?;
    //     let record = autd.finish_recording()?;

    //     println!("Calculating sound field with STM...");
    //     let mut df = record[0].sound_field(
    //         Range {
    //             x: focus.x - 30.0..=focus.x + 30.0,
    //             y: focus.y - 30.0..=focus.y + 30.0,
    //             z: focus.z..=focus.z,
    //             resolution: 2.,
    //         },
    //         RecordOption {
    //             time: Some(TimeRange {
    //                 duration: Duration::ZERO..=Duration::from_millis(20),
    //                 time_step_s: Duration::from_micros(2).as_secs_f32(),
    //             }),
    //             print_progress: true,
    //             ..Default::default()
    //         },
    //     );
    //     CsvWriter::new(std::fs::File::create("sound_field_stm.csv")?)
    //         .include_header(true)
    //         .finish(&mut df)?;
    //     println!("STM sound field data is saved as sound_field_stm.csv");
    // }

    autd.close().await?;

    println!("See plot_field.py for visualization.");

    Ok(())
}
