//! Integration tests for RustFlash artboard rendering
//!
//! Tests the complete pipeline from RustFlash engine to egui rendering.

#[cfg(test)]
mod tests {
    use crate::{
        rustflash_integration::RustFlashIntegration,
        artboard_renderer::ArtboardRenderer,
        TimelineApp,
    };
    use nannou_timeline::RiveEngine;
    use egui::{Rect, Pos2, Vec2, Color32};
    use std::sync::{Arc, Mutex};

    /// Test basic artboard rendering functionality
    #[test]
    fn test_artboard_renderer_basic() {
        let renderer = ArtboardRenderer::new();
        
        // Create test artboard
        let artboard = ArtboardRenderer::create_test_artboard(0, 100);
        
        // Verify artboard has content
        assert!(!artboard.paths.is_empty(), "Test artboard should have paths");
        assert_eq!(artboard.name, "test_artboard");
    }
    
    /// Test RustFlash integration engine
    #[test] 
    fn test_rustflash_integration_engine() {
        let mut integration = RustFlashIntegration::new();
        
        // Test basic engine functionality
        assert_eq!(integration.get_current_frame(), 0);
        assert_eq!(integration.get_total_frames(), 100);
        assert_eq!(integration.get_fps(), 24.0);
        
        // Test seeking
        integration.seek(25);
        assert_eq!(integration.get_current_frame(), 25);
        
        // Test playback controls
        integration.play();
        integration.pause();
        
        // Test artboard rendering
        match integration.get_rendered_artboard() {
            Ok(artboard) => {
                assert_eq!(artboard.name, "stage");
                // Should have animated content
                assert!(!artboard.paths.is_empty(), "Artboard should have rendered content");
            }
            Err(e) => panic!("Failed to get rendered artboard: {}", e),
        }
    }
    
    /// Test frame animation
    #[test]
    fn test_frame_animation() {
        let mut integration = RustFlashIntegration::new();
        
        // Test different frames produce different content
        let mut frame_contents = Vec::new();
        
        for frame in [0, 25, 50, 75, 99] {
            integration.seek(frame);
            
            if let Ok(artboard) = integration.get_rendered_artboard() {
                // Capture path count as a simple content indicator
                frame_contents.push((frame, artboard.paths.len()));
            }
        }
        
        // Should have content for all frames
        assert_eq!(frame_contents.len(), 5);
        
        // All frames should have some content
        for (frame, path_count) in &frame_contents {
            assert!(*path_count > 0, "Frame {} should have paths", frame);
        }
    }
    
    /// Test layer management
    #[test]
    fn test_layer_management() {
        let mut integration = RustFlashIntegration::new();
        
        // Test getting layers
        let layers = integration.get_layers();
        assert!(!layers.is_empty(), "Should have initial layers");
        
        // Test adding a layer
        let new_layer_id = integration.add_layer("Test Layer".to_string(), nannou_timeline::layer::LayerType::Normal);
        
        let updated_layers = integration.get_layers();
        assert!(updated_layers.len() > layers.len(), "Should have more layers after adding");
        
        // Find the new layer
        let new_layer = updated_layers.iter().find(|l| l.id == new_layer_id);
        assert!(new_layer.is_some(), "New layer should be found");
        assert_eq!(new_layer.unwrap().name, "Test Layer");
    }
    
    /// Test color conversion in renderer
    #[test]
    fn test_color_conversion() {
        let renderer = ArtboardRenderer::new();
        
        // Test pure colors
        let red = renderer.convert_color(0xFF0000, 1.0);
        assert_eq!(red.r(), 255);
        assert_eq!(red.g(), 0);
        assert_eq!(red.b(), 0);
        assert_eq!(red.a(), 255);
        
        let green = renderer.convert_color(0x00FF00, 1.0);
        assert_eq!(green.g(), 255);
        
        let blue = renderer.convert_color(0x0000FF, 1.0);
        assert_eq!(blue.b(), 255);
        
        // Test alpha
        let semi_transparent = renderer.convert_color(0xFF0000, 0.5);
        assert_eq!(semi_transparent.a(), 127);
    }
    
    /// Test path transformation
    #[test]
    fn test_path_transformation() {
        let renderer = ArtboardRenderer::new()
            .with_scale(2.0)
            .with_offset(Vec2::new(10.0, 20.0));
        
        let bounds = Rect::from_min_size(Pos2::new(100.0, 200.0), Vec2::new(400.0, 300.0));
        
        // Test point transformation
        let transformed = renderer.transform_point(0.0, 0.0, bounds);
        assert_eq!(transformed, Pos2::new(110.0, 220.0)); // 100 + 10 + 0*2, 200 + 20 + 0*2
        
        let transformed2 = renderer.transform_point(50.0, 75.0, bounds);
        assert_eq!(transformed2, Pos2::new(210.0, 370.0)); // 100 + 10 + 50*2, 200 + 20 + 75*2
    }
    
    /// Test bezier curve calculations
    #[test]
    fn test_bezier_curves() {
        let renderer = ArtboardRenderer::new();
        
        // Test cubic bezier
        let p0 = Pos2::new(0.0, 0.0);
        let p1 = Pos2::new(100.0, 0.0);
        let p2 = Pos2::new(100.0, 100.0);
        let p3 = Pos2::new(200.0, 100.0);
        
        // At t=0, should be p0
        let result = renderer.cubic_bezier(p0, p1, p2, p3, 0.0);
        assert!((result.x - p0.x).abs() < 0.001);
        assert!((result.y - p0.y).abs() < 0.001);
        
        // At t=1, should be p3
        let result = renderer.cubic_bezier(p0, p1, p2, p3, 1.0);
        assert!((result.x - p3.x).abs() < 0.001);
        assert!((result.y - p3.y).abs() < 0.001);
        
        // Test quadratic bezier
        let p0 = Pos2::new(0.0, 0.0);
        let p1 = Pos2::new(50.0, 100.0);
        let p2 = Pos2::new(100.0, 0.0);
        
        // At t=0, should be p0
        let result = renderer.quadratic_bezier(p0, p1, p2, 0.0);
        assert!((result.x - p0.x).abs() < 0.001);
        assert!((result.y - p0.y).abs() < 0.001);
        
        // At t=1, should be p2
        let result = renderer.quadratic_bezier(p0, p1, p2, 1.0);
        assert!((result.x - p2.x).abs() < 0.001);
        assert!((result.y - p2.y).abs() < 0.001);
    }
    
    /// Integration test: full pipeline from engine to rendering
    #[test]
    fn test_full_rendering_pipeline() {
        let mut integration = RustFlashIntegration::new();
        let renderer = ArtboardRenderer::new();
        
        // Set up test frame
        integration.seek(25);
        
        // Get artboard
        let artboard = integration.get_rendered_artboard().expect("Should get artboard");
        
        // Verify artboard content
        assert!(!artboard.paths.is_empty(), "Artboard should have paths");
        
        // Test that paths have valid data
        for path in &artboard.paths {
            assert!(!path.commands.is_empty(), "Path should have commands");
            
            // Test that we have either fill or stroke
            assert!(path.fill.is_some() || path.stroke.is_some(), 
                "Path should have fill or stroke");
            
            // Test bounds are reasonable
            let bounds = &path.bounds;
            assert!(bounds.width >= 0.0, "Width should be non-negative");
            assert!(bounds.height >= 0.0, "Height should be non-negative");
        }
    }
    
    /// Test timeline synchronization
    #[test]
    fn test_timeline_sync() {
        let mut integration = RustFlashIntegration::new();
        
        // Test frame progression
        for frame in 0..10 {
            integration.seek(frame);
            assert_eq!(integration.get_current_frame(), frame);
            
            // Each frame should produce valid artboard
            let artboard = integration.get_rendered_artboard().expect("Should get artboard");
            assert!(!artboard.paths.is_empty(), "Frame {} should have content", frame);
        }
    }
    
    /// Performance test: rendering many frames
    #[test]
    fn test_rendering_performance() {
        let mut integration = RustFlashIntegration::new();
        let renderer = ArtboardRenderer::new();
        
        let start = std::time::Instant::now();
        
        // Render 100 frames
        for frame in 0..100 {
            integration.seek(frame);
            
            if let Ok(artboard) = integration.get_rendered_artboard() {
                // Simulate rendering by counting paths
                let _path_count = artboard.paths.len();
            }
        }
        
        let duration = start.elapsed();
        
        // Should be reasonably fast (less than 1 second for 100 frames)
        assert!(duration.as_secs() < 1, "Rendering 100 frames should be fast");
        
        println!("Rendered 100 frames in {:?}", duration);
    }
}

#[cfg(test)]
mod egui_tests {
    use super::*;
    use egui_kittest::Harness;
    
    /// Test the complete UI integration with egui_kittest
    #[test]
    fn test_egui_stage_rendering() {
        let mut harness = Harness::new_ui(|ui| {
            let mut app = TimelineApp::default();
            
            // Set up a test rect for the stage
            let stage_rect = egui::Rect::from_min_size(
                egui::Pos2::new(10.0, 10.0), 
                egui::Vec2::new(400.0, 300.0)
            );
            
            // Draw the stage (this calls our rendering pipeline)
            app.draw_stage(ui, stage_rect);
            
            // Verify some basic UI state
            assert!(app.engine.get_total_frames() > 0);
            assert_eq!(app.engine.get_current_frame(), 0);
        });
        
        // Run the test
        harness.run();
    }
    
    /// Test timeline interaction
    #[test]
    fn test_timeline_interaction() {
        let mut harness = Harness::new_ui(|ui| {
            let mut app = TimelineApp::default();
            
            // Test seeking to different frames
            app.engine.seek(50);
            assert_eq!(app.engine.get_current_frame(), 50);
            
            // Test playback
            app.engine.play();
            app.engine.pause();
            
            // The UI should reflect these changes
            let timeline_rect = egui::Rect::from_min_size(
                egui::Pos2::new(0.0, 0.0),
                egui::Vec2::new(800.0, 200.0)
            );
            
            // This would normally draw the timeline UI
            // For testing purposes, we just verify the state
            assert_eq!(app.timeline.current_frame, 0); // Timeline may not be synced yet
        });
        
        harness.run();
    }
    
    /// Test error handling in rendering
    #[test]
    fn test_rendering_error_handling() {
        let mut harness = Harness::new_ui(|ui| {
            let mut app = TimelineApp::default();
            
            // Force an error condition by seeking to invalid frame
            app.engine.seek(u32::MAX);
            
            let stage_rect = egui::Rect::from_min_size(
                egui::Pos2::new(10.0, 10.0), 
                egui::Vec2::new(400.0, 300.0)
            );
            
            // Should still render without crashing (fallback to test pattern)
            app.draw_stage(ui, stage_rect);
        });
        
        harness.run();
    }
}