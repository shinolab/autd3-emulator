use std::time::Duration;

use autd3::{derive::Datagram, prelude::*, Controller};
use autd3_driver::firmware::fpga::FPGA_MAIN_CLK_FREQ;
use autd3_link_emulator::{recording::RecordOption, Emulator};
use polars::prelude::df;

#[rstest::rstest]
#[case(Silencer::disable())]
#[case(Silencer::disable().with_target(SilencerTarget::PulseWidth))]
#[tokio::test]
async fn record_drive(#[case] silencer: impl Datagram) -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Emulator::builder())
        .await?;

    autd.send(silencer).await?;
    autd.start_recording()?;
    autd.send((
        Static::with_intensity(100),
        Uniform::new((EmitIntensity::new(51), Phase::new(0x01))),
    ))
    .await?;
    autd.tick(ULTRASOUND_PERIOD)?;
    autd.send((
        Static::with_intensity(30),
        Uniform::new((EmitIntensity::new(255), Phase::new(0x02))),
    ))
    .await?;
    autd.tick(2 * ULTRASOUND_PERIOD)?;
    let record = autd.finish_recording()?;

    let to_pulse_width = |a, b| {
        let i = (a as usize * b as usize) / 255;
        ((((i as f32) / 255.).asin() / PI) * 256.).round() as u8
    };
    assert_eq!(
        df!(
            "time[s]" => [0f32, 25. / 1e6, 50. / 1e6],
            "phase" => [0x01u8, 0x02, 0x02],
            "pulsewidth" => [to_pulse_width(100, 51), to_pulse_width(30, 255), to_pulse_width(30, 255)]
        ).unwrap(),
        record[0][0].drive()
    );

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

    let v = record[0][0].output_voltage();
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
    assert_eq!(
        [expect_1, expect_2, expect_3, expect_4].concat(),
        v["voltage[V]"]
            .f32()?
            .into_no_null_iter()
            .collect::<Vec<_>>()
    );

    assert_eq!(
        Err(autd3_link_emulator::error::EmulatorError::InvalidDuration),
        record[0][0].output_voltage_within(Duration::ZERO, Duration::from_micros(1))
    );
    assert_eq!(
        Err(autd3_link_emulator::error::EmulatorError::InvalidDuration),
        record[0][0].output_voltage_within(Duration::from_micros(1), Duration::from_micros(25))
    );

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

    let v = record[0][0]
        .output_ultrasound()
        .next(30 * ULTRASOUND_PERIOD)?;

    v["time[s]"]
        .f32()?
        .into_no_null_iter()
        .enumerate()
        .for_each(|(i, t)| {
            approx::assert_abs_diff_eq!(i as f32 * (1. / FPGA_MAIN_CLK_FREQ.hz() as f32), t)
        });

    // TODO
    // assert_eq!(
    //     vec![],
    //     v["p[a.u.]"].f32()?.into_no_null_iter().collect::<Vec<_>>()
    // );
    assert_eq!(30 * 256, v["p[a.u.]"].f32()?.iter().count());

    assert_eq!(
        Err(autd3_link_emulator::error::EmulatorError::InvalidDuration),
        record[0][0]
            .output_ultrasound()
            .next(Duration::from_micros(1))
    );

    autd.close().await?;

    Ok(())
}

#[tokio::test]
async fn record_output_ultrasound_resume() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Emulator::builder())
        .await?;

    autd.send(Silencer::disable()).await?;
    autd.start_recording()?;
    autd.send(Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))))
        .await?;
    autd.tick(30 * ULTRASOUND_PERIOD)?;
    let record = autd.finish_recording()?;

    let expect = record[0][0]
        .output_ultrasound()
        .next(30 * ULTRASOUND_PERIOD)?;

    let mut output_ultrasound = record[0][0].output_ultrasound();
    let mut v1 = output_ultrasound.next(15 * ULTRASOUND_PERIOD)?;
    let v2 = output_ultrasound.next(15 * ULTRASOUND_PERIOD)?;
    v1.vstack_mut(&v2).unwrap();

    assert_eq!(expect, v1);

    autd.close().await?;

    Ok(())
}

#[tokio::test]
async fn record_sound_field() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Emulator::builder())
        .await?;

    autd.send(Silencer::disable()).await?;
    autd.start_recording()?;
    autd.send(Uniform::new((Phase::new(0x40), EmitIntensity::new(0xFF))))
        .await?;
    autd.tick(2 * ULTRASOUND_PERIOD)?;
    let record = autd.finish_recording()?;

    let point = Vector3::new(0., 0., 150. * mm);
    let df = record[0][0].sound_pressure(
        &point,
        Duration::ZERO..30 * ULTRASOUND_PERIOD,
        RecordOption {
            time_step: Duration::from_micros(1),
            ..Default::default()
        },
    )?;

    df["time[s]"]
        .f32()?
        .into_no_null_iter()
        .enumerate()
        .for_each(|(i, t)| {
            approx::assert_abs_diff_eq!(i as f32 * Duration::from_micros(1).as_secs_f32(), t)
        });

    // TODO
    // assert_eq!(
    //     vec![],
    //     df["p[Pa]@(0,0,150)"]
    //         .f32()?
    //         .into_no_null_iter()
    //         .collect::<Vec<_>>()
    // );
    assert_eq!(
        30 * ULTRASOUND_PERIOD.as_micros() as usize,
        df["p[Pa]@(0,0,150)"].f32()?.iter().count()
    );

    assert_eq!(
        Err(autd3_link_emulator::error::EmulatorError::InvalidDuration),
        record[0][0].sound_pressure(
            &point,
            Duration::ZERO..Duration::from_micros(1),
            RecordOption::default(),
        )
    );

    autd.close().await?;

    Ok(())
}
