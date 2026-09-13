// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2023 Daniel Thompson

mod gui;

use egui_miniquad::EguiMq;
use miniquad;
use oxidamp::prelude::*;
use std::collections::VecDeque;
use std::ops::RangeInclusive;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

enum Active {
    Amplifier(bool),
    DrumMachine(bool),
    Metronome(bool),
    Synth(bool),
    Tuner(bool),
}

enum Control {
    Application(Active),
    Amplifier(AmplifierConfig),
    DrumMachine(DrumMachineConfig),
    Metronome(MetronomeConfig),
    Synth(SynthConfig),
    Midi(MidiData),
}

type ControlSender = mpsc::SyncSender<Control>;

fn main() {
    let (client, _status) =
        jack::Client::new("Oxidamp", jack::ClientOptions::NO_START_SERVER).unwrap();

    let amp_in = client
        .register_port("amp_in", jack::AudioIn::default())
        .unwrap();
    let tuner_in = client
        .register_port("tuner_in", jack::AudioIn::default())
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

    // The tuner needs a whole second of audio to analyse. The audio thread
    // only copies samples into this ring buffer; the (much slower) analysis
    // runs on the GUI thread.
    let tuner_buffer = Arc::new(Mutex::new(VecDeque::<f32>::new()));
    let audio_tuner_buffer = Arc::clone(&tuner_buffer);

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
    let mut synth = VoiceBox::<KarplusStrong>::default();
    synth.setup(&ctx);

    let mut tuner_active = false;

    let (sender, receiver) = mpsc::sync_channel(16);

    let process = jack::contrib::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // handle any pending control updates
            while let Ok(ctrl) = receiver.try_recv() {
                match ctrl {
                    Control::Application(app) => match app {
                        Active::Amplifier(active) => amp_active = active,
                        Active::DrumMachine(active) => dm_active = active,
                        Active::Metronome(active) => metronome_active = active,
                        Active::Synth(active) => synth_active = active,
                        Active::Tuner(active) => tuner_active = active,
                    },
                    Control::Amplifier(cfg) => amp.set_config(cfg),
                    Control::DrumMachine(cfg) => dm.set_config(cfg),
                    Control::Metronome(cfg) => metronome.set_config(cfg),
                    Control::Synth(cfg) => synth.for_each(|v| v.set_config(cfg)),
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

            if tuner_active {
                let input = tuner_in.as_slice(ps);
                if let Ok(mut buf) = audio_tuner_buffer.try_lock() {
                    buf.extend(input.iter().copied());
                    let cap = ctx.sampling_frequency as usize;
                    while buf.len() > cap {
                        buf.pop_front();
                    }
                }
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
    miniquad::start(conf, move || {
        Box::new(Stage::new(sender, Arc::clone(&tuner_buffer), ctx))
    });

    // Deliberately leak the client instead of deactivating it. jack-rs frees
    // its callback context during deactivate/drop while the notification
    // callbacks (client_registration, port_registration, ...) remain
    // registered, so the final "client unregistered" event delivered while
    // the client is closed then dereferences freed memory and segfaults
    // (observed with PipeWire's JACK implementation). The process is exiting
    // anyway, so skip teardown and let the sound server clean up on exit.
    std::mem::forget(active_client);
}

struct Stage {
    mq_ctx: Box<dyn miniquad::RenderingBackend>,
    egui_mq: EguiMq,
    channel: ControlSender,
    settings: bool,
    amplifier: AmplifierApp,
    drum_machine: DrumMachineApp,
    metronome: MetronomeApp,
    synth: SynthApp,
    tuner: TunerApp,
}

impl Stage {
    fn new(
        channel: mpsc::SyncSender<Control>,
        tuner_buffer: Arc<Mutex<VecDeque<f32>>>,
        ctx: AudioContext,
    ) -> Self {
        let mut mq_ctx = miniquad::window::new_rendering_backend();
        let egui_mq = EguiMq::new(&mut *mq_ctx);
        let ctx_egui = egui_mq.egui_ctx();

        ctx_egui.set_pixels_per_point(1.5);
        ctx_egui.set_visuals(egui::Visuals::light());

        Self {
            mq_ctx,
            egui_mq,
            channel,
            settings: false,
            amplifier: AmplifierApp::new(),
            drum_machine: DrumMachineApp::new(),
            metronome: MetronomeApp::new(),
            synth: SynthApp::new(),
            tuner: TunerApp::new(tuner_buffer, ctx),
        }
    }
}

/// A vertical slider with its value and name centred underneath.
///
/// egui wraps a slider in its own `Ui` laid out with `Align::Min`, and that
/// inner `Ui` inherits the parent's *available* rect (which starts at the
/// parent's left edge). So the track is placed at the left no matter how the
/// surrounding column is aligned. Giving the slider a max_rect exactly as wide
/// as the track makes that internal left alignment coincide with the centre of
/// the column.
fn vertical_slider(
    ui: &mut egui::Ui,
    value: &mut f32,
    range: RangeInclusive<f32>,
    name: &str,
) -> bool {
    const COLUMN_WIDTH: f32 = 84.0;

    // Drag a full sweep over roughly 200 points, matching the feel of the
    // built-in value box.
    let speed = ((*range.end() - *range.start()) / 200.0) as f64;
    let slider_range = range.clone();

    // The same track thickness the slider computes for itself.
    let thickness = ui
        .text_style_height(&egui::TextStyle::Body)
        .max(ui.spacing().interact_size.y);
    let height = ui.spacing().slider_width + 56.0;

    let mut changed = false;
    ui.allocate_ui_with_layout(
        egui::vec2(COLUMN_WIDTH, height),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(thickness, ui.spacing().slider_width),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    changed = ui
                        .add(
                            egui::Slider::new(value, slider_range)
                                .vertical()
                                .show_value(false),
                        )
                        .changed();
                },
            );

            changed |= ui
                .add(
                    egui::DragValue::new(value)
                        .range(range)
                        .speed(speed)
                        .max_decimals(4),
                )
                .changed();
            ui.label(name);
        },
    );

    changed
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
        let mut changed = false;

        ui.horizontal(|ui| {
            // For vertical sliders this is the track height.
            ui.spacing_mut().slider_width = 120.0;

            changed |= vertical_slider(ui, &mut self.config.preamp.gain, 0.0..=96.0, "drive");
            changed |= vertical_slider(ui, &mut self.config.tonestack.bass, -24.0..=24.0, "bass");
            changed |= vertical_slider(ui, &mut self.config.tonestack.mid, -24.0..=24.0, "mid");
            changed |= vertical_slider(
                ui,
                &mut self.config.tonestack.treble,
                -24.0..=24.0,
                "treble",
            );
            changed |= vertical_slider(ui, &mut self.config.tonestack.gain, -24.0..=24.0, "gain");
        });

        if changed {
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
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
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
    config: SynthConfig,
    /// Velocity used for notes played on the on-screen keyboard. Drives the
    /// strike displacement of the pluck.
    velocity: f32,
}

impl SynthApp {
    fn new() -> Self {
        Self {
            velocity: 0.8,
            ..Self::default()
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui, ctrl_channel: &ControlSender) {
        let mut changed = false;

        ui.horizontal(|ui| {
            // For vertical sliders this is the track height.
            ui.spacing_mut().slider_width = 120.0;

            changed |= vertical_slider(ui, &mut self.config.position, 0.02..=0.98, "pluck");
            changed |= vertical_slider(ui, &mut self.config.hardness, 0.0..=1.0, "pick");
            changed |= vertical_slider(ui, &mut self.config.gain, 0.90..=0.9995, "sustain");

            // Velocity is per note, so it is not part of the config sent to the
            // audio thread; it is applied to the keyboard's note-on events.
            let _ = vertical_slider(ui, &mut self.velocity, 0.0..=1.0, "strike");
        });

        if changed {
            let _ = ctrl_channel.send(Control::Synth(self.config));
        }

        let mut tone = None;
        ui.add(gui::keyboard(&mut tone));

        // generate the appropriate midi events
        if self.tone != tone {
            let velocity = (self.velocity.clamp(0.0, 1.0) * 127.0).round() as u8;

            if let Some(tone) = self.tone {
                let note = MidiNote::new(tone + 36, velocity);
                let note_off = MidiData::NoteOff(note);
                let _ = ctrl_channel.send(Control::Midi(note_off));
            }

            self.tone = tone;
            if let Some(tone) = tone {
                let note = MidiNote::new(tone + 36, velocity);
                let note_on = MidiData::NoteOn(note);
                let _ = ctrl_channel.send(Control::Midi(note_on));
            }
        }
    }
}

/// Turn a MIDI note number into a name such as `A4`.
fn note_name(note: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = note as i32 / 12 - 1;
    format!("{}{}", NAMES[(note % 12) as usize], octave)
}

struct TunerApp {
    active: bool,
    buffer: Arc<Mutex<VecDeque<f32>>>,
    ctx: AudioContext,
    result: Option<NoteAnalysis>,
    last_analysis: Instant,
}

impl TunerApp {
    /// The analysis is comparatively expensive, so it is only run a few
    /// times per second. That is still far more often than a human can
    /// usefully read the display.
    const ANALYSIS_PERIOD: Duration = Duration::from_millis(100);

    fn new(buffer: Arc<Mutex<VecDeque<f32>>>, ctx: AudioContext) -> Self {
        Self {
            active: false,
            buffer,
            ctx,
            result: None,
            last_analysis: Instant::now(),
        }
    }

    /// Forget any buffered audio, e.g. when the tuner is switched on.
    fn reset(&mut self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.clear();
        }
        self.result = None;
    }

    fn analyse(&mut self) {
        if self.last_analysis.elapsed() < Self::ANALYSIS_PERIOD {
            return;
        }
        self.last_analysis = Instant::now();

        // Take a snapshot of the buffer so the audio thread is not blocked
        // while the note is analysed.
        let snapshot = match self.buffer.lock() {
            Ok(buf) => {
                if buf.len() < self.ctx.sampling_frequency as usize {
                    return;
                }
                buf.iter().copied().collect::<Vec<f32>>()
            }
            Err(_) => return,
        };

        self.result = Some(snapshot.analyse_note(&self.ctx));
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        self.analyse();

        let Some(result) = self.result else {
            ui.label("Listening...");
            return;
        };

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(note_name(result.note))
                    .size(36.0)
                    .strong(),
            );
            ui.vertical(|ui| {
                ui.label(format!("{:.2} Hz", result.frequency));
                ui.label(format!("{:+.1} cents", result.cents));
            });
        });

        // A simple needle showing the deviation from the nearest note.
        let bg = ui.visuals().extreme_bg_color;
        let fg = ui.visuals().weak_text_color();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(240.0, 24.0), egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, bg);

        let center = rect.center().x;
        painter.line_segment(
            [
                egui::pos2(center, rect.top()),
                egui::pos2(center, rect.bottom()),
            ],
            egui::Stroke::new(1.0_f32, fg),
        );

        let clamped = result.cents.clamp(-50.0, 50.0);
        let x = center + (clamped / 50.0) * (rect.width() / 2.0);
        let color = if result.cents.abs() < 5.0 {
            egui::Color32::from_rgb(0x2e, 0x8b, 0x57)
        } else {
            egui::Color32::from_rgb(0xc0, 0x39, 0x2b)
        };
        painter.circle_filled(egui::pos2(x, rect.center().y), 6.0, color);
    }
}

impl miniquad::EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        let Self {
            mq_ctx,
            egui_mq,
            channel,
            settings,
            amplifier,
            drum_machine,
            metronome,
            synth,
            tuner,
        } = self;

        mq_ctx.clear(Some((1., 1., 1., 1.)), None, None);
        mq_ctx.begin_default_pass(miniquad::PassAction::clear_color(0.65, 0.70, 0.65, 1.0));
        mq_ctx.end_render_pass();

        // Run the UI code:
        egui_mq.run(&mut **mq_ctx, |_mq_ctx, ctx| {
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
                    egui::widgets::global_theme_preference_switch(ui);
                });
            });

            egui::SidePanel::right("window_list")
                .resizable(false)
                .default_width(125.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                            ui.toggle_value(settings, "Settings");
                            if ui
                                .toggle_value(&mut amplifier.active, "Amplifier")
                                .clicked()
                            {
                                let _ = channel.send(Control::Application(Active::Amplifier(
                                    amplifier.active,
                                )));
                            }
                            if ui
                                .toggle_value(&mut drum_machine.active, "Drum Machine")
                                .clicked()
                            {
                                let _ = channel.send(Control::Application(Active::DrumMachine(
                                    drum_machine.active,
                                )));
                            }
                            if ui
                                .toggle_value(&mut metronome.active, "Metronome")
                                .clicked()
                            {
                                let _ = channel.send(Control::Application(Active::Metronome(
                                    metronome.active,
                                )));
                            }
                            if ui.toggle_value(&mut synth.active, "Synth").clicked() {
                                let _ =
                                    channel.send(Control::Application(Active::Synth(synth.active)));
                            }
                            if ui.toggle_value(&mut tuner.active, "Tuner").clicked() {
                                if tuner.active {
                                    tuner.reset();
                                }
                                let _ =
                                    channel.send(Control::Application(Active::Tuner(tuner.active)));
                            }
                        });
                    });
                });

            if *settings {
                let mut pixels_per_point = ctx.pixels_per_point();
                egui::Window::new("Settings").show(ctx, |ui| {
                    let response = ui
                        .add(
                            egui::Slider::new(&mut pixels_per_point, 0.75..=3.0)
                                .logarithmic(true)
                                .text("scale"),
                        )
                        .on_hover_text("Physical pixels per logical point");
                    if response.clicked() || response.drag_stopped() {
                        ctx.set_pixels_per_point(pixels_per_point);
                    }
                });
            }

            if amplifier.active {
                egui::Window::new("Amplifier").show(ctx, |ui| {
                    amplifier.draw(ui, channel);
                });
            }

            if drum_machine.active {
                egui::Window::new("Drum machine").show(ctx, |ui| {
                    drum_machine.draw(ui, channel);
                });
            }

            if metronome.active {
                egui::Window::new("Metronome").show(ctx, |ui| {
                    metronome.draw(ui, channel);
                });
            }

            if synth.active {
                egui::Window::new("synth").show(ctx, |ui| {
                    synth.draw(ui, channel);
                });
            }

            if tuner.active {
                egui::Window::new("Tuner").show(ctx, |ui| {
                    tuner.draw(ui);
                });
            }
        });

        egui_mq.draw(&mut **mq_ctx);

        mq_ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
    }

    fn mouse_wheel_event(&mut self, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(dx, dy);
    }

    fn mouse_button_down_event(&mut self, mb: miniquad::MouseButton, x: f32, y: f32) {
        self.egui_mq.mouse_button_down_event(mb, x, y);
    }

    fn mouse_button_up_event(&mut self, mb: miniquad::MouseButton, x: f32, y: f32) {
        self.egui_mq.mouse_button_up_event(mb, x, y);
    }

    fn char_event(&mut self, character: char, _keymods: miniquad::KeyMods, _repeat: bool) {
        self.egui_mq.char_event(character);
    }

    fn key_down_event(
        &mut self,
        keycode: miniquad::KeyCode,
        keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.egui_mq.key_down_event(keycode, keymods);
    }

    fn key_up_event(&mut self, keycode: miniquad::KeyCode, keymods: miniquad::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }
}
