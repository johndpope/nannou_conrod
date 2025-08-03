//! Dock manager using egui_dock for Flash-style panel management

use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use egui::*;
use serde::{Serialize, Deserialize};

/// Snap zone for window snapping
#[derive(Clone, Debug)]
pub struct SnapZone {
    pub rect: Rect,
    pub zone_type: SnapZoneType,
    pub strength: f32, // How strongly it attracts (pixels)
}

#[derive(Clone, Debug, PartialEq)]
pub enum SnapZoneType {
    WindowEdge(Edge),
    PanelEdge(String, Edge), // Panel ID and edge
    Grid(f32), // Grid size
}

#[derive(Clone, Debug, PartialEq)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

impl SnapZone {
    /// Check if a rect is within snapping distance
    pub fn should_snap(&self, rect: Rect, threshold: f32) -> Option<Rect> {
        match &self.zone_type {
            SnapZoneType::WindowEdge(edge) => {
                match edge {
                    Edge::Left => {
                        if (rect.left() - self.rect.left()).abs() < threshold {
                            Some(Rect::from_min_size(
                                pos2(self.rect.left(), rect.top()),
                                rect.size(),
                            ))
                        } else {
                            None
                        }
                    }
                    Edge::Right => {
                        if (rect.right() - self.rect.right()).abs() < threshold {
                            Some(Rect::from_min_size(
                                pos2(self.rect.right() - rect.width(), rect.top()),
                                rect.size(),
                            ))
                        } else {
                            None
                        }
                    }
                    Edge::Top => {
                        if (rect.top() - self.rect.top()).abs() < threshold {
                            Some(Rect::from_min_size(
                                pos2(rect.left(), self.rect.top()),
                                rect.size(),
                            ))
                        } else {
                            None
                        }
                    }
                    Edge::Bottom => {
                        if (rect.bottom() - self.rect.bottom()).abs() < threshold {
                            Some(Rect::from_min_size(
                                pos2(rect.left(), self.rect.bottom() - rect.height()),
                                rect.size(),
                            ))
                        } else {
                            None
                        }
                    }
                }
            }
            SnapZoneType::PanelEdge(_, edge) => {
                // Similar logic for panel edges
                match edge {
                    Edge::Left => {
                        if (rect.right() - self.rect.left()).abs() < threshold {
                            Some(Rect::from_min_size(
                                pos2(self.rect.left() - rect.width(), rect.top()),
                                rect.size(),
                            ))
                        } else {
                            None
                        }
                    }
                    Edge::Right => {
                        if (rect.left() - self.rect.right()).abs() < threshold {
                            Some(Rect::from_min_size(
                                pos2(self.rect.right(), rect.top()),
                                rect.size(),
                            ))
                        } else {
                            None
                        }
                    }
                    Edge::Top => {
                        if (rect.bottom() - self.rect.top()).abs() < threshold {
                            Some(Rect::from_min_size(
                                pos2(rect.left(), self.rect.top() - rect.height()),
                                rect.size(),
                            ))
                        } else {
                            None
                        }
                    }
                    Edge::Bottom => {
                        if (rect.top() - self.rect.bottom()).abs() < threshold {
                            Some(Rect::from_min_size(
                                pos2(rect.left(), self.rect.bottom()),
                                rect.size(),
                            ))
                        } else {
                            None
                        }
                    }
                }
            }
            SnapZoneType::Grid(grid_size) => {
                // Snap to grid
                let snapped_x = (rect.left() / grid_size).round() * grid_size;
                let snapped_y = (rect.top() / grid_size).round() * grid_size;
                Some(Rect::from_min_size(
                    pos2(snapped_x, snapped_y),
                    rect.size(),
                ))
            }
        }
    }
    
    /// Draw snap guide when active
    pub fn draw_guide(&self, painter: &Painter, active: bool) {
        if !active {
            return;
        }
        
        let color = Color32::from_rgba_unmultiplied(100, 200, 255, 150);
        let stroke = Stroke::new(2.0, color);
        
        match &self.zone_type {
            SnapZoneType::WindowEdge(edge) | SnapZoneType::PanelEdge(_, edge) => {
                match edge {
                    Edge::Left | Edge::Right => {
                        // Vertical line
                        let x = if matches!(edge, Edge::Left) { self.rect.left() } else { self.rect.right() };
                        painter.line_segment(
                            [pos2(x, self.rect.top()), pos2(x, self.rect.bottom())],
                            stroke,
                        );
                    }
                    Edge::Top | Edge::Bottom => {
                        // Horizontal line
                        let y = if matches!(edge, Edge::Top) { self.rect.top() } else { self.rect.bottom() };
                        painter.line_segment(
                            [pos2(self.rect.left(), y), pos2(self.rect.right(), y)],
                            stroke,
                        );
                    }
                }
            }
            SnapZoneType::Grid(_) => {
                // Grid pattern would be drawn differently
            }
        }
    }
}

/// Tab types for different panels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TabType {
    Timeline,
    Layers,
    Properties,
    Tools,
    Stage,
    Library,
    Actions,
    Components,
    Custom(String),
}

impl TabType {
    pub fn title(&self) -> &str {
        match self {
            TabType::Timeline => "Timeline",
            TabType::Layers => "Layers",
            TabType::Properties => "Properties",
            TabType::Tools => "Tools",
            TabType::Stage => "Stage",
            TabType::Library => "Library",
            TabType::Actions => "Actions",
            TabType::Components => "Components",
            TabType::Custom(name) => name,
        }
    }
}

/// Main dock manager that handles panel layout using egui_dock
pub struct DockManager {
    dock_state: DockState<TabType>,
    show_window_menu: bool,
    snap_zones: Vec<SnapZone>,
    active_snap_zones: Vec<usize>,
    snap_threshold: f32,
    show_snap_guides: bool,
}

impl DockManager {
    pub fn new() -> Self {
        // Create default Flash-style layout
        let mut dock_state = DockState::new(vec![TabType::Stage]);
        
        // Create the tree structure for Flash-style layout
        let [stage_node, _] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.2,
            vec![TabType::Tools],
        );
        
        let [_stage, right_panel] = dock_state.main_surface_mut().split_right(
            stage_node,
            0.7,
            vec![TabType::Layers],
        );
        
        let [_layers, properties] = dock_state.main_surface_mut().split_below(
            right_panel,
            0.6,
            vec![TabType::Properties],
        );
        
        // Add library tab to properties
        dock_state.main_surface_mut().push_to_focused_leaf(TabType::Library);
        dock_state.main_surface_mut().set_focused_node(properties);
        
        // Timeline at bottom
        let [_, _timeline] = dock_state.main_surface_mut().split_below(
            NodeIndex::root(),
            0.7,
            vec![TabType::Timeline],
        );
        
        Self {
            dock_state,
            show_window_menu: false,
            snap_zones: Vec::new(),
            active_snap_zones: Vec::new(),
            snap_threshold: 10.0,
            show_snap_guides: true,
        }
    }
    
    /// Reset to default Flash-style layout
    pub fn reset_layout(&mut self) {
        *self = Self::new();
    }
    
    /// Save current layout
    pub fn save_layout(&self) -> String {
        // TODO: Implement proper serialization when egui_dock supports it
        "{}".to_string()
    }
    
    /// Load layout from JSON
    pub fn load_layout(&mut self, _json: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement proper deserialization when egui_dock supports it
        Ok(())
    }
    
    /// Add a new tab
    pub fn add_tab(&mut self, tab: TabType) {
        self.dock_state.push_to_focused_leaf(tab);
    }
    
    /// Remove a tab
    pub fn remove_tab(&mut self, tab: &TabType) -> bool {
        self.dock_state.find_tab(tab).is_some()
    }
    
    /// Update snap zones based on current window
    pub fn update_snap_zones(&mut self, ctx: &Context) {
        self.snap_zones.clear();
        let window_rect = ctx.available_rect();
        
        // Window edges
        self.snap_zones.push(SnapZone {
            rect: Rect::from_min_size(window_rect.min, vec2(5.0, window_rect.height())),
            zone_type: SnapZoneType::WindowEdge(Edge::Left),
            strength: 15.0,
        });
        
        self.snap_zones.push(SnapZone {
            rect: Rect::from_min_size(
                pos2(window_rect.max.x - 5.0, window_rect.min.y),
                vec2(5.0, window_rect.height()),
            ),
            zone_type: SnapZoneType::WindowEdge(Edge::Right),
            strength: 15.0,
        });
        
        self.snap_zones.push(SnapZone {
            rect: Rect::from_min_size(window_rect.min, vec2(window_rect.width(), 5.0)),
            zone_type: SnapZoneType::WindowEdge(Edge::Top),
            strength: 15.0,
        });
        
        self.snap_zones.push(SnapZone {
            rect: Rect::from_min_size(
                pos2(window_rect.min.x, window_rect.max.y - 5.0),
                vec2(window_rect.width(), 5.0),
            ),
            zone_type: SnapZoneType::WindowEdge(Edge::Bottom),
            strength: 15.0,
        });
        
        // Add grid snap zone
        self.snap_zones.push(SnapZone {
            rect: window_rect,
            zone_type: SnapZoneType::Grid(25.0),
            strength: 5.0,
        });
    }
    
    /// Check snap zones for a dragged rect
    pub fn check_snap(&mut self, rect: Rect) -> Option<Rect> {
        self.active_snap_zones.clear();
        
        for (i, zone) in self.snap_zones.iter().enumerate() {
            if let Some(snapped_rect) = zone.should_snap(rect, self.snap_threshold) {
                self.active_snap_zones.push(i);
                return Some(snapped_rect);
            }
        }
        
        None
    }
    
    /// Draw active snap guides
    fn draw_snap_guides(&self, ctx: &Context) {
        if !self.show_snap_guides || self.active_snap_zones.is_empty() {
            return;
        }
        
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("snap_guides")));
        
        for &zone_idx in &self.active_snap_zones {
            if let Some(zone) = self.snap_zones.get(zone_idx) {
                zone.draw_guide(&painter, true);
            }
        }
    }
    
    /// Show the dock area
    pub fn show(&mut self, ctx: &Context, tab_viewer: &mut impl TabViewer<Tab = TabType>) {
        // Update snap zones
        self.update_snap_zones(ctx);
        // Top bar for workspace management
        TopBottomPanel::top("dock_menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Window", |ui| {
                    if ui.button("Reset Layout").clicked() {
                        self.reset_layout();
                        ui.close();
                    }
                    
                    ui.separator();
                    
                    if ui.button("New Timeline Tab").clicked() {
                        self.add_tab(TabType::Timeline);
                        ui.close();
                    }
                    
                    if ui.button("New Layers Tab").clicked() {
                        self.add_tab(TabType::Layers);
                        ui.close();
                    }
                    
                    if ui.button("New Properties Tab").clicked() {
                        self.add_tab(TabType::Properties);
                        ui.close();
                    }
                    
                    if ui.button("New Actions Tab").clicked() {
                        self.add_tab(TabType::Actions);
                        ui.close();
                    }
                });
                
                ui.separator();
                
                if ui.button("Save Layout").clicked() {
                    let json = self.save_layout();
                    // In a real app, save to file or preferences
                    println!("Layout saved: {}", json);
                }
                
                if ui.button("Load Layout").clicked() {
                    // In a real app, load from file dialog
                    println!("Load layout not implemented");
                }
            });
        });
        
        // Show the dock area
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, tab_viewer);
        
        // Draw snap guides after dock area
        self.draw_snap_guides(ctx);
        
        // Handle keyboard shortcuts for snap
        if ctx.input(|i| i.key_pressed(Key::G) && i.modifiers.ctrl) {
            self.show_snap_guides = !self.show_snap_guides;
        }
    }
}

/// Tab viewer for Flash-style panels
pub struct FlashTabViewer<'a> {
    pub timeline: &'a mut crate::Timeline,
    pub engine: &'a mut Box<dyn crate::RiveEngine>,
    pub selected_layer: Option<crate::LayerId>,
    pub selected_frame: Option<u32>,
}

impl<'a> TabViewer for FlashTabViewer<'a> {
    type Tab = TabType;
    
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title().into()
    }
    
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            TabType::Timeline => {
                // Use the actual timeline widget
                self.timeline.show(ui, self.engine);
            }
            
            TabType::Layers => {
                self.show_layers_panel(ui);
            }
            
            TabType::Properties => {
                self.show_properties_panel(ui);
            }
            
            TabType::Tools => {
                self.show_tools_panel(ui);
            }
            
            TabType::Stage => {
                self.show_stage_panel(ui);
            }
            
            TabType::Library => {
                self.show_library_panel(ui);
            }
            
            TabType::Actions => {
                self.show_actions_panel(ui);
            }
            
            TabType::Components => {
                self.show_components_panel(ui);
            }
            
            TabType::Custom(name) => {
                ui.label(format!("Custom panel: {}", name));
            }
        }
    }
    
    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        // All tabs can be closed except Stage
        !matches!(tab, TabType::Stage)
    }
    
    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        // All tabs can be undocked into separate windows
        true
    }
}

impl<'a> FlashTabViewer<'a> {
    fn show_layers_panel(&mut self, ui: &mut Ui) {
        ui.heading("Layers");
        ui.separator();
        
        ScrollArea::vertical().show(ui, |ui| {
            let layers = self.engine.get_layers();
            for (_i, layer) in layers.iter().enumerate() {
                let selected = self.selected_layer.as_ref() == Some(&layer.id);
                if ui.selectable_label(selected, &layer.name).clicked() {
                    self.selected_layer = Some(layer.id.clone());
                }
                
                // Show layer type icon inline
                let icon = match layer.layer_type {
                    crate::LayerType::Normal => "📄",
                    crate::LayerType::Folder => "📁",
                    crate::LayerType::Guide => "📍",
                    crate::LayerType::Mask => "🎭",
                    crate::LayerType::Audio => "🔊",
                    crate::LayerType::MotionGuide => "🎯",
                };
                ui.label(format!("{} {}", icon, &layer.name));
            }
        });
    }
    
    fn show_properties_panel(&mut self, ui: &mut Ui) {
        ui.heading("Properties");
        ui.separator();
        
        if let Some(layer_id) = &self.selected_layer {
            ui.label(format!("Layer ID: {:?}", layer_id));
            
            // Frame properties
            if let Some(frame) = self.selected_frame {
                ui.separator();
                ui.label(format!("Frame: {}", frame));
                
                // Show frame properties
                if ui.button("Insert Keyframe").clicked() {
                    // Insert keyframe action
                }
                
                if ui.button("Clear Keyframe").clicked() {
                    // Clear keyframe action
                }
            }
        } else {
            ui.label("No selection");
        }
    }
    
    fn show_tools_panel(&mut self, ui: &mut Ui) {
        ui.heading("Tools");
        ui.separator();
        
        let tool_size = vec2(32.0, 32.0);
        let tools = [
            ("✏", "Selection Tool"),
            ("⬛", "Rectangle Tool"),
            ("○", "Oval Tool"),
            ("▶", "Polygon Tool"),
            ("T", "Text Tool"),
            ("✋", "Hand Tool"),
            ("🔍", "Zoom Tool"),
            ("💧", "Paint Bucket"),
            ("🖌", "Brush Tool"),
            ("✨", "Eraser Tool"),
        ];
        
        ui.horizontal_wrapped(|ui| {
            for (icon, tooltip) in tools {
                if ui.add_sized(tool_size, Button::new(icon))
                    .on_hover_text(tooltip)
                    .clicked() 
                {
                    // Tool selection logic
                }
            }
        });
    }
    
    fn show_stage_panel(&mut self, ui: &mut Ui) {
        // Central panel for main content
        Frame::canvas(ui.style())
            .fill(Color32::from_gray(30))
            .show(ui, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.heading("Stage / Canvas");
                    ui.label("Main content area");
                    ui.label("(This would show your animation content)");
                });
            });
    }
    
    fn show_library_panel(&mut self, ui: &mut Ui) {
        ui.heading("Library");
        ui.separator();
        
        ui.label("Symbols:");
        ScrollArea::vertical().show(ui, |ui| {
            for i in 0..10 {
                let _ = ui.selectable_label(false, format!("Symbol {}", i + 1));
            }
        });
    }
    
    fn show_actions_panel(&mut self, ui: &mut Ui) {
        ui.heading("Actions");
        ui.separator();
        
        ui.label("ActionScript editor would go here");
        
        ScrollArea::vertical().show(ui, |ui| {
            ui.code_editor(&mut String::from(
                "// Frame actions\nstop();\n\n// Add your ActionScript here"
            ));
        });
    }
    
    fn show_components_panel(&mut self, ui: &mut Ui) {
        ui.heading("Components");
        ui.separator();
        
        let components = ["Button", "CheckBox", "ComboBox", "List", "RadioButton", "ScrollBar", "Slider", "TextArea", "TextInput"];
        
        for component in &components {
            if ui.button(*component).clicked() {
                // Add component to stage
            }
        }
    }
}