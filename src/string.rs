// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;
use std::num::Wrapping;

#[derive(Debug)]
pub struct KarplusStrong {
    delay: FracDelay<1920>,
    //filter: FIR<2, 3>,
    filter: FirstOrder,
    seed: u32,
    noise: u32,
    gain: f32,
}

impl Default for KarplusStrong {
    fn default() -> Self {
        Self {
            delay: FracDelay::default(),
            //filter: fir2_halfband(),
            filter: FirstOrder::default(),
            seed: 1,
            noise: 0,
            gain: 0.999,
        }
    }
}

impl KarplusStrong {
    pub fn setup(&mut self, ctx: &AudioContext) {
        self.delay.setup(ctx, 120.0);
        self.filter.lowpass(ctx, ctx.sampling_frequency / 4);
    }

    pub fn trigger(&mut self) {
        self.noise = 128;
        self.gain = 0.999;
    }

    pub fn mute(&mut self) {
        self.gain = 0.95;
    }

    pub fn tune(&mut self, ctx: &AudioContext, freq: f32) {
        let delay = ctx.sampling_frequency as f32 / freq;
        self.delay.setup(ctx, delay);
    }
}

impl SignalGenerator for KarplusStrong {
    fn step(&mut self) -> f32 {
        let mut spl = if self.noise > 0 {
            self.noise -= 1;
            frand31(&mut self.seed)
        } else {
            0.0
        };

        spl += self.gain * self.filter.step(self.delay.peek());
        let _ = self.delay.step(spl);

        spl
    }
}

#[derive(Debug, Default)]
pub struct VoiceBox {
    voices: [KarplusStrong; 16],
    ages: [Wrapping<i32>; 16],
    note: [u8; 16],

    t: Wrapping<i32>,
}

impl VoiceBox {
    pub fn setup(&mut self, ctx: &AudioContext) {
        for voice in &mut self.voices {
            voice.setup(ctx);
        }
    }

    fn note_on(&mut self, note: u8) -> &mut KarplusStrong {
        let zero = Wrapping(0_i32);
        let mut best = 0_usize;

        for i in 1..16 {
            let delta = self.ages[i] - self.ages[best];
            if self.note[best] != 0 {
                if self.note[i] == 0 || delta < zero {
                    best = i;
                }
            } else {
                if self.note[i] == 0 && delta < zero {
                    best = i;
                }
            }
        }

        self.ages[best] = self.t;
        self.note[best] = note;

        &mut self.voices[best]
    }

    fn note_off(&mut self, note: u8) -> Option<&mut KarplusStrong> {
        for i in 0..16 {
            if self.note[i] == note {
                self.ages[i] = self.t;
                self.note[i] = 0;

                return Some(&mut self.voices[i]);
            }
        }

        None
    }

    pub fn midi(&mut self, ctx: &AudioContext, data: &MidiData) {
        match data {
            MidiData::NoteOn(note) => {
                let voice = self.note_on(note.note);
                voice.tune(ctx, note.freq());
                voice.trigger();
            }
            MidiData::NoteOff(note) => {
                let voice = self.note_off(note.note);
                if let Some(voice) = voice {
                    voice.mute();
                }
            }
            MidiData::Raw(_) => {}
        };
    }
}

impl SignalGenerator for VoiceBox {
    fn step(&mut self) -> f32 {
        let mut spl = 0.0;

        for voice in &mut self.voices {
            spl += voice.step();
        }

        self.t += 1;

        spl
    }
}
