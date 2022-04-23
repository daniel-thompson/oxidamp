// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

#[derive(Debug, Default)]
pub struct WaveShaper<'a> {
    min: f32,
    // no need to store max since limit has this covered
    limit: f32,
    shape: &'a [f32],
}

impl<'a> WaveShaper<'a> {
    pub fn setup(&mut self, _ctx: &AudioContext, min: f32, max: f32, shape: &'a [f32]) {
        debug_assert!(min < max);
        debug_assert!(shape.len() >= 2);

        self.min = min;
        self.limit = max - min;
        self.shape = shape;
    }
}

impl Filter for WaveShaper<'_> {
    fn step(&mut self, spl: f32) -> f32 {
        if spl <= self.min {
            return self.shape[0];
        }

        let inspl = (spl - self.min) / self.limit;
        let sz = self.shape.len();
        if inspl >= 1.0 {
            return self.shape[sz - 1];
        }

        debug_assert!(0.0 < inspl && inspl < 1.0);

        let index = inspl * (sz - 1) as f32;
        let i = index as usize;

        // use the fractional part of index to calculate the weights
        let a = index - i as f32;
        let b = 1.0 - a;

        a * self.shape[i + 1] + b * self.shape[i]
    }

    fn flush(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_step {
        ($shaper:expr, $inspl:expr, $outspl:expr) => {
            assert_eq!($shaper.step($inspl), $outspl);
        };
    }

    #[test]
    fn test_hard_clamping() {
        let ctx = AudioContext::new(48000);
        let shape = [-1.0, 1.0];
        let mut shaper = WaveShaper::default();
        shaper.setup(&ctx, -1.0, 1.0, &shape);

        assert_step!(shaper, -2.0, -1.0);
        assert_step!(shaper, -1.0, -1.0);
        assert_step!(shaper, -0.5, -0.5);
        assert_step!(shaper, 0.0, 0.0);
        assert_step!(shaper, 0.5, 0.5);
        assert_step!(shaper, 1.0, 1.0);
        assert_step!(shaper, 2.0, 1.0);
    }

    #[test]
    fn test_reduce_range() {
        let ctx = AudioContext::new(48000);
        let shape = [-0.5, 0.5];
        let mut shaper = WaveShaper::default();
        shaper.setup(&ctx, -1.0, 1.0, &shape);

        assert_step!(shaper, -2.0, -0.5);
        assert_step!(shaper, -1.0, -0.5);
        assert_step!(shaper, -0.5, -0.25);
        assert_step!(shaper, 0.0, 0.0);
        assert_step!(shaper, 0.5, 0.25);
        assert_step!(shaper, 1.0, 0.5);
        assert_step!(shaper, 2.0, 0.5);
    }

    #[test]
    fn test_boost_range() {
        let ctx = AudioContext::new(48000);
        let shape = [-2.0, 2.0];
        let mut shaper = WaveShaper::default();
        shaper.setup(&ctx, -1.0, 1.0, &shape);

        assert_step!(shaper, -2.0, -2.0);
        assert_step!(shaper, -1.0, -2.0);
        assert_step!(shaper, -0.5, -1.0);
        assert_step!(shaper, 0.0, 0.0);
        assert_step!(shaper, 0.5, 1.0);
        assert_step!(shaper, 1.0, 2.0);
        assert_step!(shaper, 2.0, 2.0);
    }
    #[test]
    fn test_shape_range() {
        let ctx = AudioContext::new(48000);
        let shape = [-1.0, -0.875, -0.75, -0.5, 0.0, 0.5, 0.75, 0.875, 1.0];
        let mut shaper = WaveShaper::default();
        shaper.setup(&ctx, -2.0, 2.0, &shape);

        assert_step!(shaper, -3.0, -1.0);
        assert_step!(shaper, -2.0, -1.0);
        assert_step!(shaper, -1.5, -0.875);
        assert_step!(shaper, -1.0, -0.75);
        assert_step!(shaper, -0.75, -0.625);
        assert_step!(shaper, -0.5, -0.5);
        assert_step!(shaper, 0.0, 0.0);
        assert_step!(shaper, 0.5, 0.5);
        assert_step!(shaper, 0.75, 0.625);
        assert_step!(shaper, 1.0, 0.75);
        assert_step!(shaper, 1.5, 0.875);
        assert_step!(shaper, 2.0, 1.0);
        assert_step!(shaper, 3.0, 1.0);
    }
}
