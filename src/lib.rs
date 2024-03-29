// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

//! oxidamp - A Digital Amplifier in Rust
//!
//! oxidamp is a digital modelling amplifier written in Rust. It is currently
//! in a nascent state. It currently implements a simple fixed-function mono
//! signal chain that includes preamp, tone stack and cabinet simulation.

pub mod amplifier;
pub mod biquad;
pub mod cabsim;
pub mod dcblocker;
pub mod delay;
pub mod drummachine;
pub mod fir;
pub mod firstorder;
pub mod jack;
pub mod metronome;
pub mod midi;
pub mod preamp;
pub mod prelude;
pub mod reverb;
pub mod sbuf;
pub mod siggen;
pub mod string;
pub mod tonestack;
pub mod tubestage;
pub mod util;
pub mod voicebox;
pub mod waveshaper;

pub use amplifier::*;
pub use biquad::*;
pub use cabsim::*;
pub use dcblocker::*;
pub use delay::*;
pub use drummachine::*;
pub use fir::*;
pub use firstorder::*;
pub use metronome::*;
pub use midi::*;
pub use preamp::*;
pub use reverb::*;
pub use sbuf::*;
pub use siggen::*;
pub use string::*;
pub use tonestack::*;
pub use tubestage::*;
pub use util::*;
pub use voicebox::*;
pub use waveshaper::*;
