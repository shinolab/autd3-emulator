use autd3::{driver::defined::ultrasound_period, prelude::*};
use autd3_emulator::*;

#[test]
fn record_output_ultrasound() -> anyhow::Result<()> {
    let emulator =
        Controller::builder([AUTD3::new(Point3::origin()), AUTD3::new(Point3::origin())])
            .into_emulator();

    let record = emulator.record(|autd| {
        autd.send(Silencer::disable())?;
        autd.send(Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))))?;
        autd.tick(30 * ultrasound_period())?;
        Ok(())
    })?;

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
