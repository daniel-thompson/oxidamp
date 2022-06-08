// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use jack::RawMidi;
use std::convert::From;

#[derive(Copy, Clone, Debug)]
pub struct MidiNote {
    pub note: u8,
    pub velocity: u8,
}

impl MidiNote {
    pub fn new(note: u8, velocity: u8) -> Self {
        Self { note, velocity }
    }

    pub fn freq(&self) -> f32 {
        // TODO: convert this to a lookup table?
        440.0 * 2.0_f32.powf((self.note as f32 - 69.0) / 12.0)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MidiData {
    NoteOn(MidiNote),
    NoteOff(MidiNote),
    Raw((u8, u8, u8)),
}

#[derive(Copy, Clone, Debug)]
pub struct MidiEvent {
    pub data: MidiData,
    pub time: u32,
}

impl From<RawMidi<'_>> for MidiEvent {
    fn from(midi: RawMidi<'_>) -> Self {
        // Grab the first three bytes (or zero is packet too short)
        let len = midi.bytes.len();
        let d0 = if len > 0 { midi.bytes[0] } else { 0 };
        let d1 = if len > 1 { midi.bytes[1] } else { 0 };
        let d2 = if len > 2 { midi.bytes[2] } else { 0 };

        // For now we can manage with a very simple midi implementation.
        // We currently only pick out note on/off. For everything else
        // we just capture the first three bytes and hope...
        MidiEvent {
            data: if d0 == 144 {
                MidiData::NoteOn(MidiNote::new(d1, d2))
            } else if d0 == 128 {
                MidiData::NoteOff(MidiNote::new(d1, d2))
            } else {
                MidiData::Raw((d0, d1, d2))
            },
            time: midi.time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_freq() {
        let a0 = MidiNote::new(21, 127);
        assert_eq!(a0.freq(), 27.50);

        let c4 = MidiNote::new(60, 127);
        assert_eq!(c4.freq(), 261.62555);

        let g9 = MidiNote::new(127, 127);
        assert_eq!(g9.freq(), 12543.855);
    }
}
