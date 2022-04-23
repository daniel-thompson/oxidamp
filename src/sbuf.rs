// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

#[derive(Debug, Default)]
pub struct SampleBuffer {
    pub v: Vec<f32>,
}

impl SampleBuffer {
    pub fn new(sz: usize) -> Self {
        let mut sbuf = SampleBuffer::default();
        sbuf.v.resize(sz, 0.0);

        sbuf
    }

    pub fn analyse_peak(&self) -> f32 {
        let mut peak: f32 = 0.0;

        for spl in &self.v {
            let spl = spl.abs();
            if spl > peak {
                peak = spl;
            }
        }

        peak
    }

    pub fn analyse_rectify(&self) -> f32 {
        let mut acc: f32 = 0.0;

        for spl in &self.v {
            acc += spl.abs();
        }

        acc / (self.v.len() as f32)
    }
}

pub trait SignalGenerator {
    fn step(&mut self) -> f32;
    fn process(&mut self, samples: &mut [f32]) {
        for spl in samples {
            *spl = self.step();
        }
    }
}

pub trait SampleBufferExt {
    fn analyse_peak(&self) -> f32;
    fn analyse_rectify(&self) -> f32;
}

impl SampleBufferExt for [f32] {
    fn analyse_peak(&self) -> f32 {
        let mut peak: f32 = 0.0;

        for spl in self {
            let spl = spl.abs();
            if spl > peak {
                peak = spl;
            }
        }

        peak
    }

    fn analyse_rectify(&self) -> f32 {
        let mut acc: f32 = 0.0;

        for spl in self {
            acc += spl.abs();
        }

        acc / (self.len() as f32)
    }
}
