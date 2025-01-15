use autd3::{
    core::datagram::Datagram,
    driver::{
        defined::ultrasound_period,
        firmware::operation::{Operation, OperationGenerator},
    },
    gain,
    prelude::*,
};
use autd3_emulator::*;

use polars::{frame::DataFrame, prelude::Column};

#[rstest::rstest]
#[case(Silencer::disable())]
#[case(Silencer::disable().with_target(SilencerTarget::PulseWidth))]
#[test]
fn record_phase<D: Datagram>(#[case] silencer: D) -> anyhow::Result<()>
where
    AUTDDriverError: From<D::Error>,
    D::G: OperationGenerator,
    AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
        + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
{
    let emulator =
        Controller::builder([AUTD3::new(Point3::origin()), AUTD3::new(Point3::origin())])
            .into_emulator();

    let expect = DataFrame::new(
        [0, 25000, 50000]
            .iter()
            .zip([0, 1, 1])
            .map(|(t, o)| {
                Column::new(
                    format!("pulse_width@{}[ns]", t).into(),
                    emulator
                        .iter()
                        .flat_map(|dev| {
                            dev.iter()
                                .map(|tr| (Phase::new(o) + Phase::new(tr.dev_idx() as _)).value())
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect(),
    )
    .unwrap();

    let record = emulator.record(|autd| {
        autd.send(silencer)?;
        autd.send((
            Static::with_intensity(100),
            gain::Custom::new(|_| |tr| Phase::new(tr.dev_idx() as _)),
        ))?;
        autd.tick(ultrasound_period())?;
        autd.send((
            Static::with_intensity(200),
            gain::Custom::new(|_| |tr| Phase::new(0x01) + Phase::new(tr.dev_idx() as _)),
        ))?;
        autd.tick(2 * ultrasound_period())?;

        Ok(())
    })?;

    assert_eq!(expect, record.phase());

    Ok(())
}

#[rstest::rstest]
#[case(Silencer::disable())]
#[case(Silencer::disable().with_target(SilencerTarget::PulseWidth))]
#[test]
fn record_pulse_width<D: Datagram>(#[case] silencer: D) -> anyhow::Result<()>
where
    AUTDDriverError: From<D::Error>,
    D::G: OperationGenerator,
    AUTDDriverError: From<<<D::G as OperationGenerator>::O1 as Operation>::Error>
        + From<<<D::G as OperationGenerator>::O2 as Operation>::Error>,
{
    use polars::frame::DataFrame;

    let emulator =
        Controller::builder([AUTD3::new(Point3::origin()), AUTD3::new(Point3::origin())])
            .into_emulator();

    let to_pulse_width = |a, b| {
        let i = (a as usize * b) / 255;
        ((((i as f32) / 255.).asin() / PI) * 256.).round() as u8
    };

    let expect = DataFrame::new(
        [0, 25000, 50000]
            .iter()
            .zip([100, 200, 200])
            .map(|(t, i)| {
                Column::new(
                    format!("pulse_width@{}[ns]", t).into(),
                    emulator
                        .iter()
                        .flat_map(|dev| dev.iter().map(|tr| to_pulse_width(i, tr.idx())))
                        .collect::<Vec<_>>(),
                )
            })
            .collect(),
    )
    .unwrap();

    let record = emulator.record(|autd| {
        autd.send(silencer)?;
        autd.send((
            Static::with_intensity(100),
            gain::Custom::new(|_| |tr| EmitIntensity::new(tr.idx() as _)),
        ))?;
        autd.tick(ultrasound_period())?;
        autd.send((
            Static::with_intensity(200),
            gain::Custom::new(|_| |tr| EmitIntensity::new(tr.idx() as _)),
        ))?;
        autd.tick(2 * ultrasound_period())?;

        Ok(())
    })?;

    assert_eq!(expect, record.pulse_width());

    Ok(())
}
