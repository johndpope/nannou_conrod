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
    /// Show label management panel
    pub show_label_panel: bool,
    /// Frame range selection (start_frame, end_frame)
    pub frame_range_selection: Option<(u32, u32)>,
    /// Frame range selection mode enabled
    pub frame_range_mode: bool,
    /// Onion skin settings
    pub onion_skin_frames_before: u32,
    pub onion_skin_frames_after: u32,
    pub onion_skin_opacity: f32,
    /// Show onion skin settings panel
    pub show_onion_settings: bool,
    /// Onion skin outline mode
    pub onion_skin_outline_mode: bool,
    /// Currently scrubbing the timeline
    pub is_scrubbing: bool,
    /// Was playing before scrubbing started
    pub was_playing: bool,
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
    /// Expanded folders (track by LayerId)
    pub expanded_folders: Vec<LayerId>,
    /// Layer being dragged for reordering (layer_id, initial_y_position)
    pub dragging_layer: Option<(LayerId, f32)>,
    /// Drop target position during drag
    pub drop_target_index: Option<usize>,
    /// Layer being renamed (layer_id, new_name)
    pub renaming_layer: Option<(LayerId, String)>,
    /// Newly created layer that should be focused and renamed
    pub newly_created_layer: Option<LayerId>,
}

impl Default for TimelineState {
    fn default() -> Self {
        let mut layer_panel_state = LayerPanelState::default();
        // Expand the Effects folder by default (layer3 in mock data)
        layer_panel_state.expanded_folders.push(LayerId::new("layer3"));
        
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
            layer_panel_state,
            onion_skinning: false,
            loop_playback: false,
            show_label_panel: false,
            frame_range_selection: None,
            frame_range_mode: false,
            onion_skin_frames_before: 3,
            onion_skin_frames_after: 3,
            onion_skin_opacity: 0.3,
            show_onion_settings: false,
            onion_skin_outline_mode: false,
            is_scrubbing: false,
            was_playing: false,
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
        
        // Draw label management panel if visible
        if self.state.show_label_panel {
            self.draw_label_management_panel(ui, engine);
        }
        
        // Draw onion skin settings panel if visible
        if self.state.show_onion_settings {
            self.draw_onion_settings_panel(ui);
        }
        
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
                
                // Onion skin settings button (only show when enabled)
                if self.state.onion_skinning {
                    if ui.button("⚙").on_hover_text("Onion Skin Settings").clicked() {
                        self.state.show_onion_settings = !self.state.show_onion_settings;
                    }
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
                let frame_selection_icon = if self.state.frame_range_mode { "✓⬚" } else { "⬚" };
                if ui.selectable_label(self.state.frame_range_mode, frame_selection_icon)
                    .on_hover_text(self.get_tooltip("timeline.toolbar.frame_selection"))
                    .clicked() 
                {
                    self.state.frame_range_mode = !self.state.frame_range_mode;
                    if !self.state.frame_range_mode {
                        self.state.frame_range_selection = None;
                    }
                }
                
                ui.separator();
                
                // Label management panel toggle
                let label_panel_icon = if self.state.show_label_panel { "📋" } else { "📃" };
                if ui.selectable_label(self.state.show_label_panel, label_panel_icon)
                    .on_hover_text(self.get_tooltip("timeline.toolbar.label_panel"))
                    .clicked() 
                {
                    self.state.show_label_panel = !self.state.show_label_panel;
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
                        
                        // Track if we're currently dragging
                        let mut layer_order_changed = false;
                        let mut new_layer_order = layers.clone();
                        
                        for (idx, layer) in layers.iter().enumerate() {
                            let layer_height = self.state.track_heights
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
                            
                            // Check if this is the drop target position
                            if let Some(drop_idx) = self.state.layer_panel_state.drop_target_index {
                                if drop_idx == idx {
                                    // Draw insertion line
                                    let rect = ui.available_rect_before_wrap();
                                    let insertion_y = rect.min.y;
                                    ui.painter().line_segment(
                                        [pos2(rect.min.x, insertion_y), pos2(rect.max.x, insertion_y)],
                                        Stroke::new(2.0, Color32::from_rgb(100, 200, 255)),
                                    );
                                    ui.add_space(3.0);
                                }
                            }
                            
                            let layer_id = ui.make_persistent_id(("layer_row", idx));
                            let layer_rect = Rect::from_min_size(
                                ui.cursor().min,
                                vec2(ui.available_width(), layer_height),
                            );
                            
                            // Handle drag source
                            let response = ui.interact(layer_rect, layer_id, Sense::click_and_drag());
                            
                            if response.drag_started() && !is_locked {
                                // Start dragging this layer
                                self.state.layer_panel_state.dragging_layer = Some((layer.id.clone(), response.interact_pointer_pos().unwrap_or_default().y));
                            }
                            
                            // Handle drop target
                            if self.state.layer_panel_state.dragging_layer.is_some() {
                                if let Some(hover_pos) = response.hover_pos() {
                                    // Calculate which side of the layer we're hovering on
                                    let relative_y = hover_pos.y - layer_rect.min.y;
                                    let drop_index = if relative_y < layer_rect.height() / 2.0 {
                                        idx
                                    } else {
                                        idx + 1
                                    };
                                    self.state.layer_panel_state.drop_target_index = Some(drop_index);
                                }
                            }
                            
                            // Check if drag ended on this position
                            if response.drag_stopped() {
                                if let Some((dragged_layer_id, _)) = &self.state.layer_panel_state.dragging_layer {
                                    if let Some(drop_idx) = self.state.layer_panel_state.drop_target_index {
                                        // Find the dragged layer's current index
                                        if let Some(drag_idx) = new_layer_order.iter().position(|l| &l.id == dragged_layer_id) {
                                            // Perform the reorder
                                            let dragged_layer = new_layer_order.remove(drag_idx);
                                            let insert_idx = if drag_idx < drop_idx { drop_idx - 1 } else { drop_idx };
                                            new_layer_order.insert(insert_idx.min(new_layer_order.len()), dragged_layer);
                                            layer_order_changed = true;
                                        }
                                    }
                                    // Clear drag state
                                    self.state.layer_panel_state.dragging_layer = None;
                                    self.state.layer_panel_state.drop_target_index = None;
                                }
                            }
                            
                            // Skip this layer if it's inside a collapsed folder
                            let mut skip_layer = false;
                            if let Some(parent_id) = &layer.parent_id {
                                // Check if any parent folder is collapsed
                                let mut current_parent = Some(parent_id);
                                while let Some(pid) = current_parent {
                                    if let Some(parent_layer) = layers.iter().find(|l| &l.id == pid) {
                                        if parent_layer.layer_type == LayerType::Folder && 
                                           !self.state.layer_panel_state.expanded_folders.contains(pid) {
                                            skip_layer = true;
                                            break;
                                        }
                                        current_parent = parent_layer.parent_id.as_ref();
                                    } else {
                                        break;
                                    }
                                }
                            }
                            
                            if skip_layer {
                                continue;
                            }
                            
                            // Draw the layer content
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
                                
                                // Calculate indentation
                                let indent_level = self.calculate_layer_indent_level(&layer, &layers);
                                ui.add_space(indent_level as f32 * 16.0); // 16 pixels per indent level
                                
                                // Expand/collapse arrow for folders
                                if layer.layer_type == LayerType::Folder {
                                    let is_expanded = self.state.layer_panel_state.expanded_folders.contains(&layer.id);
                                    let arrow_icon = if is_expanded { "▼" } else { "▶" };
                                    if ui.button(arrow_icon)
                                        .on_hover_text(self.get_tooltip("timeline.layer.toggle_folder"))
                                        .clicked() 
                                    {
                                        if is_expanded {
                                            self.state.layer_panel_state.expanded_folders.retain(|id| id != &layer.id);
                                        } else {
                                            self.state.layer_panel_state.expanded_folders.push(layer.id.clone());
                                        }
                                    }
                                }
                                
                                // Layer type icon
                                let (type_icon, type_tooltip) = match layer.layer_type {
                                    crate::LayerType::Normal => ("🎬", "timeline.layer.type_normal"),
                                    crate::LayerType::Audio => ("🔊", "timeline.layer.type_audio"), 
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
                                        
                                        // If this is a newly created layer, scroll to it
                                        if let Some(newly_created_id) = &self.state.layer_panel_state.newly_created_layer {
                                            if newly_created_id == &layer.id {
                                                // Scroll to this layer
                                                ui.scroll_to_rect(layer_rect, Some(Align::Center));
                                                // Clear the newly created flag after scrolling
                                                self.state.layer_panel_state.newly_created_layer = None;
                                            }
                                        }
                                    } else {
                                        // Show normal label for other layers
                                        if ui.selectable_label(is_selected, &layer.name).clicked() {
                                            if ui.input(|i| i.modifiers.shift) && !self.state.selected_layers.is_empty() {
                                                // Range select with Shift
                                                let last_selected = self.state.selected_layers.last().unwrap();
                                                let last_idx = layers.iter().position(|l| &l.id == last_selected).unwrap_or(0);
                                                let current_idx = idx;
                                                
                                                let start = last_idx.min(current_idx);
                                                let end = last_idx.max(current_idx);
                                                
                                                // Clear existing selection
                                                self.state.selected_layers.clear();
                                                
                                                // Select all layers in range
                                                for i in start..=end {
                                                    if let Some(l) = layers.get(i) {
                                                        self.state.selected_layers.push(l.id.clone());
                                                    }
                                                }
                                            } else if ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
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
                                    // Normal label when not renaming
                                    if ui.selectable_label(is_selected, &layer.name).clicked() {
                                        if ui.input(|i| i.modifiers.shift) && !self.state.selected_layers.is_empty() {
                                            // Range select with Shift
                                            let last_selected = self.state.selected_layers.last().unwrap();
                                            let last_idx = layers.iter().position(|l| &l.id == last_selected).unwrap_or(0);
                                            let current_idx = idx;
                                            
                                            let start = last_idx.min(current_idx);
                                            let end = last_idx.max(current_idx);
                                            
                                            // Clear existing selection
                                            self.state.selected_layers.clear();
                                            
                                            // Select all layers in range
                                            for i in start..=end {
                                                if let Some(l) = layers.get(i) {
                                                    self.state.selected_layers.push(l.id.clone());
                                                }
                                            }
                                        } else if ui.input(|i| i.modifiers.ctrl || i.modifiers.command) {
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
                            
                            // Add horizontal separator line beneath each layer
                            let separator_rect = ui.available_rect_before_wrap();
                            let separator_y = separator_rect.min.y;
                            ui.painter().line_segment(
                                [pos2(separator_rect.min.x + 10.0, separator_y), pos2(separator_rect.max.x - 10.0, separator_y)],
                                Stroke::new(0.5, Color32::from_gray(60)),
                            );
                            
                            ui.add_space(2.0);
                        }
                        
                        // Apply layer order changes if any
                        if layer_order_changed {
                            // TODO: Add a reorder_layers method to RiveEngine trait
                            // For now, we'll just print the new order
                            println!("New layer order:");
                            for (i, layer) in new_layer_order.iter().enumerate() {
                                println!("  {}: {}", i, layer.name);
                            }
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
                        // Set this layer for auto-focus and renaming
                        self.state.layer_panel_state.newly_created_layer = Some(layer_id.clone());
                        self.state.layer_panel_state.renaming_layer = Some((layer_id.clone(), "New Layer".to_string()));
                        // Select the newly created layer
                        self.state.selected_layers.clear();
                        self.state.selected_layers.push(layer_id.clone());
                        println!("Added new layer: {:?}", layer_id);
                    }
                    
                    // Add folder
                    if ui.button("📁").on_hover_text(self.get_tooltip("timeline.layer.new_folder")).clicked() {
                        let layer_id = engine.add_folder_layer("New Folder".to_string());
                        // Set this folder for auto-focus and renaming
                        self.state.layer_panel_state.newly_created_layer = Some(layer_id.clone());
                        self.state.layer_panel_state.renaming_layer = Some((layer_id.clone(), "New Folder".to_string()));
                        // Select the newly created folder
                        self.state.selected_layers.clear();
                        self.state.selected_layers.push(layer_id.clone());
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
        // Draw a background to see where the frame grid area is
        ui.painter().rect_filled(
            rect,
            0.0,
            Color32::from_gray(40), // Slightly lighter gray
        );
        
        // Draw a border to debug the area
        ui.painter().rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0, Color32::GREEN),
            egui::epaint::StrokeKind::Outside,
        );
        
        // Use allocate_new_ui to properly constrain the ScrollArea
        ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
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
                    
                    // Debug: Draw a test to ensure ScrollArea is working
                    ui.label(format!("Layers: {}, Frames: {}, Size: {}x{}", layers.len(), total_frames, total_width as i32, total_height as i32));
                    
                    // Draw a simple test pattern to verify coordinate system
                    ui.painter().rect_filled(
                        Rect::from_min_size(pos2(10.0, 10.0), vec2(100.0, 50.0)),
                        0.0,
                        Color32::YELLOW,
                    );
                    ui.painter().text(
                        pos2(20.0, 20.0),
                        egui::Align2::LEFT_TOP,
                        "Test",
                        egui::FontId::default(),
                        Color32::BLACK,
                    );
                    
                    // Get viewport for optimization
                    let viewport = ui.clip_rect();
                    
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
                            // Draw onion skinning if enabled
                            if self.state.onion_skinning && !matches!(layer.layer_type, crate::LayerType::Audio) {
                                self.draw_onion_skins(ui, engine, layer, y_offset, layer_height, frame_width);
                            }
                            
                            // Draw frames based on layer type
                            if matches!(layer.layer_type, crate::LayerType::Audio) {
                                self.draw_audio_waveform(ui, layer, y_offset, layer_height, visible_start_frame..=visible_end_frame, frame_width);
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
                                            
                                            // Highlight keyframe at playhead position
                                            let is_at_playhead = frame == self.state.playhead_frame;
                                            if is_at_playhead {
                                                // Draw glow effect around keyframe at playhead
                                                let glow_rect = frame_rect.expand(2.0);
                                                ui.painter().rect_filled(
                                                    glow_rect,
                                                    2.0,
                                                    self.config.style.playhead_color.gamma_multiply(0.3),
                                                );
                                            }
                                            
                                            ui.painter().circle_filled(
                                                frame_rect.center(),
                                                3.0,
                                                if is_at_playhead {
                                                    self.config.style.playhead_color
                                                } else if is_selected {
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
            // Draw playhead line - thicker when scrubbing
            let line_width = if self.state.is_scrubbing { 3.0 } else { 2.0 };
            let playhead_color = if self.state.is_scrubbing {
                self.config.style.playhead_color.gamma_multiply(1.2)
            } else {
                self.config.style.playhead_color
            };
            
            ui.painter().line_segment(
                [pos2(x, ruler_rect.min.y), pos2(x, grid_rect.max.y)],
                Stroke::new(line_width, playhead_color),
            );

            // Draw playhead marker in ruler
            let points = vec![
                pos2(x, ruler_rect.min.y),
                pos2(x - 5.0, ruler_rect.min.y + 10.0),
                pos2(x + 5.0, ruler_rect.min.y + 10.0),
            ];
            ui.painter().add(Shape::convex_polygon(
                points,
                playhead_color,
                Stroke::NONE,
            ));
            
            // Draw scrub indicator when scrubbing
            if self.state.is_scrubbing {
                // Draw a small "SCRUB" text above the playhead
                ui.painter().text(
                    pos2(x, ruler_rect.min.y - 15.0),
                    Align2::CENTER_BOTTOM,
                    "SCRUB",
                    FontId::proportional(10.0),
                    playhead_color,
                );
            }
        }

        // Handle playhead dragging with scrub preview
        let ruler_response = ui.allocate_rect(ruler_rect, Sense::click_and_drag());
        
        // Start scrubbing on drag begin
        if ruler_response.drag_started() {
            self.state.is_scrubbing = true;
            // Store the current playing state before pausing
            self.state.was_playing = self.state.is_playing;
            if self.state.was_playing {
                engine.pause();
            }
        }
        
        // Update position while dragging
        if ruler_response.dragged() {
            if let Some(pos) = ruler_response.interact_pointer_pos() {
                let raw_x = pos.x - ruler_rect.min.x + self.state.scroll_x;
                let modifiers = ui.input(|i| i.modifiers);
                let snapped_x = self.snap_position(raw_x, &modifiers);
                let frame = (snapped_x / frame_width).round() as u32;
                let clamped_frame = frame.min(engine.get_total_frames() - 1);
                
                // Update position while scrubbing
                engine.seek(clamped_frame);
                
                // Show tooltip while scrubbing
                ui.painter().text(
                    pos2(pos.x, pos.y - 20.0),
                    Align2::CENTER_BOTTOM,
                    format!("Frame {}", clamped_frame),
                    FontId::proportional(11.0),
                    ui.style().visuals.text_color(),
                );
                
                // Force immediate redraw for responsive scrubbing
                ui.ctx().request_repaint();
            }
        }
        
        // End scrubbing
        if ruler_response.drag_stopped() {
            self.state.is_scrubbing = false;
            if self.state.was_playing {
                engine.play();
            }
        }
        
        // Handle single click to jump to position
        if ruler_response.clicked() && !ruler_response.dragged() {
            if let Some(pos) = ruler_response.interact_pointer_pos() {
                let raw_x = pos.x - ruler_rect.min.x + self.state.scroll_x;
                let modifiers = ui.input(|i| i.modifiers);
                let snapped_x = self.snap_position(raw_x, &modifiers);
                let frame = (snapped_x / frame_width).round() as u32;
                engine.seek(frame.min(engine.get_total_frames() - 1));
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
                                
                                if ui.button("⬇️ Merge Down").clicked() {
                                    // TODO: Implement merge down functionality
                                    println!("Merge down layer: {:?}", layer_id);
                                    close_menu = true;
                                }
                                
                                if ui.button("📁 Convert to Folder").clicked() {
                                    // TODO: Implement convert to folder functionality
                                    println!("Convert layer to folder: {:?}", layer_id);
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
        
        // Highlight playhead position if it's in this layer's audio
        let playhead_x = self.state.playhead_frame as f32 * frame_width;
        if self.state.playhead_frame >= *frame_range.start() && self.state.playhead_frame <= *frame_range.end() {
            // Draw vertical line at playhead position
            ui.painter().line_segment(
                [pos2(playhead_x, y_offset), pos2(playhead_x, y_offset + layer_height)],
                Stroke::new(2.0, self.config.style.playhead_color),
            );
            
            // Draw a small indicator circle at the waveform center
            ui.painter().circle_filled(
                pos2(playhead_x, center_y),
                4.0,
                self.config.style.playhead_color,
            );
        }
        
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
    
    /// Snap position to grid
    pub fn snap_position(&self, pos: f32, modifiers: &Modifiers) -> f32 {
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
    
    /// Update snap guides for visual feedback
    pub fn update_snap_guides(&mut self, pos: f32) {
        self.state.snap_guides.clear();
        
        if !self.config.snap.enabled || !self.config.snap.show_guides {
            return;
        }
        
        let frame_width = self.config.frame_width * self.state.zoom_level;
        let frame_pos = pos / frame_width;
        
        if self.config.snap.snap_to_frames {
            let nearest_frame = frame_pos.round();
            let snapped_pos = nearest_frame * frame_width;
            
            if (pos - snapped_pos).abs() < self.config.snap.threshold_pixels {
                self.state.snap_guides.push(snapped_pos);
            }
        }
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
    
    /// Draw label management panel as a popup window
    fn draw_label_management_panel(&mut self, ui: &mut Ui, engine: &mut Box<dyn RiveEngine>) {
        let ctx = ui.ctx();
        
        egui::Window::new("📋 Labels & Comments")
            .resizable(true)
            .default_width(300.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Frame Labels");
                    
                    // Frame labels section
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut labels_to_remove = Vec::new();
                        let mut jump_to_frame = None;
                        
                        for (index, label) in self.config.frame_labels.iter().enumerate() {
                            ui.horizontal(|ui| {
                                // Frame number
                                ui.label(format!("F{}", label.frame));
                                
                                // Label text
                                ui.label(&label.label);
                                
                                // Color indicator
                                if let Some(color) = label.color {
                                    ui.painter().circle_filled(ui.cursor().min + egui::vec2(5.0, 5.0), 4.0, color);
                                }
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Jump to frame button
                                    if ui.small_button("⏯").on_hover_text("Jump to frame").clicked() {
                                        jump_to_frame = Some(label.frame);
                                    }
                                    
                                    // Delete button
                                    if ui.small_button("🗑").on_hover_text("Delete label").clicked() {
                                        labels_to_remove.push(index);
                                    }
                                });
                            });
                            ui.separator();
                        }
                        
                        // Handle deletions
                        for &index in labels_to_remove.iter().rev() {
                            self.config.frame_labels.remove(index);
                        }
                        
                        // Handle jump to frame
                        if let Some(frame) = jump_to_frame {
                            self.state.playhead_frame = frame;
                            engine.seek(frame);
                        }
                    });
                    
                    ui.separator();
                    ui.heading("Frame Comments");
                    
                    // Frame comments section
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut comments_to_remove = Vec::new();
                        let mut jump_to_frame = None;
                        
                        for (index, comment) in self.config.frame_comments.iter().enumerate() {
                            ui.horizontal(|ui| {
                                // Frame number
                                ui.label(format!("F{}", comment.frame));
                                
                                // Comment text (truncated)
                                let comment_text = if comment.comment.len() > 25 {
                                    format!("{}...", &comment.comment[..22])
                                } else {
                                    comment.comment.clone()
                                };
                                ui.label(comment_text).on_hover_text(&comment.comment);
                                
                                // Color indicator
                                if let Some(color) = comment.color {
                                    ui.painter().circle_filled(ui.cursor().min + egui::vec2(5.0, 5.0), 4.0, color);
                                }
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Jump to frame button
                                    if ui.small_button("⏯").on_hover_text("Jump to frame").clicked() {
                                        jump_to_frame = Some(comment.frame);
                                    }
                                    
                                    // Delete button
                                    if ui.small_button("🗑").on_hover_text("Delete comment").clicked() {
                                        comments_to_remove.push(index);
                                    }
                                });
                            });
                            
                            // Show author and timestamp if available
                            if comment.author.is_some() || comment.timestamp.is_some() {
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);
                                    if let Some(author) = &comment.author {
                                        ui.small(format!("by {}", author));
                                    }
                                    if let Some(timestamp) = &comment.timestamp {
                                        ui.small(format!("at {}", timestamp));
                                    }
                                });
                            }
                            ui.separator();
                        }
                        
                        // Handle deletions
                        for &index in comments_to_remove.iter().rev() {
                            self.config.frame_comments.remove(index);
                        }
                        
                        // Handle jump to frame
                        if let Some(frame) = jump_to_frame {
                            self.state.playhead_frame = frame;
                            engine.seek(frame);
                        }
                    });
                    
                    ui.separator();
                    
                    // Add new label/comment section
                    ui.horizontal(|ui| {
                        if ui.button("➕ Add Label").clicked() {
                            let new_label = crate::FrameLabel::new(self.state.playhead_frame, "New Label");
                            self.config.frame_labels.push(new_label);
                        }
                        
                        if ui.button("💬 Add Comment").clicked() {
                            let new_comment = crate::FrameComment::new(self.state.playhead_frame, "New comment");
                            self.config.frame_comments.push(new_comment);
                        }
                    });
                    
                    ui.separator();
                    
                    // Statistics
                    ui.horizontal(|ui| {
                        ui.label(format!("{} labels, {} comments", 
                            self.config.frame_labels.len(), 
                            self.config.frame_comments.len()
                        ));
                    });
                });
            });
    }
    
    /// Draw onion skin frames (previous and next frames with transparency)
    fn draw_onion_skins(
        &self,
        ui: &mut Ui,
        engine: &Box<dyn RiveEngine>,
        layer: &crate::layer::LayerInfo,
        y_offset: f32,
        layer_height: f32,
        frame_width: f32,
    ) {
        let current_frame = self.state.playhead_frame;
        
        // Draw previous frames (blue tint)
        for i in 1..=self.state.onion_skin_frames_before {
            if let Some(prev_frame) = current_frame.checked_sub(i) {
                let frame_data = engine.get_frame_data(layer.id.clone(), prev_frame);
                if !matches!(frame_data.frame_type, crate::frame::FrameType::Empty) {
                    let x = prev_frame as f32 * frame_width;
                    let opacity = self.state.onion_skin_opacity / (i as f32); // Farther frames are more transparent
                    
                    let frame_rect = Rect::from_min_size(
                        pos2(x, y_offset),
                        vec2(frame_width - 1.0, layer_height - 1.0),
                    );
                    
                    // Blue tint for previous frames
                    let color = Color32::from_rgba_unmultiplied(100, 150, 255, (opacity * 255.0) as u8);
                    
                    if self.state.onion_skin_outline_mode {
                        // Outline mode - draw only border
                        let stroke = Stroke::new(2.0, color);
                        ui.painter().line_segment([frame_rect.left_top(), frame_rect.right_top()], stroke);
                        ui.painter().line_segment([frame_rect.right_top(), frame_rect.right_bottom()], stroke);
                        ui.painter().line_segment([frame_rect.right_bottom(), frame_rect.left_bottom()], stroke);
                        ui.painter().line_segment([frame_rect.left_bottom(), frame_rect.left_top()], stroke);
                    } else {
                        // Solid mode - fill the frame
                        ui.painter().rect_filled(frame_rect, 2.0, color);
                    }
                }
            }
        }
        
        // Draw next frames (green tint)
        for i in 1..=self.state.onion_skin_frames_after {
            let next_frame = current_frame + i;
            if next_frame < engine.get_total_frames() {
                let frame_data = engine.get_frame_data(layer.id.clone(), next_frame);
                if !matches!(frame_data.frame_type, crate::frame::FrameType::Empty) {
                    let x = next_frame as f32 * frame_width;
                    let opacity = self.state.onion_skin_opacity / (i as f32); // Farther frames are more transparent
                    
                    let frame_rect = Rect::from_min_size(
                        pos2(x, y_offset),
                        vec2(frame_width - 1.0, layer_height - 1.0),
                    );
                    
                    // Green tint for next frames
                    let color = Color32::from_rgba_unmultiplied(100, 255, 150, (opacity * 255.0) as u8);
                    
                    if self.state.onion_skin_outline_mode {
                        // Outline mode - draw only border
                        let stroke = Stroke::new(2.0, color);
                        ui.painter().line_segment([frame_rect.left_top(), frame_rect.right_top()], stroke);
                        ui.painter().line_segment([frame_rect.right_top(), frame_rect.right_bottom()], stroke);
                        ui.painter().line_segment([frame_rect.right_bottom(), frame_rect.left_bottom()], stroke);
                        ui.painter().line_segment([frame_rect.left_bottom(), frame_rect.left_top()], stroke);
                    } else {
                        // Solid mode - fill the frame
                        ui.painter().rect_filled(frame_rect, 2.0, color);
                    }
                }
            }
        }
    }
    
    /// Draw onion skin settings panel
    fn draw_onion_settings_panel(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx();
        
        egui::Window::new("🧅 Onion Skin Settings")
            .resizable(true)
            .default_width(300.0)
            .default_height(350.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Onion Skin Configuration");
                    ui.separator();
                    
                    // Frame range settings
                    ui.label("Frame Range");
                    ui.horizontal(|ui| {
                        ui.label("Previous Frames:");
                        ui.add(egui::DragValue::new(&mut self.state.onion_skin_frames_before)
                            .speed(1.0)
                            .range(0..=10)
                            .suffix(" frames"));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Next Frames:");
                        ui.add(egui::DragValue::new(&mut self.state.onion_skin_frames_after)
                            .speed(1.0)
                            .range(0..=10)
                            .suffix(" frames"));
                    });
                    
                    ui.separator();
                    
                    // Opacity settings
                    ui.label("Opacity Settings");
                    ui.horizontal(|ui| {
                        ui.label("Base Opacity:");
                        ui.add(egui::Slider::new(&mut self.state.onion_skin_opacity, 0.1..=0.8)
                            .show_value(true)
                            .suffix(""));
                    });
                    
                    ui.separator();
                    
                    // Display mode
                    ui.label("Display Mode");
                    ui.checkbox(&mut self.state.onion_skin_outline_mode, "Outline Mode");
                    ui.label("When enabled, shows only object outlines");
                    
                    ui.separator();
                    
                    // Color preview
                    ui.label("Color Preview");
                    ui.horizontal(|ui| {
                        // Previous frames preview
                        ui.vertical(|ui| {
                            ui.label("Previous Frames");
                            for i in 1..=3.min(self.state.onion_skin_frames_before) {
                                let opacity = self.state.onion_skin_opacity / (i as f32);
                                let color = Color32::from_rgba_unmultiplied(100, 150, 255, (opacity * 255.0) as u8);
                                ui.horizontal(|ui| {
                                    ui.colored_label(color, format!("Frame -{}", i));
                                    ui.label(format!("({:.0}% opacity)", opacity * 100.0));
                                });
                            }
                        });
                        
                        ui.separator();
                        
                        // Next frames preview
                        ui.vertical(|ui| {
                            ui.label("Next Frames");
                            for i in 1..=3.min(self.state.onion_skin_frames_after) {
                                let opacity = self.state.onion_skin_opacity / (i as f32);
                                let color = Color32::from_rgba_unmultiplied(100, 255, 150, (opacity * 255.0) as u8);
                                ui.horizontal(|ui| {
                                    ui.colored_label(color, format!("Frame +{}", i));
                                    ui.label(format!("({:.0}% opacity)", opacity * 100.0));
                                });
                            }
                        });
                    });
                    
                    ui.separator();
                    
                    // Quick presets
                    ui.label("Quick Presets");
                    ui.horizontal(|ui| {
                        if ui.button("Light").clicked() {
                            self.state.onion_skin_opacity = 0.2;
                        }
                        if ui.button("Medium").clicked() {
                            self.state.onion_skin_opacity = 0.3;
                        }
                        if ui.button("Strong").clicked() {
                            self.state.onion_skin_opacity = 0.5;
                        }
                    });
                    
                    ui.separator();
                    
                    // Close button
                    if ui.button("Close").clicked() {
                        self.state.show_onion_settings = false;
                    }
                });
            });
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