use std::time::Duration;

use autd3_driver::defined::ULTRASOUND_PERIOD;
use polars::{df, frame::DataFrame};

use crate::error::EmulatorError;

use super::TransducerRecord;

#[derive(Debug)]
pub struct OutputUltrasound<'a> {
    pub(crate) cursor: usize,
    pub(crate) record: &'a TransducerRecord<'a>,
    model: T4010A1BVDModel,
}

impl<'a> OutputUltrasound<'a> {
    pub(crate) fn _next(&mut self, n: usize) -> Vec<f32> {
        let output_volage = self.record._output_voltage_within(self.cursor, n);
        self.cursor += n;
        output_volage
            .into_iter()
            .map(|v| self.model.rk4(v))
            .collect()
    }

    pub fn next(&mut self, duration: Duration) -> Result<DataFrame, EmulatorError> {
        if duration.as_nanos() % ULTRASOUND_PERIOD.as_nanos() != 0 {
            return Err(EmulatorError::InvalidDuration);
        }
        let n = (duration.as_nanos() / ULTRASOUND_PERIOD.as_nanos()) as usize;
        let time = self.record.output_times(self.cursor, n);
        let p = self._next(n);
        Ok(df!(
            "time[s]" => &time,
            "p[a.u.]" => &p
        )
        .unwrap())
    }
}

impl<'a> TransducerRecord<'a> {
    pub fn output_ultrasound(&'a self) -> OutputUltrasound<'a> {
        OutputUltrasound {
            record: self,
            model: T4010A1BVDModel {
                state: (0., 0., 0.),
                last_v: -Self::V,
            },
            cursor: 0,
        }
    }
}

#[derive(Debug)]
struct T4010A1BVDModel {
    state: (f32, f32, f32),
    last_v: f32,
}

#[allow(non_upper_case_globals)]
impl T4010A1BVDModel {
    const Cs: f32 = 200e-9; // mF
    const L: f32 = 80e-6; // kH
    const R: f32 = 0.7; // kΩ
    const Cp: f32 = 2700e-9; // mF
    const Rd: f32 = 150e-3; // kΩ
    const h: f32 = TransducerRecord::TS;
    const NORMALIZE: f32 = 0.057430573;

    pub(crate) fn rk4(&mut self, input: f32) -> f32 {
        let state = &self.state;
        let y = state.1 * Self::NORMALIZE;
        let k00 = Self::h * Self::f0(state);
        let k01 = Self::h * self.f1(self.last_v, state);
        let k02 = Self::h * self.f2(self.last_v, state);
        let y1 = (state.0 + k00 / 2., state.1 + k01 / 2., state.2 + k02 / 2.);

        let v = (self.last_v + input) / 2.;
        let k10 = Self::h * Self::f0(&y1);
        let k11 = Self::h * self.f1(v, &y1);
        let k12 = Self::h * self.f2(v, &y1);
        let y2 = (state.0 + k10 / 2., state.1 + k11 / 2., state.2 + k12 / 2.);

        let k20 = Self::h * Self::f0(&y2);
        let k21 = Self::h * self.f1(v, &y2);
        let k22 = Self::h * self.f2(v, &y2);
        let y3 = (state.0 + k20, state.1 + k21, state.2 + k22);

        self.last_v = v;
        let k30 = Self::h * Self::f0(&y3);
        let k31 = Self::h * self.f1(input, &y3);
        let k32 = Self::h * self.f2(input, &y3);

        self.last_v = input;
        self.state = (
            state.0 + (k00 + 2. * k10 + 2. * k20 + k30) / 6.,
            state.1 + (k01 + 2. * k11 + 2. * k21 + k31) / 6.,
            state.2 + (k02 + 2. * k12 + 2. * k22 + k32) / 6.,
        );
        y
    }

    fn f0(y: &(f32, f32, f32)) -> f32 {
        y.1
    }

    fn f1(&self, v: f32, y: &(f32, f32, f32)) -> f32 {
        -y.0 / (Self::L * Self::Cs)
            - (Self::R + Self::Rd) / Self::L * y.1
            - Self::Rd / Self::L * y.2
            + v / Self::L
    }

    fn f2(&self, v: f32, y: &(f32, f32, f32)) -> f32 {
        let dt = (v - self.last_v) / Self::h * 2.;
        y.0 / (Self::L * Self::Cs)
            + (Self::R + Self::Rd) / Self::L * y.1
            + (Self::Rd / Self::L - 1. / (Self::Rd * Self::Cp)) * y.2
            + 1. / Self::Rd * dt
            - v / Self::L
    }
}
