// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use oxidamp::prelude::{Amplifier, AudioContext, Filter};

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

            // currently the Amplifier has only one output so we'll just...
            outr.copy_from_slice(outl);

            jack::Control::Continue
        },
    );

    // Activate the client, which starts the processing.
    let active_client = client
        .activate_async(oxidamp::jack::Notifications, process)
        .unwrap();

    // Wait for user input to quit
    println!("main: press enter/return to quit...");
    let mut user_input = String::new();
    std::io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}
