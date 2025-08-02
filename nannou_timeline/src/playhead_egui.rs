//! Playhead widget for timeline navigation

use egui::{*, self};

/// Playhead that indicates current position in timeline
#[derive(Clone, Debug)]
pub struct Playhead {
    pub frame: u32,
    pub color: Color32,
    pub width: f32,
}

impl Default for Playhead {
    fn default() -> Self {
        Self {
            frame: 0,
            color: Color32::from_rgb(255, 0, 0),
            width: 2.0,
        }
    }
}

impl Playhead {
    pub fn new(frame: u32) -> Self {
        Self {
            frame,
            ..Default::default()
        }
    }

    /// Draw the playhead at the given position
    pub fn draw(&self, ui: &mut Ui, x: f32, top: f32, bottom: f32) {
        ui.painter().line_segment(
            [pos2(x, top), pos2(x, bottom)],
            Stroke::new(self.width, self.color),
        );
    }

    /// Draw the playhead handle in the ruler area
    pub fn draw_handle(&self, ui: &mut Ui, x: f32, y: f32, size: f32) {
        let points = vec![
            pos2(x, y),
            pos2(x - size / 2.0, y + size),
            pos2(x + size / 2.0, y + size),
        ];
        ui.painter().add(Shape::convex_polygon(
            points,
            self.color,
            Stroke::NONE,
        ));
    }
}