//! Simple integration tests that verify basic functionality
//!
//! These tests focus on verifying that the integration compiles and
//! basic methods work without complex interactions.

use crate::artboard_renderer::{ArtboardRenderer, rustflash_types::*};
use crate::rustflash_integration::RustFlashIntegration;

#[test]
fn test_artboard_renderer_creation() {
    let renderer = ArtboardRenderer::new();
    assert_eq!(renderer.scale, 1.0);
    assert_eq!(renderer.debug_mode, false);
}

#[test]
fn test_artboard_renderer_configuration() {
    let renderer = ArtboardRenderer::new()
        .with_scale(2.0)
        .with_debug(true);
    
    assert_eq!(renderer.scale, 2.0);
    assert_eq!(renderer.debug_mode, true);
}

#[test]
fn test_rustflash_integration_creation() {
    let _integration = RustFlashIntegration::new();
    // Basic creation should not panic
}

#[test]
fn test_rive_artboard_creation() {
    let artboard = RiveArtboard::new("test".to_string());
    assert_eq!(artboard.name, "test");
    assert!(artboard.paths.is_empty());
}

#[test]
fn test_path_creation() {
    let path = RivePath {
        commands: vec![
            PathCommand::MoveTo { x: 0.0, y: 0.0 },
            PathCommand::LineTo { x: 100.0, y: 0.0 },
            PathCommand::Close,
        ],
        fill: Some(PathFill { color: 0xFF0000, alpha: 1.0 }),
        stroke: None,
        bounds: Rectangle::new(0.0, 0.0, 100.0, 50.0),
    };
    
    assert_eq!(path.commands.len(), 3);
    assert!(path.fill.is_some());
    assert!(path.stroke.is_none());
}

#[test]
fn test_color_values() {
    let fill = PathFill { color: 0xFF0000, alpha: 0.5 };
    assert_eq!(fill.color, 0xFF0000);
    assert_eq!(fill.alpha, 0.5);
    
    let stroke = PathStroke { width: 2.0, color: 0x00FF00, alpha: 1.0 };
    assert_eq!(stroke.width, 2.0);
    assert_eq!(stroke.color, 0x00FF00);
    assert_eq!(stroke.alpha, 1.0);
}

#[test] 
fn test_rectangle_creation() {
    let rect = Rectangle::new(10.0, 20.0, 100.0, 50.0);
    assert_eq!(rect.x, 10.0);
    assert_eq!(rect.y, 20.0);
    assert_eq!(rect.width, 100.0);
    assert_eq!(rect.height, 50.0);
}

#[test]
fn test_path_commands() {
    let move_to = PathCommand::MoveTo { x: 10.0, y: 20.0 };
    let line_to = PathCommand::LineTo { x: 30.0, y: 40.0 };
    let cubic_to = PathCommand::CubicTo { 
        cp1x: 10.0, cp1y: 20.0,
        cp2x: 20.0, cp2y: 30.0,
        x: 30.0, y: 40.0 
    };
    let quad_to = PathCommand::QuadTo { 
        cpx: 15.0, cpy: 25.0,
        x: 30.0, y: 40.0 
    };
    let close = PathCommand::Close;
    
    // Verify they can be created and match patterns
    match move_to {
        PathCommand::MoveTo { x, y } => {
            assert_eq!(x, 10.0);
            assert_eq!(y, 20.0);
        }
        _ => panic!("Wrong variant"),
    }
    
    match line_to {
        PathCommand::LineTo { x, y } => {
            assert_eq!(x, 30.0);
            assert_eq!(y, 40.0);
        }
        _ => panic!("Wrong variant"),
    }
    
    match cubic_to {
        PathCommand::CubicTo { cp1x, cp1y, cp2x, cp2y, x, y } => {
            assert_eq!(cp1x, 10.0);
            assert_eq!(cp1y, 20.0);
            assert_eq!(cp2x, 20.0);
            assert_eq!(cp2y, 30.0);
            assert_eq!(x, 30.0);
            assert_eq!(y, 40.0);
        }
        _ => panic!("Wrong variant"),
    }
    
    match quad_to {
        PathCommand::QuadTo { cpx, cpy, x, y } => {
            assert_eq!(cpx, 15.0);
            assert_eq!(cpy, 25.0);
            assert_eq!(x, 30.0);
            assert_eq!(y, 40.0);
        }
        _ => panic!("Wrong variant"),
    }
    
    match close {
        PathCommand::Close => {},
        _ => panic!("Wrong variant"),
    }
}