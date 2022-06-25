// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

#[derive(Debug)]
pub struct Delay<const L: usize> {
    buf: [f32; L],
    insert_at: usize,
    extract_from: usize,
}

impl<const L: usize> Default for Delay<L> {
    fn default() -> Self {
        Self {
            buf: [0.0; L],
            insert_at: 0,
            extract_from: 0,
        }
    }
}

impl<const L: usize> Delay<L> {
    pub fn setup(&mut self, _ctx: &AudioContext, n: usize) {
        let len = L;
        debug_assert!(n > 0 && n <= len);

        self.extract_from = self.insert_at + len - n;
        if self.extract_from >= len {
            self.extract_from -= len;
        }
    }

    fn double_peek(&self) -> f32 {
        if self.extract_from > L - 2 {
            self.buf[0]
        } else {
            self.buf[self.extract_from]
        }
    }

    pub fn peek(&self) -> f32 {
        self.buf[self.extract_from]
    }
}

impl<const L: usize> Filter for Delay<L> {
    fn step(&mut self, spl: f32) -> f32 {
        let len = self.buf.len();
        let res = self.peek();
        self.buf[self.insert_at] = spl;

        self.insert_at += 1;
        if self.insert_at >= len {
            self.insert_at -= len;
        }

        self.extract_from += 1;
        if self.extract_from >= len {
            self.extract_from -= len;
        }

        res
    }

    fn flush(&mut self) {
        for spl in &mut self.buf {
            *spl = 0.0;
        }
    }
}

#[derive(Debug, Default)]
pub struct FracDelay<const L: usize> {
    delay: Delay<L>,
    weight: Sample,
}

impl<const L: usize> FracDelay<L> {
    pub fn setup(&mut self, ctx: &AudioContext, n: Sample) {
        let delay = n.ceil();
        self.delay.setup(ctx, delay as usize);
        self.weight = delay - n;
    }

    pub fn peek(&self) -> f32 {
        let z1 = self.delay.peek();
        let z0 = self.delay.double_peek();
        let w = self.weight;

        z1 * (1.0 - w) + z0 * w
    }
}

impl<const L: usize> Filter for FracDelay<L> {
    fn step(&mut self, spl: Sample) -> Sample {
        let z1 = self.delay.step(spl);
        let z0 = self.delay.peek();
        let w = self.weight;

        z1 * (1.0 - w) + z0 * w
    }

    fn flush(&mut self) {
        self.delay.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_delay() {
        let mut delay = Delay::<10>::default();

        for i in -10..11 {
            let f = i as f32 / 10.0;

            if i < 0 {
                assert_eq!(0.0, delay.step(f));
            } else {
                let z = (i - 10) as f32 / 10.0;
                assert_eq!(z, delay.step(f));
            }
        }
    }

    #[test]
    fn test_delay_one() {
        let mut delay = Delay::<10>::default();
        let ctx = AudioContext::new(48000);
        delay.setup(&ctx, 1);

        for i in -10..11 {
            let f = i as f32 / 10.0;

            println!("i {}  f {}", i, f);
            if i < -9 {
                assert_eq!(0.0, delay.step(f));
            } else {
                let z = (i - 1) as f32 / 10.0;
                assert_eq!(z, delay.step(f));
            }
        }
    }

    #[test]
    fn test_delay_10() {
        let ctx = AudioContext::new(48000);
        let mut delay = Delay::<20>::default();
        delay.setup(&ctx, 10);

        for i in -10..11 {
            let f = i as f32 / 10.0;

            if i < 0 {
                assert_eq!(0.0, delay.step(f));
            } else {
                let z = (i - 10) as f32 / 10.0;
                assert_eq!(z, delay.step(f));
            }
        }
    }

    #[test]
    fn test_fracdelay_half() {
        let mut fdelay = FracDelay::<10>::default();
        let ctx = AudioContext::new(48000);
        fdelay.setup(&ctx, 0.5);

        assert_eq!(-0.5, fdelay.step(-1.0));
        assert_eq!(-0.6, fdelay.step(-0.2));
        assert_eq!(-0.1, fdelay.step(0.0));
        assert_eq!(0.1, fdelay.step(0.2));
        assert_eq!(0.6, fdelay.step(1.0));
        assert_eq!(0.5, fdelay.step(0.0));
    }

    #[test]
    fn test_fracdelay_one() {
        let mut fdelay = FracDelay::<10>::default();
        let ctx = AudioContext::new(48000);
        fdelay.setup(&ctx, 1.0);

        assert_eq!(0.0, fdelay.step(-1.0));
        assert_eq!(-1.0, fdelay.step(-0.2));
        assert_eq!(-0.2, fdelay.step(0.0));
        assert_eq!(0.0, fdelay.step(0.2));
        assert_eq!(0.2, fdelay.step(1.0));
        assert_eq!(1.0, fdelay.step(0.0));
    }

    #[test]
    fn test_fracdelay_one_and_a_quarter() {
        let mut fdelay = FracDelay::<10>::default();
        let ctx = AudioContext::new(48000);
        fdelay.setup(&ctx, 1.25);

        assert_eq!(0.0, fdelay.step(-1.0));
        assert_eq!(-0.75 + 0.0, fdelay.step(-0.2));
        assert_eq!(-0.15 - 0.25, fdelay.step(0.0));
        assert_eq!(0.0 - 0.05, fdelay.step(0.2));
        assert_eq!(0.15 + 0.0, fdelay.step(1.0));
        assert_eq!(0.75 + 0.05, fdelay.step(0.0));
    }

    #[test]
    fn test_fracdelay_one_and_a_half() {
        let mut fdelay = FracDelay::<10>::default();
        let ctx = AudioContext::new(48000);
        fdelay.setup(&ctx, 1.5);

        assert_eq!(0.0, fdelay.step(-1.0));
        assert_eq!(-0.5, fdelay.step(-0.2));
        assert_eq!(-0.6, fdelay.step(0.0));
        assert_eq!(-0.1, fdelay.step(0.2));
        assert_eq!(0.1, fdelay.step(1.0));
        assert_eq!(0.6, fdelay.step(0.0));
    }
}
