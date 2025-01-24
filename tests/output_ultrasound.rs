use autd3::{driver::defined::ultrasound_period, prelude::*};
use autd3_emulator::*;

#[test]
fn record_output_ultrasound() -> anyhow::Result<()> {
    let emulator = Emulator::new([
        AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        },
        AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        },
    ]);

    let record = emulator.record(|autd| {
        autd.send(Silencer::disable())?;
        autd.send(Uniform {
            phase: Phase(0x40),
            intensity: EmitIntensity(0xFF),
        })?;
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
