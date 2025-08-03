//! Docking system for panels

use egui::*;
use serde::{Deserialize, Serialize};
use super::PanelId;

/// Docking position relative to screen or another panel
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DockPosition {
    Left,
    Right,
    Top,
    Bottom,
    Center, // For tabbing
}

/// State of a panel (floating, docked, or grouped)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DockState {
    Floating,
    Docked {
        position: DockPosition,
        parent: Option<PanelId>, // None means docked to main window
        size_ratio: f32, // Portion of parent size (0.0 to 1.0)
    },
    Grouped {
        group_id: String,
        tab_index: usize,
    },
}

/// A docking zone that panels can be dropped into
#[derive(Clone, Debug)]
pub struct DockZone {
    pub rect: Rect,
    pub position: DockPosition,
    pub parent: Option<PanelId>,
    pub highlight_rect: Rect, // Visual feedback area
}

impl DockZone {
    /// Check if a point is within this dock zone
    pub fn contains(&self, pos: Pos2) -> bool {
        self.rect.contains(pos)
    }
    
    /// Draw dock zone highlight
    pub fn draw_highlight(&self, painter: &Painter, active: bool) {
        let color = if active {
            Color32::from_rgba_unmultiplied(100, 150, 255, 120)
        } else {
            Color32::from_rgba_unmultiplied(100, 150, 255, 40)
        };
        
        painter.rect_filled(self.highlight_rect, 4.0, color);
        
        if active {
            painter.rect_stroke(
                self.highlight_rect,
                4.0,
                Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
                egui::epaint::StrokeKind::Outside,
            );
        }
    }
    
    /// Get icon for dock position
    pub fn get_icon(&self) -> &'static str {
        match self.position {
            DockPosition::Left => "⬅",
            DockPosition::Right => "➡",
            DockPosition::Top => "⬆",
            DockPosition::Bottom => "⬇",
            DockPosition::Center => "⊞",
        }
    }
}