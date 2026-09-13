// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Daniel Thompson

//! Measure the tuning error of the [KarplusStrong] voice across the whole
//! piano range (MIDI 21..=108, i.e. A0..C8).
//!
//! Each note is excited and then held at a sustained level using
//! [KarplusStrong::set_gain] so that it lasts long enough to be analysed. The
//! resonator's DC mode is removed with a [DCBlocker] because it otherwise
//! swamps the spectrum of the high notes.
//!
//! The pitch is measured directly at the native sample rate: a coarse sweep
//! locks onto the fundamental, then the phase difference between the two
//! halves of the buffer refines it to sub-cent accuracy.
//!
//! Run with `cargo run --release --example tuning`.

use oxidamp::*;
use std::f64::consts::PI;

const RATE: i32 = 48000;

fn note_name(note: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = note as i32 / 12 - 1;
    format!("{}{}", NAMES[(note % 12) as usize], octave)
}

fn midi_freq(note: u8) -> f64 {
    440.0 * 2.0_f64.powf((note as f64 - 69.0) / 12.0)
}

fn hann(n: usize) -> Vec<f64> {
    let mut w = vec![0.0; n];
    for (i, v) in w.iter_mut().enumerate() {
        *v = 0.5 * (1.0 - (2.0 * PI * i as f64 / (n - 1) as f64).cos());
    }
    w
}

/// Windowed DFT at an arbitrary frequency.
fn dft_windowed(x: &[f32], w: &[f64], omega: f64) -> (f64, f64) {
    let (cw, sw) = (omega.cos(), omega.sin());
    let (mut c, mut s) = (1.0, 0.0);
    let (mut re, mut im) = (0.0, 0.0);
    for (xi, wi) in x.iter().zip(w.iter()) {
        let y = *xi as f64 * wi;
        re += y * c;
        im -= y * s;
        let nc = c * cw - s * sw;
        s = s * cw + c * sw;
        c = nc;
    }
    (re, im)
}

/// Estimate the pitch of `x`, expected to be close to `expected`.
fn measure(x: &[f32], rate: f64, expected: f64) -> f64 {
    let half = x.len() / 2;
    let win = hann(half);

    // Coarse: sweep either side of the expected frequency. A shorter window
    // keeps the sweep cheap while still locating the peak to well within the
    // ambiguity spacing of the phase refinement below.
    let cw = half.min(8192);
    let cwin = hann(cw);
    let step = 0.1;
    let (lo, hi) = (expected * 0.9, expected * 1.1);
    let mut coarse = expected;
    let mut best = -1.0;
    let mut f = lo;
    while f <= hi {
        let (re, im) = dft_windowed(&x[..cw], &cwin, 2.0 * PI * f / rate);
        let mag = re * re + im * im;
        if mag > best {
            best = mag;
            coarse = f;
        }
        f += step;
    }

    // Fine: the phase advance between the two halves is proportional to the
    // frequency, so it is not limited by the sweep resolution.
    let omega = 2.0 * PI * coarse / rate;
    let (ra, ia) = dft_windowed(&x[..half], &win, omega);
    let (rb, ib) = dft_windowed(&x[half..2 * half], &win, omega);

    let mut dphi = ib.atan2(rb) - ia.atan2(ra);
    while dphi > PI {
        dphi -= 2.0 * PI;
    }
    while dphi <= -PI {
        dphi += 2.0 * PI;
    }

    let h = half as f64;
    let m = (coarse * h / rate - dphi / (2.0 * PI)).round();
    (dphi + 2.0 * PI * m) * rate / (2.0 * PI * h)
}

fn main() {
    let ctx = AudioContext::new(RATE);
    let n = RATE as usize;

    let mut raw = vec![0.0f32; n];
    let mut clean = vec![0.0f32; n];

    println!(
        "KarplusStrong tuning error across the piano range @ {}Hz",
        RATE
    );
    println!(
        "{:>4} {:>4} {:>10} {:>10} {:>9}",
        "midi", "note", "target", "measured", "cents"
    );

    let mut sum_sq = 0.0f64;
    let mut max_abs = 0.0f64;

    for note in 21..=108u8 {
        let target = midi_freq(note);

        let mut ks = KarplusStrong::default();
        ks.setup(&ctx);
        ks.tune(&ctx, target as f32);
        ks.trigger(1.0);
        ks.set_gain(0.99999);
        for s in raw.iter_mut() {
            *s = ks.step();
        }

        // Remove the resonator's DC / very-low-frequency mode.
        let mut dc = DCBlocker::default();
        dc.setup(&ctx);
        dc.process(&raw, &mut clean);

        let measured = measure(&clean, RATE as f64, target);
        let cents = 1200.0 * (measured / target).log2();

        sum_sq += cents * cents;
        max_abs = max_abs.max(cents.abs());

        println!(
            "{:>4} {:>4} {:>10.2} {:>10.2} {:>9.2}",
            note,
            note_name(note),
            target,
            measured,
            cents
        );
    }

    println!();
    println!(
        "rms error {:.2} cents, max |error| {:.2} cents",
        (sum_sq / 88.0).sqrt(),
        max_abs
    );
}
