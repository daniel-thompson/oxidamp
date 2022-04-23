// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

#[derive(Debug, Default)]
pub struct Delay {
    buf: Vec<f32>,
    insert_at: usize,
    extract_from: usize,
}

impl Delay {
    pub fn new(sz: usize) -> Self {
        Self {
            buf: vec![0.0; sz],
            ..Default::default()
        }
    }

    pub fn setup(&mut self, _ctx: &AudioContext, n: usize) {
        let len = self.buf.len();
        debug_assert!(n < len);

        self.extract_from = self.insert_at + n;
        if self.extract_from > len {
            self.extract_from -= len;
        }
    }
}

impl Filter for Delay {
    fn step(&mut self, spl: f32) -> f32 {
        let len = self.buf.len();
        let res = self.buf[self.extract_from];
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
