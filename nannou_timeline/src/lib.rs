//! A Flash-inspired timeline widget for nannou using egui.
//!
//! The primary type is **Timeline** - an egui widget that mimics Adobe Flash's
//! timeline interface with layers, keyframes, and playback controls.

use egui::Color32;

pub use playhead_egui::Playhead;
pub use ruler_egui::Ruler;
pub use timeline_egui::{Timeline, TimelineState, KeyframeSelection, DragState, KeyframeClipboardItem};
pub use ui::{MockRiveEngine, MockAudioEngine};
pub use layer::{Layer, LayerId, LayerType};
pub use frame::{Frame, FrameType, KeyframeId};
pub use track_simple::Track;

pub mod playhead_egui;
pub mod ruler_egui;
pub mod timeline_egui;
pub mod layer;
pub mod frame;
pub mod track_simple;
pub mod ui;
pub mod time;
pub mod easing;
pub mod motion_editor;
pub mod audio;
pub mod i18n;
pub mod scripting;
pub mod dock_manager;

// Re-export time types
pub use time::{FrameTime, FpsPreset, FrameLabel, FrameComment};

// Re-export easing types
pub use easing::{BezierCurve, BezierPoint, EasingPreset, PropertyId};

// Re-export motion editor
pub use motion_editor::MotionEditor;

// Re-export audio types
pub use audio::{AudioId, AudioSource, AudioLayer, AudioSyncMode, AudioEngine, AudioError, WaveformData, VolumeEnvelope};

// Re-export scripting types
pub use scripting::{ScriptContext, ScriptStage, ScriptDisplayObject, ScriptEvent, ScriptManager};

// Re-export dock manager
pub use dock_manager::{DockManager, TabType, FlashTabViewer};

/// Mock interface for Rive integration
pub trait RiveEngine: Send + Sync {
    fn get_layers(&self) -> Vec<layer::LayerInfo>;
    fn get_frame_data(&self, layer_id: LayerId, frame: u32) -> frame::FrameData;
    fn play(&mut self);
    fn pause(&mut self);
    fn seek(&mut self, frame: u32);
    fn get_current_frame(&self) -> u32;
    fn get_total_frames(&self) -> u32;
    fn get_fps(&self) -> f32;
    
    // Frame operations
    fn insert_frame(&mut self, layer_id: LayerId, frame: u32);
    fn remove_frame(&mut self, layer_id: LayerId, frame: u32);
    
    // Keyframe operations
    fn insert_keyframe(&mut self, layer_id: LayerId, frame: u32);
    fn clear_keyframe(&mut self, layer_id: LayerId, frame: u32);
    
    // Tween operations
    fn create_motion_tween(&mut self, layer_id: LayerId, frame: u32);
    fn create_shape_tween(&mut self, layer_id: LayerId, frame: u32);
    
    // Keyframe manipulation operations
    fn move_keyframe(&mut self, layer_id: LayerId, from_frame: u32, to_frame: u32);
    fn copy_keyframe(&mut self, layer_id: LayerId, frame: u32) -> Option<frame::FrameData>;
    fn paste_keyframe(&mut self, layer_id: LayerId, frame: u32, data: frame::FrameData);
    fn delete_keyframe(&mut self, layer_id: LayerId, frame: u32);
    
    // Property manipulation
    fn set_property(&mut self, layer_id: LayerId, frame: u32, property: &str, value: bool);
    fn get_property(&self, layer_id: LayerId, frame: u32, property: &str) -> bool;
    
    // Layer operations
    fn rename_layer(&mut self, layer_id: LayerId, new_name: String);
    fn add_layer(&mut self, name: String, layer_type: layer::LayerType) -> LayerId;
    fn delete_layer(&mut self, layer_id: LayerId);
    fn duplicate_layer(&mut self, layer_id: LayerId) -> LayerId;
    fn add_folder_layer(&mut self, name: String) -> LayerId;
    fn add_motion_guide_layer(&mut self, name: String) -> LayerId;
}

/// Timeline configuration
#[derive(Clone, Debug)]
pub struct TimelineConfig {
    /// Width of the layer panel (left side)
    pub layer_panel_width: f32,
    /// Height of the ruler
    pub ruler_height: f32,
    /// Height of the playback controls
    pub controls_height: f32,
    /// Default track height
    pub default_track_height: f32,
    /// Frame width (for zoom)
    pub frame_width: f32,
    /// Frames per second
    pub fps: FpsPreset,
    /// Frame labels
    pub frame_labels: Vec<FrameLabel>,
    /// Frame comments
    pub frame_comments: Vec<FrameComment>,
    /// Colors and styling
    pub style: TimelineStyle,
    /// Snap-to-grid configuration
    pub snap: SnapConfig,
}

/// Snap-to-grid configuration
#[derive(Clone, Debug)]
pub struct SnapConfig {
    pub enabled: bool,
    pub snap_to_frames: bool,
    pub snap_to_keyframes: bool,
    pub snap_to_markers: bool,
    pub threshold_pixels: f32,
    pub show_guides: bool,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            layer_panel_width: 200.0,
            ruler_height: 30.0,
            controls_height: 40.0,
            default_track_height: 30.0,
            frame_width: 10.0,
            fps: FpsPreset::default(),
            frame_labels: Vec::new(),
            frame_comments: Vec::new(),
            style: TimelineStyle::default(),
            snap: SnapConfig::default(),
        }
    }
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            snap_to_frames: true,
            snap_to_keyframes: true,
            snap_to_markers: true,
            threshold_pixels: 8.0,
            show_guides: true,
        }
    }
}

/// Visual styling for the timeline
#[derive(Clone, Debug)]
pub struct TimelineStyle {
    pub background_color: Color32,
    pub grid_color: Color32,
    pub layer_background: Color32,
    pub layer_selected: Color32,
    pub frame_empty: Color32,
    pub frame_keyframe: Color32,
    pub frame_tween: Color32,
    pub playhead_color: Color32,
    pub border_color: Color32,
    pub text_color: Color32,
}

impl Default for TimelineStyle {
    fn default() -> Self {
        Self {
            background_color: Color32::from_gray(40),
            grid_color: Color32::from_gray(60),
            layer_background: Color32::from_gray(50),
            layer_selected: Color32::from_rgb(70, 130, 180),
            frame_empty: Color32::from_gray(45),
            frame_keyframe: Color32::from_gray(20),
            frame_tween: Color32::from_rgb(100, 100, 150),
            playhead_color: Color32::from_rgb(255, 0, 0),
            border_color: Color32::from_gray(80),
            text_color: Color32::from_gray(220),
        }
    }
}

