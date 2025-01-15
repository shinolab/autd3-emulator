use autd3::{driver::geometry::Complex, prelude::Point3};

use rayon::prelude::*;

use super::RmsTransducerRecord;

#[derive(Debug)]
pub(crate) struct Cpu {
    records: Vec<RmsTransducerRecord>,
    dists: Vec<Vec<f32>>,
    buffer: Vec<f32>,
}

impl Cpu {
    pub(crate) fn new(
        x: &[f32],
        y: &[f32],
        z: &[f32],
        transducer_positions: impl Iterator<Item = Point3>,
        records: Vec<RmsTransducerRecord>,
    ) -> Self {
        let transducer_positions = transducer_positions.collect::<Vec<_>>();
        let dists = itertools::izip!(x.iter(), y.iter(), z.iter())
            .map(|(&x, &y, &z)| Point3::new(x, y, z))
            .map(|p| {
                transducer_positions
                    .iter()
                    .map(|tp| (p - tp).norm())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        Self {
            records,
            dists,
            buffer: vec![0.; x.len()],
        }
    }

    pub(crate) fn compute(&mut self, idx: usize, wavenumber: f32) -> &Vec<f32> {
        self.dists
            .par_iter()
            .map(|d| {
                d.iter()
                    .zip(self.records.iter())
                    .map(|(dist, tr)| {
                        Complex::from_polar(tr.amp[idx] / dist, wavenumber * dist + tr.phase[idx])
                    })
                    .sum::<Complex>()
                    .norm()
            })
            .collect_into_vec(&mut self.buffer);
        &self.buffer
    }
}
