use std::cell::Ref;

use polars::{df, frame::DataFrame};

use super::TransducerRecord;

impl<'a> TransducerRecord<'a> {
    #[inline(always)]
    pub(crate) fn _output_ultrasound(&self) -> Ref<'_, Vec<f32>> {
        if self.output_ultrasound_cache.borrow().is_empty() {
            self.output_ultrasound_cache.replace(
                T4010A1BVDModel {
                    v: self._output_voltage(),
                }
                .rk4(),
            );
        }
        self.output_ultrasound_cache.borrow()
    }

    pub fn output_ultrasound(&self) -> DataFrame {
        let time = self.output_times();
        let p = self._output_ultrasound();
        df!(
            "time[s]" => &time,
            "p[a.u.]" => p.as_slice()
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
