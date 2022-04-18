// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;
use std::f32::consts::PI;

#[derive(Debug, Default)]
pub struct SignalGenerator {
    phase: f32,
    step: f32,
    limit: f32,
    amplitude: f32,
}

impl SignalGenerator {
    pub fn setup(&mut self, ctx: &AudioContext, freq: i32, amplitude: f32) {
        if freq != 0 {
            let limit = 2.0 * PI;
            self.step = (limit * (freq as f32)) / (ctx.sampling_frequency as f32);
            self.limit = limit;
        } else {
            self.step = 0.0;
            self.limit = amplitude + 1.0;
        }

        self.phase = 0.0 - self.step;
        self.amplitude = amplitude;
    }

    fn step(&mut self) {
        self.phase += self.step;
        if self.phase > self.limit {
            self.phase -= self.limit;
        }
    }

    pub fn sin(&mut self) -> f32 {
        self.step();
        self.amplitude * self.phase.sin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_siggen_sin() {
        let ctx = AudioContext::new(48000);
        let mut sg = SignalGenerator::default();
        sg.setup(&ctx, 400, 1.570793);
        let mut buf = [0.0_f32; 1024];

        for spl in &mut buf {
            *spl = sg.sin();
        }

        assert_fuzzeq!(buf.analyse_rectify(), 1.0, 1.05);
    }
}
