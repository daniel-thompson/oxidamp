// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2022 Daniel Thompson

use cursive::views::*;
use oxidamp::prelude::*;
use std::{cell::Cell, rc::Rc, sync::mpsc};

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
    dm.set_config(DrumMachineConfig {
        beats_per_minute: 90,
        pattern: Pattern::Rock8Beat,
    });
    let config = Rc::new(Cell::new(dm.config()));
    let mut reverb = Reverb::default();

    let (sender, receiver) = mpsc::sync_channel(16);

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // handle any pending control updates
            while let Ok(cfg) = receiver.try_recv() {
                dm.set_config(cfg);
            }

            // get the slices (all shouuld be the same length)
            let outl = out_port_l.as_mut_slice(ps);
            let outr = out_port_r.as_mut_slice(ps);

            dm.process(outl);
            reverb.process(outl, outr);

            for (l, r) in outl.iter_mut().zip(outr.iter()) {
                *l += *r * 0.33;
            }

            // currently there is only one output so we'll just...
            outl.copy_from_slice(outr);

            jack::Control::Continue
        },
    );

    // Activate the client, which starts the processing.
    let active_client = client.activate_async((), process).unwrap();

    // Build and run the UI
    let mut siv = cursive::default();

    let bpm_config = config.clone();
    let bpm_sender = sender.clone();
    let bpm_slider = SliderView::horizontal(70)
        .value((config.get().beats_per_minute as usize - 60) / 2)
        .on_change(move |_s, n| {
            let mut c = bpm_config.get();
            c.beats_per_minute = 2 * n as u32 + 60;
            bpm_config.set(c);
            let _ = bpm_sender.try_send(c);
        });

    let pattern_config = config;
    let pattern_sender = sender;
    let mut pattern = SelectView::new().on_select(move |_s, n| {
        let mut c = pattern_config.get();
        c.pattern = *n;
        pattern_config.set(c);
        let _ = pattern_sender.try_send(c);
    });
    pattern.add_item("4 beat", Pattern::Basic4Beat);
    pattern.add_item("8 beat", Pattern::Basic8Beat);
    pattern.add_item("Four to the floor", Pattern::FourToTheFloor8Beat);
    pattern.add_item("8 beat with swing", Pattern::Swing8Beat);
    pattern.add_item("8 beat rock beat", Pattern::Rock8Beat);

    siv.add_layer(
        Dialog::around(LinearLayout::vertical().child(bpm_slider).child(pattern))
            .title("Drum machine")
            .button("Quit", |s| s.quit()),
    );

    siv.run();

    active_client.deactivate().unwrap();
}
