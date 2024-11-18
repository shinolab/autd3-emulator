use autd3::driver::geometry::Device;

pub fn clone_device(dev: &Device) -> Device {
    let enable = dev.enable;
    let sound_speed = dev.sound_speed;
    let mut dev = Device::new(
        dev.idx() as _,
        *dev.rotation(),
        dev.iter().cloned().collect(),
    );
    dev.enable = enable;
    dev.sound_speed = sound_speed;
    dev
}
