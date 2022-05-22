// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

const MAX_MIDI: usize = 3;

// a fixed size container to copy data out of real-time thread
#[derive(Copy, Clone)]
pub struct MidiEvent {
    len: usize,
    data: [u8; MAX_MIDI],
    time: u32,
}

impl std::convert::From<jack::RawMidi<'_>> for MidiEvent {
    fn from(midi: jack::RawMidi<'_>) -> Self {
        let len = std::cmp::min(MAX_MIDI, midi.bytes.len());
        let mut data = [0; MAX_MIDI];
        data[..len].copy_from_slice(&midi.bytes[..len]);
        MidiEvent {
            len,
            data,
            time: midi.time,
        }
    }
}

impl std::fmt::Debug for MidiEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MidiEvent {{ time: {:4}, data: {:?} }}",
            self.time,
            &self.data[..self.len],
        )
    }
}
