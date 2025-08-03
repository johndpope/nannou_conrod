//! Panel management for window system

use egui::*;
use serde::{Deserialize, Serialize};
use super::DockState;

/// Unique identifier for panels
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PanelId(pub String);

impl PanelId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Size of a panel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

/// Position in screen coordinates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// Content that can be displayed in a panel
pub trait PanelContent {
    /// Draw the panel content
    fn draw(&mut self, ui: &mut Ui);
    
    /// Get the title for the panel
    fn title(&self) -> &str;
    
    /// Get minimum size requirements
    fn min_size(&self) -> Vec2 {
        vec2(150.0, 100.0)
    }
}

/// A single panel in the window management system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Panel {
    pub id: PanelId,
    pub title: String,
    pub position: Position,
    pub size: Size,
    pub dock_state: DockState,
    pub is_visible: bool,
    pub is_collapsed: bool,
    pub min_size: Size,
    pub can_close: bool,
    pub z_order: i32,
    #[serde(skip)]
    pub is_being_dragged: bool,
}

impl Panel {
    pub fn new(id: PanelId, title: String) -> Self {
        Self {
            id,
            title,
            position: Position { x: 100.0, y: 100.0 },
            size: Size { width: 300.0, height: 200.0 },
            dock_state: DockState::Floating,
            is_visible: true,
            is_collapsed: false,
            min_size: Size { width: 150.0, height: 100.0 },
            can_close: true,
            z_order: 0,
            is_being_dragged: false,
        }
    }
    
    /// Get the panel's current rect based on its state
    pub fn get_rect(&self) -> Rect {
        Rect::from_min_size(
            pos2(self.position.x, self.position.y),
            vec2(self.size.width, self.size.height),
        )
    }
    
    /// Draw panel in floating mode
    pub fn draw_floating(&mut self, ctx: &Context) {
        let window_id = Id::new(&self.id.0);
        
        let mut is_open = self.is_visible;
        
        Window::new(&self.title)
            .id(window_id)
            .default_pos(pos2(self.position.x, self.position.y))
            .default_size(vec2(self.size.width, self.size.height))
            .min_size(vec2(self.min_size.width, self.min_size.height))
            .collapsible(true)
            .resizable(true)
            .movable(true)
            .open(&mut is_open)
            .show(ctx, |ui| {
                // Update position and size from window
                let window_rect = ui.min_rect();
                self.position.x = window_rect.min.x;
                self.position.y = window_rect.min.y;
                self.size.width = window_rect.width();
                self.size.height = window_rect.height();
                
                // Draw panel content
                self.draw_content(ui);
            });
        
        self.is_visible = is_open;
    }
    
    /// Draw panel in docked mode
    pub fn draw_docked(&mut self, ctx: &Context, area: Rect) {
        // Update position and size from docked area
        self.position.x = area.min.x;
        self.position.y = area.min.y;
        self.size.width = area.width();
        self.size.height = area.height();
        
        // Draw as a fixed area
        let id = Id::new(&self.id.0);
        let mut ui = Ui::new(
            ctx.clone(),
            id,
            UiBuilder::new().max_rect(area),
        );
        
        Frame::window(&ctx.style())
            .fill(ctx.style().visuals.window_fill())
            .show(&mut ui, |ui| {
                // Panel header
                ui.horizontal(|ui| {
                    ui.heading(&self.title);
                    
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // Close button
                        if self.can_close && ui.small_button("×").clicked() {
                            self.is_visible = false;
                        }
                        
                        // Undock button
                        if ui.small_button("⬜").on_hover_text("Undock panel").clicked() {
                            self.dock_state = DockState::Floating;
                        }
                    });
                });
                
                ui.separator();
                
                // Panel content
                self.draw_content(ui);
            });
    }
    
    /// Draw the panel's content
    fn draw_content(&self, ui: &mut Ui) {
        // Content based on panel ID
        match self.id.0.as_str() {
            "timeline" => {
                ui.label("Timeline content would go here");
                ui.separator();
                ui.label("🎬 Frames, layers, and keyframes");
            },
            "layers" => {
                ui.label("Layers panel");
                ui.separator();
                ScrollArea::vertical().show(ui, |ui| {
                    for i in 0..5 {
                        let _ = ui.selectable_label(i == 0, format!("Layer {}", i + 1));
                    }
                });
            },
            "properties" => {
                ui.label("Properties panel");
                ui.separator();
                ui.label("Object properties would appear here");
            },
            "tools" => {
                ui.vertical(|ui| {
                    ui.label("Tools");
                    ui.separator();
                    let tool_size = vec2(32.0, 32.0);
                    ui.horizontal_wrapped(|ui| {
                        for tool in ["✏", "⬛", "○", "▶", "T", "✋", "🔍", "💧"] {
                            if ui.add_sized(tool_size, Button::new(tool)).clicked() {
                                // Tool selection
                            }
                        }
                    });
                });
            },
            "stage" => {
                ui.label("Stage/Canvas");
                ui.separator();
                ui.label("Main content area would be here");
            },
            _ => {
                ui.label(format!("Panel: {}", self.id.0));
            },
        }
    }
}