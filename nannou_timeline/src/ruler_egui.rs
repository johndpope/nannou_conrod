//! Ruler widget for frame numbers and markers

use egui::{*, self};

/// Ruler that displays frame numbers and markers
#[derive(Clone, Debug)]
pub struct Ruler {
    pub height: f32,
    pub font_size: f32,
    pub major_tick_interval: u32,
    pub minor_tick_interval: u32,
}

impl Default for Ruler {
    fn default() -> Self {
        Self {
            height: 30.0,
            font_size: 10.0,
            major_tick_interval: 10,
            minor_tick_interval: 5,
        }
    }
}

impl Ruler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Draw the ruler
    pub fn draw(
        &self,
        ui: &mut Ui,
        rect: Rect,
        _start_frame: u32,
        end_frame: u32,
        frame_width: f32,
        scroll_offset: f32,
    ) {
        // Background
        ui.painter().rect_filled(
            rect,
            0.0,
            ui.style().visuals.extreme_bg_color,
        );

        // Bottom border
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            ui.style().visuals.widgets.noninteractive.bg_stroke,
        );

        // Draw frame numbers and ticks
        let visible_start = ((scroll_offset / frame_width) as u32).saturating_sub(1);
        let visible_end = visible_start + ((rect.width() / frame_width) as u32) + 2;

        for frame in visible_start..=visible_end.min(end_frame) {
            let x = rect.min.x + (frame as f32 * frame_width) - scroll_offset;
            
            if x < rect.min.x - frame_width || x > rect.max.x + frame_width {
                continue;
            }

            // Major ticks with numbers
            if frame % self.major_tick_interval == 0 {
                ui.painter().text(
                    pos2(x, rect.center().y),
                    Align2::CENTER_CENTER,
                    frame.to_string(),
                    FontId::proportional(self.font_size),
                    ui.style().visuals.text_color(),
                );

                ui.painter().line_segment(
                    [pos2(x, rect.bottom() - 8.0), pos2(x, rect.bottom())],
                    ui.style().visuals.widgets.noninteractive.bg_stroke,
                );
            }
            // Minor ticks
            else if frame % self.minor_tick_interval == 0 {
                ui.painter().line_segment(
                    [pos2(x, rect.bottom() - 4.0), pos2(x, rect.bottom())],
                    ui.style().visuals.widgets.noninteractive.bg_stroke,
                );
            }
        }
    }

    /// Add a frame label at the specified position
    pub fn add_label(&self, ui: &mut Ui, _frame: u32, label: &str, x: f32, y: f32) {
        ui.painter().text(
            pos2(x, y - 10.0),
            Align2::CENTER_BOTTOM,
            label,
            FontId::proportional(self.font_size * 0.9),
            ui.style().visuals.strong_text_color(),
        );
    }
}