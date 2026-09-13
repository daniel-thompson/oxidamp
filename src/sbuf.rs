// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;
use rustfft::FftPlanner;
use rustfft::num_complex::Complex;
use std::iter::zip;

pub type Sample = f32;

pub trait SignalGenerator {
    /// Generate a single sample.
    fn step(&mut self) -> Sample;

    /// Generate a complete sample buffer.
    ///
    /// Most generators can use this simple default implementation that
    /// calls [SignalGenerator::step] in a loop.
    fn process(&mut self, samples: &mut [Sample]) {
        for spl in samples {
            *spl = self.step();
        }
    }
}

pub trait Filter {
    /// Process a single sample.
    fn step(&mut self, spl: Sample) -> Sample;

    /// Reset filter state.
    fn flush(&mut self);

    /// Generate a complete sample buffer.
    ///
    /// Most filters can use this simple default implementation that
    /// calls [Filter::step] in a loop.
    fn process(&mut self, inbuf: &[Sample], outbuf: &mut [Sample]) {
        for (x, y) in zip(inbuf, outbuf) {
            *y = self.step(*x);
        }
    }

    /// Stimulate the filter with a specific pure-sine wave.
    ///
    /// TODO: *Does this really need to be a method?*
    fn stimulate(&mut self, ctx: &AudioContext, gfreq: i32) -> Sample {
        let mut inbuf = [0.0_f32; 1024];
        let mut outbuf = [0.0_f32; 1024];
        let mut sg = SineGenerator::default();

        sg.setup(ctx, gfreq, 1.570793);

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
    fn analyse_frequency(&self, spectrum: &mut [Sample]);
    fn analyse_note(&self, ctx: &AudioContext) -> Sample;
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

    /// Analyse the frequency spectrum of the signal using an FFT.
    ///
    /// The buffer may be of any length; rustfft selects an appropriate
    /// algorithm internally (including for prime lengths). Bin `k` of
    /// `spectrum` holds the magnitude at `k * sampling_frequency / len`
    /// Hz, so bin 0 is DC and the highest useful bin sits at the Nyquist
    /// frequency. For real-valued input the bins above `len / 2` mirror
    /// the bins below, so `spectrum` may hold at most `len / 2 + 1` bins;
    /// a shorter spectrum is filled with just the lowest bins.
    ///
    /// The magnitudes are scaled to be directly comparable with signal
    /// amplitudes: a sine wave that completes a whole number of cycles
    /// within the buffer yields a peak equal to its amplitude in the
    /// matching bin (and the DC bin reports any offset directly). No
    /// window function is applied, so signals that are not periodic
    /// within the buffer smear energy into neighbouring bins.
    ///
    /// This is intended for offline analysis: an FFT plan is built on
    /// every call.
    fn analyse_frequency(&self, spectrum: &mut [Sample]) {
        let n = self.len();
        assert!(n > 0);
        assert!(spectrum.len() <= n / 2 + 1);

        let fft = FftPlanner::<Sample>::new().plan_fft_forward(n);
        let mut buf: Vec<Complex<Sample>> =
            self.iter().map(|&spl| Complex::new(spl, 0.0)).collect();
        fft.process(&mut buf);

        for (k, mag) in spectrum.iter_mut().enumerate() {
            // DC and (for even lengths) Nyquist are their own mirror image
            // so they only count once towards the magnitude.
            let scale = if k == 0 || 2 * k == n { 1.0 } else { 2.0 } / (n as Sample);
            *mag = buf[k].norm() * scale;
        }
    }

    /// Determine the frequency (in Hz) of the note in a monophonic buffer.
    ///
    /// A coarse estimate is taken from the strongest bin of the spectrum
    /// (see [SampleBufferExt::analyse_frequency]) and then refined by
    /// locating the true peak of the Hann-windowed spectrum with a
    /// golden-section search. The refinement matters because one FFT bin
    /// spans `sampling_frequency / len` Hz, which is worth many semitones
    /// at low frequencies; interpolating the continuous spectrum instead
    /// pins the frequency to a tiny fraction of a bin.
    ///
    /// For a steady signal the result is accurate to much better than one
    /// cent (1/100 of a semitone, about 0.05Hz at 80Hz). The signal is
    /// assumed to be monophonic with the fundamental as its strongest
    /// component: harmonics are tolerated (the window keeps their
    /// sidelobes from biasing the estimate) but a harmonic louder than
    /// the fundamental would be reported instead. Any DC offset is
    /// removed before analysis.
    ///
    /// This is offline analysis (an FFT plus a few dozen passes over the
    /// buffer) and is not intended for use in the audio callback.
    fn analyse_note(&self, ctx: &AudioContext) -> Sample {
        let n = self.len();
        assert!(n > 1);
        let fs = ctx.sampling_frequency as Sample;
        assert!(fs > 0.0);

        // Coarse estimate: strongest spectral component, ignoring DC.
        let mut spectrum = vec![0.0; n / 2 + 1];
        self.analyse_frequency(&mut spectrum);
        let k0 = spectrum[1..]
            .iter()
            .enumerate()
            .max_by(|(_, p), (_, q)| p.total_cmp(q))
            .map(|(k, _)| k + 1)
            .unwrap();

        // Window the signal for the refinement. The Hann window suppresses
        // the sidelobes of the negative-frequency image and of any
        // harmonics by 60dB or more so they cannot bias the peak location,
        // and its main lobe is smooth and unimodal, which is what the
        // search below relies on.
        let mean = self.iter().sum::<Sample>() / (n as Sample);
        let x: Vec<f64> = self
            .iter()
            .enumerate()
            .map(|(i, &spl)| {
                let window =
                    0.5 - 0.5 * (2.0 * std::f64::consts::PI * (i as f64) / (n as f64)).cos();
                ((spl - mean) as f64) * window
            })
            .collect();

        // Magnitude of the windowed DTFT at an arbitrary frequency. The
        // twiddle factors are accumulated by repeated multiplication in
        // f64, keeping the drift over even very long buffers negligible.
        let magnitude = |freq: f64| -> f64 {
            let step = -2.0 * std::f64::consts::PI * freq / (fs as f64);
            let (sin, cos) = step.sin_cos();
            let (mut tw_re, mut tw_im) = (1.0, 0.0);
            let (mut re, mut im) = (0.0, 0.0);
            for &spl in &x {
                re += spl * tw_re;
                im -= spl * tw_im;
                let next = (tw_re * cos - tw_im * sin, tw_re * sin + tw_im * cos);
                tw_re = next.0;
                tw_im = next.1;
            }
            (re * re + im * im).sqrt()
        };

        // The true peak lies between the bins adjacent to the coarse
        // estimate, well inside the main lobe of the windowed spectrum.
        let bin = (fs / (n as Sample)) as f64;
        let phi = 0.6180339887498949_f64;
        let mut lo = ((k0 - 1) as f64) * bin;
        let mut hi = ((k0 + 1) as f64) * bin;
        let mut a = hi - phi * (hi - lo);
        let mut b = lo + phi * (hi - lo);
        let (mut fa, mut fb) = (magnitude(a), magnitude(b));

        for _ in 0..100 {
            if (hi - lo) <= 1.0e-6 * (hi + lo) {
                break;
            }
            if fa > fb {
                hi = b;
                b = a;
                fb = fa;
                a = hi - phi * (hi - lo);
                fa = magnitude(a);
            } else {
                lo = a;
                a = b;
                fa = fb;
                b = lo + phi * (hi - lo);
                fb = magnitude(b);
            }
        }

        (0.5 * (lo + hi)) as Sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyse_frequency_sine() {
        let ctx = AudioContext::new(48000);
        let mut sg = SineGenerator::default();
        // 1024 samples @ 48kHz puts 3000Hz exactly in bin 64
        sg.setup(&ctx, 3000, 0.5);
        let mut buf = [0.0_f32; 1024];
        sg.process(&mut buf);

        let mut spectrum = [0.0_f32; 513];
        buf.analyse_frequency(&mut spectrum);

        assert_fuzzeq!(spectrum[64], 0.5, 1.01);
        for (k, mag) in spectrum.iter().enumerate() {
            if k != 64 {
                assert!(*mag < 0.01, "bin {} leaked: {}", k, mag);
            }
        }
    }

    #[test]
    fn test_analyse_frequency_arbitrary_length() {
        let ctx = AudioContext::new(48000);
        let mut sg = SineGenerator::default();
        // 960 samples @ 48kHz (not a power of two) puts 3000Hz in bin 60
        sg.setup(&ctx, 3000, 0.5);
        let mut buf = [0.0_f32; 960];
        sg.process(&mut buf);

        let mut spectrum = [0.0_f32; 481];
        buf.analyse_frequency(&mut spectrum);

        assert_fuzzeq!(spectrum[60], 0.5, 1.01);
        for (k, mag) in spectrum.iter().enumerate() {
            if k != 60 {
                assert!(*mag < 0.01, "bin {} leaked: {}", k, mag);
            }
        }
    }

    #[test]
    fn test_analyse_frequency_prime_length() {
        let ctx = AudioContext::new(96100);
        let mut sg = SineGenerator::default();
        // 961 = 31*31 samples @ 96.1kHz puts 3000Hz exactly in bin 30
        sg.setup(&ctx, 3000, 0.5);
        let mut buf = [0.0_f32; 961];
        sg.process(&mut buf);

        let mut spectrum = [0.0_f32; 481];
        buf.analyse_frequency(&mut spectrum);

        assert_fuzzeq!(spectrum[30], 0.5, 1.01);
    }

    #[test]
    fn test_analyse_frequency_partial_spectrum() {
        let ctx = AudioContext::new(48000);
        let mut sg = SineGenerator::default();
        sg.setup(&ctx, 3000, 0.5);
        let mut buf = [0.0_f32; 1024];
        sg.process(&mut buf);

        // only the lowest 100 bins are wanted (and computed)
        let mut spectrum = [0.0_f32; 100];
        buf.analyse_frequency(&mut spectrum);

        assert_fuzzeq!(spectrum[64], 0.5, 1.01);
    }

    #[test]
    fn test_analyse_frequency_dc() {
        let buf = [0.25_f32; 64];
        let mut spectrum = [0.0_f32; 33];
        buf.analyse_frequency(&mut spectrum);

        assert_fuzzeq!(spectrum[0], 0.25, 1.01);
        for mag in &spectrum[1..] {
            assert!(*mag < 1e-4, "non-DC bin: {}", mag);
        }
    }

    #[test]
    fn test_analyse_frequency_two_tone() {
        let ctx = AudioContext::new(48000);
        let mut sg = SineGenerator::default();
        // 1024 samples @ 48kHz: 750Hz is bin 16, 3000Hz is bin 64
        let mut buf = [0.0_f32; 1024];
        let mut tmp = [0.0_f32; 1024];
        sg.setup(&ctx, 750, 0.8);
        sg.process(&mut buf);
        sg.setup(&ctx, 3000, 0.4);
        sg.process(&mut tmp);
        for (b, t) in zip(&mut buf, &tmp) {
            *b += t;
        }

        let mut spectrum = [0.0_f32; 513];
        buf.analyse_frequency(&mut spectrum);

        assert_fuzzeq!(spectrum[16], 0.8, 1.01);
        assert_fuzzeq!(spectrum[64], 0.4, 1.01);
    }

    #[test]
    #[should_panic]
    fn test_analyse_frequency_spectrum_too_long() {
        let buf = [0.0_f32; 100];
        let mut spectrum = [0.0_f32; 52];
        buf.analyse_frequency(&mut spectrum);
    }

    /// Synthesise a steady sine at an arbitrary (fractional-Hz) frequency.
    fn sine(freq: f32, len: usize, fs: i32) -> Vec<Sample> {
        (0..len)
            .map(|i| (2.0 * std::f32::consts::PI * freq * (i as f32) / (fs as f32)).sin())
            .collect()
    }

    fn cents(f: Sample, f0: f32) -> f32 {
        1200.0 * (f / f0).log2()
    }

    #[test]
    fn test_analyse_note_around_80hz() {
        let ctx = AudioContext::new(48000);
        // 8192 samples @ 48kHz: bins are 5.86Hz wide and 80Hz falls a
        // third of the way into bin 14. Sweep the note across the bin to
        // exercise exact and fractional bin positions alike.
        let bin = 48000.0_f32 / 8192.0;
        for frac in [0.0_f32, 0.2, 0.4, 0.5, 0.65, 0.8, 0.95] {
            let f0 = (13.0 + frac) * bin;
            let buf = sine(f0, 8192, 48000);

            let f = buf.analyse_note(&ctx);
            assert!(
                cents(f, f0).abs() < 0.1,
                "frac {}: {}Hz ({:.4} cents)",
                frac,
                f,
                cents(f, f0)
            );
        }
    }

    #[test]
    fn test_analyse_note_harmonics_and_dc() {
        let ctx = AudioContext::new(48000);
        let f0 = 80.0_f32;
        let mut buf = vec![0.5; 8192];
        for (i, spl) in buf.iter_mut().enumerate() {
            let t = 2.0 * std::f32::consts::PI * (i as f32) / 48000.0;
            *spl += (t * f0).sin() + 0.4 * (t * 2.0 * f0).sin() + 0.2 * (t * 3.0 * f0).sin();
        }

        let f = buf.analyse_note(&ctx);
        assert!(
            cents(f, f0).abs() < 0.1,
            "{}Hz ({:.4} cents)",
            f,
            cents(f, f0)
        );
    }

    #[test]
    fn test_analyse_note_440hz() {
        let ctx = AudioContext::new(48000);
        // deliberately off A440 and off-bin (37.7 bins for this buffer)
        let f0 = 442.0_f32;
        let buf = sine(f0, 4096, 48000);

        let f = buf.analyse_note(&ctx);
        assert!(
            cents(f, f0).abs() < 0.1,
            "{}Hz ({:.4} cents)",
            f,
            cents(f, f0)
        );
    }
}
