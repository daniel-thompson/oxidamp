// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use crate::*;
use std::num::Wrapping;

pub trait Voice {
    fn setup(&mut self, ctx: &AudioContext);
    fn trigger(&mut self);
    fn mute(&mut self);
    fn tune(&mut self, ctx: &AudioContext, freq: f32);
}

#[derive(Debug, Default)]
pub struct DetunedPair<T> {
    voice: [T; 2],
}

impl<T: Voice> Voice for DetunedPair<T> {
    fn setup(&mut self, ctx: &AudioContext) {
        for v in &mut self.voice {
            v.setup(ctx);
        }
    }

    fn trigger(&mut self) {
        for v in &mut self.voice {
            v.trigger();
        }
    }

    fn mute(&mut self) {
        for v in &mut self.voice {
            v.mute();
        }
    }

    fn tune(&mut self, ctx: &AudioContext, freq: f32) {
        self.voice[0].tune(ctx, freq * 0.995);
        self.voice[1].tune(ctx, freq * 1.005);
    }
}

impl<T: SignalGenerator> SignalGenerator for DetunedPair<T> {
    fn step(&mut self) -> f32 {
        self.voice[0].step() + self.voice[1].step()
    }
}

macro_rules! voicebox {
    ($($num_voices:expr => $name:ident),*) => {
        $(#[derive(Debug, Default)]
        pub struct $name<T> {
            voices: [T; $num_voices],
            ages: [Wrapping<i32>; $num_voices],
            note: [u8; $num_voices],

            t: Wrapping<i32>,
        }

        impl<T: Voice> $name<T> {
            pub fn setup(&mut self, ctx: &AudioContext) {
                for voice in &mut self.voices {
                    voice.setup(ctx);
                }
            }

            fn note_on(&mut self, note: u8) -> &mut T {
                let zero = Wrapping(0_i32);
                let mut best = 0_usize;

                for i in 1..self.ages.len() {
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

            fn note_off(&mut self, note: u8) -> Option<&mut T> {
                for i in 0..self.note.len() {
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

        impl<T: SignalGenerator> SignalGenerator for $name<T> {
            fn step(&mut self) -> f32 {
                let mut spl = 0.0;

                for voice in &mut self.voices {
                    spl += voice.step();
                }

                self.t += 1;

                spl
            }
        })*
    };
}

voicebox! {
    4 => VoiceBox4,
    6 => VoiceBox6,
    8 => VoiceBox,
    16 => VoiceBox16
}
