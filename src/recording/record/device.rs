use polars::prelude::*;

use super::TransducerRecord;

use derive_more::Deref;

#[derive(Deref, Debug)]
pub struct DeviceRecord<'a> {
    #[deref]
    pub(crate) records: Vec<TransducerRecord<'a>>,
}

impl<'a> DeviceRecord<'a> {
    pub fn drive(&self) -> DataFrame {
        let mut df =
            df!("time[s]" => &TransducerRecord::time(self.records[0].drive.len())).unwrap();
        self.iter().enumerate().for_each(|(i, tr)| {
            let mut d = tr.drive();
            d.rename("phase", &format!("phase_{}", i)).unwrap();
            d.rename("intensity", &format!("intensity_{}", i)).unwrap();
            let mut d = d.take_columns();
            let intensity = d.pop().unwrap();
            let phase = d.pop().unwrap();
            df.hstack_mut(&[phase, intensity]).unwrap();
        });
        df
    }

    pub fn modulation(&self) -> DataFrame {
        self[0].modulation()
    }

    pub fn pulse_width(&self) -> DataFrame {
        let mut df =
            df!("time[s]" => &TransducerRecord::time(self.records[0].drive.len())).unwrap();
        self.iter().enumerate().for_each(|(i, tr)| {
            let mut d = tr.pulse_width();
            d.rename("pulsewidth", &format!("pulsewidth_{}", i))
                .unwrap();
            let mut d = d.take_columns();
            let pulsewidth = d.pop().unwrap();
            df.hstack_mut(&[pulsewidth]).unwrap();
        });
        df
    }

    pub fn output_voltage(&self) -> DataFrame {
        let mut df = self[0].output_voltage();
        df.rename("voltage[V]", "voltage_0[V]").unwrap();
        self.iter().enumerate().skip(1).for_each(|(i, tr)| {
            let mut d = tr.output_voltage();
            d.rename("voltage[V]", &format!("voltage_{}[V]", i))
                .unwrap();
            let mut d = d.take_columns();
            let voltage = d.pop().unwrap();
            df.hstack_mut(&[voltage]).unwrap();
        });
        df
    }

    pub fn output_ultrasound(&self) -> DataFrame {
        let mut df = self[0].output_ultrasound();
        df.rename("p[a.u.]", "p_0[a.u.]").unwrap();
        self.iter().enumerate().skip(1).for_each(|(i, tr)| {
            let mut d = tr.output_ultrasound();
            d.rename("p[a.u.]", &format!("p_{}[a.u.]", i)).unwrap();
            let mut d = d.take_columns();
            let v = d.pop().unwrap();
            df.hstack_mut(&[v]).unwrap();
        });
        df
    }
}
