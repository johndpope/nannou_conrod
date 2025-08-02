//! Main Timeline widget implementation using egui

use egui::{*, self};
use crate::{TimelineConfig, RiveEngine, LayerId};
use std::collections::HashMap;

/// Main timeline widget that displays layers, frames, and playback controls
pub struct Timeline {
    pub config: TimelineConfig,
    pub state: TimelineState,
}

/// Persistent state for the timeline
#[derive(Clone, Debug)]
pub struct TimelineState {
    /// Currently selected layers
    pub selected_layers: Vec<LayerId>,
    /// Currently selected frames (layer_id -> frame numbers)
    pub selected_frames: HashMap<LayerId, Vec<u32>>,
    /// Current playhead position
    pub playhead_frame: u32,
    /// Is timeline playing
    pub is_playing: bool,
    /// Current zoom level
    pub zoom_level: f32,
    /// Horizontal scroll position
    pub scroll_x: f32,
    /// Vertical scroll position  
    pub scroll_y: f32,
    /// Track heights that have been manually adjusted
    pub track_heights: HashMap<LayerId, f32>,
}

impl Default for TimelineState {
    fn default() -> Self {
        Self {
            selected_layers: Vec::new(),
            selected_frames: HashMap::new(),
            playhead_frame: 0,
            is_playing: false,
            zoom_level: 1.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
            track_heights: HashMap::new(),
        }
    }
}

impl Timeline {
    /// Create a new timeline with default configuration
    pub fn new() -> Self {
        Self {
            config: TimelineConfig::default(),
            state: TimelineState::default(),
        }
    }

    /// Create timeline with custom configuration
    pub fn with_config(config: TimelineConfig) -> Self {
        Self {
            config,
            state: TimelineState::default(),
        }
    }

    /// Show the timeline UI
    pub fn show(&mut self, ui: &mut Ui, engine: &mut Box<dyn RiveEngine>) -> Response {
        let available_rect = ui.available_rect_before_wrap();
        
        // Allocate space for the timeline
        let response = ui.allocate_rect(available_rect, Sense::click_and_drag());
        
        // Draw timeline background
        ui.painter().rect_filled(
            available_rect,
            0.0,
            self.config.style.background_color,
        );

        // Layout regions
        let layer_panel_rect = Rect::from_min_size(
            available_rect.min,
            vec2(self.config.layer_panel_width, available_rect.height() - self.config.controls_height),
        );

        let ruler_rect = Rect::from_min_size(
            available_rect.min + vec2(self.config.layer_panel_width, 0.0),
            vec2(available_rect.width() - self.config.layer_panel_width, self.config.ruler_height),
        );

        let frame_grid_rect = Rect::from_min_size(
            available_rect.min + vec2(self.config.layer_panel_width, self.config.ruler_height),
            vec2(
                available_rect.width() - self.config.layer_panel_width,
                available_rect.height() - self.config.ruler_height - self.config.controls_height,
            ),
        );

        let controls_rect = Rect::from_min_size(
            available_rect.min + vec2(0.0, available_rect.height() - self.config.controls_height),
            vec2(available_rect.width(), self.config.controls_height),
        );

        // Draw each section
        self.draw_layer_panel(ui, layer_panel_rect, engine);
        self.draw_ruler(ui, ruler_rect, engine);
        self.draw_frame_grid(ui, frame_grid_rect, engine);
        self.draw_playback_controls(ui, controls_rect, engine);
        self.draw_playhead(ui, ruler_rect, frame_grid_rect, engine);

        response
    }

    /// Draw the layer panel on the left
    fn draw_layer_panel(&mut self, ui: &mut Ui, rect: Rect, engine: &Box<dyn RiveEngine>) {
        // Draw background
        ui.painter().rect_filled(
            rect,
            0.0,
            self.config.style.layer_background,
        );

        // Draw border
        ui.painter().line_segment(
            [rect.right_top(), rect.right_bottom()],
            Stroke::new(1.0, self.config.style.border_color),
        );

        // Get layers from engine
        let layers = engine.get_layers();
        
        // Draw each layer
        let mut y_offset = rect.min.y;
        for (_idx, layer) in layers.iter().enumerate() {
            let layer_height = self.state.track_heights
                .get(&layer.id)
                .copied()
                .unwrap_or(self.config.default_track_height);

            let layer_rect = Rect::from_min_size(
                pos2(rect.min.x, y_offset),
                vec2(rect.width(), layer_height),
            );

            // Check if selected
            let is_selected = self.state.selected_layers.contains(&layer.id);
            if is_selected {
                ui.painter().rect_filled(
                    layer_rect,
                    0.0,
                    self.config.style.layer_selected,
                );
            }

            // Draw layer name
            ui.painter().text(
                layer_rect.min + vec2(5.0, layer_height / 2.0),
                Align2::LEFT_CENTER,
                &layer.name,
                FontId::default(),
                self.config.style.text_color,
            );

            // Draw visibility icon (stubbed)
            let eye_rect = Rect::from_center_size(
                layer_rect.right_center() - vec2(30.0, 0.0),
                vec2(20.0, 20.0),
            );
            if layer.visible {
                ui.painter().circle_filled(
                    eye_rect.center(),
                    8.0,
                    self.config.style.text_color,
                );
            }

            // Draw lock icon (stubbed)
            let lock_rect = Rect::from_center_size(
                layer_rect.right_center() - vec2(55.0, 0.0),
                vec2(20.0, 20.0),
            );
            if layer.locked {
                ui.painter().rect_filled(
                    lock_rect.shrink(4.0),
                    2.0,
                    self.config.style.text_color,
                );
            }

            y_offset += layer_height;
        }

        // Log interactions (stubbed)
        if ui.input(|i| i.pointer.primary_clicked()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if rect.contains(pos) {
                    println!("Layer panel clicked at {:?}", pos);
                }
            }
        }
    }

    /// Draw the ruler at the top
    fn draw_ruler(&mut self, ui: &mut Ui, rect: Rect, engine: &Box<dyn RiveEngine>) {
        // Draw background
        ui.painter().rect_filled(
            rect,
            0.0,
            self.config.style.layer_background,
        );

        // Draw border
        ui.painter().line_segment(
            [rect.left_bottom(), rect.right_bottom()],
            Stroke::new(1.0, self.config.style.border_color),
        );

        let total_frames = engine.get_total_frames();
        let frame_width = self.config.frame_width * self.state.zoom_level;

        // Draw frame numbers
        let visible_start_frame = (self.state.scroll_x / frame_width) as u32;
        let visible_frames = (rect.width() / frame_width).ceil() as u32;

        for frame in visible_start_frame..=(visible_start_frame + visible_frames).min(total_frames) {
            if frame % 5 == 0 {
                let x = rect.min.x + (frame as f32 - self.state.scroll_x / frame_width) * frame_width;
                
                ui.painter().text(
                    pos2(x, rect.center().y),
                    Align2::CENTER_CENTER,
                    frame.to_string(),
                    FontId::default(),
                    self.config.style.text_color,
                );

                // Draw tick mark
                ui.painter().line_segment(
                    [pos2(x, rect.bottom() - 5.0), pos2(x, rect.bottom())],
                    Stroke::new(1.0, self.config.style.border_color),
                );
            }
        }
    }

    /// Draw the frame grid
    fn draw_frame_grid(&mut self, ui: &mut Ui, rect: Rect, engine: &Box<dyn RiveEngine>) {
        // Clip to rect
        ui.painter().with_clip_rect(rect).rect_filled(
            rect,
            0.0,
            self.config.style.background_color,
        );

        let layers = engine.get_layers();
        let frame_width = self.config.frame_width * self.state.zoom_level;

        // Draw grid lines
        let visible_start_frame = (self.state.scroll_x / frame_width) as u32;
        let visible_frames = (rect.width() / frame_width).ceil() as u32;

        for frame in visible_start_frame..=(visible_start_frame + visible_frames) {
            let x = rect.min.x + (frame as f32 - self.state.scroll_x / frame_width) * frame_width;
            
            let color = if frame % 5 == 0 {
                self.config.style.grid_color
            } else {
                self.config.style.grid_color.gamma_multiply(0.5)
            };

            ui.painter().line_segment(
                [pos2(x, rect.min.y), pos2(x, rect.max.y)],
                Stroke::new(1.0, color),
            );
        }

        // Draw frames for each layer
        let mut y_offset = rect.min.y;
        for layer in &layers {
            let layer_height = self.state.track_heights
                .get(&layer.id)
                .copied()
                .unwrap_or(self.config.default_track_height);

            // Draw alternating row background
            if layers.iter().position(|l| l.id == layer.id).unwrap_or(0) % 2 == 1 {
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        pos2(rect.min.x, y_offset),
                        vec2(rect.width(), layer_height),
                    ),
                    0.0,
                    self.config.style.background_color.gamma_multiply(1.1),
                );
            }

            // Draw frames (stubbed - would get from engine)
            for frame in visible_start_frame..=(visible_start_frame + visible_frames) {
                let frame_data = engine.get_frame_data(layer.id.clone(), frame);
                let x = rect.min.x + (frame as f32 - self.state.scroll_x / frame_width) * frame_width;
                
                let frame_rect = Rect::from_min_size(
                    pos2(x, y_offset),
                    vec2(frame_width - 1.0, layer_height - 1.0),
                );

                let color = match frame_data.frame_type {
                    crate::frame::FrameType::Empty => self.config.style.frame_empty,
                    crate::frame::FrameType::Keyframe => self.config.style.frame_keyframe,
                    crate::frame::FrameType::Tween => self.config.style.frame_tween,
                };

                ui.painter().rect_filled(frame_rect, 2.0, color);

                // Draw keyframe indicator
                if matches!(frame_data.frame_type, crate::frame::FrameType::Keyframe) {
                    ui.painter().circle_filled(
                        frame_rect.center(),
                        3.0,
                        self.config.style.text_color,
                    );
                }
            }

            y_offset += layer_height;
        }

        // Handle frame selection (stubbed)
        if ui.input(|i| i.pointer.primary_clicked()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if rect.contains(pos) {
                    println!("Frame grid clicked at {:?}", pos);
                }
            }
        }
    }

    /// Draw playback controls
    fn draw_playback_controls(&mut self, ui: &mut Ui, rect: Rect, engine: &mut Box<dyn RiveEngine>) {
        // Draw background
        ui.painter().rect_filled(
            rect,
            0.0,
            self.config.style.layer_background,
        );

        // Draw border
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(1.0, self.config.style.border_color),
        );

        ui.allocate_ui_at_rect(rect, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(10.0);

                // Play/Pause button
                let play_text = if self.state.is_playing { "⏸" } else { "▶" };
                if ui.button(play_text).clicked() {
                    println!("Play/Pause clicked - would trigger Rive playback");
                    self.state.is_playing = !self.state.is_playing;
                    if self.state.is_playing {
                        engine.play();
                    } else {
                        engine.pause();
                    }
                }

                // Stop button
                if ui.button("⏹").clicked() {
                    println!("Stop clicked");
                    self.state.is_playing = false;
                    self.state.playhead_frame = 0;
                    engine.seek(0);
                }

                ui.add_space(20.0);

                // Frame display
                ui.label(format!("Frame: {}/{}", 
                    engine.get_current_frame(),
                    engine.get_total_frames()
                ));

                ui.add_space(20.0);

                // FPS display
                ui.label(format!("FPS: {:.1}", engine.get_fps()));

                ui.add_space(20.0);

                // Zoom controls
                ui.label("Zoom:");
                if ui.button("-").clicked() {
                    self.state.zoom_level = (self.state.zoom_level * 0.8).max(0.1);
                    println!("Zoom out to {}", self.state.zoom_level);
                }
                ui.label(format!("{:.0}%", self.state.zoom_level * 100.0));
                if ui.button("+").clicked() {
                    self.state.zoom_level = (self.state.zoom_level * 1.2).min(5.0);
                    println!("Zoom in to {}", self.state.zoom_level);
                }
            });
        });
    }

    /// Draw the playhead
    fn draw_playhead(&mut self, ui: &mut Ui, ruler_rect: Rect, grid_rect: Rect, engine: &mut Box<dyn RiveEngine>) {
        let current_frame = engine.get_current_frame();
        let frame_width = self.config.frame_width * self.state.zoom_level;
        let x = ruler_rect.min.x + (current_frame as f32 - self.state.scroll_x / frame_width) * frame_width;

        // Only draw if visible
        if x >= ruler_rect.min.x && x <= ruler_rect.max.x {
            // Draw playhead line
            ui.painter().line_segment(
                [pos2(x, ruler_rect.min.y), pos2(x, grid_rect.max.y)],
                Stroke::new(2.0, self.config.style.playhead_color),
            );

            // Draw playhead marker in ruler
            let points = vec![
                pos2(x, ruler_rect.min.y),
                pos2(x - 5.0, ruler_rect.min.y + 10.0),
                pos2(x + 5.0, ruler_rect.min.y + 10.0),
            ];
            ui.painter().add(Shape::convex_polygon(
                points,
                self.config.style.playhead_color,
                Stroke::NONE,
            ));
        }

        // Handle playhead dragging (stubbed)
        if ui.input(|i| i.pointer.primary_down()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if ruler_rect.contains(pos) {
                    let frame = ((pos.x - ruler_rect.min.x + self.state.scroll_x) / frame_width) as u32;
                    println!("Playhead dragged to frame {}", frame);
                    engine.seek(frame);
                }
            }
        }
    }
}