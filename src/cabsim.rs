// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;
use std::iter::zip;

/// Simple biquad based cabinet simulator.
///
/// Starting point was based on a frequency response graph of the condor from
/// runoffgroove. Nothing else comes from the condor; the "implementation"
/// is just curve fitting. Nevertheless I wanted to honour those who came
/// before by preserving the name.
///
/// ~~~ sh
/// fiview 48000 -i \
///   PkBq/0.7/-16/400 x \
///   HsBq/0.7/6/400 x \
///   HpBq/0.7/60 x \
///   LpBq/0.7/4000 x LpBq/0.7/4000
/// ~~~
///
/// The result is five biquads:
///  * Partial notch filter at 400Hz (peaking EQ) (-16dB)
///  * High boosting shelf filter at 400Hz (6dB)
///  * High pass at 60Hz
///  * 2 x low pass at 4000Hz
#[derive(Debug, Default)]
pub struct CabinetSimulator {
    notch: Biquad,
    shelf: Biquad,
    hpf: Biquad,
    lpf0: Biquad,
    lpf1: Biquad,
}

impl CabinetSimulator {
    pub fn setup(&mut self, ctx: &AudioContext) {
        self.notch.peakingeq(ctx, 400, -16.0, 0.7);
        self.shelf.highshelf(ctx, 400, 6.0, 0.7);
        self.hpf.highpass(ctx, 50, 0.35);
        self.lpf0.lowpass(ctx, 4000, 0.7);
        self.lpf1.lowpass(ctx, 4000, 0.7);
    }
}

impl Filter for CabinetSimulator {
    fn step(&mut self, spl: f32) -> f32 {
        let mut spl = spl;
        spl = self.notch.step(spl);
        spl = self.shelf.step(spl);
        spl = self.hpf.step(spl);
        spl = self.lpf0.step(spl);
        self.lpf1.step(spl)
    }

    fn process(&mut self, inbuf: &[f32], outbuf: &mut [f32]) {
        for (x, y) in zip(inbuf, outbuf) {
            *y = self.step(*x);
        }
    }

    fn flush(&mut self) {
        self.notch.flush();
        self.shelf.flush();
        self.hpf.flush();
        self.lpf0.flush();
        self.lpf1.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cabsim_step() {
        let ctx = AudioContext::new(48000);
        let mut sg = SineGenerator::default();
        let mut cabsim = CabinetSimulator::default();
        let mut inbuf = [0.0_f32; 1024];
        let mut outbuf = [0.0_f32; 1024];

        sg.setup(&ctx, 400, 1.570793);
        cabsim.setup(&ctx);

        sg.process(&mut inbuf);
        cabsim.process(&inbuf, &mut outbuf);

        assert!(inbuf[0] == 0.0);
        assert!(inbuf[1023] != 0.0);
        assert!(outbuf[0] == 0.0);
    }

    #[test]
    fn test_frequency_response() {
        let ctx = AudioContext::new(48000);
        let mut cabsim = CabinetSimulator::default();
        cabsim.setup(&ctx);
        // Ideal responses from the Marshall speaker the Condor response was
        // designed to approximate is shown in comments on the right.
        assert!(check_response(&ctx, &mut cabsim, 30, -15.0)); //   -28.5
        assert!(check_response(&ctx, &mut cabsim, 60, -8.0)); //    -15.0
        assert!(check_response(&ctx, &mut cabsim, 100, -6.5)); //    -6.0
        assert!(check_response(&ctx, &mut cabsim, 350, -13.0)); //  -10.0
        assert!(check_response(&ctx, &mut cabsim, 3200, 2.0)); //     0.0
        assert!(check_response(&ctx, &mut cabsim, 6000, -12.0)); // -12.0
        assert!(check_response(&ctx, &mut cabsim, 8000, -21.0)); // -24.0
    }
}
