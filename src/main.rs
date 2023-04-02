// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2023 Daniel Thompson

mod gui;

use egui_miniquad::EguiMq;
use miniquad;
use oxidamp::prelude::*;
use std::sync::mpsc;

enum Active {
    Amplifier(bool),
    DrumMachine(bool),
    Metronome(bool),
    Synth(bool),
}

enum Control {
    Application(Active),
    Amplifier(AmplifierConfig),
    DrumMachine(DrumMachineConfig),
    Metronome(MetronomeConfig),
    Midi(MidiData),
}

type ControlSender = mpsc::SyncSender<Control>;

fn main() {
    let (client, _status) =
        jack::Client::new("Oxidamp", jack::ClientOptions::NO_START_SERVER).unwrap();

    let amp_in = client
        .register_port("amp_in", jack::AudioIn::default())
        .unwrap();
    let mut amp_out = client
        .register_port("amp", jack::AudioOut::default())
        .unwrap();
    let mut drums_l = client
        .register_port("drums_l", jack::AudioOut::default())
        .unwrap();
    let mut drums_r = client
        .register_port("drums_r", jack::AudioOut::default())
        .unwrap();
    let mut metronome_out = client
        .register_port("metronome", jack::AudioOut::default())
        .unwrap();
    let synth_in = client
        .register_port("synth_in", jack::MidiIn::default())
        .unwrap();
    let mut synth_out = client
        .register_port("synth_out", jack::AudioOut::default())
        .unwrap();

    let ctx = AudioContext::new(client.sample_rate() as i32);

    let mut amp_active = false;
    let mut amp = Amplifier::default();
    amp.setup(&ctx);

    let mut dm_active = false;
    let mut dm = DrumMachine::default();
    dm.setup(&ctx);
    let mut reverb = Reverb::default();

    let mut metronome_active = false;
    let mut metronome = Metronome::default();
    metronome.setup(&ctx);

    let mut synth_active = false;
    let mut synth = VoiceBox::<DetunedPair<KarplusStrong>>::default();
    synth.setup(&ctx);

    let (sender, receiver) = mpsc::sync_channel(16);

    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // handle any pending control updates
            while let Ok(ctrl) = receiver.try_recv() {
                match ctrl {
                    Control::Application(app) => match app {
                        Active::Amplifier(active) => amp_active = active,
                        Active::DrumMachine(active) => dm_active = active,
                        Active::Metronome(active) => metronome_active = active,
                        Active::Synth(active) => synth_active = active,
                    },
                    Control::Amplifier(cfg) => amp.set_config(cfg),
                    Control::DrumMachine(cfg) => dm.set_config(cfg),
                    Control::Metronome(cfg) => metronome.set_config(cfg),
                    Control::Midi(mididata) => synth.midi(&ctx, &mididata),
                }
            }

            if dm_active {
                let dm_l = drums_l.as_mut_slice(ps);
                let dm_r = drums_r.as_mut_slice(ps);

                dm.process(dm_l);
                reverb.process(dm_l, dm_r);

                for (l, r) in dm_l.iter_mut().zip(dm_r.iter()) {
                    *l += *r * 0.33;
                }

                // currently there is only one output so we'll just...
                dm_l.copy_from_slice(dm_r);
            }

            if metronome_active {
                let m_out = metronome_out.as_mut_slice(ps);
                metronome.process(m_out);
            }

            if synth_active {
                let events = synth_in.iter(ps);
                for evt in events {
                    let c: MidiEvent = evt.into();
                    synth.midi(&ctx, &c.data);
                }

                let outbuf = synth_out.as_mut_slice(ps);
                synth.process(outbuf);
            }

            if amp_active {
                let input = amp_in.as_slice(ps);
                let output = amp_out.as_mut_slice(ps);

                amp.process(input, output);
            }

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

struct Stage {
    egui_mq: EguiMq,
    channel: ControlSender,
    settings: bool,
    amplifier: AmplifierApp,
    drum_machine: DrumMachineApp,
    metronome: MetronomeApp,
    synth: SynthApp,
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
            settings: false,
            amplifier: AmplifierApp::new(),
            drum_machine: DrumMachineApp::new(),
            metronome: MetronomeApp::new(),
            synth: SynthApp::new(),
        }
    }
}

#[derive(Default)]
struct AmplifierApp {
    active: bool,
    config: AmplifierConfig,
}

impl AmplifierApp {
    fn new() -> Self {
        Self::default()
    }

    fn draw(&mut self, ui: &mut egui::Ui, ctrl_channel: &ControlSender) {
        if ui
            .add(egui::Slider::new(&mut self.config.preamp.gain, 0.0..=96.0).text("drive"))
            .changed()
            | ui.add(egui::Slider::new(&mut self.config.tonestack.bass, -24.0..=24.0).text("bass"))
                .changed()
            | ui.add(egui::Slider::new(&mut self.config.tonestack.mid, -24.0..=24.0).text("mid"))
                .changed()
            | ui.add(
                egui::Slider::new(&mut self.config.tonestack.treble, -24.0..=24.0).text("treble"),
            )
            .changed()
            | ui.add(egui::Slider::new(&mut self.config.tonestack.gain, -24.0..=24.0).text("gain"))
                .changed()
        {
            let _ = ctrl_channel.send(Control::Amplifier(self.config));
        }
    }
}

#[derive(Default)]
struct DrumMachineApp {
    active: bool,
    config: DrumMachineConfig,
}

impl DrumMachineApp {
    fn new() -> Self {
        Self::default()
    }

    fn draw(&mut self, ui: &mut egui::Ui, ctrl_channel: &ControlSender) {
        if ui
            .add(
                egui::Slider::new(&mut self.config.beats_per_minute, 40..=240)
                    .text("beats per minute"),
            )
            .changed()
        {
            let _ = ctrl_channel.send(Control::DrumMachine(self.config));
        }

        egui::ComboBox::from_label("pattern")
            .selected_text(format!("{:?}", self.config.pattern))
            .show_ui(ui, |ui| {
                let config = &mut self.config;
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);
                if ui
                    .selectable_value(&mut config.pattern, Pattern::Basic4Beat, "Basic4Beat")
                    .clicked()
                    || ui
                        .selectable_value(&mut config.pattern, Pattern::Basic8Beat, "Basic8Beat")
                        .clicked()
                    || ui
                        .selectable_value(
                            &mut config.pattern,
                            Pattern::FourToTheFloor8Beat,
                            "FourToTheFloor8Beat",
                        )
                        .clicked()
                    || ui
                        .selectable_value(&mut config.pattern, Pattern::Swing8Beat, "Swing8Beat")
                        .clicked()
                    || ui
                        .selectable_value(&mut config.pattern, Pattern::Rock8Beat, "Rock8Beat")
                        .clicked()
                {
                    let _ = ctrl_channel.send(Control::DrumMachine(self.config));
                }
            });
    }
}

#[derive(Default)]
struct MetronomeApp {
    active: bool,
    config: MetronomeConfig,
}

impl MetronomeApp {
    fn new() -> Self {
        Self::default()
    }

    fn draw(&mut self, ui: &mut egui::Ui, ctrl_channel: &ControlSender) {
        if ui
            .add(
                egui::Slider::new(&mut self.config.beats_per_minute, 40..=240)
                    .text("beats per minute"),
            )
            .changed()
        {
            let _ = ctrl_channel.send(Control::Metronome(self.config));
        }
    }
}

#[derive(Default)]
struct SynthApp {
    active: bool,
    tone: Option<u8>,
}

impl SynthApp {
    fn new() -> Self {
        Self::default()
    }

    fn draw(&mut self, ui: &mut egui::Ui, ctrl_channel: &ControlSender) {
        let mut tone = None;
        ui.add(gui::keyboard(&mut tone));

        // generate the appropriate midi events
        if self.tone != tone {
            if let Some(tone) = self.tone {
                let note = MidiNote::new(tone + 36, 127);
                let note_off = MidiData::NoteOff(note);
                let _ = ctrl_channel.send(Control::Midi(note_off));
            }

            self.tone = tone;
            if let Some(tone) = tone {
                let note = MidiNote::new(tone + 36, 127);
                let note_on = MidiData::NoteOn(note);
                let _ = ctrl_channel.send(Control::Midi(note_on));
            }
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
                    ui.menu_button("🎸", |ui| {
                        if ui.button("Organize windows").clicked() {
                            ui.ctx().memory_mut(|mem| mem.reset_areas());
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if ui.button("Quit").clicked() {
                                std::process::exit(0);
                            }
                        }
                    });
                    egui::widgets::global_dark_light_mode_switch(ui);
                });
            });

            egui::SidePanel::right("window_list")
                .resizable(false)
                .default_width(125.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                            ui.toggle_value(&mut self.settings, "Settings");
                            if ui
                                .toggle_value(&mut self.amplifier.active, "Amplifier")
                                .clicked()
                            {
                                let _ = self.channel.send(Control::Application(Active::Amplifier(
                                    self.amplifier.active,
                                )));
                            }
                            if ui
                                .toggle_value(&mut self.drum_machine.active, "Drum Machine")
                                .clicked()
                            {
                                let _ = self.channel.send(Control::Application(
                                    Active::DrumMachine(self.drum_machine.active),
                                ));
                            }
                            if ui
                                .toggle_value(&mut self.metronome.active, "Metronome")
                                .clicked()
                            {
                                let _ = self.channel.send(Control::Application(Active::Metronome(
                                    self.metronome.active,
                                )));
                            }
                            if ui.toggle_value(&mut self.synth.active, "Synth").clicked() {
                                let _ = self
                                    .channel
                                    .send(Control::Application(Active::Synth(self.synth.active)));
                            }
                        });
                    });
                });

            if self.settings {
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
            }

            if self.amplifier.active {
                egui::Window::new("Amplifier").show(ctx, |ui| {
                    self.amplifier.draw(ui, &self.channel);
                });
            }

            if self.drum_machine.active {
                egui::Window::new("Drum machine").show(ctx, |ui| {
                    self.drum_machine.draw(ui, &self.channel);
                });
            }

            if self.metronome.active {
                egui::Window::new("Metronome").show(ctx, |ui| {
                    self.metronome.draw(ui, &self.channel);
                });
            }

            if self.synth.active {
                egui::Window::new("synth").show(ctx, |ui| {
                    self.synth.draw(ui, &self.channel);
                });
            }
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
