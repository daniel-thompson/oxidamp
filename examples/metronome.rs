// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022, 2023 Daniel Thompson

use cursive::views::*;
use oxidamp::prelude::*;
use std::{cell::Cell, rc::Rc, sync::mpsc};

fn main() {
    let (client, _status) =
        jack::Client::new("Oxidamp", jack::ClientOptions::NO_START_SERVER).unwrap();

    let mut out_port = client
        .register_port("metronome", jack::AudioOut::default())
        .unwrap();

    let ctx = AudioContext::new(client.sample_rate() as i32);
    let mut metronome = Metronome::default();
    metronome.setup(&ctx);
    let config = Rc::new(Cell::new(metronome.config()));

    let (sender, receiver) = mpsc::sync_channel(16);

    let process = jack::ClosureProcessHandler::new(
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

    let bpm_config = config;
    let bpm_sender = sender;
    let bpm_slider = SliderView::horizontal(70)
        .value((config.get().beats_per_minute as usize - 60) / 2)
        .on_change(move |_s, n| {
            let mut c = bpm_config.get();
            c.beats_per_minute = 2 * n as u32 + 60;
            bpm_config.set(c);
            let _ = bpm_sender.try_send(c);
        });

    siv.add_layer(
        Dialog::around(LinearLayout::vertical().child(bpm_slider))
            .title("Metronome")
            .button("Quit", |s| s.quit()),
    );

    siv.run();

    active_client.deactivate().unwrap();
}
