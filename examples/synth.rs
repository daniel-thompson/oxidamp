// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use oxidamp::prelude::*;
use std::io;

fn main() {
    let (client, _status) =
        jack::Client::new("Oxidamp", jack::ClientOptions::NO_START_SERVER).unwrap();

    let in_midi = client
        .register_port("midi_in", jack::MidiIn::default())
        .unwrap();
    let mut out_port = client
        .register_port("synth", jack::AudioOut::default())
        .unwrap();

    let ctx = AudioContext::new(client.sample_rate() as i32);
    let mut synth = VoiceBox::<DetunedPair<KarplusStrong>>::default();
    synth.setup(&ctx);

    let process = jack::contrib::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let events = in_midi.iter(ps);
            for evt in events {
                let c: MidiEvent = evt.into();
                synth.midi(&ctx, &c.data);
            }

            let outbuf = out_port.as_mut_slice(ps);
            synth.process(outbuf);

            jack::Control::Continue
        },
    );

    // Activate the client, which starts the processing.
    let active_client = client.activate_async((), process).unwrap();

    // Wait for user input to quit
    println!("main: press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    // Leak the client on exit rather than deactivating it: jack-rs frees its
    // callback context during deactivate/drop while notification callbacks
    // stay registered, so the final "client unregistered" event then hits
    // freed memory and segfaults (seen with PipeWire). See src/main.rs.
    std::mem::forget(active_client);
}
