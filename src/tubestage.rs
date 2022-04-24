// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

struct TubeTable {
    min_vi: i32,
    max_vi: i32,
    _mapping: u32,
    table: [f32; 1001],
}

include!("12AX7.rs");

pub enum Tube {
    Tube12AX7Ri68K,
    Tube12AX7Ri250K,
}

pub struct TubeStage<'a> {
    vp: f32,
    predivide: f32,

    gain: f32,
    input_filter: Biquad,
    tube: WaveShaper<'a>,
    feedback_filter: Biquad,
    output_filter: DCBlocker,

    bias: f32,
}

impl<'a> TubeStage<'a> {
    pub fn setup(
        &mut self,
        ctx: &AudioContext,
        tube: Tube,
        dbgain: f32,
        rk: f32,
        input_cutoff: i32,
        feedback_cutoff: i32,
    ) {
        let table = match tube {
            Tube::Tube12AX7Ri68K => &TUBE_12AX7_RI68K,
            Tube::Tube12AX7Ri250K => &TUBE_12AX7_RI250K,
        };

        // TODO: these should be looked up from the tube model
        self.vp = 250.0;
        let rp = 100000.0;
        self.tube
            .setup(ctx, table.min_vi as f32, table.max_vi as f32, &table.table);

        self.predivide = rk / rp;
        self.gain = db2linear(dbgain);

        self.input_filter.lowpass(ctx, input_cutoff, 0.7);
        self.feedback_filter.lowpass(ctx, feedback_cutoff, 0.7);
        self.output_filter.setup(ctx);
    }
}

impl<'a> Filter for TubeStage<'a> {
    fn step(&mut self, spl: f32) -> f32 {
        let mut spl = spl;

        spl *= self.gain;
        spl = self.input_filter.step(spl);
        spl += self.bias;
        spl = self.tube.step(spl);

        let mut feedback = self.vp - spl;
        feedback *= self.predivide;
        feedback = self.feedback_filter.step(feedback);
        self.bias = feedback;

        self.output_filter.step(spl)
    }

    fn flush(&mut self) {
        self.input_filter.flush();
        self.feedback_filter.flush();
        self.output_filter.flush();
    }
}
