use autd3::prelude::*;
use autd3_emulator::*;

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
    df["time[25us/256]"]
        .u64()?
        .into_no_null_iter()
        .enumerate()
        .for_each(|(i, t)| assert_eq!(i as u64, t));
    emulator.iter().for_each(|dev| {
        dev.iter().for_each(|tr| {
            // TODO: check the value
            assert_eq!(
                30 * 256,
                df[format!("p_{}_{}[a.u.]", tr.dev_idx(), tr.idx()).as_str()]
                    .f32()
                    .unwrap()
                    .iter()
                    .count()
            );
        });
    });

    Ok(())
}
