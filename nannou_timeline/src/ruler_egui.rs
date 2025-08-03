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

    /// Draw the ruler with frame numbers and time display
    pub fn draw(
        &self,
        ui: &mut Ui,
        rect: Rect,
        _start_frame: u32,
        end_frame: u32,
        frame_width: f32,
        scroll_offset: f32,
        frame_labels: &[crate::FrameLabel],
    ) {
        self.draw_with_fps_and_comments(ui, rect, _start_frame, end_frame, frame_width, scroll_offset, frame_labels, &[], 24.0);
    }

    /// Draw the ruler with frame numbers and time display (with FPS)
    pub fn draw_with_fps(
        &self,
        ui: &mut Ui,
        rect: Rect,
        _start_frame: u32,
        end_frame: u32,
        frame_width: f32,
        scroll_offset: f32,
        frame_labels: &[crate::FrameLabel],
        fps: f32,
    ) {
        self.draw_with_fps_and_comments(ui, rect, _start_frame, end_frame, frame_width, scroll_offset, frame_labels, &[], fps);
    }

    /// Draw the ruler with frame numbers, time display, labels and comments
    pub fn draw_with_fps_and_comments(
        &self,
        ui: &mut Ui,
        rect: Rect,
        _start_frame: u32,
        end_frame: u32,
        frame_width: f32,
        scroll_offset: f32,
        frame_labels: &[crate::FrameLabel],
        frame_comments: &[crate::FrameComment],
        fps: f32,
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

        // Calculate visible frame range
        let visible_start = ((scroll_offset / frame_width) as u32).saturating_sub(1);
        let visible_end = visible_start + ((rect.width() / frame_width) as u32) + 2;

        // Draw frame numbers and ticks
        for frame in visible_start..=visible_end.min(end_frame) {
            let x = rect.min.x + (frame as f32 * frame_width) - scroll_offset;
            
            if x < rect.min.x - frame_width || x > rect.max.x + frame_width {
                continue;
            }

            // Major ticks with numbers (every 10 frames by default, or 5 for wider spacing)
            let major_interval = if frame_width > 15.0 { 5 } else { 10 };
            if frame % major_interval == 0 {
                // Calculate time in seconds
                let time_seconds = frame as f32 / fps;
                
                // Format time display based on available space
                let time_text = if time_seconds >= 60.0 {
                    // Show minutes:seconds for longer times
                    let minutes = (time_seconds / 60.0) as u32;
                    let seconds = time_seconds % 60.0;
                    format!("{}:{:04.1}", minutes, seconds)
                } else {
                    // Show seconds with one decimal place
                    format!("{:.1}s", time_seconds)
                };
                
                // Draw frame number
                let frame_text = frame.to_string();
                ui.painter().text(
                    pos2(x, rect.center().y - 6.0),
                    Align2::CENTER_CENTER,
                    frame_text,
                    FontId::proportional(self.font_size),
                    ui.style().visuals.text_color(),
                );
                
                // Draw time below frame number if there's space
                if rect.height() > 20.0 {
                    ui.painter().text(
                        pos2(x, rect.center().y + 4.0),
                        Align2::CENTER_CENTER,
                        time_text,
                        FontId::proportional(self.font_size * 0.8),
                        ui.style().visuals.weak_text_color(),
                    );
                }

                // Draw major tick
                ui.painter().line_segment(
                    [pos2(x, rect.bottom() - 8.0), pos2(x, rect.bottom())],
                    ui.style().visuals.widgets.noninteractive.bg_stroke,
                );
            }
            // Minor ticks (every frame when zoomed in)
            else if frame_width > 8.0 {
                ui.painter().line_segment(
                    [pos2(x, rect.bottom() - 4.0), pos2(x, rect.bottom())],
                    ui.style().visuals.widgets.noninteractive.bg_stroke,
                );
            }
        }

        // Check for right-click on ruler for context menu
        let response = ui.allocate_rect(rect, Sense::click());
        if response.secondary_clicked() {
            let click_pos = response.interact_pointer_pos().unwrap_or_default();
            let clicked_frame = ((click_pos.x - rect.min.x + scroll_offset) / frame_width) as u32;
            
            ui.memory_mut(|mem| {
                mem.data.insert_temp("ruler_context_frame".into(), clicked_frame);
                mem.data.insert_temp("show_ruler_context".into(), true);
            });
        }
        
        // Show context menu if flag is set
        if ui.memory(|mem| mem.data.get_temp::<bool>("show_ruler_context".into()).unwrap_or(false)) {
            let clicked_frame = ui.memory(|mem| mem.data.get_temp::<u32>("ruler_context_frame".into()).unwrap_or(0));
            
            ui.menu_button("Ruler Menu", |ui| {
                ui.label(format!("Frame {}", clicked_frame));
                ui.separator();
                
                if ui.button("Add Frame Label").clicked() {
                    // This would trigger adding a new frame label
                    println!("Adding frame label at frame {}", clicked_frame);
                    ui.close();
                }
                
                if ui.button("Add Comment").clicked() {
                    // This would trigger adding a comment
                    println!("Adding comment at frame {}", clicked_frame);
                    ui.close();
                }
                
                ui.separator();
                
                if ui.button("Set Loop Start").clicked() {
                    println!("Setting loop start at frame {}", clicked_frame);
                    ui.close();
                }
                
                if ui.button("Set Loop End").clicked() {
                    println!("Setting loop end at frame {}", clicked_frame);
                    ui.close();
                }
                
                ui.separator();
                
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
            });
            
            // Clear the context menu flag after showing
            ui.memory_mut(|mem| {
                mem.data.remove::<bool>("show_ruler_context".into());
            });
        }

        // Draw frame labels
        for label in frame_labels {
            if label.frame >= visible_start && label.frame <= visible_end {
                let x = rect.min.x + (label.frame as f32 * frame_width) - scroll_offset;
                
                // Draw label marker
                let color = label.color.unwrap_or(ui.style().visuals.warn_fg_color);
                ui.painter().line_segment(
                    [pos2(x, rect.top()), pos2(x, rect.bottom())],
                    Stroke::new(2.0, color),
                );
                
                // Draw label text
                ui.painter().text(
                    pos2(x + 2.0, rect.top() + 2.0),
                    Align2::LEFT_TOP,
                    &label.label,
                    FontId::proportional(self.font_size * 0.9),
                    color,
                );
            }
        }
        
        // Draw frame comments
        for comment in frame_comments {
            if comment.frame >= visible_start && comment.frame <= visible_end {
                let x = rect.min.x + (comment.frame as f32 * frame_width) - scroll_offset;
                
                // Draw comment marker (slightly different from labels)
                let color = comment.color.unwrap_or(ui.style().visuals.hyperlink_color);
                ui.painter().circle_filled(
                    pos2(x, rect.center().y),
                    3.0,
                    color,
                );
                
                // Draw comment indicator line
                ui.painter().line_segment(
                    [pos2(x, rect.top() + 5.0), pos2(x, rect.bottom() - 5.0)],
                    Stroke::new(1.0, color),
                );
                
                // Draw comment text (truncated if long)
                let comment_text = if comment.comment.len() > 20 {
                    format!("{}...", &comment.comment[..17])
                } else {
                    comment.comment.clone()
                };
                
                ui.painter().text(
                    pos2(x - 2.0, rect.top() + 2.0),
                    Align2::RIGHT_TOP,
                    &comment_text,
                    FontId::proportional(self.font_size * 0.8),
                    color,
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