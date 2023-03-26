// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2023 Daniel Thompson

use egui::{Color32, Rect, Vec2};
use itertools::{chain, Itertools};

const fn is_black(tone: u8) -> bool {
    match tone % 12 {
        1 | 3 | 6 | 8 | 10 => true,
        _ => false,
    }
}

struct PianoKey {
    tone: u8,
    is_black: bool,
    bounding_box: Rect,
}

struct PianoKeyIterator {
    tone: u8,
    max_tone: u8,
    width: f32,
    white_key: Rect,
    black_key: Rect,
}

impl PianoKeyIterator {
    fn new(bounding_box: &Rect, num_tones: u8) -> Self {
        let max_tone = num_tones + is_black(num_tones - 1) as u8;
        let num_white = (0..num_tones).filter(|x| !is_black(*x)).count() as u16;

        let width = bounding_box.width() / f32::from(num_white);
        let height = bounding_box.height();

        let mut white_key = bounding_box.clone();
        white_key.set_width(width);

        let mut black_key = bounding_box.clone();
        black_key.set_width(width * 0.60);
        black_key.set_height(height * 0.60);
        black_key = black_key.translate(Vec2::new(width * 0.70, 0.0));

        Self {
            tone: 0,
            max_tone,
            width,
            white_key,
            black_key,
        }
    }
}

impl Iterator for PianoKeyIterator {
    type Item = PianoKey;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tone >= self.max_tone {
            return None;
        }

        let result = Self::Item {
            tone: self.tone,
            is_black: is_black(self.tone),
            bounding_box: if is_black(self.tone) {
                self.black_key
            } else {
                self.white_key
            },
        };

        // advance to next state
        self.tone += 1;
        if result.is_black {
            self.black_key.min.x += self.width;
            self.black_key.max.x += self.width;
        } else {
            self.white_key.min.x += self.width;
            self.white_key.max.x += self.width;
            if self.tone > 0 && !is_black(self.tone) {
                self.black_key.min.x += self.width;
                self.black_key.max.x += self.width;
            }
        }

        Some(result)
    }
}

fn keyboard_ui(ui: &mut egui::Ui, tone: &mut Option<u8>) -> egui::Response {
    const NUM_KEYS: u8 = 25;

    let num_white_keys = PianoKeyIterator::new(&Rect::NOTHING, NUM_KEYS)
        .filter(|x| !x.is_black)
        .count() as u16
        + 1;
    let desired_size = ui.spacing().interact_size.y * egui::vec2(num_white_keys.into(), 4.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::drag());

    *tone = None;
    if response.dragged() {
        if let Some(pos) = response.interact_pointer_pos() {
            *tone = PianoKeyIterator::new(&rect, NUM_KEYS)
                .filter(|r| r.bounding_box.contains(pos))
                .sorted_by(|a, b| Ord::cmp(&a.is_black, &b.is_black))
                .map(|r| r.tone as u8)
                .last();
        }
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        for rect in chain(
            PianoKeyIterator::new(&rect, NUM_KEYS).filter(|x| !x.is_black),
            PianoKeyIterator::new(&rect, NUM_KEYS).filter(|x| x.is_black),
        ) {
            let touched = if let Some(tone) = *tone {
                tone == rect.tone
            } else {
                false
            };

            let color = if touched {
                ui.style().interact_selectable(&response, true).bg_fill
            } else if rect.is_black {
                Color32::BLACK
            } else {
                Color32::WHITE
            };

            let stroke = if touched {
                ui.style().interact(&response).fg_stroke
            } else {
                ui.style().noninteractive().fg_stroke
            };

            painter.rect(rect.bounding_box, 0.0, color, stroke);
        }
    }

    response
}

pub fn keyboard(tone: &mut Option<u8>) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| keyboard_ui(ui, tone)
}
