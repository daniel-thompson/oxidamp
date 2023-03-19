// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use oxidamp::metronome;
use oxidamp::prelude::*;
use std::sync::mpsc::sync_channel;

fn main() {
    let (client, _status) =
        jack::Client::new("Oxidamp", jack::ClientOptions::NO_START_SERVER).unwrap();

    let mut out_port = client
        .register_port("metronome", jack::AudioOut::default())
        .unwrap();

    let ctx = AudioContext::new(client.sample_rate() as i32);
    let mut metronome = Metronome::default();
    metronome.setup(&ctx);

    let (sender, receiver) = sync_channel(16);
    let _ = sender.try_send(metronome::Control::BeatsPerMinute(90));

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // handle any pending control updates
            while let Ok(ctrl) = receiver.try_recv() {
                metronome.set_control(&ctrl);
            }

            // get the slices (all shouuld be the same length)
            let out = out_port.as_mut_slice(ps);
            metronome.process(out);

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
        let _ = bpm_sender.try_send(metronome::Control::BeatsPerMinute(bpm));
    });

    siv.add_layer(
        cursive::views::Dialog::around(cursive::views::LinearLayout::vertical().child(bpm_slider))
            .title("Metronome")
            .button("Quit", |s| s.quit()),
    );

    siv.run();

    active_client.deactivate().unwrap();
}
