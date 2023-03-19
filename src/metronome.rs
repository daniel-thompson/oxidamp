// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022, 2023 Daniel Thompson

use crate::*;
use std::collections::VecDeque;

pub enum Control {
    BeatsPerMinute(u32),
    BeatsPerBar(u32),
}

pub struct Sawtooth {
    value: i32,
    step: i32,
}

impl Sawtooth {
    fn new(period: u32) -> Self {
        Self {
            value: 0,
            step: (4 * 0x10000 / period) as i32,
        }
    }

    fn step(&mut self) -> i16 {
        const MAX: i32 = 0x7fff;

        let spl = self.value + self.step;
        if spl.abs() < MAX {
            self.value = spl;
        } else {
            self.value = if spl > 0 {
                2 * MAX - spl
            } else {
                -2 * MAX - spl
            };
            self.step *= -1;
        };

        self.value as i16
    }
}

const DECAY: i32 = 27500;
const DELAY_FRAMES: usize = 68;
const DELAY_DETUNE: usize = 10;

pub struct Metronome {
    sfreq: u32,
    beats_per_minute: u32,
    beats_per_bar: u32,

    delay_buffer: VecDeque<i16>,
    exitation: Sawtooth,
    frames_until_next_beat: u32,
    beat: u32,
}

impl Default for Metronome {
    fn default() -> Self {
        Metronome {
            sfreq: 48000,
            beats_per_minute: 120,
            beats_per_bar: 4,

            delay_buffer: VecDeque::new(),
            exitation: Sawtooth::new((DELAY_FRAMES - (DELAY_DETUNE / 2)) as u32),
            frames_until_next_beat: 0,
            beat: 0,
        }
    }
}

impl Metronome {
    pub fn setup(&mut self, ctx: &AudioContext) {
        self.sfreq = ctx.sampling_frequency as u32;
    }

    pub fn set_control(&mut self, ctrl: &Control) {
        match ctrl {
            Control::BeatsPerMinute(bpm) => {
                self.beats_per_minute = *bpm;
            }
            Control::BeatsPerBar(bpb) => {
                self.beats_per_bar = *bpb;
            }
        }
    }
}

impl SignalGenerator for Metronome {
    /// Produce a single metronome sample.
    fn step(&mut self) -> Sample {
        if self.frames_until_next_beat == 0 {
            // retune if needed
            match self.beat {
                0 => self.delay_buffer.resize(DELAY_FRAMES - DELAY_DETUNE, 0),
                1 => self.delay_buffer.resize(DELAY_FRAMES, 0),
                _ => {}
            }

            self.frames_until_next_beat = self.sfreq * 60 / self.beats_per_minute;
            self.beat = (self.beat + 1) % self.beats_per_bar;

            for buf in self.delay_buffer.iter_mut() {
                *buf = buf.saturating_add(self.exitation.step());
            }
        }
        self.frames_until_next_beat -= 1;

        // this is a simple karplus-strong synth
        let a = self.delay_buffer.pop_front().unwrap_or(0) as i32;
        let b = self.delay_buffer[0] as i32;
        let spl = ((DECAY * (a + b)) >> 16) as i16;
        self.delay_buffer.push_back(spl);

        // provide the result a floating point
        spl as f32 / 32767.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Just crank some beats and make sure we don't panic!
    #[test]
    fn test_metronome() {
        let ctx = AudioContext::new(48000);
        let mut m = Metronome::default();
        m.setup(&ctx);
        let mut buf = [0.0_f32; 1024];

        for _ in 0..100 {
            m.process(&mut buf);
        }
    }
}
