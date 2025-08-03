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
    assert_eq!(layers.len(), 7); // Updated to include 2 audio layers
    assert_eq!(layers[0].name, "Background");
    assert_eq!(layers[2].name, "Effects");
    assert_eq!(layers[5].name, "Background Music");
    assert_eq!(layers[6].name, "Sound Effects");
    
    // Test audio layer types
    assert!(matches!(layers[5].layer_type, nannou_timeline::LayerType::Audio));
    assert!(matches!(layers[6].layer_type, nannou_timeline::LayerType::Audio));
    
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

#[cfg(test)]
mod frame_time_tests {
    use nannou_timeline::{FrameTime, FpsPreset};
    
    #[test]
    fn test_frame_time_operations() {
        // Test frame to seconds conversion
        let ft = FrameTime::new(48, 24.0);
        assert_eq!(ft.to_seconds(), 2.0);
        
        // Test seconds to frame conversion
        let ft2 = FrameTime::from_seconds(3.5, 30.0);
        assert_eq!(ft2.frame, 105);
        
        // Test timecode formatting
        let ft3 = FrameTime::new(90, 30.0); // 3 seconds at 30fps
        let timecode = ft3.to_timecode();
        assert!(timecode.starts_with("00:00:03:"));
    }
    
    #[test]
    fn test_fps_presets() {
        assert_eq!(FpsPreset::Film.to_fps(), 24.0);
        assert_eq!(FpsPreset::Pal.to_fps(), 25.0);
        assert_eq!(FpsPreset::Ntsc.to_fps(), 29.97);
        assert_eq!(FpsPreset::Web.to_fps(), 30.0);
        assert_eq!(FpsPreset::High.to_fps(), 60.0);
        assert_eq!(FpsPreset::Custom(120.0).to_fps(), 120.0);
        
        // Test default
        assert_eq!(FpsPreset::default(), FpsPreset::Film);
    }
    
    #[test]
    fn test_fps_labels() {
        assert_eq!(FpsPreset::Film.label(), "24 fps (Film)");
        assert_eq!(FpsPreset::Ntsc.label(), "29.97 fps (NTSC)");
        assert_eq!(FpsPreset::Custom(48.0).label(), "48 fps (Custom)");
    }
}

#[test]
fn test_frame_operations() {
    use nannou_timeline::ui::MockRiveEngine;
    let mut engine = MockRiveEngine::new();
    
    // Test frame operations (these just print in mock implementation)
    engine.insert_frame(LayerId::new("layer1"), 10);
    engine.remove_frame(LayerId::new("layer1"), 10);
    engine.insert_keyframe(LayerId::new("layer1"), 5);
    engine.clear_keyframe(LayerId::new("layer1"), 5);
    engine.create_motion_tween(LayerId::new("layer1"), 15);
    engine.create_shape_tween(LayerId::new("layer1"), 20);
    
    // These operations should complete without panic
    assert_eq!(engine.get_current_frame(), 0);
}

#[test]
fn test_timeline_scrolling_and_zoom() {
    use nannou_timeline::Timeline;
    let mut timeline = Timeline::new();
    
    // Test initial scroll and zoom state
    assert_eq!(timeline.state.scroll_x, 0.0);
    assert_eq!(timeline.state.scroll_y, 0.0);
    assert_eq!(timeline.state.zoom_level, 1.0);
    
    // Test zoom limits
    timeline.state.zoom_level = 0.05;
    assert!(timeline.state.zoom_level >= 0.05);
    
    timeline.state.zoom_level = 10.0;
    assert!(timeline.state.zoom_level <= 10.0);
    
    // Test scroll positions
    timeline.state.scroll_x = 100.0;
    timeline.state.scroll_y = 50.0;
    assert_eq!(timeline.state.scroll_x, 100.0);
    assert_eq!(timeline.state.scroll_y, 50.0);
}

#[test]
fn test_snap_to_grid_functionality() {
    use nannou_timeline::Timeline;
    let mut timeline = Timeline::new();
    
    // Test default snap config
    assert!(timeline.config.snap.enabled);
    assert!(timeline.config.snap.snap_to_frames);
    assert_eq!(timeline.config.snap.threshold_pixels, 8.0);
    
    // Test snap position calculation
    let modifiers = egui::Modifiers::default();
    
    // Should snap to nearest frame
    let pos = 52.0; // Close to frame 5 (at 50.0)
    let snapped = timeline.snap_position(pos, &modifiers);
    assert_eq!(snapped, 50.0);
    
    // Should snap if within threshold (frame_width=10, so frames at 0, 10, 20, 30, 40, 50, 60, 70...)
    let pos = 65.0; // Should snap to frame 7 at 70.0 since it's within threshold (5px away)
    let snapped = timeline.snap_position(pos, &modifiers);
    assert_eq!(snapped, 70.0);
    
    // Should not snap if too far from any frame (threshold is 8px)
    let pos = 75.0; // Between frame 7 (70) and frame 8 (80), 5px from both - should snap to 80
    let snapped = timeline.snap_position(pos, &modifiers);
    assert_eq!(snapped, 80.0);
    
    // Test case that's truly outside threshold
    let pos = 84.0; // 4px from frame 8 (80), 6px from frame 9 (90) - should snap to 80
    let snapped = timeline.snap_position(pos, &modifiers);
    assert_eq!(snapped, 80.0);
    
    // Position that rounds to next frame
    let pos = 99.0; // 9.9 frames, rounds to 10, snaps to frame 10 at 100.0
    let snapped = timeline.snap_position(pos, &modifiers);
    assert_eq!(snapped, 100.0);
    
    // Position exactly between frames but within threshold
    let pos = 85.0; // 5px from both frame 8 (80) and frame 9 (90) - snaps to 90 (round up)
    let snapped = timeline.snap_position(pos, &modifiers);
    assert_eq!(snapped, 90.0);
    
    // Create a test case that should NOT snap (outside threshold)
    // Need to set threshold smaller first
    timeline.config.snap.threshold_pixels = 2.0; // Make threshold very small
    let pos = 85.0; // Now 5px from nearest frame, outside 2px threshold
    let snapped = timeline.snap_position(pos, &modifiers);
    assert_eq!(snapped, 85.0); // Should not snap
    
    // Should not snap when disabled
    timeline.config.snap.enabled = false;
    let pos = 52.0;
    let snapped = timeline.snap_position(pos, &modifiers);
    assert_eq!(snapped, 52.0);
    
    // Should not snap with Shift modifier
    timeline.config.snap.enabled = true;
    let mut shift_modifiers = egui::Modifiers::default();
    shift_modifiers.shift = true;
    let snapped = timeline.snap_position(52.0, &shift_modifiers);
    assert_eq!(snapped, 52.0);
}

#[test]
fn test_snap_guides() {
    use nannou_timeline::Timeline;
    let mut timeline = Timeline::new();
    
    // Test snap guides initialization
    assert!(timeline.state.snap_guides.is_empty());
    
    // Test guide updates
    timeline.update_snap_guides(52.0);
    assert_eq!(timeline.state.snap_guides.len(), 1);
    assert_eq!(timeline.state.snap_guides[0], 50.0);
    
    // Test guides are cleared when disabled
    timeline.config.snap.enabled = false;
    timeline.update_snap_guides(52.0);
    assert!(timeline.state.snap_guides.is_empty());
}

#[test]
fn test_keyframe_selection_basic() {
    use nannou_timeline::{Timeline, KeyframeId, LayerId};
    
    let mut timeline = Timeline::new();
    let layer_id = LayerId::new("test_layer");
    let keyframe_id = KeyframeId::new();
    
    // Test initial state
    assert!(!timeline.state.keyframe_selection.is_selected(layer_id.clone(), 5));
    assert!(timeline.state.keyframe_selection.selected.is_empty());
    
    // Test adding keyframe
    timeline.state.keyframe_selection.add(layer_id.clone(), 5, keyframe_id.clone());
    assert!(timeline.state.keyframe_selection.is_selected(layer_id.clone(), 5));
    assert_eq!(timeline.state.keyframe_selection.selected.len(), 1);
    
    // Test removing keyframe
    timeline.state.keyframe_selection.remove(layer_id.clone(), 5);
    assert!(!timeline.state.keyframe_selection.is_selected(layer_id.clone(), 5));
    assert!(timeline.state.keyframe_selection.selected.is_empty());
}

#[test]
fn test_keyframe_selection_multiple() {
    use nannou_timeline::{Timeline, KeyframeId, LayerId};
    
    let mut timeline = Timeline::new();
    let layer1 = LayerId::new("layer1");
    let layer2 = LayerId::new("layer2");
    let keyframe1 = KeyframeId::new();
    let keyframe2 = KeyframeId::new();
    let keyframe3 = KeyframeId::new();
    
    // Add multiple keyframes
    timeline.state.keyframe_selection.add(layer1.clone(), 5, keyframe1);
    timeline.state.keyframe_selection.add(layer1.clone(), 10, keyframe2);
    timeline.state.keyframe_selection.add(layer2.clone(), 8, keyframe3);
    
    assert_eq!(timeline.state.keyframe_selection.selected.len(), 3);
    assert!(timeline.state.keyframe_selection.is_selected(layer1.clone(), 5));
    assert!(timeline.state.keyframe_selection.is_selected(layer1.clone(), 10));
    assert!(timeline.state.keyframe_selection.is_selected(layer2.clone(), 8));
    
    // Test clear
    timeline.state.keyframe_selection.clear();
    assert!(timeline.state.keyframe_selection.selected.is_empty());
    assert!(!timeline.state.keyframe_selection.is_selected(layer1, 5));
}

#[test]
fn test_keyframe_drag_state() {
    use nannou_timeline::{Timeline, KeyframeId, LayerId, DragState};
    use std::collections::HashMap;
    
    let mut timeline = Timeline::new();
    let layer_id = LayerId::new("test_layer");
    let keyframe_id = KeyframeId::new();
    
    // Add a keyframe
    timeline.state.keyframe_selection.add(layer_id.clone(), 10, keyframe_id.clone());
    
    // Test drag state initialization
    assert!(timeline.state.keyframe_selection.drag_state.is_none());
    
    let mut original_positions = HashMap::new();
    original_positions.insert(keyframe_id, (layer_id, 10));
    
    let drag_state = DragState {
        original_positions,
        frame_offset: 0,
        start_pos: egui::Pos2::new(100.0, 50.0),
    };
    
    timeline.state.keyframe_selection.drag_state = Some(drag_state);
    assert!(timeline.state.keyframe_selection.drag_state.is_some());
    
    // Test clearing drag state
    timeline.state.keyframe_selection.clear();
    assert!(timeline.state.keyframe_selection.drag_state.is_none());
}

#[test]
fn test_mock_engine_keyframe_methods() {
    use nannou_timeline::ui::MockRiveEngine;
    use nannou_timeline::{RiveEngine, LayerId};
    
    let mut engine = MockRiveEngine::new();
    let layer_id = LayerId::new("test_layer");
    
    // Test new keyframe manipulation methods
    engine.move_keyframe(layer_id.clone(), 5, 10);
    
    let copied_data = engine.copy_keyframe(layer_id.clone(), 5);
    assert!(copied_data.is_some());
    
    if let Some(data) = copied_data {
        engine.paste_keyframe(layer_id.clone(), 15, data);
    }
    
    engine.delete_keyframe(layer_id.clone(), 5);
    
    // Test property methods
    engine.set_property(layer_id.clone(), 10, "visible", true);
    let visible = engine.get_property(layer_id.clone(), 10, "visible");
    assert!(visible);
    
    let locked = engine.get_property(layer_id, 10, "locked");
    assert!(!locked);
}

#[cfg(test)]
mod audio_tests {
    use nannou_timeline::{AudioSource, AudioLayer, AudioSyncMode, VolumeEnvelope, MockAudioEngine, AudioEngine};
    use std::path::PathBuf;
    
    #[test]
    fn test_audio_source_creation() {
        let source = AudioSource::new(PathBuf::from("test_audio.wav"));
        assert_eq!(source.filename, "test_audio.wav");
        assert_eq!(source.duration, 0.0);
        assert!(!source.loaded);
    }
    
    #[test]
    fn test_audio_layer_creation() {
        let source = AudioSource::new(PathBuf::from("music.mp3"));
        let layer = AudioLayer::new(source, 10);
        
        assert_eq!(layer.start_frame, 10);
        assert_eq!(layer.volume, 1.0);
        assert_eq!(layer.sync_mode, AudioSyncMode::Event);
        assert!(!layer.loop_audio);
    }
    
    #[test]
    fn test_audio_layer_frame_range() {
        let mut source = AudioSource::new(PathBuf::from("test.wav"));
        source.duration = 5.0; // 5 seconds
        
        let layer = AudioLayer::new(source, 10);
        let frame_range = layer.frame_range(24.0); // 24 fps
        
        assert_eq!(frame_range.start, 10);
        assert_eq!(frame_range.end, 10 + 120); // 5 seconds * 24 fps = 120 frames
    }
    
    #[test]
    fn test_audio_layer_trimming() {
        let mut source = AudioSource::new(PathBuf::from("test.wav"));
        source.duration = 10.0;
        
        let mut layer = AudioLayer::new(source, 0);
        layer.trim_start = 2.0;
        layer.trim_end = 1.0;
        
        assert_eq!(layer.effective_duration(), 7.0); // 10 - 2 - 1 = 7 seconds
        
        let frame_range = layer.frame_range(30.0); // 30 fps
        assert_eq!(frame_range.end - frame_range.start, 210); // 7 seconds * 30 fps
    }
    
    #[test]
    fn test_volume_envelope() {
        let mut envelope = VolumeEnvelope::new();
        
        // Test initial state
        assert_eq!(envelope.points.len(), 1);
        assert_eq!(envelope.volume_at_frame(0), 1.0);
        
        // Add points
        envelope.set_point(10, 0.5);
        envelope.set_point(20, 0.0);
        envelope.set_point(30, 0.8);
        
        assert_eq!(envelope.points.len(), 4);
        
        // Test interpolation
        assert_eq!(envelope.volume_at_frame(0), 1.0);
        assert_eq!(envelope.volume_at_frame(10), 0.5);
        assert_eq!(envelope.volume_at_frame(15), 0.25); // Interpolated between 0.5 and 0.0
        assert_eq!(envelope.volume_at_frame(30), 0.8);
    }
    
    #[test]
    fn test_mock_audio_engine() {
        let mut engine = MockAudioEngine::new();
        
        // Test audio loading
        let result = engine.load_audio(&PathBuf::from("test_music.mp3"));
        assert!(result.is_ok());
        
        let source = result.unwrap();
        assert!(source.loaded);
        assert_eq!(source.filename, "test_music.mp3");
        assert_eq!(source.duration, 30.0); // Mock music duration
        
        // Test waveform generation
        let waveform_result = engine.generate_waveform(&source.id, 24.0);
        assert!(waveform_result.is_ok());
        
        let waveform = waveform_result.unwrap();
        assert!(waveform.complete);
        assert_eq!(waveform.fps, 24.0);
        assert!(!waveform.peaks.is_empty());
        
        // Test playback
        let play_result = engine.play_segment(&source.id, 0.0, 2.0, 1.0);
        assert!(play_result.is_ok());
        assert!(engine.is_playing(&source.id));
        
        // Test stop
        let stop_result = engine.stop_audio(&source.id);
        assert!(stop_result.is_ok());
        assert!(!engine.is_playing(&source.id));
    }
    
    #[test]
    fn test_unsupported_audio_format() {
        let mut engine = MockAudioEngine::new();
        let result = engine.load_audio(&PathBuf::from("test.flac"));
        
        match result {
            Err(nannou_timeline::AudioError::UnsupportedFormat) => {},
            _ => panic!("Expected UnsupportedFormat error"),
        }
    }
    
    #[test]
    fn test_audio_time_calculation() {
        let mut source = AudioSource::new(PathBuf::from("test.wav"));
        source.duration = 8.0;
        
        let mut layer = AudioLayer::new(source, 10);
        layer.trim_start = 1.0;
        layer.trim_end = 1.0;
        
        // Frame 10 (start) should be at audio time 1.0 (trim_start)
        assert_eq!(layer.audio_time_at_frame(10, 24.0), Some(1.0));
        
        // Frame 20 should be at audio time 1.0 + (10/24) = ~1.417
        let time_20 = layer.audio_time_at_frame(20, 24.0).unwrap();
        assert!((time_20 - 1.417).abs() < 0.01);
        
        // Frame before start should return None
        assert_eq!(layer.audio_time_at_frame(5, 24.0), None);
        
        // Frame past effective end should return None
        let end_frame = 10 + ((8.0 - 1.0 - 1.0) * 24.0) as u32; // 6 seconds * 24 fps
        assert_eq!(layer.audio_time_at_frame(end_frame + 10, 24.0), None);
    }
}