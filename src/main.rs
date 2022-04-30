// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use oxidamp::*;
use std::io;
use std::iter::zip;

fn main() {
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

            // currently the Amplifier has only one output so we'll
            for (l, r) in zip(outl, outr) {
                *r = *l;
            }

            jack::Control::Continue
        },
    );

    // Activate the client, which starts the processing.
    let active_client = client.activate_async(Notifications, process).unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
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
            "JACK: port with id {} renamed from {} to {}",
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
            "JACK: ports with id {} and {} are {}",
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
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }
}
