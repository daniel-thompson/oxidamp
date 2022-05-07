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
            extract_from: L - 1,
        }
    }
}

impl<const L: usize> Delay<L> {
    pub fn setup(&mut self, _ctx: &AudioContext, n: usize) {
        let len = self.buf.len();
        debug_assert!(n < len);

        self.extract_from = self.insert_at + n;
        if self.extract_from > len {
            self.extract_from -= len;
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
