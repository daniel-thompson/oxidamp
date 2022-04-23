// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;
use std::iter::zip;

// TODO: replace with type declaration
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

pub trait Filter {
    fn step(&mut self, spl: f32) -> f32;
    fn flush(&mut self);

    fn process(&mut self, inbuf: &[f32], outbuf: &mut [f32]) {
        for (inspl, outspl) in zip(inbuf, outbuf) {
            *outspl = self.step(*inspl);
        }
    }

    /// Stimulate the filter with a specific pure-sine wave.
    ///
    /// TODO: *Does this really need to be a method?*
    fn stimulate(&mut self, ctx: &AudioContext, gfreq: i32) -> f32 {
        let mut inbuf = [0.0_f32; 1024];
        let mut outbuf = [0.0_f32; 1024];
        let mut sg = SineGenerator::default();

        sg.setup(&ctx, gfreq, 1.570793);

        // stimulate the filter
        for _ in 0..10 {
            sg.process(&mut inbuf);
            self.process(&inbuf, &mut outbuf);
        }

        // check the result
        outbuf.analyse_rectify()
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
