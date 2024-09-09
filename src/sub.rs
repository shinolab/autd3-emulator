use autd3_driver::{defined::ULTRASOUND_PERIOD, geometry::Device};
use autd3_firmware_emulator::CPUEmulator;
use polars::prelude::*;

pub struct SubDevice {
    pub(crate) device: Device,
    pub(crate) cpu: CPUEmulator,
}

impl SubDevice {
    pub fn gain(&self) -> DataFrame {
        let x = self
            .device
            .iter()
            .map(|tr| tr.position().x)
            .collect::<Vec<_>>();
        let y = self
            .device
            .iter()
            .map(|tr| tr.position().y)
            .collect::<Vec<_>>();
        let z = self
            .device
            .iter()
            .map(|tr| tr.position().z)
            .collect::<Vec<_>>();
        let w = vec![self.device.rotation().w; self.device.num_transducers()];
        let i = vec![self.device.rotation().i; self.device.num_transducers()];
        let j = vec![self.device.rotation().j; self.device.num_transducers()];
        let k = vec![self.device.rotation().k; self.device.num_transducers()];
        let phase = self
            .cpu
            .fpga()
            .drives()
            .iter()
            .map(|d| d.phase().value())
            .collect::<Vec<_>>();
        let intensity = self
            .cpu
            .fpga()
            .drives()
            .iter()
            .map(|d| d.intensity().value())
            .collect::<Vec<_>>();
        df!(
            "x" => &x,
            "y" => &y,
            "z" => &z,
            "w" => &w,
            "i" => &i,
            "j" => &j,
            "k" => &k,
            "phase" => &phase,
            "intensity" => &intensity
        )
        .unwrap()
    }

    pub fn modulation(&self) -> DataFrame {
        let segment = self.cpu.fpga().current_mod_segment();
        let cycle = self.cpu.fpga().modulation_cycle(segment);
        let m = (0..cycle)
            .flat_map(move |i| {
                std::iter::repeat(self.cpu.fpga().modulation_at(segment, i))
                    .take(self.cpu.fpga().modulation_freq_division(segment) as _)
            })
            .collect::<Vec<_>>();
        let t = (0..m.len())
            .map(|i| (i as u32 * ULTRASOUND_PERIOD).as_secs_f32())
            .collect::<Vec<_>>();
        df!(
            "time[s]" => &t,
            "modulation" => &m
        )
        .unwrap()
    }
}
