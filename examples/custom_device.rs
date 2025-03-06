use std::time::Duration;

use anyhow::Result;

use autd3::{
    core::geometry::{Device, Transducer},
    prelude::*,
};
use autd3_emulator::*;
use polars::{io::SerWriter, prelude::CsvWriter};

struct CustomDevice {
    pitch: f32,
    num_x: usize,
    num_y: usize,
}

impl From<CustomDevice> for Device {
    fn from(value: CustomDevice) -> Self {
        assert!(0 < value.num_x * value.num_y && value.num_x * value.num_y <= 256);
        Self::new(
            UnitQuaternion::identity(),
            itertools::iproduct!(0..value.num_x, 0..value.num_y)
                .map(|(x, y)| {
                    let x = x as f32 * value.pitch;
                    let y = y as f32 * value.pitch;
                    Transducer::new(Point3::new(x, y, 0.))
                })
                .collect(),
        )
    }
}

fn main() -> Result<()> {
    let emulator = Emulator::new([CustomDevice {
        pitch: 2.,
        num_x: 16,
        num_y: 16,
    }]);

    dbg!(emulator.transducer_table());

    let focus = emulator.center() + Vector3::new(0., 0., 30. * mm);

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
            x: focus.x - 200.0..=focus.x + 200.0,
            y: focus.y - 200.0..=focus.y + 200.0,
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

    println!("See plot_rms.py for visualization.");

    Ok(())
}
