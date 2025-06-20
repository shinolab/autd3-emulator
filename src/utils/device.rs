use autd3::driver::geometry::Device;

pub fn clone_device(dev: &Device) -> Device {
    let sound_speed = dev.sound_speed;
    let mut dev = Device::new(*dev.rotation(), dev.iter().cloned().collect());
    dev.sound_speed = sound_speed;
    dev
}
