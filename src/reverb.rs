// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

#[derive(Debug, Default)]
struct AllPass<const L: usize> {
    delay: Delay<L>,
    gainb: Sample,
    gainf: Sample,
}

impl<const L: usize> Filter for AllPass<L> {
    fn step(&mut self, spl: Sample) -> Sample {
        let delayout = self.delay.peek();
        let feedback = delayout * self.gainb;
        let delayin = spl + feedback;
        self.delay.step(delayin);

        delayin * self.gainf + delayout
    }

    fn flush(&mut self) {
        self.delay.flush();
    }
}

#[derive(Debug, Default)]
struct Comb<const L: usize> {
    delay: Delay<L>,
    gainb: Sample,
}

impl<const L: usize> Filter for Comb<L> {
    fn step(&mut self, spl: Sample) -> Sample {
        let delayout = self.delay.peek();
        let feedback = delayout * self.gainb;
        let combout = spl + feedback;
        self.delay.step(combout);

        combout
    }

    fn flush(&mut self) {
        self.delay.flush();
    }
}

/// A simple reverb based on jcrev by John Chowning
///
/// All filter lengths and gain tunings come directly from jcrev.
#[derive(Debug)]
pub struct Reverb {
    pregain: Sample,

    allpass347: AllPass<347>,
    allpass113: AllPass<113>,
    allpass37: AllPass<37>,

    comb1601: Comb<1601>,
    comb1867: Comb<1867>,
    comb2053: Comb<2053>,
    comb2251: Comb<2251>,
}

impl Default for Reverb {
    fn default() -> Self {
        Self {
            pregain: 0.1,

            allpass347: AllPass {
                gainb: 0.7,
                gainf: -0.7,
                ..Default::default()
            },
            allpass113: AllPass {
                gainb: 0.7,
                gainf: -0.7,
                ..Default::default()
            },
            allpass37: AllPass {
                gainb: 0.7,
                gainf: -0.7,
                ..Default::default()
            },

            comb1601: Comb {
                gainb: 0.802,
                ..Default::default()
            },
            comb1867: Comb {
                gainb: 0.802,
                ..Default::default()
            },
            comb2053: Comb {
                gainb: 0.802,
                ..Default::default()
            },
            comb2251: Comb {
                gainb: 0.802,
                ..Default::default()
            },
        }
    }
}

impl Filter for Reverb {
    fn step(&mut self, spl: Sample) -> Sample {
        let prespl = spl * self.pregain;

        // serially combine the allpass stages
        let mut ap = self.allpass347.step(prespl);
        ap = self.allpass113.step(ap);
        ap = self.allpass37.step(ap);

        // parallel combine the comb stages
        let mut sum = self.comb1601.step(ap);
        sum += self.comb1867.step(ap);
        sum += self.comb2053.step(ap);
        sum += self.comb2251.step(ap);

        spl + sum
    }

    fn flush(&mut self) {
        self.allpass347.flush();
        self.allpass113.flush();
        self.allpass37.flush();
        self.comb1601.flush();
        self.comb1867.flush();
        self.comb2053.flush();
        self.comb2251.flush();
    }
}
