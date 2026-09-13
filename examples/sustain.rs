// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Daniel Thompson

//! Measure the sustain of the [KarplusStrong] voice across the whole piano
//! range (MIDI 21..=108, i.e. A0..C8).
//!
//! For every note we render a pluck, follow the envelope and report two
//! numbers:
//!
//!  * the **initial level**, i.e. how loud the note is just after the pluck
//!    (the excitation's peak), and
//!  * the **quarter time**, i.e. how long it takes for the envelope to fall to
//!    a quarter of that initial level (two halvings, or -12dB).
//!
//! The notes are set up through [KarplusStrong::set_sustain], so the survey
//! shows the quarter time is roughly constant across the register rather than
//! the top of the keyboard dying first. A second pass requests several
//! different sustains on a few notes and checks that the measured quarter time
//! tracks what was asked for.
//!
//! Run with `cargo run --release --example sustain`.

use oxidamp::*;

const RATE: i32 = 48000;

/// Longest a single note is rendered for, in seconds.
const MAX_SECONDS: f64 = 12.0;

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

/// Render one note with the requested `sustain` and return (initial level,
/// quarter time in seconds).
///
/// The envelope is the peak of consecutive windows of about one period (at
/// least 64 samples, so the high notes still resolve). The quarter time is
/// [`None`] when the envelope never falls to a quarter of the initial level
/// within `max_seconds`.
fn measure(note: u8, sustain: f32, max_seconds: f64) -> (f32, Option<f64>) {
    let f = midi_freq(note) as f32;
    let window = ((RATE as f32 / f).round() as usize).clamp(64, 4096);
    let max = (max_seconds * RATE as f64) as usize;

    let ctx = AudioContext::new(RATE);
    let mut ks = KarplusStrong::default();
    ks.setup(&ctx);
    ks.tune(&ctx, f);
    ks.set_sustain(sustain);
    ks.trigger(1.0);

    let mut env: Vec<f32> = Vec::new();
    let mut peak = 0.0f32;
    let mut initial = 0.0f32;
    let mut i = 0;

    while i < max {
        peak = peak.max(ks.step().abs());
        i += 1;

        if i % window == 0 {
            env.push(peak);
            peak = 0.0;

            // The initial level is the peak of the first few windows, taking a
            // small maximum so the attack transient is not lost.
            if env.len() >= 3 {
                initial = env[..3].iter().fold(0.0f32, |m, s| m.max(*s));
                if initial > 0.0 && *env.last().unwrap() <= initial * 0.25 {
                    let seconds = (env.len() - 1) as f64 * window as f64 / RATE as f64;
                    return (initial, Some(seconds));
                }
            }
        }
    }

    (initial, None)
}

fn main() {
    let sustain = 1.0f32;

    println!(
        "KarplusStrong sustain across the piano range @ {}Hz (sustain {:.2}s)",
        RATE, sustain
    );
    println!(
        "{:>4} {:>4} {:>9} {:>9} {:>11} {:>8}",
        "midi", "note", "freq", "initial", "quarter(s)", "gain"
    );

    let mut done = Vec::new();
    for note in 21..=108u8 {
        let f = midi_freq(note) as f32;
        let (initial, q) = measure(note, sustain, MAX_SECONDS);
        if let Some(t) = q {
            done.push(t);
        }

        println!(
            "{:>4} {:>4} {:>9.1} {:>9.4} {:>11.4} {:>8.5}",
            note,
            note_name(note),
            f,
            initial,
            q.unwrap_or(MAX_SECONDS),
            sustain_to_gain(sustain, f)
        );
    }

    let min = done.iter().cloned().fold(f64::MAX, f64::min);
    let max = done.iter().cloned().fold(0.0f64, f64::max);
    println!(
        "quarter time spans {:.4}s .. {:.4}s, {:.1}x ({} notes never reached a quarter)",
        min,
        max,
        max / min,
        (108 - 21 + 1) - done.len()
    );

    // Request several sustains and confirm the measured quarter time tracks
    // them. It reads a little short at the top because the peak envelope is
    // dominated early on by harmonics, which decay faster than the fundamental
    // the curve targets.
    println!("\nRequested vs measured quarter time");
    println!(
        "{:>4} {:>4} {:>9} {:>10} {:>10} {:>7}",
        "midi", "note", "freq", "requested", "measured", "ratio"
    );
    for note in [45u8, 69, 93, 108] {
        let f = midi_freq(note) as f32;
        for sustain in [0.5f32, 1.0, 2.0] {
            let (_, q) = measure(note, sustain, MAX_SECONDS);
            let measured = q.unwrap_or(MAX_SECONDS);
            println!(
                "{:>4} {:>4} {:>9.1} {:>10.2} {:>10.4} {:>7.2}",
                note,
                note_name(note),
                f,
                sustain,
                measured,
                measured / sustain as f64
            );
        }
    }
}
