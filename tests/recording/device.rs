use autd3::{gain, prelude::*, Controller};
use autd3_driver::firmware::fpga::FPGA_MAIN_CLK_FREQ;
use autd3_link_emulator::Emulator;
use polars::prelude::*;

#[tokio::test]
async fn record_drive() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Emulator::builder())
        .await?;

    let to_pulse_width = |a, b| {
        let i = (a as usize * b as usize) / 255;
        ((((i as f32) / 255.).asin() / PI) * 256.).round() as u8
    };
    let expect = autd.geometry()[0].iter().fold(
        df!(
            "time[s]" => [0f32, 25. / 1e6, 50. / 1e6],
        )
        .unwrap(),
        |acc, tr| {
            let i = tr.idx() as u8;
            let pulse_width = Series::new(
                format!("pulsewidth_{}", i).into(),
                &[
                    to_pulse_width(100, i),
                    to_pulse_width(200, i),
                    to_pulse_width(200, i),
                ],
            );
            let phase = Series::new(
                format!("phase_{}", i).into(),
                &[i as u8, (0x80 + i) as _, (0x80 + i) as _]
                    .iter()
                    .copied()
                    .collect::<Vec<_>>(),
            );
            acc.hstack(&[phase, pulse_width]).unwrap()
        },
    );

    autd.send(Silencer::disable()).await?;
    autd.start_recording()?;
    autd.send((
        Static::with_intensity(100),
        gain::Custom::new(|_| |tr| (Phase::new(tr.idx() as _), EmitIntensity::new(tr.idx() as _))),
    ))
    .await?;
    autd.tick(ULTRASOUND_PERIOD)?;
    autd.send((
        Static::with_intensity(200),
        gain::Custom::new(|_| {
            |tr| {
                (
                    Phase::PI + Phase::new(tr.idx() as _),
                    EmitIntensity::new(tr.idx() as _),
                )
            }
        }),
    ))
    .await?;
    autd.tick(2 * ULTRASOUND_PERIOD)?;
    let record = autd.finish_recording()?;

    assert_eq!(expect, record[0].drive());

    autd.close().await?;

    Ok(())
}

#[tokio::test]
async fn record_output_voltage() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Emulator::builder())
        .await?;

    autd.send(Silencer::disable()).await?;
    autd.send(PulseWidthEncoder::new(|_dev| {
        |i| match i {
            0x80 => 64,
            0xFF => 128,
            _ => 0,
        }
    }))
    .await?;
    autd.start_recording()?;
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
    let record = autd.finish_recording()?;

    let v = record[0].output_voltage();
    v["time[s]"]
        .f32()?
        .into_no_null_iter()
        .enumerate()
        .for_each(|(i, t)| {
            approx::assert_abs_diff_eq!(i as f32 * (1. / FPGA_MAIN_CLK_FREQ.hz() as f32), t)
        });
    let expect_1 = [vec![12.; 64], vec![-12.; 128], vec![12.; 64]].concat();
    let expect_2 = [vec![-12.; 64], vec![12.; 128], vec![-12.; 64]].concat();
    let expect_3 = [vec![-12.; 96], vec![12.; 64], vec![-12.; 96]].concat();
    let expect_4 = vec![-12.; 256];
    let expect = [expect_1, expect_2, expect_3, expect_4].concat();
    autd.geometry()[0].iter().for_each(|tr| {
        assert_eq!(
            expect,
            v[1 + tr.idx()]
                .f32()
                .unwrap()
                .into_no_null_iter()
                .collect::<Vec<_>>()
        );
    });

    autd.close().await?;

    Ok(())
}

#[tokio::test]
async fn record_output_ultrasound() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Emulator::builder())
        .await?;

    autd.send(Silencer::disable()).await?;
    autd.start_recording()?;
    autd.send(Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))))
        .await?;
    autd.tick(30 * ULTRASOUND_PERIOD)?;
    let record = autd.finish_recording()?;

    let mut v = record[0].output_ultrasound();
    let df = v.next(30 * ULTRASOUND_PERIOD)?;
    df["time[s]"]
        .f32()?
        .into_no_null_iter()
        .enumerate()
        .for_each(|(i, t)| {
            approx::assert_abs_diff_eq!(i as f32 * (1. / FPGA_MAIN_CLK_FREQ.hz() as f32), t)
        });
    autd.geometry()[0].iter().for_each(|tr| {
        // TODO
        // assert_eq!(
        //     vec![],
        //     df["p[a.u.]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
        // );
        assert_eq!(30 * 256, df[1 + tr.idx()].f32().unwrap().iter().count());
    });

    autd.close().await?;

    Ok(())
}
