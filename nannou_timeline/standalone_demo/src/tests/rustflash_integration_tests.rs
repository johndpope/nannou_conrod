//! Tests for RustFlash Integration
//!
//! These tests verify the integration between the RustFlash engine
//! and the timeline demo, including artboard conversion and rendering.

use crate::rustflash_integration::RustFlashIntegration;
use nannou_timeline::{RiveEngine, LayerId, LayerType};
use std::sync::{Arc, Mutex};

#[test]
fn test_rustflash_integration_creation() {
    let integration = RustFlashIntegration::new();
    
    // Verify initial state  
    assert_eq!(integration.get_current_frame(), 0);
    assert_eq!(integration.get_total_frames(), 100);
    assert_eq!(integration.get_fps(), 24.0);
    
    // Verify layers are created
    let layers = integration.get_layers();
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].name, "Animation");
    assert_eq!(layers[1].name, "Background");
}

#[test]
fn test_frame_seeking() {
    let mut integration = RustFlashIntegration::new();
    
    // Test seeking to different frames
    integration.seek(25);
    assert_eq!(integration.get_current_frame(), 25);
    
    integration.seek(75);
    assert_eq!(integration.get_current_frame(), 75);
    
    // Test seeking beyond total frames (should clamp)
    integration.seek(150);
    assert_eq!(integration.get_current_frame(), 100);
}

#[test]
fn test_playback_controls() {
    let mut integration = RustFlashIntegration::new();
    
    // Test play/pause
    integration.play();
    integration.pause();
    
    // These methods should not panic
    assert_eq!(integration.get_current_frame(), 0);
}

#[test]
fn test_layer_operations() {
    let mut integration = RustFlashIntegration::new();
    
    // Test adding a new layer
    let new_layer_id = integration.add_layer("Test Layer".to_string(), LayerType::Normal);
    let layers = integration.get_layers();
    assert_eq!(layers.len(), 3);
    
    // Test renaming layer
    integration.rename_layer(new_layer_id.clone(), "Renamed Layer".to_string());
    let layers = integration.get_layers();
    let renamed_layer = layers.iter().find(|l| l.id == new_layer_id).unwrap();
    assert_eq!(renamed_layer.name, "Renamed Layer");
    
    // Test duplicating layer
    let duplicate_id = integration.duplicate_layer(new_layer_id.clone());
    let layers = integration.get_layers();
    assert_eq!(layers.len(), 4);
    
    let duplicate_layer = layers.iter().find(|l| l.id == duplicate_id).unwrap();
    assert_eq!(duplicate_layer.name, "Renamed Layer copy");
    
    // Test deleting layer
    integration.delete_layer(duplicate_id);
    let layers = integration.get_layers();
    assert_eq!(layers.len(), 3);
}

#[test]
fn test_keyframe_operations() {
    let mut integration = RustFlashIntegration::new();
    let layer_id = LayerId::new("test_layer".to_string());
    
    // Add layer first
    integration.add_layer("Test Layer".to_string(), LayerType::Normal);
    
    // Test inserting keyframe
    integration.insert_keyframe(layer_id.clone(), 10);
    let frame_data = integration.get_frame_data(layer_id.clone(), 10);
    assert!(frame_data.has_content);
    
    // Test creating motion tween
    integration.create_motion_tween(layer_id.clone(), 10);
    
    // Test moving keyframe
    integration.move_keyframe(layer_id.clone(), 10, 20);
    let old_frame = integration.get_frame_data(layer_id.clone(), 10);
    let new_frame = integration.get_frame_data(layer_id.clone(), 20);
    assert!(!old_frame.has_content);
    assert!(new_frame.has_content);
    
    // Test copying keyframe
    let copied_data = integration.copy_keyframe(layer_id.clone(), 20);
    assert!(copied_data.is_some());
    
    // Test pasting keyframe
    if let Some(data) = copied_data {
        integration.paste_keyframe(layer_id.clone(), 30, data);
        let pasted_frame = integration.get_frame_data(layer_id.clone(), 30);
        assert!(pasted_frame.has_content);
    }
    
    // Test deleting keyframe
    integration.delete_keyframe(layer_id.clone(), 30);
    let deleted_frame = integration.get_frame_data(layer_id.clone(), 30);
    assert!(!deleted_frame.has_content);
}

#[test]
fn test_layer_properties() {
    let mut integration = RustFlashIntegration::new();
    let layers = integration.get_layers();
    let layer_id = layers[0].id.clone();
    
    // Test setting and getting visibility
    integration.set_property(layer_id.clone(), 0, "visible", false);
    assert_eq!(integration.get_property(layer_id.clone(), 0, "visible"), false);
    
    integration.set_property(layer_id.clone(), 0, "visible", true);
    assert_eq!(integration.get_property(layer_id.clone(), 0, "visible"), true);
    
    // Test setting and getting locked state
    integration.set_property(layer_id.clone(), 0, "locked", true);
    assert_eq!(integration.get_property(layer_id.clone(), 0, "locked"), true);
    
    integration.set_property(layer_id.clone(), 0, "locked", false);
    assert_eq!(integration.get_property(layer_id.clone(), 0, "locked"), false);
}

#[test]
fn test_artboard_rendering() {
    let mut integration = RustFlashIntegration::new();
    
    // Test getting rendered artboard
    let artboard_result = integration.get_rendered_artboard();
    assert!(artboard_result.is_ok());
    
    let artboard = artboard_result.unwrap();
    assert_eq!(artboard.name, "stage");
    assert!(!artboard.paths.is_empty());
}

#[test]
fn test_artboard_conversion() {
    let mut integration = RustFlashIntegration::new();
    
    // Test converting to renderer format
    let renderer_artboard_result = integration.get_renderer_artboard();
    assert!(renderer_artboard_result.is_ok());
    
    let renderer_artboard = renderer_artboard_result.unwrap();
    assert_eq!(renderer_artboard.name, "stage");
    assert!(!renderer_artboard.paths.is_empty());
}

#[test]
fn test_dirty_flag_management() {
    let mut integration = RustFlashIntegration::new();
    
    // Mark as dirty
    integration.mark_dirty();
    
    // Getting artboard should clear dirty flag and re-render
    let _artboard = integration.get_rendered_artboard().unwrap();
    
    // Seeking should mark as dirty
    integration.seek(10);
    let _artboard2 = integration.get_rendered_artboard().unwrap();
}

#[test]
fn test_folder_and_guide_layers() {
    let mut integration = RustFlashIntegration::new();
    
    // Test adding folder layer
    let folder_id = integration.add_folder_layer("Folder Layer".to_string());
    let layers = integration.get_layers();
    let folder_layer = layers.iter().find(|l| l.id == folder_id).unwrap();
    assert_eq!(folder_layer.layer_type, LayerType::Folder);
    
    // Test adding motion guide layer
    let guide_id = integration.add_motion_guide_layer("Guide Layer".to_string());
    let layers = integration.get_layers();
    let guide_layer = layers.iter().find(|l| l.id == guide_id).unwrap();
    assert_eq!(guide_layer.layer_type, LayerType::Guide);
}

#[test]
fn test_frame_data_edge_cases() {
    let integration = RustFlashIntegration::new();
    let nonexistent_layer = LayerId::new("nonexistent".to_string());
    
    // Test getting frame data for nonexistent layer
    let frame_data = integration.get_frame_data(nonexistent_layer, 0);
    assert!(!frame_data.has_content);
    
    // Test getting frame data for frame beyond range
    let layers = integration.get_layers();
    let layer_id = layers[0].id.clone();
    let frame_data = integration.get_frame_data(layer_id, 9999);
    assert!(!frame_data.has_content);
}