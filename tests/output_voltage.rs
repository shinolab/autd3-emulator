use autd3::prelude::*;
use autd3_emulator::*;

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

    let v = record.output_voltage();
    v["time[25us/256]"]
        .u64()?
        .into_no_null_iter()
        .enumerate()
        .for_each(|(i, t)| assert_eq!(i as u64, t));
    let expect_1 = [vec![12.; 64], vec![-12.; 128], vec![12.; 64]].concat();
    let expect_2 = [vec![-12.; 64], vec![12.; 128], vec![-12.; 64]].concat();
    let expect_3 = [vec![-12.; 96], vec![12.; 64], vec![-12.; 96]].concat();
    let expect_4 = vec![-12.; 256];
    let expect = [expect_1, expect_2, expect_3, expect_4].concat();
    emulator.iter().for_each(|dev| {
        dev.iter().for_each(|tr| {
            assert_eq!(
                expect,
                v[format!("voltage_{}_{}[V]", tr.dev_idx(), tr.idx()).as_str()]
                    .f32()
                    .unwrap()
                    .into_no_null_iter()
                    .collect::<Vec<_>>()
            );
        });
    });

    Ok(())
}