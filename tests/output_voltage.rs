use autd3::{driver::defined::ultrasound_period, prelude::*};
use autd3_emulator::*;

#[tokio::test]
async fn record_output_voltage() -> anyhow::Result<()> {
    let emulator =
        Controller::builder([AUTD3::new(Point3::origin()), AUTD3::new(Point3::origin())])
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
            autd.tick(ultrasound_period())?;
            autd.send(Uniform::new((Phase::new(0x80), EmitIntensity::new(0xFF))))
                .await?;
            autd.tick(ultrasound_period())?;
            autd.send(Uniform::new((Phase::new(0x80), EmitIntensity::new(0x80))))
                .await?;
            autd.tick(ultrasound_period())?;
            autd.send(Uniform::new((Phase::new(0x00), EmitIntensity::new(0x00))))
                .await?;
            autd.tick(ultrasound_period())?;
            Ok(autd)
        })
        .await?;

    let df = record.output_voltage();

    assert_eq!((emulator.num_transducers(), 4 * 256), df.shape());

    df.get_column_names()
        .into_iter()
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

    let expect_1 = [vec![12.; 64], vec![-12.; 128], vec![12.; 64]].concat();
    let expect_2 = [vec![-12.; 64], vec![12.; 128], vec![-12.; 64]].concat();
    let expect_3 = [vec![-12.; 96], vec![12.; 64], vec![-12.; 96]].concat();
    let expect_4 = vec![-12.; 256];
    let expect = [expect_1, expect_2, expect_3, expect_4].concat();
    df.iter().zip(expect.into_iter()).for_each(|(c, expect)| {
        assert!(c.f32().unwrap().into_no_null_iter().all(|v| v == expect));
    });

    Ok(())
}
