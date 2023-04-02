// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pattern {
    Basic4Beat,
    Basic8Beat,
    FourToTheFloor8Beat,
    Swing8Beat,
    Rock8Beat,
}

#[derive(Clone, Copy, Debug)]
pub struct DrumMachineConfig {
    pub beats_per_minute: u32,
    pub pattern: Pattern,
}

impl Default for DrumMachineConfig {
    fn default() -> Self {
        Self {
            beats_per_minute: 112,
            pattern: Pattern::Basic4Beat,
        }
    }
}

const __: u8 = 0;
const BA: u8 = 0x01;
const SN: u8 = 0x02;
const CH: u8 = 0x10;

struct Sequence<const L: usize> {
    divisions_per_beat: u8,
    pattern: [u8; L],
}

const BASIC_4BEAT: Sequence<4> = Sequence {
    divisions_per_beat: 1,
    pattern: [CH | BA | __, CH | __ | SN, CH | BA | __, CH | __ | SN],
};

const BASIC_8BEAT: Sequence<8> = Sequence {
    divisions_per_beat: 2,
    pattern: [
        CH | BA | __,
        CH | __ | __,
        CH | __ | SN,
        CH | __ | __,
        CH | BA | __,
        CH | __ | __,
        CH | __ | SN,
        CH | __ | __,
    ],
};

const FLOOR_8BEAT: Sequence<8> = Sequence {
    divisions_per_beat: 2,
    pattern: [
        CH | BA | __,
        CH | __ | __,
        CH | BA | SN,
        CH | __ | __,
        CH | BA | __,
        CH | __ | __,
        CH | BA | SN,
        CH | __ | __,
    ],
};

const SWING_8BEAT: Sequence<12> = Sequence {
    divisions_per_beat: 3,
    pattern: [
        CH | BA | __,
        __ | __ | __,
        CH | __ | __,
        CH | __ | SN,
        __ | __ | __,
        CH | __ | __,
        CH | BA | __,
        __ | __ | __,
        CH | __ | __,
        CH | __ | SN,
        __ | __ | __,
        CH | __ | __,
    ],
};

const ROCK_8BEAT: Sequence<8> = Sequence {
    divisions_per_beat: 2,
    pattern: [
        CH | BA | __,
        CH | __ | __,
        CH | __ | SN,
        CH | __ | __,
        CH | BA | __,
        CH | BA | __,
        CH | __ | SN,
        CH | BA | __,
    ],
};

include!("lib23k/hhc_rock_b.rs");
include!("lib23k/kick_dry_b.rs");
include!("lib23k/sn_wet_b.rs");

fn lookup_sample(i: usize) -> &'static [i8] {
    let i = (1 << i) as u8;
    if i == BA {
        return &KICK_DRY_B;
    } else if i == SN {
        return &SN_WET_B;
    } else if i == CH {
        return &HHC_ROCK_B;
    }

    panic!("Invalid trigger value");
}

pub struct DrumMachine {
    config: DrumMachineConfig,
    sfreq: i32,

    cold_sample: bool,
    last_sample: f32,
    resampler: Biquad,
    coefficients: [BiquadCoeff; 16],

    division_counter: i32,
    division_reload: i32,

    voice_pointer: [usize; 8],

    divisions_per_beat: u8,
    pattern: &'static [u8],
    i: usize,

    seed: u32,
}

impl Default for DrumMachine {
    fn default() -> Self {
        Self {
            config: DrumMachineConfig::default(),
            sfreq: 0,

            cold_sample: false,
            last_sample: 0.0,
            resampler: Biquad::default(),
            coefficients: [BiquadCoeff::default(); 16],

            division_counter: 0,
            division_reload: 0,

            voice_pointer: [0; 8],

            divisions_per_beat: 1,
            pattern: &BASIC_4BEAT.pattern,
            i: 0,

            seed: 1,
        }
    }
}

impl DrumMachine {
    /// Update the state variables based on the current control values.
    fn update(&mut self) {
        match self.config.pattern {
            Pattern::Basic4Beat => {
                self.divisions_per_beat = BASIC_4BEAT.divisions_per_beat;
                self.pattern = &BASIC_4BEAT.pattern;
            }
            Pattern::Basic8Beat => {
                self.divisions_per_beat = BASIC_8BEAT.divisions_per_beat;
                self.pattern = &BASIC_8BEAT.pattern;
            }
            Pattern::FourToTheFloor8Beat => {
                self.divisions_per_beat = FLOOR_8BEAT.divisions_per_beat;
                self.pattern = &FLOOR_8BEAT.pattern;
            }
            Pattern::Swing8Beat => {
                self.divisions_per_beat = SWING_8BEAT.divisions_per_beat;
                self.pattern = &SWING_8BEAT.pattern;
            }
            Pattern::Rock8Beat => {
                self.divisions_per_beat = ROCK_8BEAT.divisions_per_beat;
                self.pattern = &ROCK_8BEAT.pattern;
            }
        }

        let beats_per_second = self.config.beats_per_minute as f32 / 60.0;
        let samples_per_second = self.sfreq / 2;
        let samples_per_beat = samples_per_second as f32 / beats_per_second;
        let samples_per_division = samples_per_beat / self.divisions_per_beat as f32;

        self.division_counter = 0;
        self.division_reload = samples_per_division as i32;
        self.i = 0;
    }

    pub fn setup(&mut self, ctx: &AudioContext) {
        for i in 0..16 {
            let shfreq = 9000 + (i * 50);
            let q = 0.55 + (0.15 * (i & 3) as f64);

            self.resampler.lowpass(ctx, shfreq, q);
            self.coefficients[i as usize] = self.resampler.coeff;
        }

        self.sfreq = ctx.sampling_frequency;
        self.update();
    }

    pub fn config(&self) -> DrumMachineConfig {
        self.config
    }

    pub fn set_config(&mut self, config: DrumMachineConfig) {
        self.config = config;
        self.update();
    }

    fn ministep(&mut self) -> f32 {
        if self.division_counter == 0 {
            // trigger the voices
            for i in 0..self.voice_pointer.len() {
                if (1 << i) & self.pattern[self.i] != 0 {
                    let len = lookup_sample(i).len();
                    let offset = rand31(&mut self.seed) as usize & 255;
                    self.voice_pointer[i] = len - 64 + offset;
                }
            }

            // restart the division counter and move to the next part of the pattern
            self.division_counter = self.division_reload;
            self.i += 1;
            if self.i >= self.pattern.len() {
                self.i = 0;
            }
        } else {
            self.division_counter -= 1;
        }

        let mut spl: i32 = 0;

        // process the voices
        for i in 0..self.voice_pointer.len() {
            if self.voice_pointer[i] != 0 {
                let sample = lookup_sample(i);
                let len = sample.len();

                if self.voice_pointer[i] <= len {
                    spl += sample[len - self.voice_pointer[i]] as i32;
                }

                self.voice_pointer[i] -= 1;
            }
        }

        spl as f32 / 256.0
    }
}

impl SignalGenerator for DrumMachine {
    /// Produce a single drum machine sample.
    ///
    /// This function is essentially just a 2x integer resampler actig on the
    /// sequence produced by ::ministep().
    fn step(&mut self) -> Sample {
        let hot_sample = !self.cold_sample;
        self.cold_sample = hot_sample;

        let spl;
        if hot_sample {
            spl = self.ministep();

            // update the filter coefficients just before the beat is
            // triggered... this is when the filter is at its most quiet so
            // we shouldn't get a pop the change
            if self.division_counter == 0 {
                let rand = rand31(&mut self.seed) as usize;
                let coeff = &self.coefficients;
                self.resampler.coeff = coeff[rand % coeff.len()];
            }
        } else {
            spl = self.last_sample;
        }

        self.last_sample = self.resampler.step(spl);
        self.last_sample
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Just crank some beats and make sure we don't panic!
    #[test]
    fn test_drummachine() {
        let ctx = AudioContext::new(48000);
        let mut dm = DrumMachine::default();
        dm.setup(&ctx);
        let mut buf = [0.0_f32; 1024];

        for _ in 0..100 {
            dm.process(&mut buf);
        }
    }
}
