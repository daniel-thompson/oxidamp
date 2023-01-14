// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;
use std::f64::consts::PI;
use std::iter::zip;

/// Biquad filter coefficients
///
/// The coefficients fully describe the filter but cannot
/// really be operated on without the rest of the filter state.
#[derive(Clone, Copy, Debug, Default)]
pub struct BiquadCoeff {
    x: [f32; 3],
    y: [f32; 2],
}

#[derive(Debug, Default)]
pub struct Biquad {
    pub coeff: BiquadCoeff,
    z: [f32; 2],
}

impl Filter for Biquad {
    fn step(&mut self, x: f32) -> f32 {
        let b0 = self.coeff.x[0];
        let b1 = self.coeff.x[1];
        let b2 = self.coeff.x[2];
        let ma1 = self.coeff.y[0];
        let ma2 = self.coeff.y[1];

        let y = b0 * x + self.z[0];
        self.z[0] = b1 * x + ma1 * y + self.z[1];
        self.z[1] = b2 * x + ma2 * y;

        y
    }

    /// Optimized processing loop for biquad filters
    ///
    /// The code here is identical to [sbuf::Filter::process] but, as of
    /// rustc 1.62, this results in *much* better code generation
    /// (>2x faster).
    fn process(&mut self, inbuf: &[f32], outbuf: &mut [f32]) {
        for (x, y) in zip(inbuf, outbuf) {
            *y = self.step(*x);
        }
    }

    fn flush(&mut self) {
        self.z = [0.0; 2];
    }
}

#[derive(Debug, Default)]
struct BiquadDesign {
    big_a: f64,
    big_g: f64,
    w0: f64,
    cosw0: f64,
    sinw0: f64,

    alpha: f64,

    b: [f64; 3],
    a: [f64; 3],
}

impl BiquadDesign {
    fn new(sfreq: i32, freq: i32, dbgain: f64, q: f64) -> Self {
        let mut design: BiquadDesign = Default::default();

        /* HACK: Many of the filters are numerically unstable when designed
         *       for 44.1K. This is a grotty workaround (and insufficient
         *       to properly clear the test suite) but it stops tintamp being
         *       a total lemon...
         */
        //let mut sfreq = sfreq;
        //if sfreq == 44100 {
        //   sfreq = 44096; // TODO: 44096?
        //}

        let base: f64 = 10.0;
        let gain = dbgain / 20.0;

        design.big_a = base.powf(gain).sqrt();
        design.big_g = base.powf(gain / 2.0);

        design.w0 = (2.0 * PI * freq as f64) / (sfreq as f64);
        design.cosw0 = design.w0.cos();
        design.sinw0 = design.w0.sin();

        design.alpha = design.sinw0 / (2.0 * q);

        design
    }

    fn apply(&mut self) -> BiquadCoeff {
        BiquadCoeff {
            x: [
                (self.b[0] / self.a[0]) as f32,
                (self.b[1] / self.a[0]) as f32,
                (self.b[2] / self.a[0]) as f32,
            ],
            y: [
                -(self.a[1] / self.a[0]) as f32,
                -(self.a[2] / self.a[0]) as f32,
            ],
        }
    }
}

impl Biquad {
    pub fn lowpass(&mut self, ctx: &AudioContext, shfreq: i32, q: f64) {
        let mut design = BiquadDesign::new(ctx.sampling_frequency, shfreq, 0.0, q);

        design.b[0] = (1.0 - design.cosw0) / 2.0;
        design.b[1] = 1.0 - design.cosw0;
        design.b[2] = (1.0 - design.cosw0) / 2.0;

        design.a[0] = 1.0 + design.alpha;
        design.a[1] = -2.0 * design.cosw0;
        design.a[2] = 1.0 - design.alpha;

        self.coeff = design.apply()
    }

    pub fn highpass(&mut self, ctx: &AudioContext, shfreq: i32, q: f64) {
        let mut design = BiquadDesign::new(ctx.sampling_frequency, shfreq, 0.0, q);

        design.b[0] = (1.0 + design.cosw0) / 2.0;
        design.b[1] = -(1.0 + design.cosw0);
        design.b[2] = (1.0 + design.cosw0) / 2.0;

        design.a[0] = 1.0 + design.alpha;
        design.a[1] = -2.0 * design.cosw0;
        design.a[2] = 1.0 - design.alpha;

        self.coeff = design.apply();
    }

    pub fn bandpass(&mut self, ctx: &AudioContext, cfreq: i32, q: f64) {
        let mut design = BiquadDesign::new(ctx.sampling_frequency, cfreq, 0.0, q);

        design.b[0] = design.alpha;
        design.b[1] = 0.0;
        design.b[2] = -design.alpha;

        design.a[0] = 1.0 + design.alpha;
        design.a[1] = -2.0 * design.cosw0;
        design.a[2] = 1.0 - design.alpha;

        self.coeff = design.apply();
    }

    pub fn bandstop(&mut self, ctx: &AudioContext, cfreq: i32, q: f64) {
        let mut design = BiquadDesign::new(ctx.sampling_frequency, cfreq, 0.0, q);

        design.b[0] = 1.0;
        design.b[1] = -2.0 * design.cosw0;
        design.b[2] = 1.0;

        design.a[0] = 1.0 + design.alpha;
        design.a[1] = -2.0 * design.cosw0;
        design.a[2] = 1.0 - design.alpha;

        self.coeff = design.apply();
    }

    pub fn allpass(&mut self, ctx: &AudioContext, csfreq: i32, q: f64) {
        let mut design = BiquadDesign::new(ctx.sampling_frequency, csfreq, 0.0, q);

        design.b[0] = 1.0 - design.alpha;
        design.b[1] = -2.0 * design.cosw0;
        design.b[2] = 1.0 + design.alpha;

        design.a[0] = design.b[2];
        design.a[1] = design.b[1];
        design.a[2] = design.b[0];

        self.coeff = design.apply();
    }

    pub fn peakingeq(&mut self, ctx: &AudioContext, cfreq: i32, dbgain: f64, q: f64) {
        let mut design = BiquadDesign::new(ctx.sampling_frequency, cfreq, dbgain, q);

        design.b[0] = 1.0 + (design.alpha * design.big_a);
        design.b[1] = -2.0 * design.cosw0;
        design.b[2] = 1.0 - (design.alpha * design.big_a);

        design.a[0] = 1.0 + (design.alpha / design.big_a);
        design.a[1] = -2.0 * design.cosw0;
        design.a[2] = 1.0 - (design.alpha / design.big_a);

        self.coeff = design.apply();
    }

    pub fn lowshelf(&mut self, ctx: &AudioContext, shfreq: i32, dbgain: f64, q: f64) {
        let mut design = BiquadDesign::new(ctx.sampling_frequency, shfreq, dbgain, q);

        let plus_minus_minus = (design.big_a + 1.0) - ((design.big_a - 1.0) * design.cosw0);
        let minus_minus_plus = (design.big_a - 1.0) - ((design.big_a + 1.0) * design.cosw0);
        let plus_plus_minus = (design.big_a + 1.0) + ((design.big_a - 1.0) * design.cosw0);
        let minus_plus_plus = (design.big_a - 1.0) + ((design.big_a + 1.0) * design.cosw0);
        let two_roota_alpha = 2.0 * design.big_a.sqrt() * design.alpha;

        design.b[0] = design.big_a * (plus_minus_minus + two_roota_alpha);
        design.b[1] = 2.0 * design.big_a * minus_minus_plus;
        design.b[2] = design.big_a * (plus_minus_minus - two_roota_alpha);

        design.a[0] = plus_plus_minus + two_roota_alpha;
        design.a[1] = -2.0 * minus_plus_plus;
        design.a[2] = plus_plus_minus - two_roota_alpha;

        self.coeff = design.apply();
    }

    pub fn highshelf(&mut self, ctx: &AudioContext, shfreq: i32, dbgain: f64, q: f64) {
        let mut design = BiquadDesign::new(ctx.sampling_frequency, shfreq, dbgain, q);

        let plus_plus_minus = (design.big_a + 1.0) + ((design.big_a - 1.0) * design.cosw0);
        let minus_plus_plus = (design.big_a - 1.0) + ((design.big_a + 1.0) * design.cosw0);
        let plus_minus_minus = (design.big_a + 1.0) - ((design.big_a - 1.0) * design.cosw0);
        let minus_minus_plus = (design.big_a - 1.0) - ((design.big_a + 1.0) * design.cosw0);
        let two_roota_alpha = 2.0 * design.big_a.sqrt() * design.alpha;

        design.b[0] = design.big_a * (plus_plus_minus + two_roota_alpha);
        design.b[1] = -2.0 * design.big_a * minus_plus_plus;
        design.b[2] = design.big_a * (plus_plus_minus - two_roota_alpha);

        design.a[0] = plus_minus_minus + two_roota_alpha;
        design.a[1] = 2.0 * minus_minus_plus;
        design.a[2] = plus_minus_minus - two_roota_alpha;

        self.coeff = design.apply();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_step() {
        let mut bq = Biquad::default();

        assert_eq!(bq.step(0.0), 0.0);
        assert_eq!(bq.step(1.0), 0.0);
        assert_eq!(bq.step(-1.0), 0.0);
    }

    #[test]
    fn test_lowpass_response() {
        let ctx = AudioContext::new(48000);
        let mut bq = Biquad::default();
        bq.lowpass(&ctx, 400, 0.7);
        assert!(check_response(&ctx, &mut bq, 400, -3.0));
        assert!(check_response(&ctx, &mut bq, 800, -12.0));
        assert!(check_response(&ctx, &mut bq, 200, 0.0));
    }

    #[test]
    fn test_highpass_response() {
        let ctx = AudioContext::new(48000);
        let mut bq = Biquad::default();
        bq.highpass(&ctx, 600, 0.7);
        assert!(check_response(&ctx, &mut bq, 600, -3.0));
        assert!(check_response(&ctx, &mut bq, 300, -12.0));
        assert!(check_response(&ctx, &mut bq, 1200, 0.0));
    }

    #[test]
    fn test_bandpass_response() {
        let ctx = AudioContext::new(48000);
        let mut bq = Biquad::default();
        bq.bandpass(&ctx, 1000, 0.7);
        assert!(check_response(&ctx, &mut bq, 250, -9.0));
        assert!(check_response(&ctx, &mut bq, 500, -3.0));
        assert!(check_response(&ctx, &mut bq, 1000, 0.0));
        assert!(check_response(&ctx, &mut bq, 2000, -3.0));
        assert!(check_response(&ctx, &mut bq, 4000, -9.0));
    }

    #[test]
    fn test_bandstop_response() {
        let ctx = AudioContext::new(48000);
        let mut bq = Biquad::default();
        bq.bandstop(&ctx, 1000, 0.7);
        assert!(check_response(&ctx, &mut bq, 125, 0.0));
        assert!(check_response(&ctx, &mut bq, 500, -3.0));
        assert!(check_response(&ctx, &mut bq, 1000, -96.0));
        assert!(check_response(&ctx, &mut bq, 2000, -3.0));
        assert!(check_response(&ctx, &mut bq, 8000, 0.0));
    }

    #[test]
    fn test_allpass_response() {
        let ctx = AudioContext::new(48000);
        let mut bq = Biquad::default();
        bq.allpass(&ctx, 400, 0.7);
        let mut f = 200;
        while f < 10000 {
            assert!(check_response(&ctx, &mut bq, f, 0.0));
            f *= 2;
        }
    }

    #[test]
    fn test_peakingeq_response() {
        let ctx = AudioContext::new(48000);
        let mut bq = Biquad::default();
        bq.peakingeq(&ctx, 400, -16.0, 8.0);
        assert!(check_response(&ctx, &mut bq, 100, 0.0));
        assert!(check_response(&ctx, &mut bq, 200, 0.0));
        assert!(check_response(&ctx, &mut bq, 400, -16.0));
        assert!(check_response(&ctx, &mut bq, 800, 0.0));
        assert!(check_response(&ctx, &mut bq, 1600, 0.0));
    }

    #[test]
    fn test_peakingeq_boost() {
        let ctx = AudioContext::new(48000);
        let mut bq = Biquad::default();
        bq.peakingeq(&ctx, 400, 6.0, 2.0);
        assert!(check_response(&ctx, &mut bq, 400, 6.0));
        assert!(check_response(&ctx, &mut bq, 1600, 0.0));
    }

    #[test]
    fn test_highshelf_response() {
        let ctx = AudioContext::new(48000);
        let mut bq = Biquad::default();
        bq.highshelf(&ctx, 750, -6.0, 0.7);
        assert!(check_response(&ctx, &mut bq, 300, 0.0));
        assert!(check_response(&ctx, &mut bq, 2000, -6.0));
    }

    #[test]
    fn test_highboost_response() {
        let ctx = AudioContext::new(48000);
        let mut bq = Biquad::default();
        bq.highshelf(&ctx, 750, 6.0, 0.7);
        assert!(check_response(&ctx, &mut bq, 300, 0.0));
        assert!(check_response(&ctx, &mut bq, 2000, 6.0));
    }

    #[test]
    fn test_lowshelf_response() {
        let ctx = AudioContext::new(48000);
        let mut bq = Biquad::default();
        bq.lowshelf(&ctx, 750, -6.0, 0.7);
        assert!(check_response(&ctx, &mut bq, 300, -6.0));
        assert!(check_response(&ctx, &mut bq, 2000, 0.0));
    }

    #[test]
    fn test_lowboost_response() {
        let ctx = AudioContext::new(48000);
        let mut bq = Biquad::default();
        bq.lowshelf(&ctx, 750, 6.0, 0.7);
        assert!(check_response(&ctx, &mut bq, 300, 6.0));
        assert!(check_response(&ctx, &mut bq, 2000, 0.0));
    }

    #[test]
    fn test_biquad_coprocess() {
        let ctx = AudioContext::new(48000);
        let mut inbuf = [0.0; 1024];

        let mut sg = SineGenerator::default();
        sg.setup(&ctx, 440, db2linear(-3.0));
        sg.process(&mut inbuf);

        let mut stepbuf = [0.0; 1024];
        let mut stepbq = Biquad::default();
        stepbq.lowpass(&ctx, 8000, 0.7);
        for (i, s) in zip(&inbuf, &mut stepbuf) {
            *s = stepbq.step(*i);
        }

        let mut procbuf = [0.0; 1024];
        let mut procbq = Biquad::default();
        procbq.lowpass(&ctx, 8000, 0.7);
        procbq.process(&inbuf, &mut procbuf);

        for (s, p) in zip(&stepbuf, &procbuf) {
            assert_eq!(*s, *p);
        }
    }
}
