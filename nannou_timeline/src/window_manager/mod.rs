//! Advanced Window Management System
//! 
//! Adobe-style panel management with docking, snapping, grouping, and workspace persistence.

pub mod panel;
pub mod dock;
pub mod snap;
pub mod workspace;

use std::collections::HashMap;
use egui::*;

pub use panel::{Panel, PanelId, PanelContent};
pub use dock::{DockState, DockPosition, DockZone};
pub use snap::{SnapGrid, SnapGuide};
pub use workspace::{WorkspaceLayout, WorkspaceManager};

/// Main window manager that handles all panel operations
pub struct WindowManager {
    pub panels: HashMap<PanelId, Panel>,
    pub dock_zones: Vec<DockZone>,
    pub snap_grid: SnapGrid,
    pub snap_guides: Vec<SnapGuide>,
    pub drag_state: Option<DragState>,
    pub workspace_manager: WorkspaceManager,
    pub show_dock_preview: bool,
    pub show_snap_guides: bool,
}

/// State during a drag operation
#[derive(Clone, Debug)]
pub struct DragState {
    pub panel_id: PanelId,
    pub start_pos: Pos2,
    pub offset: Vec2,
    pub is_docking: bool,
    pub target_dock_zone: Option<usize>,
    pub snap_targets: Vec<SnapGuide>,
}

impl WindowManager {
    pub fn new() -> Self {
        let mut manager = Self {
            panels: HashMap::new(),
            dock_zones: Vec::new(),
            snap_grid: SnapGrid::default(),
            snap_guides: Vec::new(),
            drag_state: None,
            workspace_manager: WorkspaceManager::new(),
            show_dock_preview: true,
            show_snap_guides: true,
        };
        
        // Initialize with default layout
        manager.load_default_layout();
        manager
    }
    
    /// Load the default workspace layout
    fn load_default_layout(&mut self) {
        // Timeline at bottom
        let mut timeline_panel = Panel::new(
            PanelId::new("timeline"),
            "Timeline".to_string(),
        );
        timeline_panel.dock_state = DockState::Docked {
            position: DockPosition::Bottom,
            parent: None,
            size_ratio: 0.3,
        };
        
        // Layers on right
        let mut layers_panel = Panel::new(
            PanelId::new("layers"),
            "Layers".to_string(),
        );
        layers_panel.dock_state = DockState::Docked {
            position: DockPosition::Right,
            parent: None,
            size_ratio: 0.25,
        };
        
        // Properties below layers
        let mut properties_panel = Panel::new(
            PanelId::new("properties"),
            "Properties".to_string(),
        );
        properties_panel.dock_state = DockState::Docked {
            position: DockPosition::Bottom,
            parent: Some(PanelId::new("layers")),
            size_ratio: 0.4,
        };
        
        // Tools on left
        let mut tools_panel = Panel::new(
            PanelId::new("tools"),
            "Tools".to_string(),
        );
        tools_panel.dock_state = DockState::Docked {
            position: DockPosition::Left,
            parent: None,
            size_ratio: 0.05,
        };
        
        // Stage/Canvas in center (floating for now)
        let stage_panel = Panel::new(
            PanelId::new("stage"),
            "Stage".to_string(),
        );
        
        // Add panels
        self.panels.insert(timeline_panel.id.clone(), timeline_panel);
        self.panels.insert(layers_panel.id.clone(), layers_panel);
        self.panels.insert(properties_panel.id.clone(), properties_panel);
        self.panels.insert(tools_panel.id.clone(), tools_panel);
        self.panels.insert(stage_panel.id.clone(), stage_panel);
    }
    
    /// Update dock zones based on current window and panel layout
    pub fn update_dock_zones(&mut self, window_rect: Rect) {
        self.dock_zones.clear();
        
        // Main window edges
        let edge_size = 80.0;
        let highlight_thickness = 4.0;
        
        // Left edge
        self.dock_zones.push(DockZone {
            rect: Rect::from_min_size(
                window_rect.min,
                vec2(edge_size, window_rect.height()),
            ),
            position: DockPosition::Left,
            parent: None,
            highlight_rect: Rect::from_min_size(
                window_rect.min,
                vec2(highlight_thickness, window_rect.height()),
            ),
        });
        
        // Right edge
        self.dock_zones.push(DockZone {
            rect: Rect::from_min_size(
                pos2(window_rect.max.x - edge_size, window_rect.min.y),
                vec2(edge_size, window_rect.height()),
            ),
            position: DockPosition::Right,
            parent: None,
            highlight_rect: Rect::from_min_size(
                pos2(window_rect.max.x - highlight_thickness, window_rect.min.y),
                vec2(highlight_thickness, window_rect.height()),
            ),
        });
        
        // Top edge
        self.dock_zones.push(DockZone {
            rect: Rect::from_min_size(
                window_rect.min,
                vec2(window_rect.width(), edge_size),
            ),
            position: DockPosition::Top,
            parent: None,
            highlight_rect: Rect::from_min_size(
                window_rect.min,
                vec2(window_rect.width(), highlight_thickness),
            ),
        });
        
        // Bottom edge
        self.dock_zones.push(DockZone {
            rect: Rect::from_min_size(
                pos2(window_rect.min.x, window_rect.max.y - edge_size),
                vec2(window_rect.width(), edge_size),
            ),
            position: DockPosition::Bottom,
            parent: None,
            highlight_rect: Rect::from_min_size(
                pos2(window_rect.min.x, window_rect.max.y - highlight_thickness),
                vec2(window_rect.width(), highlight_thickness),
            ),
        });
        
        // Center zone for tabbing
        let center_size = 100.0;
        self.dock_zones.push(DockZone {
            rect: Rect::from_center_size(
                window_rect.center(),
                vec2(center_size, center_size),
            ),
            position: DockPosition::Center,
            parent: None,
            highlight_rect: Rect::from_center_size(
                window_rect.center(),
                vec2(center_size, center_size),
            ),
        });
        
        // Collect panel dock zones to add
        let panel_zones: Vec<(PanelId, Rect)> = self.panels.iter()
            .filter(|(_, panel)| matches!(panel.dock_state, DockState::Docked { .. }) && panel.is_visible)
            .map(|(id, panel)| (id.clone(), panel.get_rect()))
            .collect();
        
        // Add dock zones for each docked panel
        for (panel_id, rect) in panel_zones {
            self.add_panel_dock_zones(panel_id, rect);
        }
    }
    
    /// Add dock zones around a panel for nested docking
    fn add_panel_dock_zones(&mut self, panel_id: PanelId, panel_rect: Rect) {
        let edge_size = 40.0;
        let highlight_thickness = 3.0;
        
        // Left of panel
        self.dock_zones.push(DockZone {
            rect: Rect::from_min_size(
                pos2(panel_rect.min.x - edge_size / 2.0, panel_rect.min.y),
                vec2(edge_size, panel_rect.height()),
            ),
            position: DockPosition::Left,
            parent: Some(panel_id.clone()),
            highlight_rect: Rect::from_min_size(
                panel_rect.min,
                vec2(highlight_thickness, panel_rect.height()),
            ),
        });
        
        // Right of panel
        self.dock_zones.push(DockZone {
            rect: Rect::from_min_size(
                pos2(panel_rect.max.x - edge_size / 2.0, panel_rect.min.y),
                vec2(edge_size, panel_rect.height()),
            ),
            position: DockPosition::Right,
            parent: Some(panel_id.clone()),
            highlight_rect: Rect::from_min_size(
                pos2(panel_rect.max.x - highlight_thickness, panel_rect.min.y),
                vec2(highlight_thickness, panel_rect.height()),
            ),
        });
        
        // Top of panel
        self.dock_zones.push(DockZone {
            rect: Rect::from_min_size(
                pos2(panel_rect.min.x, panel_rect.min.y - edge_size / 2.0),
                vec2(panel_rect.width(), edge_size),
            ),
            position: DockPosition::Top,
            parent: Some(panel_id.clone()),
            highlight_rect: Rect::from_min_size(
                panel_rect.min,
                vec2(panel_rect.width(), highlight_thickness),
            ),
        });
        
        // Bottom of panel
        self.dock_zones.push(DockZone {
            rect: Rect::from_min_size(
                pos2(panel_rect.min.x, panel_rect.max.y - edge_size / 2.0),
                vec2(panel_rect.width(), edge_size),
            ),
            position: DockPosition::Bottom,
            parent: Some(panel_id.clone()),
            highlight_rect: Rect::from_min_size(
                pos2(panel_rect.min.x, panel_rect.max.y - highlight_thickness),
                vec2(panel_rect.width(), highlight_thickness),
            ),
        });
        
        // Center for tabbing
        self.dock_zones.push(DockZone {
            rect: Rect::from_center_size(
                panel_rect.center(),
                panel_rect.size() * 0.8,
            ),
            position: DockPosition::Center,
            parent: Some(panel_id),
            highlight_rect: panel_rect.shrink(5.0),
        });
    }
    
    /// Handle input events
    pub fn handle_input(&mut self, ctx: &Context) {
        let input = ctx.input(|i| i.clone());
        
        // Handle keyboard shortcuts
        if input.key_pressed(Key::D) && input.modifiers.ctrl {
            // Toggle dock preview
            self.show_dock_preview = !self.show_dock_preview;
        }
        
        if input.key_pressed(Key::G) && input.modifiers.ctrl {
            // Toggle snap guides
            self.show_snap_guides = !self.show_snap_guides;
        }
        
        if input.key_pressed(Key::S) && input.modifiers.ctrl {
            // Save current workspace
            if let Some(name) = self.prompt_workspace_name(ctx) {
                self.workspace_manager.save_current(name, &self.panels);
            }
        }
    }
    
    /// Prompt user for workspace name
    fn prompt_workspace_name(&self, _ctx: &Context) -> Option<String> {
        // TODO: Implement proper dialog
        Some("Custom Workspace".to_string())
    }
    
    /// Start dragging a panel
    pub fn start_drag(&mut self, panel_id: PanelId, pointer_pos: Pos2) {
        if let Some(panel) = self.panels.get(&panel_id) {
            let offset = pointer_pos - pos2(panel.position.x, panel.position.y);
            self.drag_state = Some(DragState {
                panel_id,
                start_pos: pointer_pos,
                offset,
                is_docking: false,
                target_dock_zone: None,
                snap_targets: Vec::new(),
            });
        }
    }
    
    /// Update drag state and handle docking/snapping preview
    pub fn update_drag(&mut self, pointer_pos: Pos2, modifiers: &Modifiers) {
        // Extract what we need before mutable borrows
        let should_snap = self.snap_grid.enabled && !modifiers.shift;
        
        // Get drag state info
        if let Some(drag_state) = &self.drag_state {
            let panel_id = drag_state.panel_id.clone();
            let offset = drag_state.offset;
            
            // Calculate new position and snapping
            let new_pos = pointer_pos - offset;
            let (snapped_pos, snap_targets) = if should_snap {
                if let Some(panel) = self.panels.get(&panel_id) {
                    self.calculate_snap(new_pos, &panel.size)
                } else {
                    (new_pos, Vec::new())
                }
            } else {
                (new_pos, Vec::new())
            };
            
            // Update panel
            if let Some(panel) = self.panels.get_mut(&panel_id) {
                // Make panel floating during drag
                if !matches!(panel.dock_state, DockState::Floating) {
                    panel.dock_state = DockState::Floating;
                }
                
                panel.position.x = snapped_pos.x;
                panel.position.y = snapped_pos.y;
            }
            
            // Update drag state
            if let Some(drag_state) = &mut self.drag_state {
                drag_state.snap_targets = snap_targets;
                self.snap_guides = drag_state.snap_targets.clone();
                
                // Check for dock zones
                drag_state.target_dock_zone = None;
                drag_state.is_docking = false;
                
                for (i, zone) in self.dock_zones.iter().enumerate() {
                    if zone.rect.contains(pointer_pos) {
                        // Don't allow docking to self
                        if let Some(parent) = &zone.parent {
                            if parent == &drag_state.panel_id {
                                continue;
                            }
                        }
                        drag_state.target_dock_zone = Some(i);
                        drag_state.is_docking = true;
                        break;
                    }
                }
            }
        }
    }
    
    /// Calculate snap position based on guides and other panels
    fn calculate_snap(&self, pos: Pos2, size: &panel::Size) -> (Pos2, Vec<SnapGuide>) {
        let mut snap_targets = Vec::new();
        let mut snapped_pos = pos;
        
        // Snap to grid
        if self.snap_grid.enabled {
            snapped_pos = self.snap_grid.snap_position(pos);
        }
        
        // Snap to other panels
        let snap_distance = self.snap_grid.snap_distance;
        
        for (_other_id, other_panel) in &self.panels {
            if other_panel.is_visible {
                let other_rect = other_panel.get_rect();
                
                // Snap left edge
                if (pos.x - other_rect.max.x).abs() < snap_distance {
                    snapped_pos.x = other_rect.max.x;
                    snap_targets.push(SnapGuide::Vertical(other_rect.max.x));
                }
                if (pos.x - other_rect.min.x).abs() < snap_distance {
                    snapped_pos.x = other_rect.min.x;
                    snap_targets.push(SnapGuide::Vertical(other_rect.min.x));
                }
                
                // Snap right edge
                if ((pos.x + size.width) - other_rect.max.x).abs() < snap_distance {
                    snapped_pos.x = other_rect.max.x - size.width;
                    snap_targets.push(SnapGuide::Vertical(other_rect.max.x));
                }
                if ((pos.x + size.width) - other_rect.min.x).abs() < snap_distance {
                    snapped_pos.x = other_rect.min.x - size.width;
                    snap_targets.push(SnapGuide::Vertical(other_rect.min.x));
                }
                
                // Snap top edge
                if (pos.y - other_rect.max.y).abs() < snap_distance {
                    snapped_pos.y = other_rect.max.y;
                    snap_targets.push(SnapGuide::Horizontal(other_rect.max.y));
                }
                if (pos.y - other_rect.min.y).abs() < snap_distance {
                    snapped_pos.y = other_rect.min.y;
                    snap_targets.push(SnapGuide::Horizontal(other_rect.min.y));
                }
                
                // Snap bottom edge
                if ((pos.y + size.height) - other_rect.max.y).abs() < snap_distance {
                    snapped_pos.y = other_rect.max.y - size.height;
                    snap_targets.push(SnapGuide::Horizontal(other_rect.max.y));
                }
                if ((pos.y + size.height) - other_rect.min.y).abs() < snap_distance {
                    snapped_pos.y = other_rect.min.y - size.height;
                    snap_targets.push(SnapGuide::Horizontal(other_rect.min.y));
                }
            }
        }
        
        (snapped_pos, snap_targets)
    }
    
    /// End drag and apply docking if needed
    pub fn end_drag(&mut self) {
        if let Some(drag_state) = &self.drag_state {
            if let Some(zone_idx) = drag_state.target_dock_zone {
                if let Some(zone) = self.dock_zones.get(zone_idx) {
                    if let Some(panel) = self.panels.get_mut(&drag_state.panel_id) {
                        panel.dock_state = DockState::Docked {
                            position: zone.position.clone(),
                            parent: zone.parent.clone(),
                            size_ratio: 0.3, // Default size ratio
                        };
                    }
                }
            }
        }
        
        self.drag_state = None;
        self.snap_guides.clear();
    }
    
    /// Draw all panels and UI elements
    pub fn draw(&mut self, ctx: &Context) {
        let window_rect = ctx.available_rect();
        self.update_dock_zones(window_rect);
        
        // Handle input
        self.handle_input(ctx);
        
        // Draw workspace selector
        self.draw_workspace_selector(ctx);
        
        // Calculate docked panel areas
        let panel_areas = self.calculate_docked_areas(window_rect);
        
        // Draw docked panels
        for (panel_id, area) in &panel_areas {
            if let Some(panel) = self.panels.get_mut(panel_id) {
                if matches!(panel.dock_state, DockState::Docked { .. }) && panel.is_visible {
                    panel.draw_docked(ctx, *area);
                }
            }
        }
        
        // Draw floating panels on top
        let mut panels_to_draw: Vec<_> = self.panels.iter_mut()
            .filter(|(_, p)| matches!(p.dock_state, DockState::Floating) && p.is_visible)
            .collect();
        
        // Sort by z-order if implemented
        panels_to_draw.sort_by_key(|(_, p)| p.z_order);
        
        for (_panel_id, panel) in panels_to_draw {
            panel.draw_floating(ctx);
        }
        
        // Draw dock zone highlights during drag
        if self.show_dock_preview {
            if let Some(drag_state) = &self.drag_state {
                if let Some(zone_idx) = drag_state.target_dock_zone {
                    if let Some(zone) = self.dock_zones.get(zone_idx) {
                        // Draw highlight
                        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("dock_preview")));
                        painter.rect_filled(
                            zone.highlight_rect,
                            4.0,
                            Color32::from_rgba_unmultiplied(100, 150, 255, 120),
                        );
                        
                        // Draw dock preview
                        let preview_rect = self.calculate_dock_preview(zone, window_rect);
                        painter.rect_filled(
                            preview_rect,
                            4.0,
                            Color32::from_rgba_unmultiplied(100, 150, 255, 60),
                        );
                        painter.rect_stroke(
                            preview_rect,
                            4.0,
                            Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
                            egui::epaint::StrokeKind::Outside,
                        );
                    }
                }
            }
        }
        
        // Draw snap guides
        if self.show_snap_guides && !self.snap_guides.is_empty() {
            for guide in &self.snap_guides {
                guide.draw(ctx, window_rect);
            }
        }
    }
    
    /// Calculate areas for docked panels
    fn calculate_docked_areas(&self, window_rect: Rect) -> HashMap<PanelId, Rect> {
        let mut areas = HashMap::new();
        let mut available_rect = window_rect;
        
        // First pass: dock to main window
        for (panel_id, panel) in &self.panels {
            if let DockState::Docked { position, parent: None, size_ratio } = &panel.dock_state {
                if panel.is_visible {
                    let (panel_rect, new_available) = self.split_rect(available_rect, position, *size_ratio);
                    areas.insert(panel_id.clone(), panel_rect);
                    available_rect = new_available;
                }
            }
        }
        
        // Second pass: dock to other panels
        for (panel_id, panel) in &self.panels {
            if let DockState::Docked { position, parent: Some(parent_id), size_ratio } = &panel.dock_state {
                if panel.is_visible {
                    if let Some(parent_rect) = areas.get(parent_id).cloned() {
                        let (panel_rect, _) = self.split_rect(parent_rect, position, *size_ratio);
                        areas.insert(panel_id.clone(), panel_rect);
                    }
                }
            }
        }
        
        areas
    }
    
    /// Split a rect based on dock position
    fn split_rect(&self, rect: Rect, position: &DockPosition, size_ratio: f32) -> (Rect, Rect) {
        match position {
            DockPosition::Left => {
                let split_x = rect.min.x + rect.width() * size_ratio;
                (
                    Rect::from_min_max(rect.min, pos2(split_x, rect.max.y)),
                    Rect::from_min_max(pos2(split_x, rect.min.y), rect.max),
                )
            },
            DockPosition::Right => {
                let split_x = rect.max.x - rect.width() * size_ratio;
                (
                    Rect::from_min_max(pos2(split_x, rect.min.y), rect.max),
                    Rect::from_min_max(rect.min, pos2(split_x, rect.max.y)),
                )
            },
            DockPosition::Top => {
                let split_y = rect.min.y + rect.height() * size_ratio;
                (
                    Rect::from_min_max(rect.min, pos2(rect.max.x, split_y)),
                    Rect::from_min_max(pos2(rect.min.x, split_y), rect.max),
                )
            },
            DockPosition::Bottom => {
                let split_y = rect.max.y - rect.height() * size_ratio;
                (
                    Rect::from_min_max(pos2(rect.min.x, split_y), rect.max),
                    Rect::from_min_max(rect.min, pos2(rect.max.x, split_y)),
                )
            },
            DockPosition::Center => {
                // For center (tabbing), return the full rect
                (rect, rect)
            },
        }
    }
    
    /// Calculate preview rect for docking
    fn calculate_dock_preview(&self, zone: &DockZone, window_rect: Rect) -> Rect {
        let size_ratio = 0.3; // Default preview size
        
        if let Some(parent_id) = &zone.parent {
            // Docking to another panel
            if let Some(panel) = self.panels.get(parent_id) {
                let parent_rect = panel.get_rect();
                let (preview_rect, _) = self.split_rect(parent_rect, &zone.position, size_ratio);
                preview_rect
            } else {
                Rect::ZERO
            }
        } else {
            // Docking to main window
            let (preview_rect, _) = self.split_rect(window_rect, &zone.position, size_ratio);
            preview_rect
        }
    }
    
    /// Draw workspace selector UI
    fn draw_workspace_selector(&mut self, ctx: &Context) {
        TopBottomPanel::top("workspace_selector").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Workspace:");
                
                // Workspace dropdown
                ComboBox::from_label("")
                    .selected_text(&self.workspace_manager.active_workspace)
                    .show_ui(ui, |ui| {
                        for name in self.workspace_manager.list_workspaces() {
                            ui.selectable_value(
                                &mut self.workspace_manager.active_workspace,
                                name.clone(),
                                &name,
                            );
                        }
                    });
                
                ui.separator();
                
                // Save button
                if ui.button("Save").clicked() {
                    if let Some(name) = self.prompt_workspace_name(ctx) {
                        self.workspace_manager.save_current(name, &self.panels);
                    }
                }
                
                // Save As button
                if ui.button("Save As...").clicked() {
                    if let Some(name) = self.prompt_workspace_name(ctx) {
                        self.workspace_manager.save_current(name, &self.panels);
                    }
                }
                
                // Reset button
                if ui.button("Reset to Default").clicked() {
                    self.load_default_layout();
                }
            });
        });
    }
}