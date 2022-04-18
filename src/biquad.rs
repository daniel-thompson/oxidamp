// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;
use std::f64::consts::PI;

/// Biquad filter coefficients
///
/// The coefficients fully describe the filter but cannot
/// really be operated on without the rest of the filter state.
#[derive(Debug, Default)]
pub struct BiquadCoeff {
    x: [f32; 3],
    y: [f32; 2],
}

#[derive(Debug, Default)]
pub struct Biquad {
    coeff: BiquadCoeff,
    zx: [f32; 2],
    zy: [f32; 2],
    zi: usize,
}

impl Biquad {
    pub fn step(&mut self, x: f32) -> f32 {
        let nextzi = (self.zi == 0) as usize;

        let mut y = self.coeff.x[0] * x;
        y += self.coeff.x[1] * self.zx[self.zi];
        y += self.coeff.x[2] * self.zx[nextzi];
        y += self.coeff.y[0] * self.zy[self.zi];
        y += self.coeff.y[1] * self.zy[nextzi];

        self.zx[nextzi] = x;
        self.zy[nextzi] = y;
        self.zi = nextzi;

        y
    }

    // TODO: can we macro the process method?

    pub fn flush(&mut self) {
        self.zx = [0.0; 2];
        self.zy = [0.0; 2];
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
    fn new(sfreq: i32, freq: i32, dbgain: i32, q: f64) -> Self {
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
        let gain = (dbgain as f64) / 20.0;

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
        let mut design = BiquadDesign::new(ctx.sampling_frequency, shfreq, 0, q);

        design.b[0] = (1.0 - design.cosw0) / 2.0;
        design.b[1] = 1.0 - design.cosw0;
        design.b[2] = (1.0 - design.cosw0) / 2.0;

        design.a[0] = 1.0 + design.alpha;
        design.a[1] = -2.0 * design.cosw0;
        design.a[2] = 1.0 - design.alpha;

        self.coeff = design.apply()
    }

    pub fn highpass(&mut self, ctx: &AudioContext, shfreq: i32, q: f64) {
        let mut design = BiquadDesign::new(ctx.sampling_frequency, shfreq, 0, q);

        design.b[0] = (1.0 + design.cosw0) / 2.0;
        design.b[1] = -(1.0 + design.cosw0);
        design.b[2] = (1.0 + design.cosw0) / 2.0;

        design.a[0] = 1.0 + design.alpha;
        design.a[1] = -2.0 * design.cosw0;
        design.a[2] = 1.0 - design.alpha;

        self.coeff = design.apply();
    }

    pub fn bandpass(&mut self, ctx: &AudioContext, cfreq: i32, q: f64) {
        let mut design = BiquadDesign::new(ctx.sampling_frequency, cfreq, 0, q);

        design.b[0] = design.alpha;
        design.b[1] = 0.0;
        design.b[2] = -design.alpha;

        design.a[0] = 1.0 + design.alpha;
        design.a[1] = -2.0 * design.cosw0;
        design.a[2] = 1.0 - design.alpha;

        self.coeff = design.apply();
    }

    pub fn bandstop(&mut self, ctx: &AudioContext, cfreq: i32, q: f64) {
        let mut design = BiquadDesign::new(ctx.sampling_frequency, cfreq, 0, q);

        design.b[0] = 1.0;
        design.b[1] = -2.0 * design.cosw0;
        design.b[2] = 1.0;

        design.a[0] = 1.0 + design.alpha;
        design.a[1] = -2.0 * design.cosw0;
        design.a[2] = 1.0 - design.alpha;

        self.coeff = design.apply();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sbuf::SampleBufferExt;
    use crate::siggen::SignalGenerator;
    use crate::util::*;
    use std::iter::zip;

    fn stimulate(ctx: &AudioContext, bq: &mut Biquad, gfreq: i32) -> f32 {
        let mut inbuf = [0.0_f32; 1024];
        let mut outbuf = [0.0_f32; 1024];
        let mut sg = SignalGenerator::default();

        sg.setup(&ctx, gfreq, 1.570793);

        // stimulate the filter
        for _ in 0..10 {
            for it in zip(&mut inbuf, &mut outbuf) {
                let (inspl, outspl) = it;
                *inspl = sg.sin();
                *outspl = bq.step(*inspl);
            }
        }

        // check the result
        outbuf.analyse_rectify()
    }

    fn check_response(ctx: &AudioContext, bq: &mut Biquad, gfreq: i32, db: f32) -> bool {
        let level = stimulate(ctx, bq, gfreq);
        let ok;

        if 0.0 == db {
            ok = fuzzcmp(level, 1.0, 1.05);

            println!(
                "    {:4}Hz@{}Hz    {:6.2} {}=   1.00              [ {} ]",
                gfreq,
                ctx.sampling_frequency,
                level,
                if ok { '~' } else { '!' },
                if ok { " OK " } else { "FAIL" }
            );
        } else {
            let dblevel = linear2db(level);

            /* special case for very quiet signals */
            if db <= -96.0 {
                ok = dblevel <= (db + 6.0);
            } else {
                ok = fuzzcmp(dblevel, db, 1.1);
            }

            println!(
                "    {:4}Hz@{}Hz    {:6.2} {}= {:6.2}dB            [ {} ]",
                gfreq,
                ctx.sampling_frequency,
                dblevel,
                if ok { '~' } else { '!' },
                db,
                if ok { " OK " } else { "FAIL" }
            );
        }

        ok
    }

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
        assert!(check_response(&ctx, &mut bq, 1000, 0.0));
        assert!(check_response(&ctx, &mut bq, 500, -3.0));
        assert!(check_response(&ctx, &mut bq, 2000, -3.0));
    }

    #[test]
    fn test_bandstop_response() {
        let ctx = AudioContext::new(48000);
        let mut bq = Biquad::default();
        bq.bandstop(&ctx, 1000, 0.7);
        assert!(check_response(&ctx, &mut bq, 1000, -96.0));
        assert!(check_response(&ctx, &mut bq, 500, -3.0));
        assert!(check_response(&ctx, &mut bq, 2000, -3.0));
    }
}
