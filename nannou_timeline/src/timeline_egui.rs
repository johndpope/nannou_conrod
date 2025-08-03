//! Main Timeline widget implementation using egui

use egui::{*, self};
use crate::{TimelineConfig, RiveEngine, LayerId, KeyframeId, MotionEditor};
use std::collections::HashMap;

/// Keyframe selection state for interactive manipulation
#[derive(Clone, Debug, Default)]
pub struct KeyframeSelection {
    /// Currently selected keyframes (layer_id, frame) -> keyframe_id
    pub selected: HashMap<(LayerId, u32), KeyframeId>,
    /// Drag operation state
    pub drag_state: Option<DragState>,
    /// Copied keyframes for paste operations
    pub clipboard: Vec<KeyframeClipboardItem>,
}

/// State tracking an active drag operation
#[derive(Clone, Debug)]
pub struct DragState {
    /// Original positions of all selected keyframes
    pub original_positions: HashMap<KeyframeId, (LayerId, u32)>,
    /// Current drag offset in frames
    pub frame_offset: i32,
    /// Mouse position where drag started
    pub start_pos: egui::Pos2,
}

/// Clipboard item for copy/paste operations
#[derive(Clone, Debug)]
pub struct KeyframeClipboardItem {
    pub layer_id: LayerId,
    pub relative_frame: u32,
    pub data: crate::frame::FrameData,
}

impl KeyframeSelection {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Clear all selections
    pub fn clear(&mut self) {
        self.selected.clear();
        self.drag_state = None;
    }
    
    /// Add a keyframe to selection
    pub fn add(&mut self, layer_id: LayerId, frame: u32, keyframe_id: KeyframeId) {
        self.selected.insert((layer_id, frame), keyframe_id);
    }
    
    /// Remove a keyframe from selection
    pub fn remove(&mut self, layer_id: LayerId, frame: u32) {
        self.selected.remove(&(layer_id, frame));
    }
    
    /// Check if a keyframe is selected
    pub fn is_selected(&self, layer_id: LayerId, frame: u32) -> bool {
        self.selected.contains_key(&(layer_id, frame))
    }
    
    /// Get selected keyframes as list
    pub fn get_selected(&self) -> Vec<(LayerId, u32, KeyframeId)> {
        self.selected.iter()
            .map(|((layer_id, frame), keyframe_id)| (layer_id.clone(), *frame, keyframe_id.clone()))
            .collect()
    }
}

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
    /// Right-click context menu state
    pub context_menu: Option<ContextMenuState>,
    /// Active snap guides (frame positions)
    pub snap_guides: Vec<f32>,
    /// Keyframe selection and manipulation state
    pub keyframe_selection: KeyframeSelection,
    /// Motion Editor for easing curves
    pub motion_editor: MotionEditor,
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
            context_menu: None,
            snap_guides: Vec::new(),
            keyframe_selection: KeyframeSelection::new(),
            motion_editor: MotionEditor::new(),
        }
    }
}

impl Timeline {
    /// Snap a position to the nearest valid grid position
    pub fn snap_position(&self, pos: f32, modifiers: &egui::Modifiers) -> f32 {
        // Disable snapping if Shift is held
        if modifiers.shift || !self.config.snap.enabled {
            return pos;
        }
        
        let frame_width = self.config.frame_width * self.state.zoom_level;
        let frame_pos = pos / frame_width;
        
        // Snap to frame boundaries
        if self.config.snap.snap_to_frames {
            let nearest_frame = frame_pos.round();
            let snapped_pos = nearest_frame * frame_width;
            
            if (pos - snapped_pos).abs() < self.config.snap.threshold_pixels {
                return snapped_pos;
            }
        }
        
        pos
    }
    
    /// Find snap targets for intelligent snapping
    pub fn find_snap_targets(&self, layer_id: &LayerId, frame: u32, engine: &Box<dyn crate::RiveEngine>) -> Vec<u32> {
        let mut targets = Vec::new();
        
        if self.config.snap.snap_to_keyframes {
            // Add keyframes from same layer
            for test_frame in 0..engine.get_total_frames() {
                let frame_data = engine.get_frame_data(layer_id.clone(), test_frame);
                if matches!(frame_data.frame_type, crate::frame::FrameType::Keyframe) {
                    targets.push(test_frame);
                }
            }
            
            // Add keyframes from other layers at same time position
            let layers = engine.get_layers();
            for layer in &layers {
                if layer.id != *layer_id {
                    let frame_data = engine.get_frame_data(layer.id.clone(), frame);
                    if matches!(frame_data.frame_type, crate::frame::FrameType::Keyframe) {
                        targets.push(frame);
                    }
                }
            }
        }
        
        if self.config.snap.snap_to_markers {
            // Add frame labels
            for label in &self.config.frame_labels {
                targets.push(label.frame);
            }
        }
        
        targets.sort_unstable();
        targets.dedup();
        targets
    }
    
    /// Update snap guides based on cursor position
    pub fn update_snap_guides(&mut self, cursor_pos: f32) {
        self.state.snap_guides.clear();
        
        if !self.config.snap.show_guides || !self.config.snap.enabled {
            return;
        }
        
        let frame_width = self.config.frame_width * self.state.zoom_level;
        let frame_pos = cursor_pos / frame_width;
        
        // Add guide for nearest frame
        if self.config.snap.snap_to_frames {
            let nearest_frame = frame_pos.round();
            let guide_pos = nearest_frame * frame_width;
            
            if (cursor_pos - guide_pos).abs() < self.config.snap.threshold_pixels {
                self.state.snap_guides.push(guide_pos);
            }
        }
    }
    
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
        
        // Handle keyboard shortcuts
        self.handle_keyboard_shortcuts(ui, engine);
        
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
        
        // Handle context menu (needs mutable engine access)
        self.handle_context_menu(ui, engine);
        
        // Draw snap guides
        self.draw_snap_guides(ui, frame_grid_rect);
        
        // Show Motion Editor if open
        self.state.motion_editor.show(ui.ctx());

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
        
        // Apply vertical scroll offset to sync with frame grid
        let scroll_offset = self.state.scroll_y;
        
        // Clip to the visible area
        ui.set_clip_rect(rect);
        
        // Draw each layer
        let mut y_offset = rect.min.y - scroll_offset;
        for (_idx, layer) in layers.iter().enumerate() {
            let layer_height = self.state.track_heights
                .get(&layer.id)
                .copied()
                .unwrap_or(self.config.default_track_height);

            // Skip layers outside visible area
            if y_offset + layer_height < rect.min.y || y_offset > rect.max.y {
                y_offset += layer_height;
                continue;
            }

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
        let ruler = crate::Ruler::new();
        let total_frames = engine.get_total_frames();
        let frame_width = self.config.frame_width * self.state.zoom_level;
        
        ruler.draw(
            ui,
            rect,
            0,
            total_frames,
            frame_width,
            self.state.scroll_x,
            &self.config.frame_labels,
        );
    }

    /// Draw the frame grid
    fn draw_frame_grid(&mut self, ui: &mut Ui, rect: Rect, engine: &mut Box<dyn RiveEngine>) {
        // Use ScrollArea for both horizontal and vertical scrolling
        egui::ScrollArea::both()
            .id_source("timeline_scroll")
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .show_viewport(ui, |ui, viewport| {
                // Update scroll state
                self.state.scroll_x = viewport.min.x;
                self.state.scroll_y = viewport.min.y;

                let layers = engine.get_layers();
                let frame_width = self.config.frame_width * self.state.zoom_level;
                let total_frames = engine.get_total_frames();
                
                // Calculate content size
                let total_width = total_frames as f32 * frame_width;
                let total_height = layers.iter()
                    .map(|l| self.state.track_heights.get(&l.id).copied()
                        .unwrap_or(self.config.default_track_height))
                    .sum::<f32>();
                
                // Set the content size for scrolling
                ui.set_min_size(vec2(total_width, total_height));
                
                // Calculate visible range
                let visible_start_frame = (viewport.min.x / frame_width).floor() as u32;
                let visible_end_frame = ((viewport.max.x / frame_width).ceil() as u32).min(total_frames);
                let _visible_frames = visible_end_frame.saturating_sub(visible_start_frame);

                // Draw background
                ui.painter().rect_filled(
                    ui.max_rect(),
                    0.0,
                    self.config.style.background_color,
                );
                
                // Draw grid lines
                for frame in visible_start_frame..=visible_end_frame {
                    let x = frame as f32 * frame_width;
            
            let color = if frame % 5 == 0 {
                self.config.style.grid_color
            } else {
                self.config.style.grid_color.gamma_multiply(0.5)
            };

                    ui.painter().line_segment(
                        [pos2(x, viewport.min.y), pos2(x, viewport.min.y + total_height)],
                        Stroke::new(1.0, color),
                    );
                }

                // Draw frames for each layer
                let mut y_offset = 0.0;
                for (layer_idx, layer) in layers.iter().enumerate() {
            let layer_height = self.state.track_heights
                .get(&layer.id)
                .copied()
                .unwrap_or(self.config.default_track_height);

                    // Only draw if layer is visible
                    if y_offset + layer_height < viewport.min.y || y_offset > viewport.max.y {
                        y_offset += layer_height;
                        continue;
                    }
                    
                    // Draw alternating row background
                    if layer_idx % 2 == 1 {
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                pos2(0.0, y_offset),
                                vec2(total_width, layer_height),
                            ),
                            0.0,
                            self.config.style.background_color.gamma_multiply(1.1),
                        );
                    }

                    // Draw frames
                    for frame in visible_start_frame..=visible_end_frame {
                        let frame_data = engine.get_frame_data(layer.id.clone(), frame);
                        let x = frame as f32 * frame_width;
                
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
                            let is_selected = self.state.keyframe_selection.is_selected(layer.id.clone(), frame);
                            
                            // Draw selection highlight if selected
                            if is_selected {
                                ui.painter().circle_stroke(
                                    frame_rect.center(),
                                    5.0,
                                    Stroke::new(2.0, egui::Color32::from_rgb(70, 130, 255)),
                                );
                            }
                            
                            // Draw keyframe circle
                            let keyframe_color = if is_selected {
                                egui::Color32::from_rgb(100, 150, 255)
                            } else {
                                self.config.style.text_color
                            };
                            
                            ui.painter().circle_filled(
                                frame_rect.center(),
                                3.0,
                                keyframe_color,
                            );
                        }
                    }

                    y_offset += layer_height;
                }
            });

        
        // Handle frame interactions
        let response = ui.interact(rect, ui.id().with("frame_grid"), Sense::click_and_drag());
        
        // Handle mouse wheel scrolling
        if let Some(hover_pos) = response.hover_pos() {
            if rect.contains(hover_pos) {
                let scroll_delta = ui.input(|i| i.scroll_delta);
                
                // Horizontal scroll with shift or horizontal wheel
                if ui.input(|i| i.modifiers.shift) || scroll_delta.x != 0.0 {
                    self.state.scroll_x = (self.state.scroll_x - scroll_delta.y * 20.0).max(0.0);
                }
                // Zoom with ctrl/cmd
                else if ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                    let zoom_delta = scroll_delta.y * 0.001;
                    let old_zoom = self.state.zoom_level;
                    self.state.zoom_level = (self.state.zoom_level * (1.0 + zoom_delta)).clamp(0.1, 5.0);
                    
                    // Zoom centered on mouse position
                    if old_zoom != self.state.zoom_level {
                        let mouse_frame_pos = (hover_pos.x - rect.min.x + self.state.scroll_x) / (self.config.frame_width * old_zoom);
                        let new_mouse_x = mouse_frame_pos * self.config.frame_width * self.state.zoom_level;
                        self.state.scroll_x = (new_mouse_x - (hover_pos.x - rect.min.x)).max(0.0);
                    }
                }
                // Vertical scroll normally
                else {
                    self.state.scroll_y = (self.state.scroll_y - scroll_delta.y).max(0.0);
                }
            }
        }
        
        // Handle keyframe selection clicks
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let modifiers = ui.input(|i| i.modifiers);
                self.handle_keyframe_click(pos, rect, &modifiers, engine);
            }
        }
        
        // Handle keyframe dragging
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.handle_keyframe_drag(pos, rect, response.drag_delta(), engine);
            }
        }
        
        // Handle drag end
        if response.drag_released() {
            self.handle_keyframe_drag_end(engine);
        }
        
        // Handle right-click for context menu
        if response.secondary_clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let frame_width = self.config.frame_width * self.state.zoom_level;
                let clicked_frame = ((pos.x - rect.min.x + self.state.scroll_x) / frame_width) as u32;
                
                // Find which layer was clicked
                let layers = engine.get_layers();
                let mut clicked_layer = None;
                let mut y_offset = self.state.scroll_y;
                for layer in &layers {
                    let layer_height = self.state.track_heights
                        .get(&layer.id)
                        .copied()
                        .unwrap_or(self.config.default_track_height);
                    
                    if pos.y >= rect.min.y + y_offset && pos.y < rect.min.y + y_offset + layer_height {
                        clicked_layer = Some(layer.id.clone());
                        break;
                    }
                    y_offset += layer_height;
                }
                
                if let Some(layer_id) = clicked_layer {
                    self.state.context_menu = Some(ContextMenuState {
                        position: pos,
                        frame: clicked_frame,
                        layer_id,
                    });
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

                // FPS selector
                ui.label("FPS:");
                egui::ComboBox::from_id_source("fps_selector")
                    .selected_text(self.config.fps.label())
                    .show_ui(ui, |ui| {
                        for preset in crate::FpsPreset::all_presets() {
                            if ui.selectable_value(&mut self.config.fps, preset, preset.label()).clicked() {
                                println!("FPS changed to: {}", preset.to_fps());
                                // TODO: Update engine FPS when connected
                            }
                        }
                        
                        ui.separator();
                        
                        // Custom FPS option
                        if ui.button("Custom...").clicked() {
                            // TODO: Open dialog for custom FPS
                            println!("Custom FPS dialog would open");
                        }
                    });

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
                
                ui.add_space(20.0);
                
                // Snap controls
                ui.separator();
                ui.add_space(5.0);
                
                let snap_icon = if self.config.snap.enabled { "🧲" } else { "⚫" };
                if ui.selectable_label(self.config.snap.enabled, format!("{} Snap", snap_icon)).clicked() {
                    self.config.snap.enabled = !self.config.snap.enabled;
                    println!("Snap {}", if self.config.snap.enabled { "enabled" } else { "disabled" });
                }
                
                if self.config.snap.enabled {
                    ui.menu_button("⚙", |ui| {
                        ui.label("Snap Settings:");
                        ui.separator();
                        
                        ui.checkbox(&mut self.config.snap.snap_to_frames, "Snap to frames");
                        ui.checkbox(&mut self.config.snap.snap_to_keyframes, "Snap to keyframes");
                        ui.checkbox(&mut self.config.snap.snap_to_markers, "Snap to markers");
                        ui.checkbox(&mut self.config.snap.show_guides, "Show guides");
                        
                        ui.separator();
                        ui.label("Snap distance:");
                        ui.add(egui::Slider::new(&mut self.config.snap.threshold_pixels, 1.0..=20.0)
                            .suffix(" px"));
                    });
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

        // Handle playhead dragging with snapping
        if ui.input(|i| i.pointer.primary_down()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if ruler_rect.contains(pos) {
                    let raw_x = pos.x - ruler_rect.min.x + self.state.scroll_x;
                    let modifiers = ui.input(|i| i.modifiers);
                    let snapped_x = self.snap_position(raw_x, &modifiers);
                    let frame = (snapped_x / frame_width) as u32;
                    
                    println!("Playhead dragged to frame {} (snapped: {})", frame, modifiers.shift);
                    engine.seek(frame);
                }
            }
        }
        
        // Update snap guides on hover
        if let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if ruler_rect.contains(hover_pos) || grid_rect.contains(hover_pos) {
                let raw_x = hover_pos.x - ruler_rect.min.x + self.state.scroll_x;
                self.update_snap_guides(raw_x);
            } else {
                self.state.snap_guides.clear();
            }
        }
    }
    
    /// Draw snap guides as vertical lines
    fn draw_snap_guides(&self, ui: &mut Ui, grid_rect: Rect) {
        if !self.config.snap.show_guides {
            return;
        }
        
        for &guide_x in &self.state.snap_guides {
            let x = grid_rect.min.x + guide_x - self.state.scroll_x;
            
            // Only draw if visible
            if x >= grid_rect.min.x && x <= grid_rect.max.x {
                ui.painter().line_segment(
                    [pos2(x, grid_rect.min.y), pos2(x, grid_rect.max.y)],
                    Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 0)), // Yellow guide
                );
                
                // Add a small indicator at the top
                ui.painter().circle_filled(
                    pos2(x, grid_rect.min.y + 3.0),
                    2.0,
                    egui::Color32::from_rgb(255, 255, 0),
                );
            }
        }
    }
    
    /// Handle keyboard shortcuts
    fn handle_keyboard_shortcuts(&mut self, ui: &mut Ui, engine: &mut Box<dyn RiveEngine>) {
        let ctx = ui.ctx();
        
        // Get selected layer and frame
        if let Some(layer_id) = self.state.selected_layers.first() {
            let current_frame = engine.get_current_frame();
            
            // F5: Insert Frame
            if ctx.input(|i| i.key_pressed(egui::Key::F5) && !i.modifiers.shift) {
                println!("F5: Insert frame at {} on layer {:?}", current_frame, layer_id);
                engine.insert_frame(layer_id.clone(), current_frame);
            }
            
            // Shift+F5: Remove Frame
            if ctx.input(|i| i.key_pressed(egui::Key::F5) && i.modifiers.shift) {
                println!("Shift+F5: Remove frame at {} on layer {:?}", current_frame, layer_id);
                engine.remove_frame(layer_id.clone(), current_frame);
            }
            
            // F6: Insert Keyframe
            if ctx.input(|i| i.key_pressed(egui::Key::F6) && !i.modifiers.shift) {
                println!("F6: Insert keyframe at {} on layer {:?}", current_frame, layer_id);
                engine.insert_keyframe(layer_id.clone(), current_frame);
            }
            
            // Shift+F6: Clear Keyframe
            if ctx.input(|i| i.key_pressed(egui::Key::F6) && i.modifiers.shift) {
                println!("Shift+F6: Clear keyframe at {} on layer {:?}", current_frame, layer_id);
                engine.clear_keyframe(layer_id.clone(), current_frame);
            }
        }
        
        // Spacebar: Play/Pause
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.state.is_playing = !self.state.is_playing;
            if self.state.is_playing {
                engine.play();
            } else {
                engine.pause();
            }
        }
        
        // Home: Go to first frame
        if ctx.input(|i| i.key_pressed(egui::Key::Home)) {
            engine.seek(0);
        }
        
        // End: Go to last frame
        if ctx.input(|i| i.key_pressed(egui::Key::End)) {
            let total_frames = engine.get_total_frames();
            engine.seek(total_frames.saturating_sub(1));
        }
        
        // Left Arrow: Previous frame
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            let current = engine.get_current_frame();
            if current > 0 {
                engine.seek(current - 1);
            }
        }
        
        // Right Arrow: Next frame
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            let current = engine.get_current_frame();
            let total = engine.get_total_frames();
            if current < total - 1 {
                engine.seek(current + 1);
            }
        }
    }
    
    /// Handle the right-click context menu
    fn handle_context_menu(&mut self, ui: &mut Ui, engine: &mut Box<dyn RiveEngine>) {
        if let Some(menu_state) = self.state.context_menu.clone() {
            self.draw_context_menu(ui, &menu_state, engine);
        }
    }
    
    /// Draw the right-click context menu
    fn draw_context_menu(&mut self, ui: &mut Ui, menu_state: &ContextMenuState, engine: &mut Box<dyn RiveEngine>) {
        let mut close_menu = false;
        
        egui::Area::new("frame_context_menu")
            .fixed_pos(menu_state.position)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .show(ui, |ui| {
                        ui.set_min_width(150.0);
                        
                        // Insert Frame (F5)
                        if ui.button("Insert Frame (F5)").clicked() {
                            println!("Insert frame at {} on layer {:?}", menu_state.frame, menu_state.layer_id);
                            engine.insert_frame(menu_state.layer_id.clone(), menu_state.frame);
                            close_menu = true;
                        }
                        
                        // Remove Frame (Shift+F5)
                        if ui.button("Remove Frame (Shift+F5)").clicked() {
                            println!("Remove frame at {} on layer {:?}", menu_state.frame, menu_state.layer_id);
                            engine.remove_frame(menu_state.layer_id.clone(), menu_state.frame);
                            close_menu = true;
                        }
                        
                        ui.separator();
                        
                        // Insert Keyframe (F6)
                        if ui.button("Insert Keyframe (F6)").clicked() {
                            println!("Insert keyframe at {} on layer {:?}", menu_state.frame, menu_state.layer_id);
                            engine.insert_keyframe(menu_state.layer_id.clone(), menu_state.frame);
                            close_menu = true;
                        }
                        
                        // Clear Keyframe (Shift+F6)
                        if ui.button("Clear Keyframe (Shift+F6)").clicked() {
                            println!("Clear keyframe at {} on layer {:?}", menu_state.frame, menu_state.layer_id);
                            engine.clear_keyframe(menu_state.layer_id.clone(), menu_state.frame);
                            close_menu = true;
                        }
                        
                        ui.separator();
                        
                        // Create Motion Tween
                        if ui.button("Create Motion Tween").clicked() {
                            println!("Create motion tween at {} on layer {:?}", menu_state.frame, menu_state.layer_id);
                            engine.create_motion_tween(menu_state.layer_id.clone(), menu_state.frame);
                            close_menu = true;
                        }
                        
                        // Create Shape Tween
                        if ui.button("Create Shape Tween").clicked() {
                            println!("Create shape tween at {} on layer {:?}", menu_state.frame, menu_state.layer_id);
                            engine.create_shape_tween(menu_state.layer_id.clone(), menu_state.frame);
                            close_menu = true;
                        }
                        
                        ui.separator();
                        
                        // Insert Frame Label
                        if ui.button("Insert Frame Label...").clicked() {
                            println!("Insert frame label at {} on layer {:?}", menu_state.frame, menu_state.layer_id);
                            // TODO: Open dialog for frame label
                            close_menu = true;
                        }
                        
                        ui.separator();
                        
                        // Edit Easing - Motion Editor
                        if ui.button("Edit Easing...").clicked() {
                            println!("Opening Motion Editor for frame {} on layer {:?}", menu_state.frame, menu_state.layer_id);
                            self.state.motion_editor.open();
                            close_menu = true;
                        }
                    });
            });
        
        // Close menu if clicked outside or an action was taken
        if close_menu || ui.input(|i| i.pointer.primary_clicked()) {
            self.state.context_menu = None;
        }
    }
}

/// State for the right-click context menu
#[derive(Clone, Debug)]
pub struct ContextMenuState {
    pub position: Pos2,
    pub frame: u32,
    pub layer_id: LayerId,
}

// Implementation methods for Timeline keyframe interaction
impl Timeline {
    /// Handle keyframe click for selection
    fn handle_keyframe_click(&mut self, pos: Pos2, rect: Rect, modifiers: &egui::Modifiers, engine: &Box<dyn RiveEngine>) {
        let frame_width = self.config.frame_width * self.state.zoom_level;
        let clicked_frame = ((pos.x - rect.min.x + self.state.scroll_x) / frame_width) as u32;
        
        // Find which layer was clicked
        let layers = engine.get_layers();
        let mut y_offset = self.state.scroll_y;
        let mut clicked_layer_id = None;
        
        for layer in &layers {
            let layer_height = self.state.track_heights
                .get(&layer.id)
                .copied()
                .unwrap_or(self.config.default_track_height);
            
            if pos.y >= rect.min.y + y_offset && pos.y < rect.min.y + y_offset + layer_height {
                clicked_layer_id = Some(layer.id.clone());
                break;
            }
            y_offset += layer_height;
        }
        
        if let Some(layer_id) = clicked_layer_id {
            let frame_data = engine.get_frame_data(layer_id.clone(), clicked_frame);
            
            // Only handle clicks on keyframes
            if matches!(frame_data.frame_type, crate::frame::FrameType::Keyframe) {
                let keyframe_id = frame_data.id;
                
                // Handle different selection modes
                if modifiers.ctrl || modifiers.command {
                    // Toggle selection with Ctrl/Cmd
                    if self.state.keyframe_selection.is_selected(layer_id.clone(), clicked_frame) {
                        self.state.keyframe_selection.remove(layer_id.clone(), clicked_frame);
                        println!("Deselected keyframe at frame {} on layer {:?}", clicked_frame, layer_id);
                    } else {
                        self.state.keyframe_selection.add(layer_id.clone(), clicked_frame, keyframe_id);
                        println!("Added keyframe to selection at frame {} on layer {:?}", clicked_frame, layer_id);
                    }
                } else if modifiers.shift {
                    // Range selection with Shift (TODO: implement range selection)
                    self.state.keyframe_selection.add(layer_id.clone(), clicked_frame, keyframe_id);
                    println!("Range select keyframe at frame {} on layer {:?}", clicked_frame, layer_id);
                } else {
                    // Single selection
                    self.state.keyframe_selection.clear();
                    self.state.keyframe_selection.add(layer_id.clone(), clicked_frame, keyframe_id);
                    println!("Selected keyframe at frame {} on layer {:?}", clicked_frame, layer_id);
                }
            } else {
                // Clear selection if clicking on empty frame
                if !modifiers.ctrl && !modifiers.command {
                    self.state.keyframe_selection.clear();
                    println!("Cleared keyframe selection");
                }
            }
        }
    }
    
    /// Handle keyframe dragging
    fn handle_keyframe_drag(&mut self, pos: Pos2, _rect: Rect, _delta: egui::Vec2, _engine: &mut Box<dyn RiveEngine>) {
        if self.state.keyframe_selection.selected.is_empty() {
            return;
        }
        
        let frame_width = self.config.frame_width * self.state.zoom_level;
        
        // Initialize drag state if not already dragging
        if self.state.keyframe_selection.drag_state.is_none() {
            let mut original_positions = HashMap::new();
            for ((layer_id, frame), keyframe_id) in self.state.keyframe_selection.selected.iter() {
                original_positions.insert(keyframe_id.clone(), (layer_id.clone(), *frame));
            }
            
            self.state.keyframe_selection.drag_state = Some(DragState {
                original_positions,
                frame_offset: 0,
                start_pos: pos,
            });
        }
        
        if let Some(ref mut drag_state) = self.state.keyframe_selection.drag_state {
            // Calculate frame offset from drag
            let total_delta_x = pos.x - drag_state.start_pos.x;
            let new_frame_offset = (total_delta_x / frame_width).round() as i32;
            
            if new_frame_offset != drag_state.frame_offset {
                drag_state.frame_offset = new_frame_offset;
                println!("Dragging keyframes by {} frames", new_frame_offset);
            }
        }
    }
    
    /// Handle end of keyframe drag operation
    fn handle_keyframe_drag_end(&mut self, engine: &mut Box<dyn RiveEngine>) {
        if let Some(drag_state) = self.state.keyframe_selection.drag_state.take() {
            if drag_state.frame_offset != 0 {
                // Apply the drag operation to move keyframes
                for (keyframe_id, (layer_id, original_frame)) in drag_state.original_positions {
                    let new_frame = (original_frame as i32 + drag_state.frame_offset).max(0) as u32;
                    
                    println!("Moving keyframe {:?} from frame {} to frame {} on layer {:?}", 
                        keyframe_id, original_frame, new_frame, layer_id);
                    
                    // Move the keyframe in the engine
                    engine.move_keyframe(layer_id.clone(), original_frame, new_frame);
                    
                    // Update selection to new position
                    self.state.keyframe_selection.remove(layer_id.clone(), original_frame);
                    self.state.keyframe_selection.add(layer_id, new_frame, keyframe_id);
                }
            }
        }
    }
}