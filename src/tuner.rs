// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Daniel Thompson

//! Monophonic instrument tuner.
//!
//! [analyse_note] estimates the fundamental frequency of a monophonic
//! musical signal and reports the nearest equal tempered note together
//! with the tuning error in cents.
//!
//! The estimator works in two stages:
//!
//! 1. A coarse estimate is obtained from a decimated copy of the input.
//!    Decimation keeps the arithmetic (and the FFT) small even at high
//!    sample rates. The fundamental is identified from the magnitude
//!    spectrum as the lowest peak that the strongest peak is an integer
//!    multiple of (which copes with notes whose fundamental is weaker
//!    than their harmonics) and a parabolic fit to that peak provides a
//!    coarse frequency.
//!
//! 2. The coarse estimate is refined to sub-cent accuracy by measuring
//!    the phase difference of the fundamental between the two halves of
//!    the buffer. For a stationary tone the phase difference is exactly
//!    proportional to the frequency, so the estimate is not limited by
//!    the FFT bin spacing.
//!
//! The decimation filter is linear phase, so the phase it adds to the
//! fundamental is identical in both halves of the buffer and cancels
//! during the refinement.

use crate::*;
use std::f64::consts::PI;

/// The result of analysing a buffer for a musical note.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteAnalysis {
    /// MIDI note number (0..=127) of the nearest note.
    pub note: u8,
    /// Frequency of the detected note in Hz.
    pub frequency: f32,
    /// Tuning error relative to the nearest note, in cents.
    pub cents: f32,
}

impl Default for NoteAnalysis {
    fn default() -> Self {
        Self {
            note: 0,
            frequency: 0.0,
            cents: 0.0,
        }
    }
}

/// Lowest fundamental we attempt to detect. A0 is 27.5Hz.
const F_MIN: f64 = 25.0;

/// Highest fundamental we attempt to detect. C8 is 4186Hz.
const F_MAX: f64 = 4500.0;

/// Target sample rate for the coarse analysis stage.
const COARSE_RATE: f64 = 24000.0;

/// Length of the anti-alias FIR used when decimating.
const DECIM_FILTER_LEN: usize = 128;

/// Analyse a buffer of monophonic audio and estimate the note being played.
///
/// The buffer is expected to hold roughly one second of audio sampled at
/// `ctx.sampling_frequency`. Longer buffers improve the accuracy, shorter
/// buffers reduce it. The returned [NoteAnalysis] contains the nearest
/// equal tempered note and the error in cents.
pub fn analyse_note(samples: &[Sample], ctx: &AudioContext) -> NoteAnalysis {
    let sr = ctx.sampling_frequency as f64;
    let len = samples.len();

    if sr <= 0.0 || len < 256 {
        return NoteAnalysis::default();
    }

    // Remove any DC offset; it would otherwise leak into the low bins.
    let mean = samples.iter().map(|&x| x as f64).sum::<f64>() / len as f64;

    // Decimate to a manageable rate. The anti-alias filter is linear phase
    // so the phase difference measured later is unaffected by it.
    let m = ((sr / COARSE_RATE).floor() as usize).max(1);
    let fa = sr / m as f64;
    let dec = decimate(samples, mean, m);

    if dec.len() < 64 {
        return NoteAnalysis::default();
    }

    // Stage 1: coarse frequency.
    let f0_coarse = coarse_frequency(&dec, fa);
    if !(F_MIN..=F_MAX).contains(&f0_coarse) {
        return NoteAnalysis::default();
    }

    // Stage 2: phase based refinement.
    let f0 = refine_frequency(&dec, fa, f0_coarse);

    note_from_frequency(f0)
}

/// Convert a frequency into the nearest equal tempered note.
fn note_from_frequency(freq: f64) -> NoteAnalysis {
    if !freq.is_finite() || freq <= 0.0 {
        return NoteAnalysis::default();
    }

    let midi = 69.0 + 12.0 * (freq / 440.0).log2();
    let note = midi.round().clamp(0.0, 127.0) as u8;
    let note_freq = 440.0 * 2.0_f64.powf((note as f64 - 69.0) / 12.0);
    let cents = 1200.0 * (freq / note_freq).log2();

    NoteAnalysis {
        note,
        frequency: freq as f32,
        cents: cents as f32,
    }
}

/// Low pass and downsample by `m`.
///
/// The filter is a windowed sinc with a cutoff of `0.4 * sr / m`, i.e. it
/// passes everything up to a little below the new Nyquist frequency. Only
/// the samples that are actually kept are computed.
fn decimate(samples: &[Sample], mean: f64, m: usize) -> Vec<f64> {
    if m <= 1 {
        return samples.iter().map(|&x| x as f64 - mean).collect();
    }

    let h = design_decimation_filter(m);
    let l = h.len();
    let n = samples.len();

    if n < l {
        return Vec::new();
    }

    let out_len = (n - l) / m + 1;
    let mut out = Vec::with_capacity(out_len);

    for j in 0..out_len {
        let base = j * m;
        let mut acc = 0.0;
        for i in 0..l {
            acc += h[i] * samples[base + i] as f64;
        }
        // The coefficients sum to one so subtracting the mean afterwards is
        // equivalent to removing it from the input.
        out.push(acc - mean);
    }

    out
}

/// Design a windowed sinc low pass filter for decimation by `m`.
///
/// The cutoff is `0.4 / m` (normalised to the input sample rate) which
/// leaves a little room for the transition band below the new Nyquist
/// frequency of `0.5 / m`.
fn design_decimation_filter(m: usize) -> Vec<f64> {
    let fc = 0.4 / m as f64;
    let len = DECIM_FILTER_LEN;
    let center = (len - 1) as f64 / 2.0;
    let mut h = vec![0.0; len];
    let mut sum = 0.0;

    for (i, coeff) in h.iter_mut().enumerate() {
        let x = i as f64 - center;
        let a = 2.0 * PI * fc * x;
        let sinc = if a.abs() < 1e-12 { 1.0 } else { a.sin() / a };
        let w = 0.42 - 0.5 * (2.0 * PI * i as f64 / (len - 1) as f64).cos()
            + 0.08 * (4.0 * PI * i as f64 / (len - 1) as f64).cos();
        *coeff = 2.0 * fc * sinc * w;
        sum += *coeff;
    }

    for coeff in &mut h {
        *coeff /= sum;
    }

    h
}

/// Estimate the fundamental frequency from a decimated buffer.
fn coarse_frequency(dec: &[f64], fa: f64) -> f64 {
    let n = dec.len();

    // Half a second is plenty to resolve the fundamental and its harmonics
    // while keeping the FFT size bounded.
    let n_seg = n.min((fa * 0.5) as usize).max(64);
    let n_fft = n_seg.next_power_of_two();

    // Magnitude spectrum of the windowed segment. A Blackman window is used
    // because its very low sidelobes let us use a low peak threshold without
    // picking up window skirts as if they were harmonics.
    let win = blackman(n_seg);
    let mut re = vec![0.0; n_fft];
    let mut im = vec![0.0; n_fft];
    for i in 0..n_seg {
        re[i] = dec[i] * win[i];
    }
    fft(&mut re, &mut im);

    let n_bins = n_fft / 2 + 1;
    let mut mags = vec![0.0; n_bins];
    for k in 0..n_bins {
        mags[k] = (re[k] * re[k] + im[k] * im[k]).sqrt();
    }

    let bin_hz = fa / n_fft as f64;
    spectral_f0(&mags, bin_hz)
}

/// Find the fundamental from a magnitude spectrum.
///
/// The fundamental is taken to be the lowest spectral peak that the
/// strongest peak is an (approximately) integer multiple of. This finds the
/// fundamental even when it is much weaker than the harmonics, while a pure
/// tone simply reports its own peak.
fn spectral_f0(mags: &[f64], bin_hz: f64) -> f64 {
    let n = mags.len();
    if n < 5 {
        return 0.0;
    }

    // Find the strongest peak.
    let mut max_mag = 0.0;
    let mut max_bin = 1;
    for (k, &m) in mags.iter().enumerate().skip(1) {
        if m > max_mag {
            max_mag = m;
            max_bin = k;
        }
    }
    if max_mag <= 0.0 {
        return 0.0;
    }

    let threshold = max_mag * 0.005;

    // Collect local maxima above the threshold.
    let mut peaks: Vec<usize> = Vec::new();
    for k in 1..n - 1 {
        if mags[k] > threshold && mags[k] >= mags[k - 1] && mags[k] > mags[k + 1] {
            peaks.push(k);
        }
    }

    // Discard peaks that sit in the skirt of a stronger peak.
    let mut clean: Vec<usize> = Vec::new();
    for &p in &peaks {
        let dominated = peaks
            .iter()
            .any(|&q| mags[q] > mags[p] && (q as isize - p as isize).unsigned_abs() <= 3);
        if !dominated {
            clean.push(p);
        }
    }

    // The fundamental is the lowest peak that the strongest peak is an
    // (approximately) integer multiple of. Interpolated peak positions are
    // used so that the ratio is not distorted by bin quantisation.
    let f_max = parabolic(mags, max_bin);
    for &p in &clean {
        let f_p = parabolic(mags, p);
        if f_p <= 0.0 {
            continue;
        }
        let ratio = f_max / f_p;
        let nearest = ratio.round();
        if nearest >= 1.0 && (ratio - nearest).abs() < 0.02 * nearest {
            return f_p * bin_hz;
        }
    }

    // Fall back to the strongest peak.
    f_max * bin_hz
}

/// Parabolic interpolation of a peak at index `k`.
fn parabolic(y: &[f64], k: usize) -> f64 {
    if k == 0 || k + 1 >= y.len() {
        return k as f64;
    }

    let a = y[k - 1];
    let b = y[k];
    let c = y[k + 1];
    let denom = a - 2.0 * b + c;

    if denom.abs() < 1e-30 {
        return k as f64;
    }

    k as f64 + 0.5 * (a - c) / denom
}

/// Refine a coarse frequency using the phase difference between the two
/// halves of the buffer.
///
/// For a tone at frequency `f` the phase of the DFT at a nearby frequency
/// advances by `2*pi*f*H/fa` between two windows separated by `H` samples.
/// Measuring that advance gives `f` directly, independent of the bin
/// spacing.
fn refine_frequency(dec: &[f64], fa: f64, f0_coarse: f64) -> f64 {
    let n = dec.len();
    let half = n / 2;

    if half < 16 {
        return f0_coarse;
    }

    let win = hann(half);
    let omega = 2.0 * PI * f0_coarse / fa;

    let (ra, ia) = dft_windowed(&dec[..half], &win, omega);
    let (rb, ib) = dft_windowed(&dec[half..2 * half], &win, omega);

    let pa = ia.atan2(ra);
    let pb = ib.atan2(rb);

    let mut dphi = pb - pa;
    while dphi > PI {
        dphi -= 2.0 * PI;
    }
    while dphi <= -PI {
        dphi += 2.0 * PI;
    }

    let h = half as f64;
    let m = (f0_coarse * h / fa - dphi / (2.0 * PI)).round();

    (dphi + 2.0 * PI * m) * fa / (2.0 * PI * h)
}

/// Evaluate the windowed DFT at an arbitrary frequency.
///
/// Returns the real and imaginary parts of `sum y[n] * exp(-j*omega*n)`.
fn dft_windowed(x: &[f64], w: &[f64], omega: f64) -> (f64, f64) {
    let (cw, sw) = (omega.cos(), omega.sin());
    let (mut c, mut s) = (1.0, 0.0);
    let mut re = 0.0;
    let mut im = 0.0;

    for (xi, wi) in x.iter().zip(w.iter()) {
        let y = xi * wi;
        re += y * c;
        im -= y * s;

        let nc = c * cw - s * sw;
        s = s * cw + c * sw;
        c = nc;
    }

    (re, im)
}

/// A Hann window of length `n`.
fn hann(n: usize) -> Vec<f64> {
    let mut w = vec![0.0; n];

    if n == 1 {
        w[0] = 1.0;
        return w;
    }

    for (i, v) in w.iter_mut().enumerate() {
        *v = 0.5 * (1.0 - (2.0 * PI * i as f64 / (n - 1) as f64).cos());
    }

    w
}

/// A Blackman window of length `n`.
fn blackman(n: usize) -> Vec<f64> {
    let mut w = vec![0.0; n];

    if n == 1 {
        w[0] = 1.0;
        return w;
    }

    for (i, v) in w.iter_mut().enumerate() {
        let t = 2.0 * PI * i as f64 / (n - 1) as f64;
        *v = 0.42 - 0.5 * t.cos() + 0.08 * (2.0 * t).cos();
    }

    w
}

/// In-place iterative radix-2 FFT.
fn fft(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    debug_assert!(n.is_power_of_two());
    debug_assert_eq!(n, im.len());

    // Bit reversal permutation.
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }

    // Butterflies.
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * PI / len as f64;
        let (wr, wi) = (ang.cos(), ang.sin());
        let half = len / 2;

        let mut i = 0;
        while i < n {
            let (mut cr, mut ci) = (1.0, 0.0);
            for k in 0..half {
                let ur = re[i + k];
                let ui = im[i + k];
                let xr = re[i + k + half];
                let xi = im[i + k + half];

                let vr = xr * cr - xi * ci;
                let vi = xr * ci + xi * cr;

                re[i + k] = ur + vr;
                im[i + k] = ui + vi;
                re[i + k + half] = ur - vr;
                im[i + k + half] = ui - vi;

                let ncr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = ncr;
            }
            i += len;
        }

        len <<= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate a one second tone from a list of harmonic amplitudes.
    fn make_tone(ctx: &AudioContext, freq: f64, harmonics: &[f64]) -> Vec<Sample> {
        let sr = ctx.sampling_frequency as f64;
        let n = ctx.sampling_frequency as usize;
        let mut buf = vec![0.0 as Sample; n];

        for (h, &amp) in harmonics.iter().enumerate() {
            let hf = freq * (h as f64 + 1.0);
            if hf >= sr / 2.0 {
                break;
            }

            // Phase accumulator to avoid a sin() call per sample.
            let omega = 2.0 * PI * hf / sr;
            let (cw, sw) = (omega.cos(), omega.sin());
            let (mut c, mut s) = (0.0, 1.0);
            for v in buf.iter_mut() {
                *v += (amp * s) as Sample;
                let nc = c * cw - s * sw;
                s = s * cw + c * sw;
                c = nc;
            }
        }

        let peak = buf.iter().fold(0.0 as Sample, |m, x| m.max(x.abs()));
        if peak > 0.0 {
            for v in buf.iter_mut() {
                *v /= peak;
            }
        }

        buf
    }

    fn midi_freq(note: u8) -> f64 {
        440.0 * 2.0_f64.powf((note as f64 - 69.0) / 12.0)
    }

    fn check_note(ctx: &AudioContext, note: u8, harmonics: &[f64]) {
        let freq = midi_freq(note);
        let buf = make_tone(ctx, freq, harmonics);
        let result = buf.analyse_note(ctx);

        assert_eq!(
            result.note, note,
            "note {} @ {}Hz detected as {} ({}Hz, {} cents)",
            note, ctx.sampling_frequency, result.note, result.frequency, result.cents
        );
        assert!(
            result.cents.abs() < 0.5,
            "note {} @ {}Hz error {} cents",
            note,
            ctx.sampling_frequency,
            result.cents
        );
    }

    #[test]
    fn test_note_from_frequency() {
        let a4 = note_from_frequency(440.0);
        assert_eq!(a4.note, 69);
        assert!(a4.cents.abs() < 1e-3);

        let a0 = note_from_frequency(27.5);
        assert_eq!(a0.note, 21);
        assert!(a0.cents.abs() < 1e-3);

        let c8 = note_from_frequency(4186.009);
        assert_eq!(c8.note, 108);
        assert!(c8.cents.abs() < 1e-2);

        // A quarter tone sharp should report the same note but positive cents.
        let sharp = note_from_frequency(440.0 * 2.0_f64.powf(0.25 / 12.0));
        assert_eq!(sharp.note, 69);
        assert!(sharp.cents > 20.0 && sharp.cents < 30.0);
    }

    #[test]
    fn test_pure_tones_all_notes() {
        let ctx = AudioContext::new(48000);
        for note in 21..=108 {
            check_note(&ctx, note, &[1.0]);
        }
    }

    #[test]
    fn test_harmonic_tones_all_notes() {
        let ctx = AudioContext::new(48000);
        for note in 21..=108 {
            check_note(&ctx, note, &[1.0, 0.6, 0.4, 0.25, 0.15]);
        }
    }

    #[test]
    fn test_weak_fundamental() {
        let ctx = AudioContext::new(48000);
        // The fundamental is much weaker than the second and third harmonics.
        for note in [21, 24, 28, 33, 40, 48, 60, 72, 84, 96, 108] {
            check_note(&ctx, note, &[0.1, 1.0, 0.8, 0.5, 0.3]);
        }
    }

    #[test]
    fn test_sample_rates() {
        // Cover every note at the extremes of the supported rate range.
        for rate in [32000, 192000] {
            let ctx = AudioContext::new(rate);
            for note in 21..=108 {
                check_note(&ctx, note, &[1.0, 0.6, 0.4, 0.25, 0.15]);
            }
        }

        // A representative subset at the other rates.
        let notes = [21, 24, 28, 33, 40, 48, 60, 72, 84, 96, 108];
        for rate in [44100, 88200, 96000, 176400] {
            let ctx = AudioContext::new(rate);
            for note in notes {
                check_note(&ctx, note, &[1.0, 0.6, 0.4, 0.25, 0.15]);
            }
        }
    }

    #[test]
    fn test_rich_harmonics() {
        // A piano-like spectrum with many harmonics. At high sample rates the
        // upper harmonics are above the decimated Nyquist frequency and
        // exercise the anti-alias filter.
        let harmonics: Vec<f64> = (1..=40).map(|h| 1.0 / h as f64).collect();

        for rate in [32000, 48000, 96000, 192000] {
            let ctx = AudioContext::new(rate);
            for note in [21, 28, 36, 45, 57, 69, 81, 93, 105, 108] {
                check_note(&ctx, note, &harmonics);
            }
        }
    }

    #[test]
    fn test_detuned_note() {
        let ctx = AudioContext::new(48000);
        let freq = 440.0 * 2.0_f64.powf(0.4 / 12.0);
        let buf = make_tone(&ctx, freq, &[1.0, 0.5, 0.3]);
        let result = buf.analyse_note(&ctx);

        assert_eq!(result.note, 69);
        assert!(
            (result.cents - 40.0).abs() < 0.5,
            "expected ~40 cents, got {}",
            result.cents
        );
    }

    #[test]
    fn test_silence() {
        let ctx = AudioContext::new(48000);
        let buf = vec![0.0 as Sample; 48000];
        let result = buf.analyse_note(&ctx);
        assert_eq!(result, NoteAnalysis::default());
    }

    #[test]
    fn test_short_buffer() {
        let ctx = AudioContext::new(48000);
        let buf = [0.0 as Sample; 16];
        let result = buf.analyse_note(&ctx);
        assert_eq!(result, NoteAnalysis::default());
    }
}
