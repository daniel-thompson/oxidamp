// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use oxidamp::drummachine;
use oxidamp::prelude::*;
use std::sync::mpsc::sync_channel;

fn main() {
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
            outr.copy_from_slice(outl);

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
