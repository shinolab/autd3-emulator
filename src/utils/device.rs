use autd3::driver::geometry::Device;

pub fn clone_device(dev: &Device) -> Device {
    Device::new(
        dev.idx() as _,
        *dev.rotation(),
        dev.iter().cloned().collect(),
    )
}
