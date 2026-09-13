// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

/// Longest delay line we can represent, in samples, and therefore the largest
/// excitation buffer we ever need (one period of the lowest note).
const MAX_PERIOD: usize = 1920;

/// Widest plectrum/finger contact, as a fraction of the string length. Used
/// when the pluck is at its softest and removes the most high frequency
/// energy.
const MAX_CONTACT: f32 = 0.04;

/// Narrowest contact, as a fraction of the string length. A hard, thin pick
/// leaves an almost ideal corner so the excitation keeps its full upper
/// harmonics.
const MIN_CONTACT: f32 = 0.004;

/// Bounds of the sustain control, in seconds.
const MIN_SUSTAIN: f32 = 0.01;
const MAX_SUSTAIN: f32 = 30.0;

/// Front panel controls for a [KarplusStrong] voice.
///
/// The strike displacement is not here: that is supplied per note by the
/// velocity passed to [Voice::trigger].
#[derive(Clone, Copy, Debug)]
pub struct SynthConfig {
    /// Pluck point as a fraction of the string length from one end.
    pub position: f32,
    /// How strongly the strike displacement sharpens the pluck.
    pub hardness: f32,
    /// How long the string rings, as the time in seconds for the note to decay
    /// to a quarter of its initial level. Converted to a per-note loop gain by
    /// [sustain_to_gain] every time the frequency changes.
    pub sustain: f32,
}

impl Default for SynthConfig {
    fn default() -> Self {
        Self {
            position: 0.25,
            hardness: 0.6,
            sustain: 1.0,
        }
    }
}

/// Q of the narrow peaking EQ used to cancel the loop filter's loss at the
/// note frequency. Low enough to be broad compared with the note spacing, high
/// enough that it does not lift the harmonics above unity once the loop gain is
/// folded in.
const EQ_Q: f64 = 1.5;

/// Magnitude of everything in the feedback path at `freq` apart from the loop
/// gain: the two tap average that sets the tuning, and the fractional delay's
/// interpolation. It is the loop gain's job to compensate this.
fn loop_loss(freq: f32, sfreq: f32) -> f32 {
    let w = 2.0 * std::f32::consts::PI * freq / sfreq;

    // The two tap average used for tuning has |H| = |cos(w/2)|.
    let average = (0.5 * w).cos().abs();

    // The fractional delay interpolates between two taps, which costs a little
    // more the further the weight sits from an integer. `frac` is the weight
    // [KarplusStrong::tune] will program into the delay line.
    let n = sfreq / freq - 0.5;
    let frac = n.ceil() - n;
    let interp = (1.0 - 4.0 * frac * (1.0 - frac) * (0.5 * w).sin().powi(2))
        .max(0.0)
        .sqrt();

    (average * interp).max(1.0e-6)
}

/// Compensation curve: the loop gain that gives a note at `freq` a quarter
/// time of `sustain` seconds.
///
/// The string is a delay loop whose signal is multiplied by the loop gain once
/// per round trip. Since there are `freq` round trips per second, the note has
/// decayed to a quarter after
///
/// ```text
///     sustain = ln(4) / (freq · ln(1 / G))
/// ```
///
/// seconds, where `G` is the round trip gain. Solving for `G` gives the curve
///
/// ```text
///     gain = 0.25^(1 / (freq · sustain))
/// ```
///
/// The exponent is the compensation: without it the high notes, which complete
/// many more round trips per second, always decay first. The loop's own
/// frequency dependent loss ([loop_loss]) is cancelled separately, by a narrow
/// peaking EQ at the note frequency - folding it into the gain would need a gain
/// above one, which makes the loop unstable.
pub fn sustain_to_gain(sustain: f32, freq: f32) -> f32 {
    let q = sustain.clamp(MIN_SUSTAIN, MAX_SUSTAIN);
    let f = freq.max(1.0);

    0.25f32.powf(1.0 / (f * q))
}

/// A plucked string voice.
///
/// This is a Karplus-Strong resonator excited with a physical model of the
/// pluck rather than a burst of noise.
///
/// Picking a string displaces it into (approximately) a triangle peaked at the
/// pluck point. The triangle is loaded into the delay line for one period and
/// then left to circulate through the loop filter, so the resonator rings with
/// the harmonics of that initial shape. For an ideal string of length normalised
/// to one, plucked a fraction `p` from the end with an initial displacement `d`,
/// the harmonic amplitudes are
///
/// ```text
/// A_n ∝ d / (n² · p(1-p)) · sin(nπp)
/// ```
///
/// which gives two of the classic, physically meaningful controls for free:
///
///  * the `sin(nπp)` term is a comb with nulls at harmonics `k/p`, so
///    [KarplusStrong::set_position] decides *which* harmonics are present
///    (pluck towards the bridge for a thin, bright tone; towards the middle for
///    a hollow, flute-like tone); and
///  * the `1/n²` term is the natural mellow tilt of a plucked string.
///
/// The remaining `d` is the strike displacement. In the ideal linear model it
/// is pure level, but a real pluck also gets brighter the harder it is played
/// (a larger displacement and a firmer pick leave a sharper corner at the
/// pluck point). The velocity passed to [Voice::trigger] sets `d`, so harder
/// notes are louder and - through [KarplusStrong::set_hardness] - brighter too.
#[derive(Debug)]
pub struct KarplusStrong {
    delay: FracDelay<MAX_PERIOD>,
    filter: FirstOrder,
    /// Narrow peaking EQ at the note frequency. It cancels the loop's own loss
    /// at the fundamental so that the loop gain can stay below one while the
    /// note sustains as long as [sustain_to_gain] asks for.
    eq: Biquad,

    /// One period of initial string displacement, generated by [trigger].
    excite: [f32; MAX_PERIOD],
    excite_len: usize,
    excite_pos: usize,

    /// Pluck point as a fraction of the string length from one end, 0 < p < 1.
    position: f32,
    /// Strike displacement, i.e. how far/hard the string is pulled.
    strike: f32,
    /// How strongly the strike sharpens the pluck. 0 makes the strike pure
    /// level (ideal linear string); 1 makes a hard strike much brighter.
    hardness: f32,

    /// Sampling frequency, cached from [AudioContext].
    sfreq: f32,
    /// Frequency of the current note, in Hz, set by [Voice::tune].
    freq: f32,
    /// Loop period in samples, set by [Voice::tune].
    period: f32,
    /// Current loop gain.
    gain: f32,
    /// Sustain control, i.e. the time in seconds for the note to decay to a
    /// quarter of its initial level. [Voice::tune] converts it to a per-note
    /// loop gain via [sustain_to_gain].
    sustain: f32,
}

impl Default for KarplusStrong {
    fn default() -> Self {
        Self {
            delay: FracDelay::default(),
            filter: FirstOrder::default(),
            eq: Biquad::default(),
            excite: [0.0; MAX_PERIOD],
            excite_len: 0,
            excite_pos: 0,
            position: 0.25,
            strike: 0.8,
            hardness: 0.6,
            sfreq: 48000.0,
            freq: 120.0,
            period: 400.0,
            gain: 0.999,
            sustain: 1.0,
        }
    }
}

impl KarplusStrong {
    /// Set how long the string sustains.
    ///
    /// `sustain` is the time in seconds for the note to decay to a quarter of
    /// its initial level. The loop gain needed to achieve it is looked up in
    /// the compensation curve ([sustain_to_gain]) and is re-derived every time
    /// the frequency changes, so every note on the keyboard decays at the same
    /// rate rather than the high notes dying first.
    pub fn set_sustain(&mut self, sustain: f32) {
        self.sustain = sustain.clamp(MIN_SUSTAIN, MAX_SUSTAIN);
        self.gain = sustain_to_gain(self.sustain, self.freq);
    }

    /// Set the pluck point, as a fraction of the string length from one end.
    ///
    /// Values towards the ends give a thin, bright tone (harmonics at multiples
    /// of `1/p` are suppressed); a value near 0.5 plucks the middle and gives a
    /// hollow tone with the even harmonics missing. Applied on the next
    /// [Voice::trigger].
    pub fn set_position(&mut self, position: f32) {
        self.position = position.clamp(1.0e-3, 1.0 - 1.0e-3);
    }

    /// Set how strongly the strike displacement sharpens the pluck.
    ///
    /// At 0 the strike affects level only, as in an ideal linear string. At 1
    /// a hard strike leaves a very sharp corner and a lot of high frequency
    /// energy, while a light strike is rounded off and mellow. Applied on the
    /// next [Voice::trigger].
    pub fn set_hardness(&mut self, hardness: f32) {
        self.hardness = hardness.clamp(0.0, 1.0);
    }

    /// Apply a set of front panel controls.
    ///
    /// [KarplusStrong::set_position] and [KarplusStrong::set_hardness] take
    /// effect on the next [Voice::trigger]; the sustain applies immediately.
    pub fn set_config(&mut self, config: SynthConfig) {
        self.set_position(config.position);
        self.set_hardness(config.hardness);
        self.set_sustain(config.sustain);
    }

    /// Build one period of initial string displacement into `excite`.
    fn build_excitation(&mut self) {
        let n = (self.period.round() as usize).clamp(2, MAX_PERIOD);
        let p = self.position.clamp(1.0e-3, 1.0 - 1.0e-3);

        // A firmer strike and/or a harder pick leave a sharper corner at the
        // pluck point. Model this as a one pole smoother over a fraction of the
        // period: a narrow contact barely rounds the corner (bright) and a wide
        // one rolls off the upper harmonics (mellow). Working in fractions of
        // the period keeps the character consistent across the register rather
        // than letting it drift with absolute frequency.
        let hardness = (self.strike * self.hardness).clamp(0.0, 1.0);
        let contact = MIN_CONTACT + (MAX_CONTACT - MIN_CONTACT) * (1.0 - hardness);
        let width = (contact * n as f32).max(0.25);
        let alpha = 1.0 - (-1.0 / width).exp();

        // Two passes: the first settles the smoother so the shape is smoothed
        // cyclically, with no discontinuity where the period wraps around.
        let mut y = 0.0;
        for i in 0..(2 * n) {
            let phase = (i % n) as f32 / n as f32;
            let tri = if phase < p {
                phase / p
            } else {
                (1.0 - phase) / (1.0 - p)
            };

            y += alpha * (tri - y);

            if i >= n {
                // The triangle's mean is exactly 1/2, so removing it excites no
                // DC mode. The factor of two restores the +/-strike peak.
                self.excite[i - n] = (y - 0.5) * 2.0 * self.strike;
            }
        }

        self.excite_len = n;
        self.excite_pos = 0;

        // Remove any residual DC exactly. The loop filter preserves DC, so even
        // though the loop gain is below one, there is no reason to excite a mode
        // that only decays slowly.
        let sum: f64 = self.excite[..n].iter().map(|s| *s as f64).sum();
        let mean = (sum / n as f64) as f32;
        for s in &mut self.excite[..n] {
            *s -= mean;
        }
    }
}

impl Voice for KarplusStrong {
    fn setup(&mut self, ctx: &AudioContext) {
        self.sfreq = ctx.sampling_frequency as f32;
        self.filter.lowpass(ctx, ctx.sampling_frequency / 4);
        self.tune(ctx, self.freq);
    }

    fn trigger(&mut self, velocity: f32) {
        // Velocity is the strike displacement: pulling the string further
        // injects more energy and, through the hardness coupling, is also
        // brighter. Clamp so a velocity of 0 stays silent.
        self.strike = velocity.clamp(0.0, 1.0);
        self.build_excitation();

        // The loop gain was derived from the sustain when the frequency was
        // set; do not reset it here.
    }

    fn mute(&mut self) {
        // Damp the string by asking [sustain_to_gain] for the gain that would
        // give a sustain 50 times shorter, i.e. a note that dies away in a few
        // round trips. Going through the curve keeps this below the sounding
        // gain, so muting can only ever reduce it; the full sustain is restored
        // by [Voice::tune] the next time the frequency changes.
        self.gain = sustain_to_gain(self.sustain / 50.0, self.freq);
    }

    fn tune(&mut self, ctx: &AudioContext, freq: f32) {
        self.freq = freq;
        self.period = self.sfreq / freq;

        // The loop is delay + two tap average. The average has a group delay of
        // exactly half a sample, so the delay line is half a sample shorter than
        // the period.
        self.delay.setup(ctx, self.period - 0.5);

        // Cancel the loop's loss at this note with a narrow peaking EQ centred
        // on the note. Its phase is zero at the centre, so tuning is unchanged,
        // and being narrow it does not lift any other mode near unity.
        let loss = loop_loss(freq, self.sfreq);
        let boost_db = -20.0 * loss.log10();
        self.eq
            .peakingeq(ctx, freq.round() as i32, boost_db as f64, EQ_Q);
        self.eq.flush();

        // Sustain is frequency dependent, so re-derive the loop gain here,
        // whenever the note's frequency changes.
        self.gain = sustain_to_gain(self.sustain, self.freq);
    }
}

impl SignalGenerator for KarplusStrong {
    fn step(&mut self) -> f32 {
        let mut spl = if self.excite_pos < self.excite_len {
            let s = self.excite[self.excite_pos];
            self.excite_pos += 1;
            s
        } else {
            0.0
        };

        spl += self.gain * self.eq.step(self.filter.step(self.delay.peek()));
        let _ = self.delay.step(spl);

        spl
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    /// Magnitude of the `h`-th harmonic of a periodic buffer.
    fn harmonic(x: &[f32], h: usize) -> f32 {
        let n = x.len();
        let (mut re, mut im) = (0.0_f32, 0.0_f32);

        for (i, s) in x.iter().enumerate() {
            let th = 2.0 * PI * h as f32 * i as f32 / n as f32;
            re += s * th.cos();
            im -= s * th.sin();
        }

        (re * re + im * im).sqrt()
    }

    fn excited(position: f32, velocity: f32, hardness: f32) -> KarplusStrong {
        let ctx = AudioContext::new(48000);
        let mut ks = KarplusStrong::default();
        ks.setup(&ctx);
        ks.set_position(position);
        ks.set_hardness(hardness);
        ks.tune(&ctx, 200.0);
        ks.trigger(velocity);
        ks
    }

    #[test]
    fn test_excitation_is_dc_free() {
        let ks = excited(0.3, 0.8, 0.5);
        let n = ks.excite_len;
        let mean: f32 = ks.excite[..n].iter().sum::<f32>() / n as f32;

        assert!(mean.abs() < 0.01, "mean {mean}");
    }

    #[test]
    fn test_pluck_position_comb() {
        // Plucking the midpoint suppresses the even harmonics.
        let ks = excited(0.5, 1.0, 0.0);
        let h1 = harmonic(&ks.excite[..ks.excite_len], 1);
        let h2 = harmonic(&ks.excite[..ks.excite_len], 2);

        assert!(h1 > 0.0);
        assert!(h2 < h1 * 0.05, "h1 {h1} h2 {h2}");
    }

    #[test]
    fn test_velocity_sets_level() {
        let quiet = excited(0.25, 0.25, 0.0);
        let loud = excited(0.25, 1.0, 0.0);
        let q1 = harmonic(&quiet.excite[..quiet.excite_len], 1);
        let l1 = harmonic(&loud.excite[..loud.excite_len], 1);

        assert!(l1 > q1 * 3.0, "quiet {q1} loud {l1}");
    }

    #[test]
    fn test_hardness_adds_top_end() {
        // p = 0.2 nulls harmonics at multiples of 5, so the 12th harmonic is
        // well clear of the comb and isolates the contact-width smoothing.
        let soft = excited(0.2, 1.0, 0.0);
        let hard = excited(0.2, 1.0, 1.0);
        let hs = harmonic(&soft.excite[..soft.excite_len], 12);
        let hh = harmonic(&hard.excite[..hard.excite_len], 12);

        assert!(hh > hs * 1.5, "soft {hs} hard {hh}");
    }

    #[test]
    fn test_config_applies_to_trigger() {
        let ctx = AudioContext::new(48000);
        let mut ks = KarplusStrong::default();
        ks.setup(&ctx);
        ks.tune(&ctx, 200.0);
        ks.set_config(SynthConfig {
            position: 0.4,
            hardness: 0.2,
            sustain: 3.0,
        });
        ks.trigger(0.5);

        // The configured sustain is turned into the loop gain for this note.
        assert_eq!(ks.sustain, 3.0);
        assert_eq!(ks.gain, sustain_to_gain(3.0, 200.0));
        assert!(ks.excite_len > 0);
    }

    /// Render `ks` and return the time in seconds for its envelope to fall to a
    /// quarter of the initial level. [`None`] if it never gets there.
    fn quarter_time(ks: &mut KarplusStrong, max_seconds: f32) -> Option<f32> {
        // Long enough to smooth a period of the lowest note under test but
        // short enough to resolve the decay.
        const WINDOW: usize = 960;
        let max = (max_seconds * 48000.0) as usize;
        let mut env: Vec<f32> = Vec::new();
        let mut peak = 0.0f32;

        for i in 0..max {
            peak = peak.max(ks.step().abs());

            if i % WINDOW == WINDOW - 1 {
                env.push(peak);
                peak = 0.0;

                let head = env.len().min(3);
                let initial = env[..head].iter().fold(0.0f32, |m, s| m.max(*s));
                if initial > 0.0 && *env.last().unwrap() <= initial * 0.25 {
                    return Some((env.len() - 1) as f32 * WINDOW as f32 / 48000.0);
                }
            }
        }

        None
    }

    #[test]
    fn test_sustain_is_frequency_independent() {
        let ctx = AudioContext::new(48000);
        let mut times = Vec::new();

        for &f in &[110.0f32, 440.0, 1760.0, 4186.0] {
            let mut ks = KarplusStrong::default();
            ks.setup(&ctx);
            ks.tune(&ctx, f);
            ks.set_sustain(1.0);
            ks.trigger(1.0);
            times.push(quarter_time(&mut ks, 4.0).expect("note should decay"));
        }

        // Every note should take about a second to fall by 12dB.
        for t in &times {
            assert!((*t - 1.0).abs() < 0.3, "quarter times {times:?}");
        }
    }

    #[test]
    fn test_high_note_is_stable() {
        // The top of the range needs the most loop loss cancelled, so it is the
        // note most likely to expose an over-eager peaking EQ.
        let ctx = AudioContext::new(48000);

        for &f in &[4186.0f32, 2793.8, 1975.5, 3520.0] {
            let mut ks = KarplusStrong::default();
            ks.setup(&ctx);
            ks.tune(&ctx, f);
            ks.set_sustain(10.0);

            // A compensated loop always keeps the gain below one.
            assert!(ks.gain < 1.0, "gain {} at {f}Hz", ks.gain);

            ks.trigger(1.0);
            let mut peak = 0.0f32;
            for i in 0..(48000 * 3) {
                let s = ks.step();
                if i > 12000 {
                    peak = peak.max(s.abs());
                }
            }

            assert!(peak < 10.0, "runaway at {f}Hz, peak {peak}");
            assert!(peak > 0.0);
        }
    }

    #[test]
    fn test_mute_only_reduces_gain() {
        // Muting must never make the note louder, whatever the sustain control
        // asks for - including a sustain so short that the gain is already low
        // enough for the 20x shorter mute to clamp back onto the sounding gain.
        let ctx = AudioContext::new(48000);

        for &sustain in &[0.01f32, 0.1, 1.0, 30.0] {
            let mut ks = KarplusStrong::default();
            ks.setup(&ctx);
            ks.tune(&ctx, 440.0);
            ks.set_sustain(sustain);

            let sounding = ks.gain;
            ks.mute();
            assert!(
                ks.gain <= sounding,
                "sustain {sustain}: {sounding} -> {}",
                ks.gain
            );
        }
    }
}
