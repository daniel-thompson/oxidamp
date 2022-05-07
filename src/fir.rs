// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;
use std::iter::zip;

type Sample = f32;

#[derive(Debug)]
pub struct FIR<const O: usize, const P: usize> {
    coeffs: [Sample; O],
    zbuf: [Sample; P],
    zi: usize,
}

// Normally we'd use derive(Default) for this but the const generics appear
// to confuse the macro
impl<const O: usize, const P: usize> Default for FIR<O, P> {
    fn default() -> Self {
        Self {
            coeffs: [0.0; O],
            zbuf: [0.0; P],
            zi: 0,
        }
    }
}

impl<const O: usize, const P: usize> FIR<O, P> {
    pub fn setup(&mut self, coeffs: &[Sample]) {
        for (p, q) in zip(&mut self.coeffs, coeffs) {
            *p = *q;
        }
    }

    /// Inner MAC loop.
    ///
    /// Multiply the provided history buffer and the coefficients and
    /// return the results. The primary reason to factor this out into an
    /// inline is that this makes is super-obvious to a casual reader that the
    /// core MAC loop can be vectorized automatically.
    fn micro_step(&self, zbuf: &[Sample]) -> Sample {
        let mut spl = 0.0 as Sample;
        for (z, c) in zip(zbuf, &self.coeffs) {
            spl += z * *c;
        }
        spl
    }
}

impl<const O: usize, const P: usize> Filter for FIR<O, P> {
    fn step(&mut self, spl: Sample) -> Sample {
        if self.zi < (P - O) {
            // inject a new sample into the Z buffer
            self.zbuf[self.zi + O] = spl;
            self.zi += 1;
        } else {
            // reposition the history buffer and append the latest sample
            self.zbuf.copy_within((P - O + 1)..P, 0);
            self.zbuf[O - 1] = spl;
            self.zi = 0;
        }

        self.micro_step(&self.zbuf[self.zi..(self.zi + O)])
    }

    fn process(&mut self, inbuf: &[Sample], outbuf: &mut [Sample]) {
        assert!(inbuf.len() == outbuf.len());
        let len = inbuf.len();

        // Initial priming until we can use inbuf directly as the z buffer
        for i in 0..O {
            outbuf[i] = self.step(inbuf[i]);
        }

        // Rip though the rest of the samples
        for i in O..len {
            let j = i + 1;
            outbuf[i] = self.micro_step(&inbuf[(j - O)..j]);
        }

        // Update the history buffer
        self.zbuf[0..O].copy_from_slice(&inbuf[(len - O)..len]);
        self.zi = 0;
    }

    fn flush(&mut self) {
        self.zbuf = [0.0; P];
        self.zi = 0;
    }
}

pub fn fir16_halfband() -> FIR<16, 31> {
    FIR::<16, 31> {
        coeffs: [
            -1.0658427656939486e-05,
            -0.00033168888351401706,
            0.002034312933821322,
            0.007595268660206917,
            -0.021569714800516036,
            -0.05238230981932257,
            0.12389807012472395,
            0.4407667202122573,
            0.4407667202122573,
            0.12389807012472395,
            -0.05238230981932257,
            -0.021569714800516036,
            0.007595268660206917,
            0.002034312933821322,
            -0.00033168888351401706,
            -1.0658427656939486e-05,
        ],
        ..Default::default()
    }
}

pub fn fir64_halfband() -> FIR<64, 64> {
    FIR::<64, 64> {
        coeffs: [
            1.1741898894196146e-05,
            3.946877907395632e-05,
            -9.113103950383317e-06,
            -0.00012622147323243955,
            -9.801069542248084e-05,
            0.00021800061731000242,
            0.0004079265182793447,
            -0.00011393808187227738,
            -0.0009016126785682302,
            -0.0005034415086256887,
            0.0012458603206347953,
            0.0018410269657195928,
            -0.0007282429831308869,
            -0.003592996464068533,
            -0.0014721594130666293,
            0.004609534840395266,
            0.005616005232427697,
            -0.003007585520122609,
            -0.010587572317815758,
            -0.003026392328422272,
            0.013450734175940005,
            0.013882073481272916,
            -0.009854019477921592,
            -0.027186904507968915,
            -0.004746883115706027,
            0.037076343116378474,
            0.03407056158565557,
            -0.0335386358353364,
            -0.08373781455466003,
            -0.005899019722973638,
            0.19798444296382173,
            0.37868278295794966,
            0.37868278295794966,
            0.19798444296382173,
            -0.005899019722973638,
            -0.08373781455466003,
            -0.0335386358353364,
            0.03407056158565557,
            0.037076343116378474,
            -0.004746883115706027,
            -0.027186904507968915,
            -0.009854019477921592,
            0.013882073481272916,
            0.013450734175940005,
            -0.003026392328422272,
            -0.010587572317815758,
            -0.003007585520122609,
            0.005616005232427697,
            0.004609534840395266,
            -0.0014721594130666293,
            -0.003592996464068533,
            -0.0007282429831308869,
            0.0018410269657195928,
            0.0012458603206347953,
            -0.0005034415086256887,
            -0.0009016126785682302,
            -0.00011393808187227738,
            0.0004079265182793447,
            0.00021800061731000242,
            -9.801069542248084e-05,
            -0.00012622147323243955,
            -9.113103950383317e-06,
            3.946877907395632e-05,
            1.1741898894196146e-05,
        ],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Whitebox test to verify zi is managed as expected.
    #[test]
    fn test_16_31_steps() {
        let mut fir = FIR::<16, 31>::default();
        for i in 0..64 {
            assert_eq!(fir.step(i as f32), 0.0);
            assert_eq!(fir.zi, (i + 1) % 16);
        }
    }

    #[test]
    fn test_16_16_steps() {
        let mut fir = FIR::<16, 16>::default();
        for i in 0..64 {
            assert_eq!(fir.step(i as f32), 0.0);
            assert_eq!(fir.zi, 0);
        }
    }

    #[test]
    fn test_fir16_halfband() {
        let ctx = AudioContext::new(48000);
        let mut fir = fir16_halfband();
        // assert the responses predicted by the filter designer
        assert!(check_response(&ctx, &mut fir, 4800, 0.0));
        assert!(check_response(&ctx, &mut fir, 2 * 4800, -2.5));
        assert!(check_response(&ctx, &mut fir, 3 * 4800, -12.0));
        assert!(check_response(&ctx, &mut fir, 4 * 4800, -42.0));
        assert!(check_response(&ctx, &mut fir, 5 * 4800, -96.0));
    }

    #[test]
    fn test_fir64_halfband() {
        let ctx = AudioContext::new(48000);
        let mut fir = fir64_halfband();
        // assert the responses predicted by the filter designer
        assert!(check_response(&ctx, &mut fir, 1000, 0.0));
        assert!(check_response(&ctx, &mut fir, 4000, 0.0));
        assert!(check_response(&ctx, &mut fir, 8000, 0.0));
        assert!(check_response(&ctx, &mut fir, 9000, -1.3));
        assert!(check_response(&ctx, &mut fir, 10000, -9.0));
        assert!(check_response(&ctx, &mut fir, 11000, -29.0));
        assert!(check_response(&ctx, &mut fir, 12000, -96.0));
    }

    #[test]
    fn test_fir16_step() {
        let mut fir = FIR::<16, 31> {
            coeffs: [0.5; 16],
            ..Default::default()
        };

        for i in 0..64 {
            let inspl = if 0 == i % 16 { 1.0 } else { 0.0 };
            let outspl = fir.step(inspl);
            assert_eq!(outspl, 0.5);
        }
    }

    #[test]
    fn test_fir16_process() {
        let mut fir = FIR::<16, 31> {
            coeffs: [0.5; 16],
            ..Default::default()
        };

        let mut inbuf = [0.0; 64];
        inbuf[0] = 1.0;
        inbuf[16] = 1.0;
        inbuf[32] = 1.0;
        inbuf[48] = 1.0;

        let mut outbuf = [0.0; 64];
        fir.process(&inbuf, &mut outbuf);

        for outspl in &outbuf {
            assert_eq!(*outspl, 0.5);
        }
    }

    #[test]
    fn test_fir64_coprocess() {
        let ctx = AudioContext::new(48000);
        let mut inbuf = [0.0; 1024];

        let mut sg = SineGenerator::default();
        sg.setup(&ctx, 440, db2linear(-3.0));
        sg.process(&mut inbuf);

        let mut stepbuf = [0.0; 1024];
        let mut stepfir = fir64_halfband();
        for (i, s) in zip(&inbuf, &mut stepbuf) {
            *s = stepfir.step(*i);
        }

        let mut procbuf = [0.0; 1024];
        let mut procfir = fir64_halfband();
        procfir.process(&inbuf, &mut procbuf);

        for (s, p) in zip(&stepbuf, &procbuf) {
            assert_eq!(*s, *p);
        }
    }
}
