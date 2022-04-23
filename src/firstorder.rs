// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;
use std::f64::consts::PI;

#[derive(Debug, Default)]
pub struct FirstOrderCoeff {
    x: [f32; 2],
    y: f32,
}

#[derive(Debug, Default)]
pub struct FirstOrder {
    coeff: FirstOrderCoeff,
    zx: f32,
    zy: f32,
}

impl Filter for FirstOrder {
    fn step(&mut self, x: f32) -> f32 {
        let mut y = self.coeff.x[0] * x;
        y += self.coeff.x[1] * self.zx;
        y += self.coeff.y * self.zy;

        self.zx = x;
        self.zy = y;

        y
    }

    fn flush(&mut self) {
        self.zx = 0.0;
        self.zy = 0.0;
    }
}

#[derive(Debug, Default)]
struct FirstOrderDesign {
    w: f64,
    big_k: f64,
    alpha: f64,

    b: [f64; 2],
    a: [f64; 2],
}

/// Uses equations from Filter Design Equations from Apogee
/// (http://www.apogeebio.com/ddx/PDFs/AN-06.pdf ).
impl FirstOrderDesign {
    fn new(sfreq: i32, freq: i32) -> Self {
        let mut design: FirstOrderDesign = Default::default();

        design.w = 2.0 * PI * (freq as f64) / (sfreq as f64);
        design.big_k = (design.w / 2.0).tan();
        design.alpha = 1.0 + design.big_k;

        design
    }

    fn apply(&mut self) -> FirstOrderCoeff {
        FirstOrderCoeff {
            x: [
                (self.b[0] / self.a[0]) as f32,
                (self.b[1] / self.a[0]) as f32,
            ],
            y: -(self.a[1] / self.a[0]) as f32,
        }
    }
}

impl FirstOrder {
    pub fn lowpass(&mut self, ctx: &AudioContext, shfreq: i32) {
        let mut design = FirstOrderDesign::new(ctx.sampling_frequency, shfreq);

        design.b[0] = design.big_k / design.alpha;
        design.b[1] = design.big_k / design.alpha;

        design.a[0] = 1.0;
        design.a[1] = -((1.0 - design.big_k) / design.alpha);

        self.coeff = design.apply();
    }

    pub fn highpass(&mut self, ctx: &AudioContext, shfreq: i32) {
        let mut design = FirstOrderDesign::new(ctx.sampling_frequency, shfreq);

        design.b[0] = 1.0 / design.alpha;
        design.b[1] = -1.0 / design.alpha;

        design.a[0] = 1.0;
        design.a[1] = -((1.0 - design.big_k) / design.alpha);

        self.coeff = design.apply();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_step() {
        let mut f = FirstOrder::default();

        assert_eq!(f.step(0.0), 0.0);
        assert_eq!(f.step(1.0), 0.0);
        assert_eq!(f.step(-1.0), 0.0);
    }

    #[test]
    fn test_lowpass_response() {
        let ctx = AudioContext::new(48000);
        let mut f = FirstOrder::default();
        f.lowpass(&ctx, 400);
        assert!(check_response(&ctx, &mut f, 400, -3.0));
        assert!(check_response(&ctx, &mut f, 1600, -12.0));
        assert!(check_response(&ctx, &mut f, 3200, -18.0));
        assert!(check_response(&ctx, &mut f, 100, 0.0));
    }

    #[test]
    fn test_highpass_response() {
        let ctx = AudioContext::new(48000);
        let mut f = FirstOrder::default();
        f.highpass(&ctx, 600);
        assert!(check_response(&ctx, &mut f, 600, -3.0));
        assert!(check_response(&ctx, &mut f, 150, -12.0));
        assert!(check_response(&ctx, &mut f, 75, -18.0));
        assert!(check_response(&ctx, &mut f, 2400, 0.0));
    }
}
