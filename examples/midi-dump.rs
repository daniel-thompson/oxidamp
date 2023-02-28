// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use oxidamp::prelude::*;
use std::io;
use std::sync::mpsc::sync_channel;

fn main() {
    let (client, _status) =
        jack::Client::new("Oxidamp", jack::ClientOptions::NO_START_SERVER).unwrap();

    let in_midi = client
        .register_port("midi_in", jack::MidiIn::default())
        .unwrap();
    let (sender, receiver) = sync_channel(64);

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let events = in_midi.iter(ps);
            for evt in events {
                let c: MidiEvent = evt.into();
                let _ = sender.try_send(c);
            }
            jack::Control::Continue
        },
    );

    let active_client = client.activate_async((), process).unwrap();

    // spawn a non-real-time thread that prints out the midi messages we get
    std::thread::spawn(move || {
        while let Ok(m) = receiver.recv() {
            println!("midi: {:?}", m);
        }
    });

    println!("main: press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}
