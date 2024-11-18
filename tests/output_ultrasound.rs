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

    assert_eq!((emulator.num_transducers(), 30 * 256), df.shape());

    df.get_column_names()
        .into_iter()
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

    // TODO: check the value

    Ok(())
}
