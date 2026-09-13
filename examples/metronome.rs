// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022, 2023 Daniel Thompson

use cursive::views::*;
use oxidamp::prelude::*;
use std::sync::{mpsc, Arc, Mutex};

fn main() {
    let (client, _status) =
        jack::Client::new("Oxidamp", jack::ClientOptions::NO_START_SERVER).unwrap();

    let mut out_port = client
        .register_port("metronome", jack::AudioOut::default())
        .unwrap();

    let ctx = AudioContext::new(client.sample_rate() as i32);
    let mut metronome = Metronome::default();
    metronome.setup(&ctx);
    let config = Arc::new(Mutex::new(metronome.config()));

    let (sender, receiver) = mpsc::channel();

    let process = jack::contrib::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // handle any pending control updates
            while let Ok(cfg) = receiver.try_recv() {
                metronome.set_config(cfg);
            }

            let out = out_port.as_mut_slice(ps);
            metronome.process(out);

            jack::Control::Continue
        },
    );

    // Activate the client, which starts the processing.
    let active_client = client.activate_async((), process).unwrap();

    // Build and run the UI
    let mut siv = cursive::default();

    let bpm_config = Arc::clone(&config);
    let bpm_sender = sender;
    let bpm_slider = SliderView::horizontal(70)
        .value((config.lock().unwrap().beats_per_minute as usize - 60) / 2)
        .on_change(move |_s, n| {
            let mut c = bpm_config.lock().unwrap().clone();
            c.beats_per_minute = 2 * n as u32 + 60;
            *bpm_config.lock().unwrap() = c;
            let _ = bpm_sender.send(c);
        });

    siv.add_layer(
        Dialog::around(LinearLayout::vertical().child(bpm_slider))
            .title("Metronome")
            .button("Quit", |s| s.quit()),
    );

    siv.run();

    // Leak the client on exit rather than deactivating it: jack-rs frees its
    // callback context during deactivate/drop while notification callbacks
    // stay registered, so the final "client unregistered" event then hits
    // freed memory and segfaults (seen with PipeWire). See src/main.rs.
    std::mem::forget(active_client);
}
