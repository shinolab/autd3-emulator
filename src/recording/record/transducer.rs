use autd3_firmware_emulator::FPGAEmulator;
use polars::prelude::*;

use autd3_driver::{
    defined::{ULTRASOUND_FREQ, ULTRASOUND_PERIOD, ULTRASOUND_PERIOD_COUNT},
    derive::Builder,
    firmware::fpga::Drive,
    firmware::fpga::SilencerTarget,
};

use derive_more::Debug;

#[derive(Builder, Debug)]
pub struct TransducerRecord<'a> {
    pub(crate) drive: Vec<Drive>,
    pub(crate) modulation: Vec<u8>,
    #[debug(skip)]
    pub(crate) fpga: &'a FPGAEmulator,
}

impl<'a> TransducerRecord<'a> {
    const TS: f32 = 1. / (ULTRASOUND_FREQ.hz() as f32 * ULTRASOUND_PERIOD_COUNT as f32);

    pub(crate) fn time(n: usize) -> Series {
        (0..n)
            .map(|i| (i as u32 * ULTRASOUND_PERIOD).as_secs_f32())
            .collect()
    }

    pub fn drive(&self) -> DataFrame {
        let time = Self::time(self.drive.len());
        let phase = self
            .drive
            .iter()
            .map(|d| d.phase().value())
            .collect::<Vec<_>>();
        let intensity = self
            .drive
            .iter()
            .map(|d| d.intensity().value())
            .collect::<Vec<_>>();
        df!(
            "time[s]" => &time,
            "phase" => &phase,
            "intensity" => &intensity
        )
        .unwrap()
    }

    pub fn modulation(&self) -> DataFrame {
        let time = Self::time(self.modulation.len());
        df!(
            "time[s]" => &time,
            "modulation" => &self
            .modulation
        )
        .unwrap()
    }

    fn pulse_width_after_silencer(&self) -> Vec<u8> {
        match self.fpga.silencer_target() {
            SilencerTarget::Intensity => {
                let intensity = self
                    .drive
                    .iter()
                    .zip(self.modulation.iter())
                    .map(|(d, &m)| ((d.intensity().value() as u16 * m as u16) / 255) as u8)
                    .collect::<Vec<_>>();
                self.fpga
                    .apply_silencer(0, &intensity, false)
                    .into_iter()
                    .map(|intensity| self.fpga.pulse_width_encoder_table_at(intensity as _))
                    .collect::<Vec<_>>()
            }
            SilencerTarget::PulseWidth => {
                let pulse_width = self
                    .drive
                    .iter()
                    .zip(self.modulation.iter())
                    .map(|(d, &m)| self.fpga.to_pulse_width(d.intensity(), m))
                    .collect::<Vec<_>>();
                self.fpga.apply_silencer(0, &pulse_width, false)
            }
        }
    }

    pub fn pulse_width(&self) -> DataFrame {
        let pulse_width = self.pulse_width_after_silencer();
        let time = Self::time(pulse_width.len());
        df!(
            "time[s]" => &time,
            "pulsewidth" => &pulse_width
        )
        .unwrap()
    }

    fn _output_voltage(&self) -> Vec<f32> {
        const V: f32 = 12.0;
        self.pulse_width_after_silencer()
            .into_iter()
            .zip(self.drive.iter())
            .flat_map(|(pw, d)| {
                let rise = d.phase().value().wrapping_sub(pw / 2);
                let fall = d.phase().value().wrapping_add(pw / 2 + (pw & 0x01));
                (0..=255u8).map(move |i| {
                    #[allow(clippy::collapsible_else_if)]
                    if rise <= fall {
                        if (rise <= i) && (i < fall) {
                            V
                        } else {
                            -V
                        }
                    } else {
                        if (i < fall) || (rise <= i) {
                            V
                        } else {
                            -V
                        }
                    }
                })
            })
            .collect::<Vec<_>>()
    }

    pub fn output_voltage(&self) -> DataFrame {
        let time = (0..self.modulation.len())
            .map(|i| i as u32 * ULTRASOUND_PERIOD)
            .flat_map(|t| (0..=255u8).map(move |i| t.as_secs_f32() + i as f32 * Self::TS))
            .collect::<Vec<_>>();
        let v = self._output_voltage();
        df!(
            "time[s]" => &time,
            "voltage[V]" => &v
        )
        .unwrap()
    }

    pub fn output_ultrasound(&self) -> DataFrame {
        let time = (0..self.modulation.len())
            .map(|i| i as u32 * ULTRASOUND_PERIOD)
            .flat_map(|t| (0..=255u8).map(move |i| t.as_secs_f32() + i as f32 * Self::TS))
            .collect::<Vec<_>>();
        df!(
            "time[s]" => &time,
            "p[a.u.]" => &T4010A1BVDModel {
                v: self._output_voltage(),
            }.rk4()
        )
        .unwrap()
    }
}

struct T4010A1BVDModel {
    v: Vec<f32>,
}

#[allow(non_upper_case_globals)]
impl T4010A1BVDModel {
    const Cs: f32 = 200e-9; // mF
    const L: f32 = 80e-6; // kH
    const R: f32 = 0.7; // kΩ
    const Cp: f32 = 2700e-9; // mF
    const Rd: f32 = 150e-3; // kΩ
    const h: f32 = TransducerRecord::TS;
    const NORMALIZE: f32 = 0.057_522_15;

    fn rk4(&self) -> Vec<f32> {
        (0..self.v.len())
            .scan((0., 0., 0.), |state, i| {
                let y = state.1 * Self::NORMALIZE;
                let k00 = Self::h * Self::f0(state);
                let k01 = Self::h * self.f1(2 * i, state);
                let k02 = Self::h * self.f2(2 * i, state);
                let y1 = (state.0 + k00 / 2., state.1 + k01 / 2., state.2 + k02 / 2.);
                let k10 = Self::h * Self::f0(&y1);
                let k11 = Self::h * self.f1(2 * i + 1, &y1);
                let k12 = Self::h * self.f2(2 * i + 1, &y1);
                let y2 = (state.0 + k10 / 2., state.1 + k11 / 2., state.2 + k12 / 2.);
                let k20 = Self::h * Self::f0(&y2);
                let k21 = Self::h * self.f1(2 * i + 1, &y2);
                let k22 = Self::h * self.f2(2 * i + 1, &y2);
                let y3 = (state.0 + k20, state.1 + k21, state.2 + k22);
                let k30 = Self::h * Self::f0(&y3);
                let k31 = Self::h * self.f1(2 * i + 2, &y3);
                let k32 = Self::h * self.f2(2 * i + 2, &y3);
                *state = (
                    state.0 + (k00 + 2. * k10 + 2. * k20 + k30) / 6.,
                    state.1 + (k01 + 2. * k11 + 2. * k21 + k31) / 6.,
                    state.2 + (k02 + 2. * k12 + 2. * k22 + k32) / 6.,
                );
                Some(y)
            })
            .collect()
    }

    fn f0(y: &(f32, f32, f32)) -> f32 {
        y.1
    }

    fn f1(&self, t: usize, y: &(f32, f32, f32)) -> f32 {
        -y.0 / (Self::L * Self::Cs)
            - (Self::R + Self::Rd) / Self::L * y.1
            - Self::Rd / Self::L * y.2
            + self.v.get(t / 2).unwrap_or(&0.) / Self::L
    }

    fn f2(&self, t: usize, y: &(f32, f32, f32)) -> f32 {
        let dt = match t {
            0 => 2. * self.v[0],
            1 => 0.,
            t if t / 2 + 1 < self.v.len() => self.v[t / 2 + 1] - self.v[t / 2 - 1],
            _ => 0.,
        } / (2. * Self::h);
        y.0 / (Self::L * Self::Cs)
            + (Self::R + Self::Rd) / Self::L * y.1
            + (Self::Rd / Self::L - 1. / (Self::Rd * Self::Cp)) * y.2
            + 1. / Self::Rd * dt
            - self.v.get(t / 2).unwrap_or(&0.) / Self::L
    }
}
