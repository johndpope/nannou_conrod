//! egui_kittest tests for artboard rendering functionality
//!
//! These tests verify that the ArtboardRenderer correctly converts RustFlash
//! RiveArtboard content into egui painting commands and renders properly.

use crate::artboard_renderer::{ArtboardRenderer, rustflash_types::*};
use egui::{Pos2, Vec2, Rect, Color32};
use egui_kittest::{Harness, kittest::Queryable};

/// Test basic artboard renderer initialization
#[test]
fn test_artboard_renderer_creation() {
    let renderer = ArtboardRenderer::new();
    assert_eq!(renderer.scale, 1.0);
    assert_eq!(renderer.offset, Pos2::ZERO);
    assert_eq!(renderer.debug_mode, false);
}

/// Test artboard renderer with custom configuration
#[test]
fn test_artboard_renderer_configuration() {
    let renderer = ArtboardRenderer::new()
        .with_scale(2.0)
        .with_offset(Pos2::new(10.0, 20.0))
        .with_debug(true);
    
    assert_eq!(renderer.scale, 2.0);
    assert_eq!(renderer.offset, Pos2::new(10.0, 20.0));
    assert_eq!(renderer.debug_mode, true);
}

/// Test color conversion from u32 to egui Color32
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
    
    // Test green color
    let green = renderer.convert_color(0x00FF00, 0.8);
    assert_eq!(green, Color32::from_rgba_unmultiplied(0, 255, 0, 204));
}

/// Test point transformation with scale and offset
#[test]
fn test_point_transformation() {
    let renderer = ArtboardRenderer::new()
        .with_scale(2.0)
        .with_offset(Pos2::new(10.0, 5.0));
    
    let canvas_rect = Rect::from_min_size(Pos2::new(100.0, 100.0), Vec2::new(400.0, 300.0));
    let transformed = renderer.transform_point(50.0, 25.0, canvas_rect);
    
    // Expected: (50 * 2 + 10) + 100 = 210, (25 * 2 + 5) + 100 = 155
    assert_eq!(transformed, Pos2::new(210.0, 155.0));
}

/// Test quadratic bezier curve calculations
#[test]
fn test_quadratic_bezier_calculations() {
    let renderer = ArtboardRenderer::new();
    
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

/// Test cubic bezier curve calculations
#[test]
fn test_cubic_bezier_calculations() {
    let renderer = ArtboardRenderer::new();
    
    let p0 = Pos2::new(0.0, 0.0);
    let p1 = Pos2::new(33.0, 0.0);
    let p2 = Pos2::new(67.0, 100.0);
    let p3 = Pos2::new(100.0, 100.0);
    
    let start = renderer.cubic_bezier_point(p0, p1, p2, p3, 0.0);
    let end = renderer.cubic_bezier_point(p0, p1, p2, p3, 1.0);
    
    assert_eq!(start, p0);
    assert_eq!(end, p3);
}

/// Test artboard rendering with egui_kittest
#[test]
fn test_artboard_rendering_integration() {
    let mut harness = Harness::new(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let renderer = ArtboardRenderer::new().with_debug(true);
            let canvas_rect = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(400.0, 300.0));
            
            // Create a simple test artboard with a rectangle
            let mut artboard = RiveArtboard::new("test_artboard".to_string());
            let rect_path = RivePath {
                commands: vec![
                    PathCommand::MoveTo { x: 50.0, y: 50.0 },
                    PathCommand::LineTo { x: 150.0, y: 50.0 },
                    PathCommand::LineTo { x: 150.0, y: 100.0 },
                    PathCommand::LineTo { x: 50.0, y: 100.0 },
                    PathCommand::Close,
                ],
                fill: Some(PathFill { color: 0xFF0000, alpha: 1.0 }),
                stroke: None,
                bounds: Rectangle::new(50.0, 50.0, 100.0, 50.0),
            };
            artboard.add_path(rect_path);
            
            renderer.render_artboard(ui.painter(), &artboard, canvas_rect);
            
            ui.label("Artboard rendered successfully");
        });
    });
    
    // Verify the UI rendered without panicking
    harness.run();
}

/// Test test pattern rendering
#[test]
fn test_pattern_rendering() {
    let mut harness = Harness::new(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let renderer = ArtboardRenderer::new();
            let canvas_rect = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(400.0, 300.0));
            
            // Render test pattern for frame 0
            renderer.render_test_pattern(ui.painter(), canvas_rect, 0);
            
            ui.label("Test pattern rendered for frame 0");
        });
    });
    
    harness.run();
}

/// Test artboard rendering with complex paths
#[test]
fn test_complex_path_rendering() {
    let mut harness = Harness::new(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let renderer = ArtboardRenderer::new();
            let canvas_rect = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(400.0, 300.0));
            
            // Create artboard with bezier curves
            let mut artboard = RiveArtboard::new("complex_artboard".to_string());
            let curve_path = RivePath {
                commands: vec![
                    PathCommand::MoveTo { x: 100.0, y: 200.0 },
                    PathCommand::CubicTo { 
                        cp1x: 100.0, cp1y: 100.0,
                        cp2x: 200.0, cp2y: 100.0,
                        x: 200.0, y: 200.0 
                    },
                    PathCommand::QuadTo { 
                        cpx: 250.0, cpy: 150.0,
                        x: 300.0, y: 200.0 
                    },
                ],
                fill: None,
                stroke: Some(PathStroke { width: 3.0, color: 0x00FF00, alpha: 1.0 }),
                bounds: Rectangle::new(100.0, 100.0, 200.0, 100.0),
            };
            artboard.add_path(curve_path);
            
            renderer.render_artboard(ui.painter(), &artboard, canvas_rect);
            
            ui.label("Complex bezier path rendered");
        });
    });
    
    harness.run();
}

/// Test artboard with multiple paths and mixed fill/stroke
#[test]
fn test_multi_path_artboard() {
    let mut harness = Harness::new(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let renderer = ArtboardRenderer::new();
            let canvas_rect = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(400.0, 300.0));
            
            let mut artboard = RiveArtboard::new("multi_path_artboard".to_string());
            
            // Add filled rectangle
            let filled_rect = RivePath {
                commands: vec![
                    PathCommand::MoveTo { x: 20.0, y: 20.0 },
                    PathCommand::LineTo { x: 80.0, y: 20.0 },
                    PathCommand::LineTo { x: 80.0, y: 60.0 },
                    PathCommand::LineTo { x: 20.0, y: 60.0 },
                    PathCommand::Close,
                ],
                fill: Some(PathFill { color: 0x0000FF, alpha: 0.7 }),
                stroke: None,
                bounds: Rectangle::new(20.0, 20.0, 60.0, 40.0),
            };
            artboard.add_path(filled_rect);
            
            // Add stroked circle approximation
            let circle_path = RivePath {
                commands: vec![
                    PathCommand::MoveTo { x: 150.0, y: 100.0 },
                    // Simplified circle with quadratic curves
                    PathCommand::QuadTo { cpx: 150.0, cpy: 70.0, x: 120.0, y: 70.0 },
                    PathCommand::QuadTo { cpx: 90.0, cpy: 70.0, x: 90.0, y: 100.0 },
                    PathCommand::QuadTo { cpx: 90.0, cpy: 130.0, x: 120.0, y: 130.0 },
                    PathCommand::QuadTo { cpx: 150.0, cpy: 130.0, x: 150.0, y: 100.0 },
                    PathCommand::Close,
                ],
                fill: Some(PathFill { color: 0xFFFF00, alpha: 0.5 }),
                stroke: Some(PathStroke { width: 2.0, color: 0xFF0000, alpha: 1.0 }),
                bounds: Rectangle::new(90.0, 70.0, 60.0, 60.0),
            };
            artboard.add_path(circle_path);
            
            renderer.render_artboard(ui.painter(), &artboard, canvas_rect);
            
            ui.label("Multi-path artboard with fills and strokes");
        });
    });
    
    harness.run();
}

/// Test renderer error handling with empty artboard
#[test]
fn test_empty_artboard_handling() {
    let mut harness = Harness::new(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let renderer = ArtboardRenderer::new();
            let canvas_rect = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(400.0, 300.0));
            
            // Create empty artboard
            let artboard = RiveArtboard::new("empty_artboard".to_string());
            
            // Should not panic or error
            renderer.render_artboard(ui.painter(), &artboard, canvas_rect);
            
            ui.label("Empty artboard handled gracefully");
        });
    });
    
    harness.run();
}

/// Test animated test pattern across multiple frames
#[test]
fn test_animated_test_pattern() {
    for frame in [0, 10, 25, 50, 100] {
        let mut harness = Harness::new(move |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let renderer = ArtboardRenderer::new();
                let canvas_rect = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(400.0, 300.0));
                
                renderer.render_test_pattern(ui.painter(), canvas_rect, frame);
                
                ui.label(format!("Test pattern frame {}", frame));
            });
        });
        
        harness.run();
    }
}