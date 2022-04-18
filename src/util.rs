// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

#[derive(Clone, Copy, Debug, Default)]
pub struct AudioContext {
    pub sampling_frequency: i32,
}

impl AudioContext {
    pub fn new(sfreq: i32) -> Self {
        AudioContext {
            sampling_frequency: sfreq,
        }
    }
}

pub fn fuzzcmp(a: f32, b: f32, delta: f32) -> bool {
    assert!(delta >= 1.0);

    let aa = a.abs();
    let ab = b.abs();

    // check for matching sign
    if (aa == a) != (ab == b) {
        return false;
    }

    if aa <= ab {
        return (aa * delta) >= ab;
    } else {
        return (ab * delta) >= aa;
    }
}

#[macro_export]
macro_rules! assert_fuzzeq {
    ($a:expr, $b:expr, $delta:expr) => {
        assert!(fuzzcmp($a, $b, $delta));
    };
}

pub fn db2linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

pub fn linear2db(spl: f32) -> f32 {
    spl.log10() * 20.0
}
