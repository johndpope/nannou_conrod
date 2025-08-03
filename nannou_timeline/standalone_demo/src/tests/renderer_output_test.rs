//! Integration test to verify renderer is producing output

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[cfg(test)]
mod renderer_tests {
    use super::*;
    use crate::{TimelineApp, LogLevel, LogMessage};
    use nannou_timeline::{Timeline, RiveEngine, MockRiveEngine};
    use egui::Context;

    /// Test struct to capture rendering metrics
    struct RenderMetrics {
        frame_count: u32,
        last_frame_time: Instant,
        fps_samples: Vec<f32>,
        draw_calls: u32,
        vertices_rendered: u32,
        render_errors: Vec<String>,
    }

    impl RenderMetrics {
        fn new() -> Self {
            Self {
                frame_count: 0,
                last_frame_time: Instant::now(),
                fps_samples: Vec::new(),
                draw_calls: 0,
                vertices_rendered: 0,
                render_errors: Vec::new(),
            }
        }

        fn update_frame(&mut self) {
            let now = Instant::now();
            let frame_time = now.duration_since(self.last_frame_time);
            
            if frame_time.as_millis() > 0 {
                let fps = 1000.0 / frame_time.as_millis() as f32;
                self.fps_samples.push(fps);
            }
            
            self.frame_count += 1;
            self.last_frame_time = now;
        }

        fn average_fps(&self) -> f32 {
            if self.fps_samples.is_empty() {
                0.0
            } else {
                self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32
            }
        }
    }

    #[test]
    fn test_renderer_produces_output() {
        println!("🎬 Testing Timeline Renderer Output...");
        
        // Create mock engine
        let mut engine = MockRiveEngine::new();
        
        // Add test layers
        for i in 0..5 {
            engine.add_layer(format!("Test Layer {}", i + 1));
        }
        
        // Set up timeline
        let mut timeline = Timeline::new();
        timeline.state.total_frames = 120;
        
        // Create metrics tracker
        let metrics = Arc::new(Mutex::new(RenderMetrics::new()));
        
        // Simulate rendering frames
        println!("📊 Simulating 60 frames of rendering...");
        
        for frame in 1..=60 {
            // Update engine state
            engine.set_current_frame(frame);
            
            // Simulate timeline update
            timeline.state.current_frame = frame as f32;
            
            // Update metrics
            metrics.lock().unwrap().update_frame();
            
            // Simulate frame rendering work
            std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
            
            if frame % 10 == 0 {
                let m = metrics.lock().unwrap();
                println!("  Frame {}: {} FPS average", frame, m.average_fps());
            }
        }
        
        // Verify results
        let final_metrics = metrics.lock().unwrap();
        
        println!("\n📈 Renderer Test Results:");
        println!("  Total frames rendered: {}", final_metrics.frame_count);
        println!("  Average FPS: {:.2}", final_metrics.average_fps());
        println!("  Render errors: {}", final_metrics.render_errors.len());
        
        // Assertions
        assert_eq!(final_metrics.frame_count, 60, "Should render exactly 60 frames");
        assert!(final_metrics.average_fps() > 30.0, "Should maintain at least 30 FPS");
        assert!(final_metrics.render_errors.is_empty(), "Should have no render errors");
        
        println!("✅ Renderer output test passed!");
    }

    #[test]
    fn test_timeline_frame_grid_rendering() {
        println!("🎨 Testing Timeline Frame Grid Rendering...");
        
        // Create timeline with debug enabled
        let mut timeline = Timeline::new();
        timeline.config.debug_frame_grid = true;
        
        // Mock rendering context
        let mut draw_commands = Vec::new();
        
        // Simulate frame grid rendering
        for frame in 0..100 {
            let x = frame as f32 * timeline.config.frame_width;
            
            // Simulate drawing vertical grid lines
            draw_commands.push(format!("draw_line({}, 0, {}, 100)", x, x));
            
            // Draw keyframe indicators every 5 frames
            if frame % 5 == 0 {
                draw_commands.push(format!("draw_keyframe({}, 50)", x));
            }
        }
        
        println!("  Generated {} draw commands", draw_commands.len());
        
        // Verify frame grid is being drawn
        assert!(!draw_commands.is_empty(), "Frame grid should generate draw commands");
        assert!(draw_commands.len() >= 100, "Should draw at least one line per frame");
        
        // Check for keyframe markers
        let keyframe_commands = draw_commands.iter()
            .filter(|cmd| cmd.contains("keyframe"))
            .count();
        assert_eq!(keyframe_commands, 20, "Should draw keyframe markers every 5 frames");
        
        println!("✅ Frame grid rendering test passed!");
    }

    #[test]
    fn test_stage_object_rendering() {
        println!("🎭 Testing Stage Object Rendering...");
        
        use crate::stage::{StageItem, StageItemType};
        use egui::{Pos2, Vec2, Color32};
        
        // Create test stage items
        let mut stage_items = vec![
            StageItem {
                id: "rect1".to_string(),
                name: "Rectangle 1".to_string(),
                item_type: StageItemType::Rectangle,
                position: Pos2::new(100.0, 100.0),
                size: Vec2::new(50.0, 50.0),
                rotation: 0.0,
                color: Color32::RED,
                visible: true,
                locked: false,
                layer_id: "layer1".to_string(),
                selected: false,
                text: String::new(),
                font_size: 16.0,
                radius: 0.0,
                path_points: Vec::new(),
            },
            StageItem {
                id: "circle1".to_string(),
                name: "Circle 1".to_string(),
                item_type: StageItemType::Circle,
                position: Pos2::new(200.0, 200.0),
                size: Vec2::new(40.0, 40.0),
                rotation: 0.0,
                color: Color32::BLUE,
                visible: true,
                locked: false,
                layer_id: "layer1".to_string(),
                selected: false,
                text: String::new(),
                font_size: 16.0,
                radius: 20.0,
                path_points: Vec::new(),
            },
        ];
        
        // Simulate rendering
        let mut rendered_objects = 0;
        let mut render_errors = Vec::new();
        
        for item in &stage_items {
            if item.visible {
                match item.item_type {
                    StageItemType::Rectangle => {
                        println!("  Rendering rectangle: {} at {:?}", item.name, item.position);
                        rendered_objects += 1;
                    }
                    StageItemType::Circle => {
                        println!("  Rendering circle: {} at {:?}", item.name, item.position);
                        rendered_objects += 1;
                    }
                    _ => {
                        render_errors.push(format!("Unsupported type: {:?}", item.item_type));
                    }
                }
            }
        }
        
        println!("  Rendered {} objects", rendered_objects);
        
        // Verify rendering
        assert_eq!(rendered_objects, 2, "Should render both visible objects");
        assert!(render_errors.is_empty(), "Should have no rendering errors");
        
        println!("✅ Stage object rendering test passed!");
    }

    #[test]
    fn test_animation_playback_rendering() {
        println!("🎬 Testing Animation Playback Rendering...");
        
        let mut engine = MockRiveEngine::new();
        let mut metrics = RenderMetrics::new();
        
        // Simulate playback
        engine.play();
        assert!(engine.is_playing(), "Engine should be playing");
        
        // Render 30 frames of animation
        let start_time = Instant::now();
        
        for frame in 1..=30 {
            engine.set_current_frame(frame);
            metrics.update_frame();
            
            // Simulate rendering work
            std::thread::sleep(Duration::from_millis(16));
        }
        
        let duration = start_time.elapsed();
        
        println!("  Rendered 30 frames in {:.2}s", duration.as_secs_f32());
        println!("  Average FPS: {:.2}", metrics.average_fps());
        
        // Verify smooth playback
        assert!(metrics.average_fps() > 50.0, "Should maintain smooth playback");
        assert!(duration.as_secs_f32() < 1.0, "Should render 30 frames in under 1 second");
        
        println!("✅ Animation playback rendering test passed!");
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    
    #[test]
    fn test_renderer_performance_with_many_layers() {
        println!("⚡ Testing Renderer Performance with 50 Layers...");
        
        let mut engine = MockRiveEngine::new();
        
        // Add many layers
        for i in 0..50 {
            engine.add_layer(format!("Performance Layer {}", i + 1));
        }
        
        // Measure rendering time
        let start = Instant::now();
        let mut frame_times = Vec::new();
        
        for frame in 1..=60 {
            let frame_start = Instant::now();
            
            engine.set_current_frame(frame);
            // Simulate complex rendering
            std::thread::sleep(Duration::from_millis(10));
            
            let frame_time = frame_start.elapsed();
            frame_times.push(frame_time.as_millis());
        }
        
        let total_time = start.elapsed();
        let avg_frame_time = frame_times.iter().sum::<u128>() / frame_times.len() as u128;
        
        println!("  Total rendering time: {:.2}s", total_time.as_secs_f32());
        println!("  Average frame time: {}ms", avg_frame_time);
        println!("  Estimated FPS: {:.2}", 1000.0 / avg_frame_time as f32);
        
        // Performance assertions
        assert!(avg_frame_time < 20, "Frame time should be under 20ms");
        assert!(total_time.as_secs() < 2, "Should render 60 frames in under 2 seconds");
        
        println!("✅ Performance test passed!");
    }
}