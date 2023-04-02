// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct ToneStackConfig {
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    pub gain: f32,
}

#[derive(Debug, Default)]
pub struct ToneStack {
    config: ToneStackConfig,

    ctx: AudioContext,

    bass_shelf: Biquad,
    treble_shelf: Biquad,
    linear_gain: f32,
}

impl ToneStack {
    pub fn setup(&mut self, ctx: &AudioContext) {
        self.ctx = *ctx;
        self.linear_gain = 1.0;
        self.update_filters();
    }

    pub fn config(&mut self) -> ToneStackConfig {
        self.config
    }

    pub fn set_config(&mut self, config: ToneStackConfig) {
        self.config = config;
        self.update_filters();
    }

    fn update_filters(&mut self) {
        // mid takes effect solely by influencing other controls
        let mid = self.config.mid;

        // calculate the changes
        let bass = self.config.bass - mid;
        let treble = self.config.treble - mid;
        let gain = self.config.gain + mid;

        // design the filters and convert the gain to linear
        self.bass_shelf.lowshelf(&self.ctx, 300, bass as f64, 0.8);
        self.treble_shelf
            .highshelf(&self.ctx, 1000, treble as f64, 0.8);
        self.linear_gain = db2linear(gain);
    }
}

impl Filter for ToneStack {
    fn step(&mut self, spl: f32) -> f32 {
        let mut spl = spl;
        spl = self.bass_shelf.step(spl);
        spl = self.treble_shelf.step(spl);
        spl * self.linear_gain
    }

    fn flush(&mut self) {
        self.bass_shelf.flush();
        self.treble_shelf.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_step() {
        let mut tone = ToneStack::default();

        assert_eq!(tone.step(0.0), 0.0);
        assert_eq!(tone.step(1.0), 0.0);
        assert_eq!(tone.step(-1.0), 0.0);
    }

    #[test]
    fn test_flat_response() {
        let ctx = AudioContext::new(48000);
        let mut tone = ToneStack::default();
        tone.setup(&ctx);
        assert!(check_response(&ctx, &mut tone, 100, 0.0));
        assert!(check_response(&ctx, &mut tone, 270, 0.0));
        assert!(check_response(&ctx, &mut tone, 300, 0.0));
        assert!(check_response(&ctx, &mut tone, 330, 0.0));
        assert!(check_response(&ctx, &mut tone, 550, 0.0));
        assert!(check_response(&ctx, &mut tone, 900, 0.0));
        assert!(check_response(&ctx, &mut tone, 1000, 0.0));
        assert!(check_response(&ctx, &mut tone, 1100, 0.0));
        assert!(check_response(&ctx, &mut tone, 1500, 0.0));
    }

    //
    // For the digital tonestack the logarithmic mid-point between the
    // bass corner frequency (300) and the treble corner frequency (1000)
    // is approximately 550Hz. However the bass and treble filters are tuned
    // to that this midpoint is slightly affected by bass/treble changes.
    // For that reason the target volumes at 550Hz don't strictly follow the
    // value of the "mid" control.
    //

    #[test]
    fn test_gain_cut() {
        let ctx = AudioContext::new(48000);
        let mut tone = ToneStack::default();
        tone.setup(&ctx);
        tone.set_config(ToneStackConfig {
            gain: -24.0,
            ..Default::default()
        });

        assert!(check_response(&ctx, &mut tone, 100, -24.0));
        assert!(check_response(&ctx, &mut tone, 550, -24.0));
        assert!(check_response(&ctx, &mut tone, 3000, -24.0));
    }

    #[test]
    fn test_gain_boost() {
        let ctx = AudioContext::new(48000);
        let mut tone = ToneStack::default();
        tone.setup(&ctx);
        tone.set_config(ToneStackConfig {
            gain: 24.0,
            ..Default::default()
        });

        assert!(check_response(&ctx, &mut tone, 100, 24.0));
        assert!(check_response(&ctx, &mut tone, 550, 24.0));
        assert!(check_response(&ctx, &mut tone, 3000, 24.0));
    }

    #[test]
    fn test_bass_cut() {
        let ctx = AudioContext::new(48000);
        let mut tone = ToneStack::default();
        tone.setup(&ctx);
        tone.set_config(ToneStackConfig {
            bass: -24.0,
            ..Default::default()
        });

        assert!(check_response(&ctx, &mut tone, 100, -24.0));
        assert!(check_response(&ctx, &mut tone, 550, -3.0));
        assert!(check_response(&ctx, &mut tone, 1000, 0.0));
        assert!(check_response(&ctx, &mut tone, 3000, 0.0));
    }

    #[test]
    fn test_bass_boost() {
        let ctx = AudioContext::new(48000);
        let mut tone = ToneStack::default();
        tone.setup(&ctx);
        tone.set_config(ToneStackConfig {
            bass: 24.0,
            ..Default::default()
        });

        assert!(check_response(&ctx, &mut tone, 100, 24.0));
        assert!(check_response(&ctx, &mut tone, 550, 3.0));
        assert!(check_response(&ctx, &mut tone, 1000, 0.0));
        assert!(check_response(&ctx, &mut tone, 3000, 0.0));
    }

    #[test]
    fn test_mid_cut() {
        let ctx = AudioContext::new(48000);
        let mut tone = ToneStack::default();
        tone.setup(&ctx);
        tone.set_config(ToneStackConfig {
            mid: -24.0,
            ..Default::default()
        });

        assert!(check_response(&ctx, &mut tone, 100, 0.0));
        assert!(check_response(&ctx, &mut tone, 550, -18.0));
        assert!(check_response(&ctx, &mut tone, 3000, 0.0));
    }

    #[test]
    fn test_mid_boost() {
        let ctx = AudioContext::new(48000);
        let mut tone = ToneStack::default();
        tone.setup(&ctx);
        tone.set_config(ToneStackConfig {
            mid: 24.0,
            ..Default::default()
        });

        assert!(check_response(&ctx, &mut tone, 100, 0.0));
        assert!(check_response(&ctx, &mut tone, 550, 18.0));
        assert!(check_response(&ctx, &mut tone, 3000, 0.0));
    }

    #[test]
    fn test_treble_cut() {
        let ctx = AudioContext::new(48000);
        let mut tone = ToneStack::default();
        tone.setup(&ctx);
        tone.set_config(ToneStackConfig {
            treble: -24.0,
            ..Default::default()
        });

        assert!(check_response(&ctx, &mut tone, 100, 0.0));
        assert!(check_response(&ctx, &mut tone, 300, 0.0));
        assert!(check_response(&ctx, &mut tone, 550, -3.0));
        assert!(check_response(&ctx, &mut tone, 3000, -24.0));
    }

    #[test]
    fn test_treble_boost() {
        let ctx = AudioContext::new(48000);
        let mut tone = ToneStack::default();
        tone.setup(&ctx);
        tone.set_config(ToneStackConfig {
            treble: 24.0,
            ..Default::default()
        });

        assert!(check_response(&ctx, &mut tone, 100, 0.0));
        assert!(check_response(&ctx, &mut tone, 300, 0.0));
        assert!(check_response(&ctx, &mut tone, 550, 3.0));
        assert!(check_response(&ctx, &mut tone, 3000, 24.0));
    }
}
