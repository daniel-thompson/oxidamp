// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2023 Daniel Thompson

use egui_miniquad::EguiMq;
use miniquad;
use oxidamp::drummachine;
use oxidamp::prelude::*;
use std::sync::mpsc;

struct Stage {
    egui_mq: EguiMq,
    channel: mpsc::SyncSender<Control>,
    bpm: u32,
    pattern: Pattern,
}

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
    let mut reverb = Reverb::default();

    let (sender, receiver) = mpsc::sync_channel(16);
    let _ = sender.try_send(drummachine::Control::BeatsPerMinute(90));
    let _ = sender.try_send(drummachine::Control::Pattern(Pattern::Rock8Beat));

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

    // ... then use the main thread to run the GUI
    let conf = miniquad::conf::Conf {
        window_title: "Oxidamp".to_string(),
        high_dpi: true,
        window_width: 1200,
        window_height: 1024,
        ..Default::default()
    };
    miniquad::start(conf, |mut ctx| Box::new(Stage::new(&mut ctx, sender)));

    active_client.deactivate().unwrap();
}

impl Stage {
    fn new(mq_ctx: &mut miniquad::Context, channel: mpsc::SyncSender<Control>) -> Self {
        let egui_mq = EguiMq::new(mq_ctx);
        let ctx = egui_mq.egui_ctx();

        ctx.set_pixels_per_point(1.5);
        ctx.set_visuals(egui::Visuals::light());

        Self {
            egui_mq,
            channel,
            bpm: 112,
            pattern: Pattern::Rock8Beat,
        }
    }
}

impl miniquad::EventHandler for Stage {
    fn update(&mut self, _ctx: &mut miniquad::Context) {}

    fn draw(&mut self, mq_ctx: &mut miniquad::Context) {
        mq_ctx.clear(Some((1., 1., 1., 1.)), None, None);
        mq_ctx.begin_default_pass(miniquad::PassAction::clear_color(0.65, 0.70, 0.65, 1.0));
        mq_ctx.end_render_pass();

        // Run the UI code:
        self.egui_mq.run(mq_ctx, |_mq_ctx, ctx| {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.menu_button("🎸", |ui| {
                            if ui.button("Quit").clicked() {
                                std::process::exit(0);
                            }
                        });
                        egui::widgets::global_dark_light_mode_switch(ui);
                    }
                });
            });

            let mut pixels_per_point = ctx.pixels_per_point();
            egui::Window::new("Settings").show(ctx, |ui| {
                let response = ui
                    .add(
                        egui::Slider::new(&mut pixels_per_point, 0.75..=3.0)
                            .logarithmic(true)
                            .text("scale"),
                    )
                    .on_hover_text("Physical pixels per logical point");
                if response.clicked() || response.drag_released() {
                    ctx.set_pixels_per_point(pixels_per_point);
                }
            });

            egui::Window::new("Drum machine").show(ctx, |ui| {
                ui.heading("Configuration");
                if ui
                    .add(egui::Slider::new(&mut self.bpm, 40..=240).text("beats per minute"))
                    .changed()
                {
                    let _ = self.channel.send(Control::BeatsPerMinute(self.bpm));
                }

                egui::ComboBox::from_label("pattern")
                    .selected_text(format!("{:?}", self.pattern))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(60.0);
                        if ui
                            .selectable_value(&mut self.pattern, Pattern::Basic4Beat, "Basic4Beat")
                            .clicked()
                            || ui
                                .selectable_value(
                                    &mut self.pattern,
                                    Pattern::Basic8Beat,
                                    "Basic8Beat",
                                )
                                .clicked()
                            || ui
                                .selectable_value(
                                    &mut self.pattern,
                                    Pattern::Swing8Beat,
                                    "Swing8Beat",
                                )
                                .clicked()
                            || ui
                                .selectable_value(
                                    &mut self.pattern,
                                    Pattern::Rock8Beat,
                                    "Rock8Beat",
                                )
                                .clicked()
                        {
                            let _ = self.channel.send(Control::Pattern(self.pattern));
                        }
                    });

                ui.heading("Control");
            });
        });

        self.egui_mq.draw(mq_ctx);

        mq_ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, _: &mut miniquad::Context, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, _: &mut miniquad::Context, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut miniquad::Context,
        mb: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_down_event(ctx, mb, x, y);
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut miniquad::Context,
        mb: miniquad::MouseButton,
        x: f32,
        y: f32,
    ) {
        self.egui_mq.mouse_button_up_event(ctx, mb, x, y);
    }

    fn char_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        character: char,
        _keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.key_down_event(ctx, keycode, keymods);
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut miniquad::Context,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
    ) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}
