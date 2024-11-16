use autd3::prelude::*;
use autd3_emulator::*;
use polars::prelude::Column;

#[tokio::test]
async fn record_output_voltage() -> anyhow::Result<()> {
    let emulator =
        Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
            .into_emulator();

    let record = emulator
        .record(|mut autd| async {
            autd.send(Silencer::disable()).await?;
            autd.send(PulseWidthEncoder::new(|_dev| {
                |i| match i {
                    0x80 => 64,
                    0xFF => 128,
                    _ => 0,
                }
            }))
            .await?;
            autd.send(Uniform::new((Phase::new(0x00), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(ULTRASOUND_PERIOD)?;
            autd.send(Uniform::new((Phase::new(0x80), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(ULTRASOUND_PERIOD)?;
            autd.send(Uniform::new((Phase::new(0x80), EmitIntensity::new(0x80))))
                .await?;
            autd.tick(ULTRASOUND_PERIOD)?;
            autd.send(Uniform::new((Phase::new(0x00), EmitIntensity::new(0x00))))
                .await?;
            autd.tick(ULTRASOUND_PERIOD)?;
            Ok(autd)
        })
        .await?;

    let df = record.output_voltage();

    assert_eq!((emulator.num_transducers(), 5 + 4 * 256), df.shape());

    df.get_column_names()
        .into_iter()
        .skip(5)
        .enumerate()
        .for_each(|(i, n)| {
            assert_eq!(
                i,
                n.as_str()
                    .replace("voltage[V]@", "")
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

    let expect_1 = [vec![12.; 64], vec![-12.; 128], vec![12.; 64]].concat();
    let expect_2 = [vec![-12.; 64], vec![12.; 128], vec![-12.; 64]].concat();
    let expect_3 = [vec![-12.; 96], vec![12.; 64], vec![-12.; 96]].concat();
    let expect_4 = vec![-12.; 256];
    let expect = [expect_1, expect_2, expect_3, expect_4].concat();
    df.iter()
        .skip(5)
        .zip(expect.into_iter())
        .for_each(|(c, expect)| {
            assert!(c.f32().unwrap().into_no_null_iter().all(|v| v == expect));
        });

    Ok(())
}
