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
        (aa * delta) >= ab
    } else {
        (ab * delta) >= aa
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

/// Generate a 31-bit random number.
///
/// Derived from <http://www.firstpr.com.au/dsp/rand31/>, this is the
/// Park-Miller "minimal standard" 31 bit pseudo-random number generator,
/// implemented with David G. Carta's optimization with 32 bit math and
/// without division.
///
/// For most applications, this is largely a historical curiosity. It is
/// fast and has a tiny, tiny internal state which is great but that is
/// mostly because it is extremely primitive. It can't be used for things
/// like white noise but we do use it for low frequency "humanization" in
/// a couple of places.
pub fn rand31(seed: &mut u32) -> u32 {
    let lo = 16807 * (*seed & 0xffff);
    let hi = 16807 * (*seed >> 16);

    let mid = lo + ((hi & 0x7fff) << 16) + (hi >> 15);

    *seed = if mid > 0x7fffffff {
        mid - 0x7fffffff
    } else {
        mid
    };

    *seed
}

pub fn frand31(seed: &mut u32) -> f32 {
    ((rand31(seed) as f32) / 1073741824.0) - 1.0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rand31_first_ten() {
        let mut seed = 1;

        assert_eq!(16807, rand31(&mut seed));
        assert_eq!(282475249, rand31(&mut seed));
        assert_eq!(1622650073, rand31(&mut seed));
        assert_eq!(984943658, rand31(&mut seed));
        assert_eq!(1144108930, rand31(&mut seed));
        assert_eq!(470211272, rand31(&mut seed));
        assert_eq!(101027544, rand31(&mut seed));
        assert_eq!(1457850878, rand31(&mut seed));
        assert_eq!(1458777923, rand31(&mut seed));
        assert_eq!(2007237709, rand31(&mut seed));
    }

    #[test]
    fn test_frand31_spread() {
        let mut seed = 1;
        let mut acc = 0.0;
        let mut abs = 0.0;

        for _i in 0..100000 {
            let v = frand31(&mut seed);
            acc += v;
            abs += v.abs();

            assert!(-1.0 <= v && v <= 1.0);
        }

        let mean = acc / 100000.0;
        let abs_mean = abs / 100000.0;

        assert!(mean < 0.001);
        assert!(abs_mean > 0.499);
        assert!(abs_mean < 0.501);
    }
}
