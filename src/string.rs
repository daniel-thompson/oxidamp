// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

#[derive(Debug)]
pub struct KarplusStrong {
    delay: FracDelay<1920>,
    //filter: FIR<2, 3>,
    filter: FirstOrder,
    noise: WhiteNoise,
    stimulate: u32,
    gain: f32,
}

impl Default for KarplusStrong {
    fn default() -> Self {
        Self {
            delay: FracDelay::default(),
            //filter: fir2_halfband(),
            filter: FirstOrder::default(),
            noise: WhiteNoise::new(),
            stimulate: 0,
            gain: 0.999,
        }
    }
}

impl Voice for KarplusStrong {
    fn setup(&mut self, ctx: &AudioContext) {
        self.delay.setup(ctx, 120.0);
        self.filter.lowpass(ctx, ctx.sampling_frequency / 4);
    }

    fn trigger(&mut self) {
        self.stimulate = 128;
        self.gain = 0.999;
    }

    fn mute(&mut self) {
        self.gain = 0.95;
    }

    fn tune(&mut self, ctx: &AudioContext, freq: f32) {
        let delay = (ctx.sampling_frequency as f32 / freq) - 1.20;
        self.delay.setup(ctx, delay);
    }
}

impl SignalGenerator for KarplusStrong {
    fn step(&mut self) -> f32 {
        let mut spl = if self.stimulate > 0 {
            self.stimulate -= 1;
            self.noise.next().unwrap()
        } else {
            0.0
        };

        spl += self.gain * self.filter.step(self.delay.peek());
        let _ = self.delay.step(spl);

        spl
    }
}
