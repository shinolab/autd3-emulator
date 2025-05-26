use autd3::{driver::common::ULTRASOUND_PERIOD, prelude::*};
use autd3_emulator::*;

#[test]
fn record_output_voltage() -> anyhow::Result<()> {
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
        autd.send(PulseWidthEncoder::new(|_dev| {
            |i| match i {
                EmitIntensity(0x80) => PulseWidth::new(128).unwrap(),
                EmitIntensity(0xFF) => PulseWidth::new(256).unwrap(),
                _ => PulseWidth::new(0).unwrap(),
            }
        }))?;
        autd.send(Uniform {
            phase: Phase(0x00),
            intensity: EmitIntensity(0xFF),
        })?;
        autd.tick(ULTRASOUND_PERIOD)?;
        autd.send(Uniform {
            phase: Phase(0x80),
            intensity: EmitIntensity(0xFF),
        })?;
        autd.tick(ULTRASOUND_PERIOD)?;
        autd.send(Uniform {
            phase: Phase(0x80),
            intensity: EmitIntensity(0x80),
        })?;
        autd.tick(ULTRASOUND_PERIOD)?;
        autd.send(Uniform {
            phase: Phase(0x00),
            intensity: EmitIntensity(0x00),
        })?;
        autd.tick(ULTRASOUND_PERIOD)?;
        Ok(())
    })?;

    let df = record.output_voltage();

    assert_eq!((emulator.num_transducers(), 4 * 512), df.shape());

    df.get_column_names()
        .into_iter()
        .enumerate()
        .for_each(|(i, n)| {
            assert_eq!(
                i,
                n.as_str()
                    .replace("voltage[V]@", "")
                    .replace("[25us/512]", "")
                    .parse::<usize>()
                    .unwrap()
            )
        });

    let expect_1 = [vec![12.; 128], vec![-12.; 256], vec![12.; 128]].concat();
    let expect_2 = [vec![-12.; 128], vec![12.; 256], vec![-12.; 128]].concat();
    let expect_3 = [vec![-12.; 192], vec![12.; 128], vec![-12.; 192]].concat();
    let expect_4 = vec![-12.; 512];
    let expect = [expect_1, expect_2, expect_3, expect_4].concat();
    df.iter().zip(expect).for_each(|(c, expect)| {
        assert!(c.f32().unwrap().into_no_null_iter().all(|v| v == expect));
    });

    Ok(())
}
