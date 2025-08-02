//! Tests for the Flash-inspired timeline widget

use nannou_timeline::{Timeline, TimelineConfig, ui::MockRiveEngine, LayerId, RiveEngine};

#[test]
fn test_timeline_creation() {
    let timeline = Timeline::new();
    assert!(timeline.state.playhead_frame == 0);
    assert!(!timeline.state.is_playing);
    assert_eq!(timeline.state.zoom_level, 1.0);
}

#[test]
fn test_timeline_with_custom_config() {
    let config = TimelineConfig {
        layer_panel_width: 250.0,
        ruler_height: 40.0,
        controls_height: 50.0,
        default_track_height: 35.0,
        frame_width: 15.0,
        ..Default::default()
    };
    
    let timeline = Timeline::with_config(config.clone());
    assert_eq!(timeline.config.layer_panel_width, 250.0);
    assert_eq!(timeline.config.ruler_height, 40.0);
}

#[test]
fn test_mock_rive_engine() {
    let mut engine = MockRiveEngine::new();
    
    // Test layer retrieval
    let layers = engine.get_layers();
    assert_eq!(layers.len(), 5);
    assert_eq!(layers[0].name, "Background");
    assert_eq!(layers[2].name, "Effects");
    
    // Test playback controls
    assert_eq!(engine.get_current_frame(), 0);
    engine.seek(50);
    assert_eq!(engine.get_current_frame(), 50);
    
    engine.play();
    engine.pause();
    
    // Test frame data
    let frame_data = engine.get_frame_data(LayerId::new("layer1"), 10);
    assert_eq!(frame_data.frame_number, 10);
}

#[test]
fn test_layer_visibility() {
    let engine = MockRiveEngine::new();
    let layers = engine.get_layers();
    
    // Check initial visibility states
    assert!(layers[0].visible); // Background
    assert!(layers[1].visible); // Character
    assert!(!layers[3].visible); // Glow (should be hidden)
}

#[test]
fn test_layer_hierarchy() {
    let engine = MockRiveEngine::new();
    let layers = engine.get_layers();
    
    // Check folder structure
    let effects_folder = &layers[2];
    assert_eq!(effects_folder.children.len(), 2);
    
    // Check parent relationships
    let glow_layer = &layers[3];
    assert_eq!(glow_layer.parent_id, Some(LayerId::new("layer3")));
}

#[test]
fn test_frame_types() {
    let engine = MockRiveEngine::new();
    
    // Test keyframe pattern (every 10th frame)
    let keyframe = engine.get_frame_data(LayerId::new("layer1"), 10);
    assert!(matches!(keyframe.frame_type, nannou_timeline::frame::FrameType::Keyframe));
    
    // Test tween pattern (frames 1-4)
    let tween = engine.get_frame_data(LayerId::new("layer1"), 3);
    assert!(matches!(tween.frame_type, nannou_timeline::frame::FrameType::Tween));
    
    // Test empty frame
    let empty = engine.get_frame_data(LayerId::new("layer1"), 7);
    assert!(matches!(empty.frame_type, nannou_timeline::frame::FrameType::Empty));
}

#[cfg(test)]
mod visual_tests {
    use super::*;
    use egui::Color32;
    
    #[test]
    fn test_timeline_style_colors() {
        let config = TimelineConfig::default();
        let style = &config.style;
        
        // Test default colors
        assert_eq!(style.background_color, Color32::from_gray(40));
        assert_eq!(style.playhead_color, Color32::from_rgb(255, 0, 0));
        assert_eq!(style.layer_selected, Color32::from_rgb(70, 130, 180));
    }
}

#[cfg(test)]
mod state_tests {
    use nannou_timeline::TimelineState;
    
    #[test]
    fn test_timeline_state_defaults() {
        let state = TimelineState::default();
        
        assert_eq!(state.playhead_frame, 0);
        assert!(!state.is_playing);
        assert_eq!(state.zoom_level, 1.0);
        assert_eq!(state.scroll_x, 0.0);
        assert_eq!(state.scroll_y, 0.0);
        assert!(state.selected_layers.is_empty());
        assert!(state.selected_frames.is_empty());
    }
}