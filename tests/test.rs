mod recording;

use autd3::{
    prelude::{Vector3, AUTD3},
    Controller,
};
use autd3_link_calc::Calc;

#[tokio::test]
async fn update() -> anyhow::Result<()> {
    let mut autd = Controller::builder([AUTD3::new(Vector3::zeros())])
        .open(Calc::builder())
        .await?;

    autd.geometry_mut()[0].translate(Vector3::new(10., 0., 0.));
    let df = autd.gain();
    assert_eq!(0., df["x"].f32()?.into_no_null_iter().next().unwrap());

    autd.send(autd3::gain::Null::new()).await?;
    let df = autd.gain();
    assert_eq!(10., df["x"].f32()?.into_no_null_iter().next().unwrap());

    autd.close().await?;

    Ok(())
}
