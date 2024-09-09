use autd3::{
    prelude::{Drive, EmitIntensity, Phase, Silencer, Vector3, AUTD3, ULTRASOUND_PERIOD},
    Controller,
};
use autd3_link_calc::Calc;
use polars::df;

#[tokio::test]
async fn gain() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    let df = autd.gain();
    assert!(autd
        .geometry()
        .iter()
        .flat_map(|dev| dev.iter())
        .zip(df["x"].f32()?.into_no_null_iter())
        .all(|(tr, x)| tr.position().x == x));
    assert!(autd
        .geometry()
        .iter()
        .flat_map(|dev| dev.iter())
        .zip(df["y"].f32()?.into_no_null_iter())
        .all(|(tr, y)| tr.position().y == y));
    assert!(autd
        .geometry()
        .iter()
        .flat_map(|dev| dev.iter())
        .zip(df["z"].f32()?.into_no_null_iter())
        .all(|(tr, z)| tr.position().z == z));
    let rot = autd.geometry()[0].rotation();
    assert!(df["w"].f32()?.into_no_null_iter().all(|w| rot.w == w));
    assert!(df["i"].f32()?.into_no_null_iter().all(|i| rot.i == i));
    assert!(df["j"].f32()?.into_no_null_iter().all(|j| rot.j == j));
    assert!(df["k"].f32()?.into_no_null_iter().all(|k| rot.k == k));
    assert!(df["phase"].u8()?.into_no_null_iter().all(|k| 0 == k));
    assert!(df["intensity"].u8()?.into_no_null_iter().all(|k| 0 == k));

    let expect = Drive::new(Phase::new(0x80), EmitIntensity::new(0x81));
    autd.send(autd3::gain::Uniform::new(expect)).await?;
    let df = autd.gain();
    assert!(df["phase"]
        .u8()?
        .into_no_null_iter()
        .all(|k| expect.phase().value() == k));
    assert!(df["intensity"]
        .u8()?
        .into_no_null_iter()
        .all(|k| expect.intensity().value() == k));

    Ok(())
}

#[tokio::test]
async fn modulation() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    let t = (0..65535 * 2)
        .map(|i| (i * ULTRASOUND_PERIOD).as_secs_f32())
        .collect::<Vec<_>>();
    assert_eq!(
        df!(
            "time[s]" => &t,
            "modulation" => &vec![0xFFu8; t.len()]
        )
        .unwrap(),
        autd[0].modulation()
    );

    let expect = vec![0x80, 0x81];
    autd.send((
        Silencer::disable(),
        autd3::modulation::Custom::new(expect, 2 * ULTRASOUND_PERIOD)?,
    ))
    .await?;
    let t = (0..4)
        .map(|i| (i * ULTRASOUND_PERIOD).as_secs_f32())
        .collect::<Vec<_>>();
    assert_eq!(
        df!(
            "time[s]" => &t,
            "modulation" => &vec![0x80, 0x80, 0x81, 0x81]
        )
        .unwrap(),
        autd[0].modulation()
    );

    autd.close().await?;

    Ok(())
}
