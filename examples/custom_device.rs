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

impl autd3::driver::geometry::IntoDevice for CustomDevice {
    fn into_device(self, dev_idx: u16) -> Device {
        assert!(0 < self.num_x * self.num_y && self.num_x * self.num_y <= 256);
        Device::new(
            dev_idx,
            UnitQuaternion::identity(),
            itertools::iproduct!(0..self.num_x, 0..self.num_y)
                .enumerate()
                .map(|(i, (x, y))| {
                    let x = x as f32 * self.pitch;
                    let y = y as f32 * self.pitch;
                    Transducer::new(i as u8, dev_idx, Point3::new(x, y, 0.))
                })
                .collect(),
        )
    }
}

fn main() -> Result<()> {
    let emulator = Controller::builder([CustomDevice {
        pitch: 2.,
        num_x: 16,
        num_y: 16,
    }])
    .into_emulator();

    dbg!(emulator.transducer_table());

    let focus = emulator.center() + Vector3::new(0., 0., 30. * mm);

    let record = emulator.record(|autd| {
        autd.send(Silencer::disable())?;
        autd.send((Static::with_intensity(0xFF), Focus::new(focus)))?;
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
