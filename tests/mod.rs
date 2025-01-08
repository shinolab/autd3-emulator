mod drive;
mod output_ultrasound;
mod output_voltage;
mod rms;
mod sound_field;

use autd3::{driver::defined::ultrasound_period, prelude::*};
use autd3_emulator::*;
use polars::prelude::Column;

#[tokio::test]
async fn invalid_tick() -> anyhow::Result<()> {
    let emulator = Controller::builder([AUTD3::new(Point3::origin())]).into_emulator();

    let record = emulator
        .record(|mut autd| async {
            autd.send(Silencer::disable()).await?;
            autd.tick(ultrasound_period() / 2)?;
            Ok(autd)
        })
        .await;

    assert!(record.is_err());

    Ok(())
}

#[tokio::test]
async fn transducer_table() {
    let emulator = Controller::builder([AUTD3::new(Point3::origin())]).into_emulator();

    let df = emulator.transducer_table();

    assert_eq!(
        &Column::new(
            "dev_idx".into(),
            &emulator
                .iter()
                .flat_map(|dev| dev.iter().map(|tr| tr.dev_idx() as u16))
                .collect::<Vec<_>>()
        ),
        &df[0]
    );
    assert_eq!(
        &Column::new(
            "tr_idx".into(),
            &emulator
                .iter()
                .flat_map(|dev| dev.iter().map(|tr| tr.idx() as u8))
                .collect::<Vec<_>>()
        ),
        &df[1]
    );
    assert_eq!(
        &Column::new(
            "x[mm]".into(),
            &emulator
                .iter()
                .flat_map(|dev| dev.iter().map(|tr| tr.position().x))
                .collect::<Vec<_>>()
        ),
        &df[2]
    );
    assert_eq!(
        &Column::new(
            "y[mm]".into(),
            &emulator
                .iter()
                .flat_map(|dev| dev.iter().map(|tr| tr.position().y))
                .collect::<Vec<_>>()
        ),
        &df[3]
    );
    assert_eq!(
        &Column::new(
            "z[mm]".into(),
            &emulator
                .iter()
                .flat_map(|dev| dev.iter().map(|tr| tr.position().z))
                .collect::<Vec<_>>()
        ),
        &df[4]
    );
    assert_eq!(
        &Column::new(
            "nx".into(),
            &emulator
                .iter()
                .flat_map(|dev| dev
                    .iter()
                    .map(|tr| emulator[tr.dev_idx()].axial_direction().x))
                .collect::<Vec<_>>()
        ),
        &df[5]
    );
    assert_eq!(
        &Column::new(
            "ny".into(),
            &emulator
                .iter()
                .flat_map(|dev| dev
                    .iter()
                    .map(|tr| emulator[tr.dev_idx()].axial_direction().y))
                .collect::<Vec<_>>()
        ),
        &df[6]
    );
    assert_eq!(
        &Column::new(
            "nz".into(),
            &emulator
                .iter()
                .flat_map(|dev| dev
                    .iter()
                    .map(|tr| emulator[tr.dev_idx()].axial_direction().z))
                .collect::<Vec<_>>()
        ),
        &df[7]
    );
}
