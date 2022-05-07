// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;
use std::iter::zip;

pub type Sample = f32;

pub trait SignalGenerator {
    fn step(&mut self) -> Sample;
    fn process(&mut self, samples: &mut [Sample]) {
        for spl in samples {
            *spl = self.step();
        }
    }
}

pub trait Filter {
    fn step(&mut self, spl: Sample) -> Sample;
    fn flush(&mut self);

    fn process(&mut self, inbuf: &[Sample], outbuf: &mut [Sample]) {
        for (inspl, outspl) in zip(inbuf, outbuf) {
            *outspl = self.step(*inspl);
        }
    }

    /// Stimulate the filter with a specific pure-sine wave.
    ///
    /// TODO: *Does this really need to be a method?*
    fn stimulate(&mut self, ctx: &AudioContext, gfreq: i32) -> Sample {
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
    fn analyse_peak(&self) -> Sample;
    fn analyse_rectify(&self) -> Sample;
}

impl SampleBufferExt for [Sample] {
    fn analyse_peak(&self) -> Sample {
        let mut peak: Sample = 0.0;

        for spl in self {
            let spl = spl.abs();
            if spl > peak {
                peak = spl;
            }
        }

        peak
    }

    fn analyse_rectify(&self) -> Sample {
        let mut acc: Sample = 0.0;

        for spl in self {
            acc += spl.abs();
        }

        acc / (self.len() as Sample)
    }
}
