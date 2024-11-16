use autd3::prelude::*;
use autd3_emulator::*;
use polars::prelude::Column;

#[tokio::test]
async fn record_output_ultrasound() -> anyhow::Result<()> {
    let emulator =
        Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
            .into_emulator();

    let record = emulator
        .record(|mut autd| async {
            autd.send(Silencer::disable()).await?;
            autd.send(Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(30 * ULTRASOUND_PERIOD)?;
            Ok(autd)
        })
        .await?;

    let df = record.output_ultrasound();

    assert_eq!((emulator.num_transducers(), 5 + 30 * 256), df.shape());

    df.get_column_names()
        .into_iter()
        .skip(5)
        .enumerate()
        .for_each(|(i, n)| {
            assert_eq!(
                i,
                n.as_str()
                    .replace("p[a.u.]@", "")
                    .replace("[25us/256]", "")
                    .parse::<usize>()
                    .unwrap()
            )
        });

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

    // TODO: check the value

    Ok(())
}
