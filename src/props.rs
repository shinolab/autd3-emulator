use polars::prelude::*;

use crate::Emulator;

impl Emulator {
    pub fn gain(&self) -> DataFrame {
        let temp = self.sub_devices[0].gain();
        self.sub_devices
            .iter()
            .skip(1)
            .fold(temp, |acc, sub| acc.vstack(&sub.gain()).unwrap())
    }
}
