//! Snapping system for panel alignment

use egui::*;
use serde::{Deserialize, Serialize};

/// Snap grid for aligning panels
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapGrid {
    pub enabled: bool,
    pub size: f32,
    pub snap_distance: f32,
    pub show_grid: bool,
}

impl Default for SnapGrid {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 10.0,
            snap_distance: 10.0,
            show_grid: false,
        }
    }
}

impl SnapGrid {
    /// Snap a position to the grid
    pub fn snap_position(&self, pos: Pos2) -> Pos2 {
        if !self.enabled {
            return pos;
        }
        
        pos2(
            (pos.x / self.size).round() * self.size,
            (pos.y / self.size).round() * self.size,
        )
    }
    
    /// Draw the grid
    pub fn draw(&self, painter: &Painter, rect: Rect) {
        if !self.show_grid || !self.enabled {
            return;
        }
        
        let grid_color = Color32::from_rgba_unmultiplied(100, 100, 100, 20);
        
        // Vertical lines
        let mut x = rect.min.x;
        while x <= rect.max.x {
            painter.line_segment(
                [pos2(x, rect.min.y), pos2(x, rect.max.y)],
                Stroke::new(1.0, grid_color),
            );
            x += self.size;
        }
        
        // Horizontal lines
        let mut y = rect.min.y;
        while y <= rect.max.y {
            painter.line_segment(
                [pos2(rect.min.x, y), pos2(rect.max.x, y)],
                Stroke::new(1.0, grid_color),
            );
            y += self.size;
        }
    }
}

/// A snap guide line
#[derive(Clone, Debug)]
pub enum SnapGuide {
    Vertical(f32),
    Horizontal(f32),
}

impl SnapGuide {
    /// Draw the snap guide
    pub fn draw(&self, ctx: &Context, window_rect: Rect) {
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("snap_guides")));
        let guide_color = Color32::from_rgb(100, 150, 255);
        let guide_stroke = Stroke::new(1.0, guide_color);
        
        match self {
            SnapGuide::Vertical(x) => {
                painter.line_segment(
                    [pos2(*x, window_rect.min.y), pos2(*x, window_rect.max.y)],
                    guide_stroke,
                );
                
                // Draw measurement
                let label = format!("{:.0}px", x);
                painter.text(
                    pos2(*x + 5.0, window_rect.center().y),
                    Align2::LEFT_CENTER,
                    label,
                    FontId::default(),
                    guide_color,
                );
            },
            SnapGuide::Horizontal(y) => {
                painter.line_segment(
                    [pos2(window_rect.min.x, *y), pos2(window_rect.max.x, *y)],
                    guide_stroke,
                );
                
                // Draw measurement
                let label = format!("{:.0}px", y);
                painter.text(
                    pos2(window_rect.center().x, *y - 5.0),
                    Align2::CENTER_BOTTOM,
                    label,
                    FontId::default(),
                    guide_color,
                );
            },
        }
    }
}