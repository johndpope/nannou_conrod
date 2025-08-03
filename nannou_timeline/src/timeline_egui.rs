//! Complete Flash CS6/Animate IDE-style Timeline implementation using egui
//! Fixed version with proper layout and no crashes

use egui::{*, self};
use crate::{TimelineConfig, RiveEngine, LayerId, KeyframeId, MotionEditor, layer::LayerType};
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
    pub i18n: I18n,
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
    /// Layer panel state
    pub layer_panel_state: LayerPanelState,
    /// Show onion skinning
    pub onion_skinning: bool,
    /// Loop playback
    pub loop_playback: bool,
}

/// State for the enhanced layer panel
#[derive(Clone, Debug, Default)]
pub struct LayerPanelState {
    /// Layer visibility states
    pub layer_visibility: HashMap<LayerId, bool>,
    /// Layer lock states
    pub layer_locked: HashMap<LayerId, bool>,
    /// Layer outline mode
    pub layer_outline: HashMap<LayerId, bool>,
    /// Expanded folders
    pub expanded_folders: Vec<String>,
    /// Layer being dragged for reordering
    pub dragging_layer: Option<(LayerId, f32)>,
    /// Layer being renamed (layer_id, new_name)
    pub renaming_layer: Option<(LayerId, String)>,
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
            layer_panel_state: LayerPanelState::default(),
            onion_skinning: false,
            loop_playback: false,
        }
    }
}

impl Timeline {
    /// Get icon for layer type
    fn get_layer_type_icon(layer_type: LayerType) -> &'static str {
        match layer_type {
            LayerType::Normal => "📄",     // Normal layer - document icon
            LayerType::Folder => "📁",     // Folder layer - folder icon  
            LayerType::Mask => "🎭",       // Mask layer - mask icon
            LayerType::Guide => "📐",      // Guide layer - ruler icon
            LayerType::MotionGuide => "🛤",  // Motion guide layer - railway track icon
            LayerType::Audio => "🔊",      // Audio layer - speaker icon
            LayerType::Video => "🎥",      // Video layer - video camera icon
        }
    }
    
    /// Calculate indentation level for layer based on parent hierarchy
    fn calculate_layer_indent_level(&self, layer: &crate::layer::LayerInfo, all_layers: &[crate::layer::LayerInfo]) -> usize {
        let mut indent_level = 0;
        let mut current_parent = layer.parent_id.clone();
        
        // Walk up the parent chain to count nesting level
        while let Some(parent_id) = current_parent {
            indent_level += 1;
            // Find the parent layer and get its parent
            current_parent = all_layers.iter()
                .find(|l| l.id == parent_id)
                .and_then(|parent_layer| parent_layer.parent_id.clone());
                
            // Prevent infinite loops
            if indent_level > 10 {
                break;
            }
        }
        
        indent_level
    }
    /// Create a new timeline with default configuration
    pub fn new() -> Self {
        Self {
            config: TimelineConfig::default(),
            state: TimelineState::default(),
            i18n: I18n::new("en"),
        }
    }

    /// Create timeline with custom configuration
    pub fn with_config(config: TimelineConfig) -> Self {
        Self {
            config,
            state: TimelineState::default(),
            i18n: I18n::new("en"),
        }
    }
    
    /// Get localized tooltip text
    fn get_tooltip(&self, key: &str) -> String {
        self.i18n.get(key)
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

        // Calculate layout regions
        let toolbar_height = 35.0;
        let controls_height = 40.0;
        let ruler_height = 25.0;
        let layer_panel_width = 250.0; // Wider for Flash-style controls
        
        // Timeline toolbar (top)
        let toolbar_rect = Rect::from_min_size(
            available_rect.min,
            vec2(available_rect.width(), toolbar_height),
        );
        
        // Layer panel (left side, below toolbar)
        let layer_panel_rect = Rect::from_min_size(
            available_rect.min + vec2(0.0, toolbar_height),
            vec2(layer_panel_width, available_rect.height() - toolbar_height - controls_height),
        );

        // Ruler (top of frame area)
        let ruler_rect = Rect::from_min_size(
            available_rect.min + vec2(layer_panel_width, toolbar_height),
            vec2(available_rect.width() - layer_panel_width, ruler_height),
        );

        // Frame grid (main timeline area)
        let frame_grid_rect = Rect::from_min_size(
            available_rect.min + vec2(layer_panel_width, toolbar_height + ruler_height),
            vec2(
                available_rect.width() - layer_panel_width,
                available_rect.height() - toolbar_height - ruler_height - controls_height,
            ),
        );

        // Playback controls (bottom)
        let controls_rect = Rect::from_min_size(
            available_rect.min + vec2(0.0, available_rect.height() - controls_height),
            vec2(available_rect.width(), controls_height),
        );

        // Draw each section
        self.draw_timeline_toolbar(ui, toolbar_rect);
        self.draw_enhanced_layer_panel(ui, layer_panel_rect, engine);
        self.draw_ruler(ui, ruler_rect, engine);
        self.draw_frame_grid_fixed(ui, frame_grid_rect, engine);
        self.draw_enhanced_playback_controls(ui, controls_rect, engine);
        self.draw_playhead(ui, ruler_rect, frame_grid_rect, engine);
        
        // Handle context menu
        self.handle_context_menu(ui, engine);
        
        // Draw snap guides
        self.draw_snap_guides(ui, frame_grid_rect);
        
        // Show Motion Editor if open
        self.state.motion_editor.show(ui.ctx());

        response
    }

    /// Draw the Flash-style timeline toolbar
    fn draw_timeline_toolbar(&mut self, ui: &mut Ui, rect: Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(55));
            
            // Border
            ui.painter().line_segment(
                [rect.left_bottom(), rect.right_bottom()],
                Stroke::new(1.0, Color32::from_gray(80)),
            );
            
            ui.horizontal(|ui| {
                ui.add_space(5.0);
                
                // Frame navigation buttons
                if ui.button("⏮").on_hover_text(self.get_tooltip("timeline.toolbar.first_frame")).clicked() {
                    self.go_to_first_frame(ui.ctx());
                }
                
                if ui.button("◀").on_hover_text(self.get_tooltip("timeline.toolbar.previous_frame")).clicked() {
                    self.go_to_previous_frame(ui.ctx());
                }
                
                if ui.button("▶").on_hover_text(self.get_tooltip("timeline.toolbar.next_frame")).clicked() {
                    self.go_to_next_frame(ui.ctx());
                }
                
                if ui.button("⏭").on_hover_text(self.get_tooltip("timeline.toolbar.last_frame")).clicked() {
                    self.go_to_last_frame(ui.ctx());
                }
                
                ui.separator();
                
                // Onion skinning toggle
                let onion_label = if self.state.onion_skinning { "🧅 On" } else { "🧅 Off" };
                if ui.selectable_label(self.state.onion_skinning, onion_label)
                    .on_hover_text(self.get_tooltip("timeline.toolbar.onion_skinning"))
                    .clicked() 
                {
                    self.state.onion_skinning = !self.state.onion_skinning;
                }
                
                ui.separator();
                
                // Loop toggle
                let loop_label = if self.state.loop_playback { "🔁" } else { "➡️" };
                if ui.selectable_label(self.state.loop_playback, loop_label)
                    .on_hover_text(self.get_tooltip("timeline.toolbar.loop_playback"))
                    .clicked() 
                {
                    self.state.loop_playback = !self.state.loop_playback;
                }
                
                ui.separator();
                
                // Center frame button
                if ui.button("⊡").on_hover_text(self.get_tooltip("timeline.toolbar.center_playhead")).clicked() {
                    self.center_playhead();
                }
                
                ui.separator();
                
                // Edit multiple frames toggle
                if ui.button("📑").on_hover_text(self.get_tooltip("timeline.toolbar.edit_multiple_frames")).clicked() {
                    println!("Edit multiple frames mode");
                }
                
                // Frame-based selection toggle  
                if ui.button("⬚").on_hover_text(self.get_tooltip("timeline.toolbar.frame_selection")).clicked() {
                    println!("Frame-based selection mode");
                }
            });
        });
    }

    /// Draw the enhanced Flash-style layer panel
    fn draw_enhanced_layer_panel(&mut self, ui: &mut Ui, rect: Rect, engine: &mut Box<dyn RiveEngine>) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(45));
            
            // Border
            ui.painter().line_segment(
                [rect.right_top(), rect.right_bottom()],
                Stroke::new(1.0, Color32::from_gray(80)),
            );
            
            // Layer controls at bottom
            let controls_height = 35.0;
            let controls_rect = Rect::from_min_size(
                pos2(rect.min.x, rect.max.y - controls_height),
                vec2(rect.width(), controls_height),
            );
            
            // Layer list area
            let list_rect = Rect::from_min_size(
                rect.min,
                vec2(rect.width(), rect.height() - controls_height),
            );
            
            // Draw layer list with scroll
            ui.scope_builder(UiBuilder::new().max_rect(list_rect), |ui| {
                ScrollArea::vertical()
                    .id_salt("layer_list")
                    .show(ui, |ui| {
                        let layers = engine.get_layers();
                        
                        for (idx, layer) in layers.iter().enumerate() {
                            let _layer_height = self.state.track_heights
                                .get(&layer.id)
                                .copied()
                                .unwrap_or(self.config.default_track_height);
                            
                            let is_selected = self.state.selected_layers.contains(&layer.id);
                            let is_visible = *self.state.layer_panel_state.layer_visibility
                                .get(&layer.id)
                                .unwrap_or(&true);
                            let is_locked = *self.state.layer_panel_state.layer_locked
                                .get(&layer.id)
                                .unwrap_or(&false);
                            let is_outline = *self.state.layer_panel_state.layer_outline
                                .get(&layer.id)
                                .unwrap_or(&false);
                            
                            ui.horizontal(|ui| {
                                // Selection background
                                if is_selected {
                                    let rect = ui.available_rect_before_wrap();
                                    ui.painter().rect_filled(
                                        rect,
                                        0.0,
                                        Color32::from_rgb(70, 130, 180),
                                    );
                                }
                                
                                // Layer type icon
                                let (type_icon, type_tooltip) = match layer.layer_type {
                                    crate::LayerType::Normal => ("🎬", "timeline.layer.type_normal"),
                                    crate::LayerType::Audio => ("🔊", "timeline.layer.type_audio"), 
                                    crate::LayerType::Video => ("🎥", "timeline.layer.type_video"),
                                    crate::LayerType::Folder => ("📁", "timeline.layer.type_folder"),
                                    crate::LayerType::Mask => ("🎭", "timeline.layer.type_mask"),
                                    crate::LayerType::Guide => ("📐", "timeline.layer.type_guide"),
                                    crate::LayerType::MotionGuide => ("🛤", "timeline.layer.type_motion_guide"),
                                };
                                ui.label(type_icon).on_hover_text(self.i18n.get(type_tooltip));
                                
                                // Eye icon (visibility)
                                let eye_icon = if is_visible { "👁" } else { "⚫" };
                                if ui.selectable_label(false, eye_icon)
                                    .on_hover_text(self.i18n.get("timeline.layer.visibility_tooltip"))
                                    .clicked() 
                                {
                                    self.state.layer_panel_state.layer_visibility
                                        .insert(layer.id.clone(), !is_visible);
                                }
                                
                                // Lock icon
                                let lock_icon = if is_locked { "🔒" } else { "🔓" };
                                if ui.selectable_label(false, lock_icon)
                                    .on_hover_text(self.i18n.get("timeline.layer.lock_tooltip"))
                                    .clicked() 
                                {
                                    self.state.layer_panel_state.layer_locked
                                        .insert(layer.id.clone(), !is_locked);
                                }
                                
                                // Outline icon
                                let outline_icon = if is_outline { "⬚" } else { "⬛" };
                                if ui.selectable_label(false, outline_icon)
                                    .on_hover_text(self.get_tooltip("timeline.layer.outline_mode"))
                                    .clicked() 
                                {
                                    self.state.layer_panel_state.layer_outline
                                        .insert(layer.id.clone(), !is_outline);
                                }
                                
                                // Layer name (selectable or editable if renaming)
                                if let Some((renaming_id, ref mut new_name)) = &mut self.state.layer_panel_state.renaming_layer {
                                    if renaming_id == &layer.id {
                                        // Show text edit for renaming
                                        let response = ui.text_edit_singleline(new_name);
                                        
                                        // Handle Enter to confirm or Escape to cancel
                                        if response.lost_focus() {
                                            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                                // Apply the rename through the engine
                                                engine.rename_layer(layer.id.clone(), new_name.clone());
                                                self.state.layer_panel_state.renaming_layer = None;
                                            } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                                // Cancel rename
                                                self.state.layer_panel_state.renaming_layer = None;
                                            }
                                        }
                                        
                                        // Auto-focus the text edit
                                        if response.gained_focus() || !response.has_focus() {
                                            response.request_focus();
                                        }
                                    } else {
                                        // Show normal label for other layers with icon and indentation
                                        let layer_icon = Self::get_layer_type_icon(layer.layer_type);
                                        
                                        // Calculate indentation level based on parent hierarchy
                                        let indent_level = self.calculate_layer_indent_level(&layer, &layers);
                                        let indent = "  ".repeat(indent_level); // Two spaces per level
                                        
                                        let layer_text = format!("{}{} {}", indent, layer_icon, layer.name);
                                        if ui.selectable_label(is_selected, layer_text).clicked() {
                                            if ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                                                // Multi-select with Ctrl/Cmd
                                                if is_selected {
                                                    self.state.selected_layers.retain(|id| id != &layer.id);
                                                } else {
                                                    self.state.selected_layers.push(layer.id.clone());
                                                }
                                            } else {
                                                // Single select
                                                self.state.selected_layers.clear();
                                                self.state.selected_layers.push(layer.id.clone());
                                            }
                                        }
                                    }
                                } else {
                                    // Normal label when not renaming with icon and indentation
                                    let layer_icon = Self::get_layer_type_icon(layer.layer_type);
                                    
                                    // Calculate indentation level based on parent hierarchy
                                    let indent_level = self.calculate_layer_indent_level(&layer, &layers);
                                    let indent = "  ".repeat(indent_level); // Two spaces per level
                                    
                                    let layer_text = format!("{}{} {}", indent, layer_icon, layer.name);
                                    if ui.selectable_label(is_selected, layer_text).clicked() {
                                        if ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                                            // Multi-select with Ctrl/Cmd
                                            if is_selected {
                                                self.state.selected_layers.retain(|id| id != &layer.id);
                                            } else {
                                                self.state.selected_layers.push(layer.id.clone());
                                            }
                                        } else {
                                            // Single select
                                            self.state.selected_layers.clear();
                                            self.state.selected_layers.push(layer.id.clone());
                                        }
                                    }
                                }
                                
                                // Right-click context menu
                                if ui.interact(ui.available_rect_before_wrap(), ui.id().with(("layer", idx)), Sense::click())
                                    .secondary_clicked() 
                                {
                                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                                        self.state.context_menu = Some(ContextMenuState {
                                            position: pos,
                                            menu_type: ContextMenuType::Layer(layer.id.clone()),
                                        });
                                    }
                                }
                            });
                            
                            ui.add_space(2.0);
                        }
                    });
            });
            
            // Draw layer controls
            ui.scope_builder(UiBuilder::new().max_rect(controls_rect), |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                    
                    // Add layer
                    if ui.button("➕").on_hover_text(self.get_tooltip("timeline.layer.new_layer")).clicked() {
                        let layer_id = engine.add_layer("New Layer".to_string(), crate::layer::LayerType::Normal);
                        println!("Added new layer: {:?}", layer_id);
                    }
                    
                    // Add folder
                    if ui.button("📁").on_hover_text(self.get_tooltip("timeline.layer.new_folder")).clicked() {
                        let layer_id = engine.add_folder_layer("New Folder".to_string());
                        println!("Added new folder layer: {:?}", layer_id);
                    }
                    
                    // Delete layer
                    let can_delete = !self.state.selected_layers.is_empty();
                    if ui.add_enabled(can_delete, Button::new("🗑"))
                        .on_hover_text(self.get_tooltip("timeline.layer.delete_layer"))
                        .clicked() 
                    {
                        for layer_id in self.state.selected_layers.clone() {
                            engine.delete_layer(layer_id.clone());
                            println!("Deleted layer: {:?}", layer_id);
                        }
                        self.state.selected_layers.clear();
                    }
                    
                    // Duplicate layer
                    if ui.add_enabled(can_delete, Button::new("📋"))
                        .on_hover_text(self.get_tooltip("timeline.layer.duplicate_layer"))
                        .clicked() 
                    {
                        for layer_id in self.state.selected_layers.clone() {
                            let new_layer_id = engine.duplicate_layer(layer_id.clone());
                            println!("Duplicated layer {:?} to {:?}", layer_id, new_layer_id);
                        }
                    }
                    
                    ui.separator();
                    
                    // Show/hide all layers
                    if ui.button("👁").on_hover_text(self.get_tooltip("timeline.layer.toggle_visibility_all")).clicked() {
                        let all_visible = engine.get_layers().iter()
                            .all(|l| *self.state.layer_panel_state.layer_visibility.get(&l.id).unwrap_or(&true));
                        
                        for layer in engine.get_layers() {
                            self.state.layer_panel_state.layer_visibility
                                .insert(layer.id.clone(), !all_visible);
                        }
                    }
                    
                    // Lock/unlock all layers
                    if ui.button("🔒").on_hover_text(self.get_tooltip("timeline.layer.toggle_lock_all")).clicked() {
                        let all_locked = engine.get_layers().iter()
                            .all(|l| *self.state.layer_panel_state.layer_locked.get(&l.id).unwrap_or(&false));
                        
                        for layer in engine.get_layers() {
                            self.state.layer_panel_state.layer_locked
                                .insert(layer.id.clone(), !all_locked);
                        }
                    }
                });
            });
        });
    }

    /// Draw the frame grid with proper scrolling
    fn draw_frame_grid_fixed(&mut self, ui: &mut Ui, rect: Rect, engine: &mut Box<dyn RiveEngine>) {
        // Properly scope the ScrollArea within the given rect
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            ScrollArea::both()
                .id_salt("timeline_frame_grid")
                .auto_shrink([false, false])
                .scroll_bar_visibility(scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    let layers = engine.get_layers();
                    let frame_width = self.config.frame_width * self.state.zoom_level;
                    let total_frames = engine.get_total_frames();
                    
                    // Calculate content size
                    let total_width = total_frames as f32 * frame_width;
                    let total_height = layers.iter()
                        .map(|l| self.state.track_heights.get(&l.id).copied()
                            .unwrap_or(self.config.default_track_height))
                        .sum::<f32>()
                        .max(rect.height()); // Ensure minimum height
                    
                    // Set the content size for scrolling
                    ui.set_min_size(vec2(total_width, total_height));
                    
                    // Get viewport for optimization
                    let viewport = ui.clip_rect();
                    
                    // Update scroll state
                    self.state.scroll_x = viewport.min.x;
                    self.state.scroll_y = viewport.min.y;
                    
                    // Calculate visible range
                    let visible_start_frame = ((viewport.min.x / frame_width).floor() as u32).saturating_sub(1);
                    let visible_end_frame = ((viewport.max.x / frame_width).ceil() as u32 + 1).min(total_frames);
                    
                    // Draw background
                    ui.painter().rect_filled(
                        Rect::from_min_size(pos2(0.0, 0.0), vec2(total_width, total_height)),
                        0.0,
                        self.config.style.background_color,
                    );
                    
                    // Draw vertical grid lines
                    for frame in visible_start_frame..=visible_end_frame {
                        let x = frame as f32 * frame_width;
                        
                        let color = if frame % 5 == 0 {
                            self.config.style.grid_color
                        } else {
                            self.config.style.grid_color.gamma_multiply(0.5)
                        };
                        
                        ui.painter().line_segment(
                            [pos2(x, 0.0), pos2(x, total_height)],
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
                        
                        // Skip layers outside visible area
                        if y_offset > viewport.max.y || y_offset + layer_height < viewport.min.y {
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
                        
                        // Draw horizontal grid line
                        ui.painter().line_segment(
                            [pos2(0.0, y_offset + layer_height), pos2(total_width, y_offset + layer_height)],
                            Stroke::new(1.0, self.config.style.grid_color.gamma_multiply(0.3)),
                        );
                        
                        // Check if layer is visible
                        let is_visible = *self.state.layer_panel_state.layer_visibility
                            .get(&layer.id)
                            .unwrap_or(&true);
                        
                        if is_visible {
                            // Draw frames based on layer type
                            if matches!(layer.layer_type, crate::LayerType::Audio) {
                                self.draw_audio_waveform(ui, layer, y_offset, layer_height, visible_start_frame..=visible_end_frame, frame_width);
                            } else if matches!(layer.layer_type, crate::LayerType::Video) {
                                self.draw_video_thumbnails(ui, layer, y_offset, layer_height, visible_start_frame..=visible_end_frame, frame_width);
                            } else {
                                // Draw regular frames
                                for frame in visible_start_frame..=visible_end_frame {
                                    let frame_data = engine.get_frame_data(layer.id.clone(), frame);
                                    let x = frame as f32 * frame_width;
                                    
                                    let frame_rect = Rect::from_min_size(
                                        pos2(x, y_offset),
                                        vec2(frame_width - 1.0, layer_height - 1.0),
                                    );
                                    
                                    // Interactive frame area
                                    let frame_response = ui.interact(frame_rect, ui.id().with(("frame", layer.id.clone(), frame)), Sense::click());
                                    
                                    // Handle double-click to edit frame content (check first to avoid borrow issues)
                                    let was_double_clicked = frame_response.double_clicked();
                                    
                                    // Draw frame based on type
                                    match frame_data.frame_type {
                                        crate::frame::FrameType::Empty => {
                                            // Empty frames - no fill
                                            if frame_response.hovered() {
                                                frame_response.on_hover_text(self.i18n.get("timeline.tooltips.frame_empty"));
                                            }
                                        }
                                        crate::frame::FrameType::Keyframe => {
                                            // Keyframe
                                            ui.painter().rect_filled(
                                                frame_rect,
                                                2.0,
                                                self.config.style.frame_keyframe,
                                            );
                                            
                                            // Draw keyframe indicator
                                            let is_selected = self.state.keyframe_selection.is_selected(layer.id.clone(), frame);
                                            
                                            if is_selected {
                                                // Draw selection border with line segments (egui 0.32 workaround)
                                                let selection_rect = frame_rect.expand(1.0);
                                                let selection_stroke = Stroke::new(2.0, Color32::from_rgb(100, 150, 255));
                                                ui.painter().line_segment([selection_rect.left_top(), selection_rect.right_top()], selection_stroke);
                                                ui.painter().line_segment([selection_rect.right_top(), selection_rect.right_bottom()], selection_stroke);
                                                ui.painter().line_segment([selection_rect.right_bottom(), selection_rect.left_bottom()], selection_stroke);
                                                ui.painter().line_segment([selection_rect.left_bottom(), selection_rect.left_top()], selection_stroke);
                                            }
                                            
                                            ui.painter().circle_filled(
                                                frame_rect.center(),
                                                3.0,
                                                if is_selected {
                                                    Color32::from_rgb(100, 150, 255)
                                                } else {
                                                    self.config.style.text_color
                                                },
                                            );
                                            
                                            if frame_response.hovered() {
                                                frame_response.on_hover_text(self.i18n.get("timeline.tooltips.frame_keyframe"));
                                            }
                                        }
                                        crate::frame::FrameType::Tween => {
                                            // Tween frame
                                            ui.painter().rect_filled(
                                                frame_rect,
                                                2.0,
                                                self.config.style.frame_tween,
                                            );
                                            
                                            // Draw tween arrow
                                            let arrow_start = frame_rect.left_center() + vec2(5.0, 0.0);
                                            let arrow_end = frame_rect.right_center() - vec2(5.0, 0.0);
                                            ui.painter().arrow(
                                                arrow_start,
                                                arrow_end - arrow_start,
                                                Stroke::new(1.0, self.config.style.text_color),
                                            );
                                            
                                            if frame_response.hovered() {
                                                frame_response.on_hover_text(self.i18n.get("timeline.tooltips.frame_tween"));
                                            }
                                        }
                                    }
                                    
                                    // Process double-click if it occurred
                                    if was_double_clicked {
                                        // TODO: Open frame content editor
                                        // For now, just clear and add a keyframe as a placeholder
                                        match frame_data.frame_type {
                                            crate::frame::FrameType::Empty => {
                                                engine.insert_keyframe(layer.id.clone(), frame);
                                            }
                                            crate::frame::FrameType::Keyframe => {
                                                // Open content editor in the future
                                                // For now, show in logs
                                                println!("Double-clicked keyframe at layer {} frame {}", layer.name, frame);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        
                        y_offset += layer_height;
                    }
                });
        });
        
        // Handle interactions outside the scroll area
        let response = ui.interact(rect, ui.id().with("frame_grid_interact"), Sense::click_and_drag());
        
        // Handle mouse interactions
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let modifiers = ui.input(|i| i.modifiers);
                self.handle_frame_click(pos, rect, &modifiers, engine);
            }
        }
        
        // Handle right-click for context menu
        if response.secondary_clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.handle_frame_right_click(pos, rect, engine);
            }
        }
    }

    /// Draw enhanced playback controls
    fn draw_enhanced_playback_controls(&mut self, ui: &mut Ui, rect: Rect, engine: &mut Box<dyn RiveEngine>) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(50));
            
            // Border
            ui.painter().line_segment(
                [rect.left_top(), rect.right_top()],
                Stroke::new(1.0, Color32::from_gray(80)),
            );
            
            ui.horizontal_centered(|ui| {
                ui.add_space(10.0);
                
                // Playback controls
                if ui.button("⏹").on_hover_text(self.get_tooltip("timeline.controls.stop")).clicked() {
                    self.state.is_playing = false;
                    self.state.playhead_frame = 0;
                    engine.seek(0);
                }
                
                let play_text = if self.state.is_playing { "⏸" } else { "▶" };
                if ui.button(play_text).on_hover_text(self.get_tooltip("timeline.controls.play_pause")).clicked() {
                    self.state.is_playing = !self.state.is_playing;
                    if self.state.is_playing {
                        engine.play();
                    } else {
                        engine.pause();
                    }
                }
                
                ui.separator();
                
                // Frame counter
                ui.label("Frame:");
                let mut frame_text = engine.get_current_frame().to_string();
                let response = ui.add(
                    TextEdit::singleline(&mut frame_text)
                        .desired_width(50.0)
                        .horizontal_align(Align::Center)
                ).on_hover_text("Enter frame number to jump to");
                if response.changed() {
                    if let Ok(frame) = frame_text.parse::<u32>() {
                        engine.seek(frame.min(engine.get_total_frames() - 1));
                    }
                }
                ui.label(format!("/ {}", engine.get_total_frames()))
                    .on_hover_text("Total frames in timeline");
                
                ui.separator();
                
                // FPS selector
                ui.label("FPS:");
                ComboBox::from_id_salt("fps_selector")
                    .selected_text(self.config.fps.label())
                    .show_ui(ui, |ui| {
                        for preset in crate::FpsPreset::all_presets() {
                            if ui.selectable_value(&mut self.config.fps, preset, preset.label()).clicked() {
                                println!("FPS changed to: {}", preset.to_fps());
                            }
                        }
                        
                        ui.separator();
                        
                        if ui.button("Custom...").clicked() {
                            println!("Custom FPS dialog");
                        }
                    });
                
                ui.separator();
                
                // Zoom controls
                ui.label("Zoom:");
                if ui.button("−").clicked() {
                    self.state.zoom_level = (self.state.zoom_level * 0.8).max(0.1);
                }
                
                let mut zoom_text = format!("{:.0}%", self.state.zoom_level * 100.0);
                if ui.add(
                    TextEdit::singleline(&mut zoom_text)
                        .desired_width(50.0)
                        .horizontal_align(Align::Center)
                ).changed() {
                    if let Ok(percent) = zoom_text.trim_end_matches('%').parse::<f32>() {
                        self.state.zoom_level = (percent / 100.0).clamp(0.1, 5.0);
                    }
                }
                
                if ui.button("+").clicked() {
                    self.state.zoom_level = (self.state.zoom_level * 1.2).min(5.0);
                }
                
                // Zoom slider
                ui.add(
                    Slider::new(&mut self.state.zoom_level, 0.1..=5.0)
                        .show_value(false)
                        .clamping(egui::SliderClamping::Always)
                );
                
                ui.separator();
                
                // Snap controls
                let snap_icon = if self.config.snap.enabled { "🧲" } else { "⚫" };
                if ui.selectable_label(self.config.snap.enabled, format!("{} Snap", snap_icon)).clicked() {
                    self.config.snap.enabled = !self.config.snap.enabled;
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
                        ui.add(Slider::new(&mut self.config.snap.threshold_pixels, 1.0..=20.0)
                            .suffix(" px"));
                    });
                }
                
                ui.add_space(10.0);
            });
        });
    }

    /// Draw the ruler at the top
    fn draw_ruler(&mut self, ui: &mut Ui, rect: Rect, engine: &Box<dyn RiveEngine>) {
        let ruler = crate::Ruler::new();
        let total_frames = engine.get_total_frames();
        let frame_width = self.config.frame_width * self.state.zoom_level;
        let fps = engine.get_fps();
        
        ruler.draw_with_fps_and_comments(
            ui,
            rect,
            0,
            total_frames,
            frame_width,
            self.state.scroll_x,
            &self.config.frame_labels,
            &self.config.frame_comments,
            fps,
        );
    }

    /// Draw the playhead
    fn draw_playhead(&mut self, ui: &mut Ui, ruler_rect: Rect, grid_rect: Rect, engine: &mut Box<dyn RiveEngine>) {
        let current_frame = engine.get_current_frame();
        let frame_width = self.config.frame_width * self.state.zoom_level;
        let x = ruler_rect.min.x + (current_frame as f32 * frame_width) - self.state.scroll_x;

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

        // Handle playhead dragging
        if ui.input(|i| i.pointer.primary_down()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if ruler_rect.contains(pos) {
                    let raw_x = pos.x - ruler_rect.min.x + self.state.scroll_x;
                    let modifiers = ui.input(|i| i.modifiers);
                    let snapped_x = self.snap_position(raw_x, &modifiers);
                    let frame = (snapped_x / frame_width).round() as u32;
                    engine.seek(frame.min(engine.get_total_frames() - 1));
                }
            }
        }
    }
    
    /// Draw snap guides
    fn draw_snap_guides(&self, ui: &mut Ui, grid_rect: Rect) {
        if !self.config.snap.show_guides {
            return;
        }
        
        for &guide_x in &self.state.snap_guides {
            let x = grid_rect.min.x + guide_x - self.state.scroll_x;
            
            if x >= grid_rect.min.x && x <= grid_rect.max.x {
                ui.painter().line_segment(
                    [pos2(x, grid_rect.min.y), pos2(x, grid_rect.max.y)],
                    Stroke::new(1.0, Color32::from_rgb(255, 255, 0)),
                );
                
                ui.painter().circle_filled(
                    pos2(x, grid_rect.min.y + 3.0),
                    2.0,
                    Color32::from_rgb(255, 255, 0),
                );
            }
        }
    }
    
    /// Handle keyboard shortcuts
    fn handle_keyboard_shortcuts(&mut self, ui: &mut Ui, engine: &mut Box<dyn RiveEngine>) {
        let ctx = ui.ctx();
        
        // Spacebar: Play/Pause
        if ctx.input(|i| i.key_pressed(Key::Space)) {
            self.state.is_playing = !self.state.is_playing;
            if self.state.is_playing {
                engine.play();
            } else {
                engine.pause();
            }
        }
        
        // Home: First frame
        if ctx.input(|i| i.key_pressed(Key::Home)) {
            engine.seek(0);
        }
        
        // End: Last frame
        if ctx.input(|i| i.key_pressed(Key::End)) {
            engine.seek(engine.get_total_frames().saturating_sub(1));
        }
        
        // Left/Right arrows for frame navigation
        if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
            let current = engine.get_current_frame();
            if current > 0 {
                engine.seek(current - 1);
            }
        }
        
        if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
            let current = engine.get_current_frame();
            if current < engine.get_total_frames() - 1 {
                engine.seek(current + 1);
            }
        }
        
        // Frame operations (if layer selected)
        if let Some(layer_id) = self.state.selected_layers.first() {
            let current_frame = engine.get_current_frame();
            
            // F5: Insert Frame
            if ctx.input(|i| i.key_pressed(Key::F5) && !i.modifiers.shift) {
                engine.insert_frame(layer_id.clone(), current_frame);
            }
            
            // Shift+F5: Remove Frame
            if ctx.input(|i| i.key_pressed(Key::F5) && i.modifiers.shift) {
                engine.remove_frame(layer_id.clone(), current_frame);
            }
            
            // F6: Insert Keyframe
            if ctx.input(|i| i.key_pressed(Key::F6) && !i.modifiers.shift) {
                engine.insert_keyframe(layer_id.clone(), current_frame);
            }
            
            // Shift+F6: Clear Keyframe
            if ctx.input(|i| i.key_pressed(Key::F6) && i.modifiers.shift) {
                engine.clear_keyframe(layer_id.clone(), current_frame);
            }
        }
    }
    
    /// Handle frame click
    fn handle_frame_click(&mut self, pos: Pos2, rect: Rect, modifiers: &Modifiers, engine: &Box<dyn RiveEngine>) {
        let frame_width = self.config.frame_width * self.state.zoom_level;
        let clicked_frame = ((pos.x - rect.min.x + self.state.scroll_x) / frame_width) as u32;
        
        // Find which layer was clicked
        let layers = engine.get_layers();
        let mut y_offset = self.state.scroll_y;
        
        for layer in &layers {
            let layer_height = self.state.track_heights
                .get(&layer.id)
                .copied()
                .unwrap_or(self.config.default_track_height);
            
            if pos.y >= rect.min.y + y_offset && pos.y < rect.min.y + y_offset + layer_height {
                let frame_data = engine.get_frame_data(layer.id.clone(), clicked_frame);
                
                if matches!(frame_data.frame_type, crate::frame::FrameType::Keyframe) {
                    let keyframe_id = frame_data.id;
                    
                    if modifiers.ctrl || modifiers.command {
                        // Toggle selection
                        if self.state.keyframe_selection.is_selected(layer.id.clone(), clicked_frame) {
                            self.state.keyframe_selection.remove(layer.id.clone(), clicked_frame);
                        } else {
                            self.state.keyframe_selection.add(layer.id.clone(), clicked_frame, keyframe_id);
                        }
                    } else {
                        // Single selection
                        self.state.keyframe_selection.clear();
                        self.state.keyframe_selection.add(layer.id.clone(), clicked_frame, keyframe_id);
                    }
                } else if !modifiers.ctrl && !modifiers.command {
                    self.state.keyframe_selection.clear();
                }
                break;
            }
            y_offset += layer_height;
        }
    }
    
    /// Handle frame right-click
    fn handle_frame_right_click(&mut self, pos: Pos2, rect: Rect, engine: &Box<dyn RiveEngine>) {
        let frame_width = self.config.frame_width * self.state.zoom_level;
        let clicked_frame = ((pos.x - rect.min.x + self.state.scroll_x) / frame_width) as u32;
        
        // Find which layer was clicked
        let layers = engine.get_layers();
        let mut y_offset = self.state.scroll_y;
        
        for layer in &layers {
            let layer_height = self.state.track_heights
                .get(&layer.id)
                .copied()
                .unwrap_or(self.config.default_track_height);
            
            if pos.y >= rect.min.y + y_offset && pos.y < rect.min.y + y_offset + layer_height {
                self.state.context_menu = Some(ContextMenuState {
                    position: pos,
                    menu_type: ContextMenuType::Frame {
                        layer_id: layer.id.clone(),
                        frame: clicked_frame,
                    },
                });
                break;
            }
            y_offset += layer_height;
        }
    }
    
    /// Handle context menu
    fn handle_context_menu(&mut self, ui: &mut Ui, engine: &mut Box<dyn RiveEngine>) {
        if let Some(menu_state) = self.state.context_menu.clone() {
            let mut close_menu = false;
            
            Area::new(ui.id().with("timeline_context_menu"))
                .fixed_pos(menu_state.position)
                .order(Order::Foreground)
                .show(ui.ctx(), |ui| {
                    Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_min_width(180.0);
                        
                        match &menu_state.menu_type {
                            ContextMenuType::Layer(layer_id) => {
                                ui.label("Layer Options");
                                ui.separator();
                                
                                if ui.button("➕ Insert Layer Above").clicked() {
                                    println!("Insert layer above {:?}", layer_id);
                                    close_menu = true;
                                }
                                
                                if ui.button("➕ Insert Layer Below").clicked() {
                                    println!("Insert layer below {:?}", layer_id);
                                    close_menu = true;
                                }
                                
                                if ui.button("📁 Insert Folder").clicked() {
                                    let layer_id = engine.add_folder_layer("New Folder".to_string());
                                    println!("Added new folder layer: {:?}", layer_id);
                                    close_menu = true;
                                }
                                
                                ui.separator();
                                
                                if ui.button("📋 Duplicate Layer").clicked() {
                                    let new_layer_id = engine.duplicate_layer(layer_id.clone());
                                    println!("Duplicated layer {:?} to {:?}", layer_id, new_layer_id);
                                    close_menu = true;
                                }
                                
                                if ui.button("🗑 Delete Layer").clicked() {
                                    engine.delete_layer(layer_id.clone());
                                    println!("Deleted layer: {:?}", layer_id);
                                    close_menu = true;
                                }
                                
                                ui.separator();
                                
                                if ui.button("✏️ Rename Layer...").clicked() {
                                    // Start renaming - find the layer name
                                    if let Some(layer) = engine.get_layers().iter().find(|l| &l.id == layer_id) {
                                        self.state.layer_panel_state.renaming_layer = Some((layer_id.clone(), layer.name.clone()));
                                    }
                                    close_menu = true;
                                }
                                
                                if ui.button("🎨 Layer Properties...").clicked() {
                                    println!("Layer properties {:?}", layer_id);
                                    close_menu = true;
                                }
                                
                                ui.separator();
                                
                                if ui.button("🎭 Mask").clicked() {
                                    println!("Convert to mask {:?}", layer_id);
                                    close_menu = true;
                                }
                                
                                if ui.button("📐 Guide").clicked() {
                                    println!("Convert to guide {:?}", layer_id);
                                    close_menu = true;
                                }
                                
                                if ui.button("🛤 Add Motion Guide").clicked() {
                                    let guide_id = engine.add_motion_guide_layer("Motion Guide".to_string());
                                    println!("Added motion guide layer: {:?}", guide_id);
                                    close_menu = true;
                                }
                                
                                ui.separator();
                                
                                if ui.button("📋 Select All Frames").clicked() {
                                    // Select all keyframes in this layer
                                    self.state.keyframe_selection.clear();
                                    let total_frames = engine.get_total_frames();
                                    for frame in 0..=total_frames {
                                        let frame_data = engine.get_frame_data(layer_id.clone(), frame);
                                        if matches!(frame_data.frame_type, crate::frame::FrameType::Keyframe) {
                                            self.state.keyframe_selection.add(layer_id.clone(), frame, frame_data.id);
                                        }
                                    }
                                    close_menu = true;
                                }
                            }
                            ContextMenuType::Frame { layer_id, frame } => {
                                let frame_data = engine.get_frame_data(layer_id.clone(), *frame);
                                let is_keyframe = matches!(frame_data.frame_type, crate::frame::FrameType::Keyframe);
                                
                                ui.label(format!("Frame {}", frame));
                                ui.separator();
                                
                                if ui.button("⬜ Insert Frame (F5)").clicked() {
                                    engine.insert_frame(layer_id.clone(), *frame);
                                    close_menu = true;
                                }
                                
                                if ui.button("❌ Remove Frame (Shift+F5)").clicked() {
                                    engine.remove_frame(layer_id.clone(), *frame);
                                    close_menu = true;
                                }
                                
                                ui.separator();
                                
                                if is_keyframe {
                                    if ui.button("🚫 Clear Keyframe (Shift+F6)").clicked() {
                                        engine.clear_keyframe(layer_id.clone(), *frame);
                                        close_menu = true;
                                    }
                                } else {
                                    if ui.button("🔑 Insert Keyframe (F6)").clicked() {
                                        engine.insert_keyframe(layer_id.clone(), *frame);
                                        close_menu = true;
                                    }
                                    
                                    if ui.button("⬜ Insert Blank Keyframe (F7)").clicked() {
                                        engine.insert_keyframe(layer_id.clone(), *frame);
                                        close_menu = true;
                                    }
                                    
                                    // Convert to Keyframe (only for non-keyframe types)
                                    let frame_data = engine.get_frame_data(layer_id.clone(), *frame);
                                    if !matches!(frame_data.frame_type, crate::frame::FrameType::Keyframe) {
                                        if ui.button("🔄 Convert to Keyframe").clicked() {
                                            // Clear existing frame first, then insert keyframe
                                            engine.remove_frame(layer_id.clone(), *frame);
                                            engine.insert_keyframe(layer_id.clone(), *frame);
                                            close_menu = true;
                                        }
                                    }
                                }
                                
                                ui.separator();
                                
                                if ui.button("➡️ Create Motion Tween").clicked() {
                                    engine.create_motion_tween(layer_id.clone(), *frame);
                                    close_menu = true;
                                }
                                
                                if ui.button("🔄 Create Shape Tween").clicked() {
                                    engine.create_shape_tween(layer_id.clone(), *frame);
                                    close_menu = true;
                                }
                                
                                if ui.button("🔧 Create Classic Tween").clicked() {
                                    println!("Create classic tween at frame {}", frame);
                                    close_menu = true;
                                }
                                
                                ui.separator();
                                
                                if ui.button("📋 Copy Frames").clicked() {
                                    println!("Copy frames from {}", frame);
                                    close_menu = true;
                                }
                                
                                if ui.button("📄 Paste Frames").clicked() {
                                    println!("Paste frames at {}", frame);
                                    close_menu = true;
                                }
                                
                                if ui.button("🚮 Clear Frames").clicked() {
                                    println!("Clear frames at {}", frame);
                                    close_menu = true;
                                }
                                
                                ui.separator();
                                
                                if ui.button("🏷 Insert Frame Label...").clicked() {
                                    println!("Insert frame label at {}", frame);
                                    close_menu = true;
                                }
                                
                                if ui.button("📈 Edit Easing...").clicked() {
                                    self.state.motion_editor.open();
                                    close_menu = true;
                                }
                            }
                        }
                    });
                });
            
            // Close menu if clicked outside or action taken
            if close_menu || ui.input(|i| i.pointer.primary_clicked()) {
                self.state.context_menu = None;
            }
        }
    }
    
    /// Draw audio waveform
    fn draw_audio_waveform(&self, ui: &mut Ui, layer: &crate::layer::LayerInfo, y_offset: f32, layer_height: f32, frame_range: std::ops::RangeInclusive<u32>, frame_width: f32) {
        let waveform_color = Color32::from_rgb(100, 200, 255);
        let center_y = y_offset + layer_height / 2.0;
        let amplitude_scale = layer_height * 0.4;
        
        // Draw background
        let layer_rect = Rect::from_min_size(
            pos2(*frame_range.start() as f32 * frame_width, y_offset),
            vec2((*frame_range.end() - *frame_range.start()) as f32 * frame_width, layer_height),
        );
        ui.painter().rect_filled(layer_rect, 2.0, Color32::from_gray(35));
        
        // Generate mock waveform
        let mut waveform_points = Vec::new();
        let sample_count = ((*frame_range.end() - *frame_range.start()) as f32 * frame_width / 2.0) as usize;
        
        for i in 0..sample_count {
            let x = *frame_range.start() as f32 * frame_width + (i as f32 * 2.0);
            let time = i as f32 * 0.1;
            
            let base_frequency = if layer.name.contains("Music") { 220.0 } else { 440.0 };
            let envelope = 1.0 - (time * 0.05).min(1.0);
            let signal = envelope * (
                0.6 * (2.0 * std::f32::consts::PI * base_frequency * time * 0.01).sin() +
                0.3 * (2.0 * std::f32::consts::PI * base_frequency * 2.0 * time * 0.01).sin() +
                0.1 * (time * 17.3).sin()
            );
            
            let amplitude = signal.abs() * amplitude_scale;
            let y_min = center_y - amplitude;
            let y_max = center_y + amplitude;
            
            waveform_points.push((pos2(x, y_min), pos2(x, y_max)));
        }
        
        // Draw waveform
        for (top, bottom) in waveform_points {
            ui.painter().line_segment([top, bottom], Stroke::new(1.0, waveform_color));
        }
        
        // Draw center line
        ui.painter().line_segment(
            [pos2(*frame_range.start() as f32 * frame_width, center_y), 
             pos2(*frame_range.end() as f32 * frame_width, center_y)],
            Stroke::new(0.5, waveform_color.gamma_multiply(0.3)),
        );
        
        // Draw audio label with high contrast color
        if layer_rect.width() > 100.0 {
            ui.painter().text(
                layer_rect.min + vec2(5.0, 5.0),
                Align2::LEFT_TOP,
                format!("♪ {}", layer.name),
                FontId::monospace(12.0),
                Color32::WHITE, // Changed from waveform_color to white for better contrast
            );
        }
    }
    
    /// Draw video thumbnails for a video layer
    fn draw_video_thumbnails(&self, ui: &mut Ui, layer: &crate::layer::LayerInfo, y_offset: f32, layer_height: f32, frame_range: std::ops::RangeInclusive<u32>, frame_width: f32) {
        // Background with film strip appearance
        let layer_rect = Rect::from_min_size(
            pos2(*frame_range.start() as f32 * frame_width, y_offset),
            vec2((*frame_range.end() - *frame_range.start()) as f32 * frame_width, layer_height),
        );
        ui.painter().rect_filled(layer_rect, 2.0, Color32::from_gray(25));
        
        // Film strip perforations (top and bottom edges)
        let perforation_color = Color32::from_gray(15);
        let perforation_size = 3.0;
        let perforation_spacing = 8.0;
        
        for i in 0..((layer_rect.width() / perforation_spacing) as i32) {
            let x = layer_rect.min.x + (i as f32) * perforation_spacing;
            // Top perforations
            ui.painter().circle_filled(
                pos2(x, y_offset + 3.0),
                perforation_size / 2.0,
                perforation_color,
            );
            // Bottom perforations
            ui.painter().circle_filled(
                pos2(x, y_offset + layer_height - 3.0),
                perforation_size / 2.0,
                perforation_color,
            );
        }
        
        // Calculate thumbnail dimensions
        let thumb_height = layer_height - 12.0; // Leave space for perforations
        let thumb_width = thumb_height * (16.0 / 9.0); // Assume 16:9 aspect ratio
        
        // Draw video thumbnails
        let mut current_x = *frame_range.start() as f32 * frame_width;
        let thumb_y = y_offset + 6.0;
        
        while current_x < (*frame_range.end() as f32 * frame_width) {
            let thumb_rect = Rect::from_min_size(
                pos2(current_x, thumb_y),
                vec2(thumb_width.min(frame_width), thumb_height),
            );
            
            // Draw thumbnail placeholder with gradient
            let gradient_start = Color32::from_rgb(80, 80, 120);
            let gradient_end = Color32::from_rgb(40, 40, 80);
            
            // Simple gradient simulation
            ui.painter().rect_filled(thumb_rect, 1.0, gradient_start);
            ui.painter().rect_filled(
                Rect::from_min_size(
                    thumb_rect.min + vec2(0.0, thumb_rect.height() * 0.7),
                    vec2(thumb_rect.width(), thumb_rect.height() * 0.3),
                ),
                0.0,
                gradient_end,
            );
            
            // Mock thumbnail content - simple geometric pattern
            let center = thumb_rect.center();
            let time_factor = (current_x / frame_width * 0.1) % 1.0;
            
            // Rotating square pattern
            let square_size = thumb_rect.height() * 0.3;
            let angle = time_factor * 2.0 * std::f32::consts::PI;
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            
            let square_points = [
                pos2(-square_size / 2.0, -square_size / 2.0),
                pos2(square_size / 2.0, -square_size / 2.0),
                pos2(square_size / 2.0, square_size / 2.0),
                pos2(-square_size / 2.0, square_size / 2.0),
            ];
            
            let rotated_points: Vec<Pos2> = square_points
                .iter()
                .map(|p| {
                    let x = p.x * cos_a - p.y * sin_a;
                    let y = p.x * sin_a + p.y * cos_a;
                    center + vec2(x, y)
                })
                .collect();
            
            // Draw the rotated square
            if rotated_points.len() >= 4 {
                ui.painter().add(Shape::convex_polygon(
                    rotated_points,
                    Color32::from_rgb(255, 200, 100),
                    Stroke::new(1.0, Color32::from_rgb(255, 255, 255)),
                ));
            }
            
            // Draw thumbnail border
            ui.painter().rect_stroke(thumb_rect, 1.0, Stroke::new(1.0, Color32::from_gray(60)), egui::epaint::StrokeKind::Outside);
            
            current_x += thumb_width;
        }
        
        // Draw video timeline overlay info
        if layer_rect.width() > 150.0 {
            ui.painter().text(
                layer_rect.min + vec2(5.0, layer_height / 2.0 - 6.0),
                Align2::LEFT_CENTER,
                format!("🎥 {}", layer.name),
                FontId::monospace(11.0),
                Color32::from_rgb(255, 255, 255),
            );
            
            // Show video duration/info if space allows
            if layer_rect.width() > 250.0 {
                ui.painter().text(
                    layer_rect.min + vec2(5.0, layer_height / 2.0 + 6.0),
                    Align2::LEFT_CENTER,
                    "30fps • 1920x1080",
                    FontId::monospace(9.0),
                    Color32::from_rgb(180, 180, 180),
                );
            }
        }
    }
    
    /// Snap position to grid
    fn snap_position(&self, pos: f32, modifiers: &Modifiers) -> f32 {
        if modifiers.shift || !self.config.snap.enabled {
            return pos;
        }
        
        let frame_width = self.config.frame_width * self.state.zoom_level;
        let frame_pos = pos / frame_width;
        
        if self.config.snap.snap_to_frames {
            let nearest_frame = frame_pos.round();
            let snapped_pos = nearest_frame * frame_width;
            
            if (pos - snapped_pos).abs() < self.config.snap.threshold_pixels {
                return snapped_pos;
            }
        }
        
        pos
    }
    
    // Navigation helpers
    fn go_to_first_frame(&mut self, _ctx: &Context) {
        // Will be implemented with engine
        println!("Go to first frame");
    }
    
    fn go_to_previous_frame(&mut self, _ctx: &Context) {
        // Will be implemented with engine
        println!("Go to previous frame");
    }
    
    fn go_to_next_frame(&mut self, _ctx: &Context) {
        // Will be implemented with engine
        println!("Go to next frame");
    }
    
    fn go_to_last_frame(&mut self, _ctx: &Context) {
        // Will be implemented with engine
        println!("Go to last frame");
    }
    
    fn center_playhead(&mut self) {
        // Center the playhead in the visible area
        let _frame_width = self.config.frame_width * self.state.zoom_level;
        // This would need the visible width from the UI
        println!("Center playhead");
    }
}

/// State for the right-click context menu
#[derive(Clone, Debug)]
pub struct ContextMenuState {
    pub position: Pos2,
    pub menu_type: ContextMenuType,
}

/// Type of context menu to show
#[derive(Clone, Debug)]
pub enum ContextMenuType {
    Layer(LayerId),
    Frame { layer_id: LayerId, frame: u32 },
}

// Re-export the original implementation's audio waveform and other methods
// use crate::timeline_egui::Timeline as OriginalTimeline; // Unused import

/// Internationalization support for timeline UI
#[derive(Clone, Debug)]
pub struct I18n {
    language: String,
    translations: HashMap<String, String>,
}

impl I18n {
    /// Create new i18n instance with specified language
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
            translations: crate::i18n::load_translations(language),
        }
    }
    
    /// Get translated string for key
    pub fn get(&self, key: &str) -> String {
        self.translations.get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
    
    /// Change language at runtime
    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
        self.translations = crate::i18n::load_translations(language);
    }
}