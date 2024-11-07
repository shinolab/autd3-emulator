use autd3::{derive::Datagram, gain, prelude::*};
use autd3_emulator::*;

use polars::prelude::{df, Column};

#[rstest::rstest]
#[case(Silencer::disable())]
#[case(Silencer::disable().with_target(SilencerTarget::PulseWidth))]
#[tokio::test]
async fn record_drive(#[case] silencer: impl Datagram) -> anyhow::Result<()> {
    let emulator =
        Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
            .into_emulator();

    let to_pulse_width = |a, b| {
        let i = (a as usize * b as usize) / 255;
        ((((i as f32) / 255.).asin() / PI) * 256.).round() as u8
    };
    let mut expect = df!(
        "time[ns]" => [0u64, 25000, 50000],
    )
    .unwrap();
    let series = emulator
        .geometry()
        .iter()
        .flat_map(|dev| {
            dev.iter().flat_map(|tr| {
                let dev_idx = tr.dev_idx() as u8;
                let tr_idx = tr.idx() as u8;
                let pulse_width = Column::new(
                    format!("pulsewidth_{}_{}", dev_idx, tr_idx).into(),
                    &[
                        to_pulse_width(100, tr_idx),
                        to_pulse_width(200, tr_idx),
                        to_pulse_width(200, tr_idx),
                    ],
                );
                let phase = Column::new(
                    format!("phase_{}_{}", dev_idx, tr_idx).into(),
                    &[dev_idx, (0x01 + dev_idx) as _, (0x01 + dev_idx) as _],
                );
                [phase, pulse_width]
            })
        })
        .collect::<Vec<_>>();
    expect.hstack_mut(&series).unwrap();

    let record = emulator
        .record(|mut autd| async {
            autd.send(silencer).await?;
            autd.send((
                Static::with_intensity(100),
                gain::Custom::new(|_| {
                    |tr| {
                        (
                            Phase::new(tr.dev_idx() as _),
                            EmitIntensity::new(tr.idx() as _),
                        )
                    }
                }),
            ))
            .await?;
            autd.tick(ULTRASOUND_PERIOD)?;
            autd.send((
                Static::with_intensity(200),
                gain::Custom::new(|_| {
                    |tr| {
                        (
                            Phase::new(0x01) + Phase::new(tr.dev_idx() as _),
                            EmitIntensity::new(tr.idx() as _),
                        )
                    }
                }),
            ))
            .await?;
            autd.tick(2 * ULTRASOUND_PERIOD)?;

            Ok(autd)
        })
        .await?;

    assert_eq!(expect, record.drive());

    Ok(())
}
