// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

#[derive(Debug, Default)]
pub struct DCBlocker {
    filter: FirstOrder,
}

impl DCBlocker {
    pub fn setup(&mut self, ctx: &AudioContext) {
        self.filter.highpass(ctx, 31);
    }
}

impl Filter for DCBlocker {
    fn step(&mut self, x: f32) -> f32 {
        self.filter.step(x)
    }

    fn flush(&mut self) {
        self.filter.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dc() {
        let ctx = AudioContext::new(48000);
        let mut dc = DCBlocker::default();
        let mut outbuf = [0.0_f32; 1024];

        dc.setup(&ctx);

        // stimulate the filter
        for _ in 0..100 {
            for spl in &mut outbuf {
                *spl = dc.step(1.0);
            }
        }

        assert!(outbuf[0] < 0.0001);
        assert!(outbuf.analyse_rectify() < 0.0001);
    }

    #[test]
    fn test_ac() {
        let ctx = AudioContext::new(48000);
        let mut dc = DCBlocker::default();
        let mut sg = SineGenerator::default();
        let mut inbuf = [0.0_f32; 1024];
        let mut outbuf = [0.0_f32; 1024];

        dc.setup(&ctx);
        sg.setup(&ctx, 100, 1.0);

        for _ in 0..10 {
            sg.process(&mut inbuf);
            dc.process(&inbuf, &mut outbuf);
        }

        assert_fuzzeq!(inbuf.analyse_rectify(), 0.64, 1.03);
        assert_fuzzeq!(outbuf.analyse_rectify(), 0.64, 1.03);
    }
}
