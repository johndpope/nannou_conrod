//! Artboard Renderer - Converts RustFlash RiveArtboard to egui painting commands
//!
//! This module bridges the gap between RustFlash's Rive-based rendering system
//! and egui's immediate mode GUI painter, allowing RiveArtboard content to be
//! displayed in the timeline stage canvas.

use egui::{Painter, Color32, Stroke, Pos2, Shape, Rect};
use std::f32::consts::PI;

/// Mock RustFlash types for integration (replace with actual imports when available)
pub mod rustflash_types {
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone)]
    pub struct RiveArtboard {
        pub name: String,
        pub paths: Vec<RivePath>,
        pub bounds: Rectangle,
    }
    
    impl RiveArtboard {
        pub fn new(name: String) -> Self {
            Self {
                name,
                paths: Vec::new(),
                bounds: Rectangle::new(0.0, 0.0, 800.0, 600.0),
            }
        }
        
        pub fn add_path(&mut self, path: RivePath) {
            self.paths.push(path);
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct RivePath {
        pub commands: Vec<PathCommand>,
        pub fill: Option<PathFill>,
        pub stroke: Option<PathStroke>,
        pub bounds: Rectangle,
    }
    
    #[derive(Debug, Clone)]
    pub enum PathCommand {
        MoveTo { x: f32, y: f32 },
        LineTo { x: f32, y: f32 },
        CubicTo { cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32 },
        QuadTo { cpx: f32, cpy: f32, x: f32, y: f32 },
        Close,
    }
    
    #[derive(Debug, Clone)]
    pub struct PathFill {
        pub color: u32,
        pub alpha: f32,
    }
    
    #[derive(Debug, Clone)]
    pub struct PathStroke {
        pub width: f32,
        pub color: u32,
        pub alpha: f32,
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub struct Rectangle {
        pub x: f32,
        pub y: f32,
        pub width: f32,
        pub height: f32,
    }
    
    impl Rectangle {
        pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
            Self { x, y, width, height }
        }
    }
}

pub use rustflash_types::*;

/// Converts RustFlash artboard content to egui painting commands
pub struct ArtboardRenderer {
    /// Scale factor for rendering
    pub scale: f32,
    /// Offset for positioning
    pub offset: Pos2,
    /// Debug mode for showing additional info
    pub debug_mode: bool,
}

impl ArtboardRenderer {
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            offset: Pos2::ZERO,
            debug_mode: false,
        }
    }
    
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
    
    pub fn with_offset(mut self, offset: Pos2) -> Self {
        self.offset = offset;
        self
    }
    
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug_mode = debug;
        self
    }
    
    /// Main rendering function - converts artboard to egui shapes
    pub fn render_artboard(&self, painter: &Painter, artboard: &RiveArtboard, canvas_rect: Rect) {
        // Clear the canvas with a dark background
        painter.rect_filled(canvas_rect, 0.0, Color32::from_gray(25));
        
        if self.debug_mode {
            // Draw debug info
            painter.text(
                canvas_rect.left_top() + egui::vec2(10.0, 10.0),
                egui::Align2::LEFT_TOP,
                format!("Artboard: {} ({} paths)", artboard.name, artboard.paths.len()),
                egui::FontId::monospace(12.0),
                Color32::WHITE,
            );
        }
        
        // Render each path in the artboard
        for (i, path) in artboard.paths.iter().enumerate() {
            self.render_path(painter, path, canvas_rect);
            
            if self.debug_mode && i < 5 {
                // Show path info for first few paths
                painter.text(
                    canvas_rect.left_top() + egui::vec2(10.0, 30.0 + i as f32 * 15.0),
                    egui::Align2::LEFT_TOP,
                    format!("Path {}: {} commands", i, path.commands.len()),
                    egui::FontId::monospace(10.0),
                    Color32::LIGHT_GRAY,
                );
            }
        }
        
        // Draw border around canvas
        painter.rect_stroke(canvas_rect, 0.0, Stroke::new(1.0, Color32::GRAY), egui::epaint::StrokeKind::Outside);
    }
    
    /// Renders a single RivePath to egui shapes
    fn render_path(&self, painter: &Painter, path: &RivePath, canvas_rect: Rect) {
        if path.commands.is_empty() {
            return;
        }
        
        // Convert path commands to egui points
        let points = self.path_commands_to_egui_points(&path.commands, canvas_rect);
        
        if points.is_empty() {
            return;
        }
        
        // Handle filled paths
        if let Some(fill) = &path.fill {
            let fill_color = self.convert_color(fill.color, fill.alpha);
            
            if points.len() >= 3 {
                // Create a filled polygon
                let shape = Shape::convex_polygon(points.clone(), fill_color, Stroke::NONE);
                painter.add(shape);
            } else if points.len() == 2 {
                // For lines, draw a thick stroke instead
                painter.line_segment([points[0], points[1]], Stroke::new(2.0, fill_color));
            }
        }
        
        // Handle stroked paths
        if let Some(stroke) = &path.stroke {
            let stroke_color = self.convert_color(stroke.color, stroke.alpha);
            let stroke_width = stroke.width * self.scale;
            
            if points.len() >= 2 {
                // Draw stroke as connected line segments
                let path_stroke = Stroke::new(stroke_width, stroke_color);
                for window in points.windows(2) {
                    painter.line_segment([window[0], window[1]], path_stroke);
                }
            }
        }
    }
    
    /// Converts path commands to egui screen coordinates
    fn path_commands_to_egui_points(&self, commands: &[PathCommand], canvas_rect: Rect) -> Vec<Pos2> {
        let mut points = Vec::new();
        let mut current_pos = Pos2::ZERO;
        
        for command in commands {
            match command {
                PathCommand::MoveTo { x, y } => {
                    current_pos = self.transform_point(*x, *y, canvas_rect);
                    points.push(current_pos);
                }
                PathCommand::LineTo { x, y } => {
                    current_pos = self.transform_point(*x, *y, canvas_rect);
                    points.push(current_pos);
                }
                PathCommand::CubicTo { cp1x, cp1y, cp2x, cp2y, x, y } => {
                    // Sample cubic bezier curve into line segments
                    let end_pos = self.transform_point(*x, *y, canvas_rect);
                    let cp1 = self.transform_point(*cp1x, *cp1y, canvas_rect);
                    let cp2 = self.transform_point(*cp2x, *cp2y, canvas_rect);
                    
                    let bezier_points = self.sample_cubic_bezier(current_pos, cp1, cp2, end_pos, 10);
                    points.extend(bezier_points.iter().skip(1)); // Skip first point (already added)
                    current_pos = end_pos;
                }
                PathCommand::QuadTo { cpx, cpy, x, y } => {
                    // Sample quadratic bezier curve into line segments
                    let end_pos = self.transform_point(*x, *y, canvas_rect);
                    let cp = self.transform_point(*cpx, *cpy, canvas_rect);
                    
                    let bezier_points = self.sample_quadratic_bezier(current_pos, cp, end_pos, 8);
                    points.extend(bezier_points.iter().skip(1)); // Skip first point (already added)
                    current_pos = end_pos;
                }
                PathCommand::Close => {
                    // Close the path by connecting back to the first point
                    if !points.is_empty() {
                        points.push(points[0]);
                    }
                }
            }
        }
        
        points
    }
    
    /// Transform artboard coordinates to screen coordinates
    fn transform_point(&self, x: f32, y: f32, canvas_rect: Rect) -> Pos2 {
        let scaled_x = x * self.scale + self.offset.x;
        let scaled_y = y * self.scale + self.offset.y;
        
        Pos2::new(
            canvas_rect.left() + scaled_x,
            canvas_rect.top() + scaled_y,
        )
    }
    
    /// Sample a cubic bezier curve into line segments
    fn sample_cubic_bezier(&self, p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, segments: usize) -> Vec<Pos2> {
        let mut points = Vec::with_capacity(segments + 1);
        
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let point = self.cubic_bezier_point(p0, p1, p2, p3, t);
            points.push(point);
        }
        
        points
    }
    
    /// Sample a quadratic bezier curve into line segments
    fn sample_quadratic_bezier(&self, p0: Pos2, p1: Pos2, p2: Pos2, segments: usize) -> Vec<Pos2> {
        let mut points = Vec::with_capacity(segments + 1);
        
        for i in 0..=segments {
            let t = i as f32 / segments as f64 as f32;
            let point = self.quadratic_bezier_point(p0, p1, p2, t);
            points.push(point);
        }
        
        points
    }
    
    /// Calculate a point on a cubic bezier curve
    fn cubic_bezier_point(&self, p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;
        
        let x = uuu * p0.x + 3.0 * uu * t * p1.x + 3.0 * u * tt * p2.x + ttt * p3.x;
        let y = uuu * p0.y + 3.0 * uu * t * p1.y + 3.0 * u * tt * p2.y + ttt * p3.y;
        
        Pos2::new(x, y)
    }
    
    /// Calculate a point on a quadratic bezier curve
    fn quadratic_bezier_point(&self, p0: Pos2, p1: Pos2, p2: Pos2, t: f32) -> Pos2 {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        
        let x = uu * p0.x + 2.0 * u * t * p1.x + tt * p2.x;
        let y = uu * p0.y + 2.0 * u * t * p1.y + tt * p2.y;
        
        Pos2::new(x, y)
    }
    
    /// Convert RustFlash color (u32) and alpha to egui Color32
    fn convert_color(&self, color: u32, alpha: f32) -> Color32 {
        let r = ((color >> 16) & 0xFF) as u8;
        let g = ((color >> 8) & 0xFF) as u8; 
        let b = (color & 0xFF) as u8;
        let a = (alpha.clamp(0.0, 1.0) * 255.0) as u8;
        
        Color32::from_rgba_unmultiplied(r, g, b, a)
    }
    
    /// Render a test pattern when no artboard is available
    pub fn render_test_pattern(&self, painter: &Painter, canvas_rect: Rect, frame: u32) {
        // Clear background
        painter.rect_filled(canvas_rect, 0.0, Color32::from_gray(30));
        
        let center = canvas_rect.center();
        let time = frame as f32 * 0.1;
        
        // Animated background grid
        let grid_size = 50.0;
        let grid_color = Color32::from_rgba_unmultiplied(60, 60, 60, 100);
        
        let mut x = canvas_rect.left();
        while x <= canvas_rect.right() {
            painter.line_segment(
                [Pos2::new(x, canvas_rect.top()), Pos2::new(x, canvas_rect.bottom())],
                Stroke::new(1.0, grid_color),
            );
            x += grid_size;
        }
        
        let mut y = canvas_rect.top();
        while y <= canvas_rect.bottom() {
            painter.line_segment(
                [Pos2::new(canvas_rect.left(), y), Pos2::new(canvas_rect.right(), y)],
                Stroke::new(1.0, grid_color),
            );
            y += grid_size;
        }
        
        // Moving rectangle
        let rect_size = 40.0;
        let rect_x = center.x + (time * 2.0).sin() * 150.0 - rect_size / 2.0;
        let rect_y = center.y - 100.0 - rect_size / 2.0;
        let rect_color = Color32::from_rgb(255, 100, 100);
        
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(rect_x, rect_y), egui::Vec2::splat(rect_size)),
            5.0,
            rect_color,
        );
        
        // Pulsating circle
        let circle_radius = 30.0 + (time * 3.0).sin() * 10.0;
        let circle_color = Color32::from_rgb(100, 255, 100);
        
        painter.circle_filled(center, circle_radius, circle_color);
        
        // Orbiting smaller circle
        let orbit_radius = 80.0;
        let orbit_x = center.x + (time * 1.5).cos() * orbit_radius;
        let orbit_y = center.y + (time * 1.5).sin() * orbit_radius;
        let orbit_color = Color32::from_rgb(100, 100, 255);
        
        painter.circle_filled(Pos2::new(orbit_x, orbit_y), 15.0, orbit_color);
        
        // Frame counter
        painter.text(
            canvas_rect.left_top() + egui::vec2(10.0, 10.0),
            egui::Align2::LEFT_TOP,
            format!("Frame: {}", frame),
            egui::FontId::monospace(14.0),
            Color32::WHITE,
        );
        
        // Draw border
        painter.rect_stroke(canvas_rect, 0.0, Stroke::new(2.0, Color32::DARK_GRAY), egui::epaint::StrokeKind::Outside);
    }
}

impl Default for ArtboardRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_color_conversion() {
        let renderer = ArtboardRenderer::new();
        
        // Test red color
        let red = renderer.convert_color(0xFF0000, 1.0);
        assert_eq!(red, Color32::from_rgb(255, 0, 0));
        
        // Test with alpha
        let semi_red = renderer.convert_color(0xFF0000, 0.5);
        assert_eq!(semi_red, Color32::from_rgba_unmultiplied(255, 0, 0, 127));
        
        // Test blue color
        let blue = renderer.convert_color(0x0000FF, 1.0);
        assert_eq!(blue, Color32::from_rgb(0, 0, 255));
    }
    
    #[test]
    fn test_point_transformation() {
        let renderer = ArtboardRenderer::new()
            .with_scale(2.0)
            .with_offset(Pos2::new(10.0, 5.0));
        
        let canvas_rect = Rect::from_min_size(Pos2::new(100.0, 100.0), egui::Vec2::new(400.0, 300.0));
        let transformed = renderer.transform_point(50.0, 25.0, canvas_rect);
        
        // Expected: (50 * 2 + 10) + 100 = 210, (25 * 2 + 5) + 100 = 155
        assert_eq!(transformed, Pos2::new(210.0, 155.0));
    }
    
    #[test]
    fn test_bezier_calculations() {
        let renderer = ArtboardRenderer::new();
        
        // Test quadratic bezier
        let p0 = Pos2::new(0.0, 0.0);
        let p1 = Pos2::new(50.0, 100.0);
        let p2 = Pos2::new(100.0, 0.0);
        
        let start = renderer.quadratic_bezier_point(p0, p1, p2, 0.0);
        let middle = renderer.quadratic_bezier_point(p0, p1, p2, 0.5);
        let end = renderer.quadratic_bezier_point(p0, p1, p2, 1.0);
        
        assert_eq!(start, p0);
        assert_eq!(end, p2);
        assert_eq!(middle, Pos2::new(50.0, 50.0)); // Peak of the curve
    }
}