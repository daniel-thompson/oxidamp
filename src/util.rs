// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

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

pub fn check_response(ctx: &AudioContext, f: &mut impl Filter, gfreq: i32, db: f32) -> bool {
    let level = f.stimulate(ctx, gfreq);
    let ok;

    if 0.0 == db {
        ok = fuzzcmp(level, 1.0, 1.05);

        println!(
            "    {:4}Hz@{}Hz    {:6.2} {}=   1.00              [ {} ]",
            gfreq,
            ctx.sampling_frequency,
            level,
            if ok { '~' } else { '!' },
            if ok { " OK " } else { "FAIL" }
        );
    } else {
        let dblevel = linear2db(level);

        /* special case for very quiet signals */
        if db <= -96.0 {
            ok = dblevel <= (db + 6.0);
        } else {
            ok = fuzzcmp(dblevel, db, 1.1);
        }

        println!(
            "    {:4}Hz@{}Hz    {:6.2} {}= {:6.2}dB            [ {} ]",
            gfreq,
            ctx.sampling_frequency,
            dblevel,
            if ok { '~' } else { '!' },
            db,
            if ok { " OK " } else { "FAIL" }
        );
    }

    ok
}
