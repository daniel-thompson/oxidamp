// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use clap::{Parser, Subcommand};
use oxidamp::*;
use std::io;
use std::sync::mpsc::sync_channel;

/// A digital amplifier in Rust.
///
/// Amplifier, synth, drum machine and more...
#[derive(Parser)]
#[clap(version)]
struct Cli {
    #[clap(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Digital amplifier
    Amplifier,

    /// Simple lo-fi drum machine
    DrumMachine,

    /// Dump MIDI packets for debugging
    MidiDump,

    /// Karplus-Strong based strings synthesizer
    Synth,
}

fn amp() {
    let (client, _status) =
        jack::Client::new("Oxidamp", jack::ClientOptions::NO_START_SERVER).unwrap();

    let in_port = client
        .register_port("amp_in", jack::AudioIn::default())
        .unwrap();
    let mut out_port_l = client
        .register_port("amp_l", jack::AudioOut::default())
        .unwrap();
    let mut out_port_r = client
        .register_port("amp_r", jack::AudioOut::default())
        .unwrap();

    let ctx = AudioContext::new(client.sample_rate() as i32);
    let mut amp = Amplifier::default();
    amp.setup(&ctx);

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // get the slices (all shouuld be the same length)
            let inbuf = in_port.as_slice(ps);
            let outl = out_port_l.as_mut_slice(ps);
            let outr = out_port_r.as_mut_slice(ps);

            amp.process(inbuf, outl);

            // currently the Amplifier has only one output so we'll just...
            outr.copy_from_slice(&outl);

            jack::Control::Continue
        },
    );

    // Activate the client, which starts the processing.
    let active_client = client.activate_async(Notifications, process).unwrap();

    // Wait for user input to quit
    println!("main: press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}

fn drum_machine() {
    let (client, _status) =
        jack::Client::new("Oxidamp", jack::ClientOptions::NO_START_SERVER).unwrap();

    let mut out_port_l = client
        .register_port("drums_l", jack::AudioOut::default())
        .unwrap();
    let mut out_port_r = client
        .register_port("drums_r", jack::AudioOut::default())
        .unwrap();

    let ctx = AudioContext::new(client.sample_rate() as i32);
    let mut dm = DrumMachine::default();
    dm.setup(&ctx);

    let (sender, receiver) = sync_channel(16);
    let _ = sender.try_send(drummachine::Control::BeatsPerMinute(90));
    let _ = sender.try_send(drummachine::Control::Pattern(3));

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // handle any pending control updates
            while let Ok(ctrl) = receiver.try_recv() {
                dm.set_control(&ctrl);
            }

            // get the slices (all shouuld be the same length)
            let outl = out_port_l.as_mut_slice(ps);
            let outr = out_port_r.as_mut_slice(ps);

            dm.process(outl);

            // currently there is only one output so we'll just...
            outr.copy_from_slice(&outl);

            jack::Control::Continue
        },
    );

    // Activate the client, which starts the processing.
    let active_client = client.activate_async((), process).unwrap();

    // Build and run the UI
    let mut siv = cursive::default();

    let bpm_sender = sender.clone();
    let bpm_slider = cursive::views::SliderView::horizontal(70).on_change(move |_s, n| {
        let bpm = 2 * n as u32 + 60;
        let _ = bpm_sender.try_send(drummachine::Control::BeatsPerMinute(bpm));
    });

    let pattern_sender = sender.clone();
    let mut pattern = cursive::views::SelectView::new().on_select(move |_s, n| {
        let _ = pattern_sender.try_send(drummachine::Control::Pattern(*n));
    });
    pattern.add_item("4 beat", 0);
    pattern.add_item("8 beat", 1);
    pattern.add_item("8 beat with swing", 2);
    pattern.add_item("8 beat rock", 3);

    siv.add_layer(
        cursive::views::Dialog::around(
            cursive::views::LinearLayout::vertical()
                .child(bpm_slider)
                .child(pattern),
        )
        .title("Drum machine")
        .button("Quit", |s| s.quit()),
    );

    siv.run();

    active_client.deactivate().unwrap();
}

fn midi_dump() {
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

    let active_client = client.activate_async(Notifications, process).unwrap();

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

fn synth() {
    let (client, _status) =
        jack::Client::new("Oxidamp", jack::ClientOptions::NO_START_SERVER).unwrap();

    let in_midi = client
        .register_port("midi_in", jack::MidiIn::default())
        .unwrap();
    let mut out_port = client
        .register_port("synth", jack::AudioOut::default())
        .unwrap();

    let ctx = AudioContext::new(client.sample_rate() as i32);
    let mut ks = KarplusStrong::default();
    ks.setup(&ctx);

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let events = in_midi.iter(ps);
            for evt in events {
                let c: MidiEvent = evt.into();
                match c.data {
                    MidiData::NoteOn(note) => {
                        ks.tune(&ctx, note.freq());
                        ks.trigger();
                    }
                    MidiData::NoteOff(_) => {
                        ks.mute();
                    }
                    MidiData::Raw(_) => {}
                }
            }

            let outbuf = out_port.as_mut_slice(ps);
            ks.process(outbuf);

            jack::Control::Continue
        },
    );

    // Activate the client, which starts the processing.
    let active_client = client.activate_async((), process).unwrap();

    // Build and run the UI
    let mut siv = cursive::default();

    siv.add_layer(
        cursive::views::Dialog::around(cursive::views::LinearLayout::vertical())
            .title("Bitsichord")
            .button("Quit", |s| s.quit()),
    );

    siv.run();

    active_client.deactivate().unwrap();
}

fn main() {
    let args = Cli::parse();

    match args.cmd {
        Commands::Amplifier => amp(),
        Commands::DrumMachine => drum_machine(),
        Commands::MidiDump => midi_dump(),
        Commands::Synth => synth(),
    }
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("jack: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "jack: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "jack: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("jack: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "jack: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "jack: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "jack: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "jack: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("jack: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("jack: xrun occurred");
        jack::Control::Continue
    }
}
