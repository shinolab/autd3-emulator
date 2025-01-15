use autd3::{driver::defined::ultrasound_period, prelude::*};
use autd3_emulator::*;

#[test]
fn record_output_voltage() -> anyhow::Result<()> {
    let emulator =
        Controller::builder([AUTD3::new(Point3::origin()), AUTD3::new(Point3::origin())])
            .into_emulator();

    let record = emulator.record(|autd| {
        autd.send(Silencer::disable())?;
        autd.send(PulseWidthEncoder::new(|_dev| {
            |i| match i {
                0x80 => 64,
                0xFF => 128,
                _ => 0,
            }
        }))?;
        autd.send(Uniform::new((Phase::new(0x00), EmitIntensity::new(0xFF))))?;
        autd.tick(ultrasound_period())?;
        autd.send(Uniform::new((Phase::new(0x80), EmitIntensity::new(0xFF))))?;
        autd.tick(ultrasound_period())?;
        autd.send(Uniform::new((Phase::new(0x80), EmitIntensity::new(0x80))))?;
        autd.tick(ultrasound_period())?;
        autd.send(Uniform::new((Phase::new(0x00), EmitIntensity::new(0x00))))?;
        autd.tick(ultrasound_period())?;
        Ok(())
    })?;

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
    df.iter().zip(expect).for_each(|(c, expect)| {
        assert!(c.f32().unwrap().into_no_null_iter().all(|v| v == expect));
    });

    Ok(())
}
