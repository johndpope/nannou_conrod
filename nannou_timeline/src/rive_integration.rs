//! Rive-Timeline Integration Interface
//! 
//! This module defines clean interfaces for integrating the Timeline UI with the Rive rendering engine.
//! It separates concerns between timeline operations, data queries, and event handling.

use crate::{LayerId, KeyframeId, layer::LayerInfo};
use serde::{Serialize, Deserialize};

/// Commands that the timeline sends to the Rive engine for animation control
pub trait RiveController: Send + Sync {
    // Playback control
    fn play(&mut self);
    fn pause(&mut self);
    fn stop(&mut self);
    fn seek_to_frame(&mut self, frame: u32);
    fn set_fps(&mut self, fps: f32);
    
    // Layer operations
    fn add_layer(&mut self, name: &str) -> LayerId;
    fn remove_layer(&mut self, layer_id: LayerId);
    fn reorder_layers(&mut self, layer_ids: Vec<LayerId>);
    fn set_layer_visibility(&mut self, layer_id: LayerId, visible: bool);
    fn set_layer_locked(&mut self, layer_id: LayerId, locked: bool);
    fn rename_layer(&mut self, layer_id: LayerId, new_name: String);
    
    // Keyframe operations
    fn add_keyframe(&mut self, layer_id: LayerId, frame: u32) -> KeyframeId;
    fn remove_keyframe(&mut self, keyframe_id: KeyframeId);
    fn move_keyframe(&mut self, keyframe_id: KeyframeId, new_frame: u32);
    fn copy_keyframe(&mut self, keyframe_id: KeyframeId) -> Option<KeyframeData>;
    fn paste_keyframe(&mut self, layer_id: LayerId, frame: u32, data: KeyframeData);
    
    // Frame operations
    fn insert_frame(&mut self, layer_id: LayerId, frame: u32);
    fn remove_frame(&mut self, layer_id: LayerId, frame: u32);
    fn extend_frame(&mut self, layer_id: LayerId, frame: u32);
    
    // Tween operations
    fn create_tween(&mut self, layer_id: LayerId, start_frame: u32, end_frame: u32, tween_type: TweenType);
    fn set_easing(&mut self, tween_id: TweenId, easing: EasingFunction);
    fn remove_tween(&mut self, tween_id: TweenId);
}

/// Events that Rive sends to the timeline for state synchronization
pub trait TimelineEventHandler: Send + Sync {
    fn on_frame_changed(&mut self, frame: u32);
    fn on_playback_started(&mut self);
    fn on_playback_stopped(&mut self);
    fn on_layer_added(&mut self, layer: LayerInfo);
    fn on_layer_removed(&mut self, layer_id: LayerId);
    fn on_layer_reordered(&mut self, layer_ids: Vec<LayerId>);
    fn on_keyframe_added(&mut self, layer_id: LayerId, frame: u32, keyframe_id: KeyframeId);
    fn on_keyframe_removed(&mut self, keyframe_id: KeyframeId);
    fn on_animation_loaded(&mut self, animation_info: AnimationInfo);
    fn on_animation_error(&mut self, error: RiveError);
}

/// Timeline queries Rive for current state and data
pub trait RiveDataProvider: Send + Sync {
    fn get_layers(&self) -> Vec<LayerInfo>;
    fn get_keyframes(&self, layer_id: LayerId) -> Vec<KeyframeInfo>;
    fn get_frame_data(&self, layer_id: LayerId, frame: u32) -> Option<RiveFrameData>;
    fn get_current_frame(&self) -> u32;
    fn get_total_frames(&self) -> u32;
    fn get_fps(&self) -> f32;
    fn get_animation_bounds(&self) -> (u32, u32); // start, end
    fn get_layer_hierarchy(&self) -> Vec<LayerHierarchyNode>;
    fn get_tweens(&self, layer_id: LayerId) -> Vec<TweenInfo>;
    fn is_playing(&self) -> bool;
}

/// Combined interface for full Rive integration
pub trait RiveEngine: RiveController + RiveDataProvider + Send + Sync {
    /// Register an event handler to receive timeline updates
    fn set_event_handler(&mut self, handler: Box<dyn TimelineEventHandler>);
    
    /// Get a reference to the event handler
    fn event_handler(&self) -> Option<&dyn TimelineEventHandler>;
    
    /// Initialize the engine with timeline configuration
    fn initialize(&mut self, config: RiveConfig) -> Result<(), RiveError>;
    
    /// Shutdown and cleanup resources
    fn shutdown(&mut self);
}

// ================== Shared Data Types ==================

/// Information about a keyframe in the animation
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyframeInfo {
    pub id: KeyframeId,
    pub layer_id: LayerId,
    pub frame: u32,
    pub keyframe_type: KeyframeType,
    pub has_tween: bool,
    pub tween_id: Option<TweenId>,
}

/// Data associated with a keyframe for copy/paste operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyframeData {
    pub keyframe_type: KeyframeType,
    pub properties: std::collections::HashMap<String, PropertyValue>,
    pub tween_data: Option<TweenData>,
}

/// Type of keyframe in the timeline
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyframeType {
    Empty,
    Keyframe,
    BlankKeyframe,
}

/// Type of tween animation between keyframes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TweenType {
    Motion,
    Shape,
    Classic,
}

/// Unique identifier for tweens
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TweenId(pub uuid::Uuid);

impl TweenId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

/// Information about a tween animation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TweenInfo {
    pub id: TweenId,
    pub layer_id: LayerId,
    pub start_frame: u32,
    pub end_frame: u32,
    pub tween_type: TweenType,
    pub easing: EasingFunction,
}

/// Data for creating or modifying tweens
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TweenData {
    pub tween_type: TweenType,
    pub duration_frames: u32,
    pub easing: EasingFunction,
}

/// Easing function for animations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    Custom(BezierCurve),
}

/// Bezier curve definition for custom easing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BezierCurve {
    pub control_points: Vec<(f32, f32)>,
}

/// Property value that can be animated
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PropertyValue {
    Bool(bool),
    Float(f32),
    Int(i32),
    String(String),
    Color(Color),
    Transform(Transform),
}

/// Color representation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// 2D transformation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transform {
    pub position: (f32, f32),
    pub rotation: f32,
    pub scale: (f32, f32),
    pub skew: (f32, f32),
}

/// Layer hierarchy node for complex layer structures
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerHierarchyNode {
    pub layer_info: LayerInfo,
    pub children: Vec<LayerHierarchyNode>,
    pub depth: u32,
}

/// Information about the loaded animation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimationInfo {
    pub name: String,
    pub duration_frames: u32,
    pub fps: f32,
    pub layer_count: usize,
    pub bounds: AnimationBounds,
}

/// Animation canvas bounds
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimationBounds {
    pub width: f32,
    pub height: f32,
    pub min_x: f32,
    pub min_y: f32,
}

/// Configuration for Rive engine initialization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiveConfig {
    pub canvas_size: (f32, f32),
    pub default_fps: f32,
    pub enable_audio: bool,
    pub quality_settings: QualitySettings,
}

/// Rendering quality settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualitySettings {
    pub antialiasing: bool,
    pub texture_filtering: bool,
    pub shadow_quality: ShadowQuality,
}

/// Shadow rendering quality levels
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowQuality {
    None,
    Low,
    Medium,
    High,
}

/// Errors that can occur during Rive operations
#[derive(Clone, Debug, thiserror::Error, Serialize, Deserialize)]
pub enum RiveError {
    #[error("Animation file not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Invalid animation format: {reason}")]
    InvalidFormat { reason: String },
    
    #[error("Layer not found: {layer_id:?}")]
    LayerNotFound { layer_id: LayerId },
    
    #[error("Keyframe not found: {keyframe_id:?}")]
    KeyframeNotFound { keyframe_id: KeyframeId },
    
    #[error("Invalid frame number: {frame} (max: {max_frame})")]
    InvalidFrame { frame: u32, max_frame: u32 },
    
    #[error("Engine not initialized")]
    NotInitialized,
    
    #[error("Operation failed: {message}")]
    OperationFailed { message: String },
    
    #[error("Rive runtime error: {error}")]
    RiveRuntimeError { error: String },
}

// ================== Default Implementations ==================

impl Default for EasingFunction {
    fn default() -> Self {
        EasingFunction::Linear
    }
}

impl Default for RiveConfig {
    fn default() -> Self {
        Self {
            canvas_size: (800.0, 600.0),
            default_fps: 24.0,
            enable_audio: true,
            quality_settings: QualitySettings::default(),
        }
    }
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self {
            antialiasing: true,
            texture_filtering: true,
            shadow_quality: ShadowQuality::Medium,
        }
    }
}

// ================== Helper Implementations ==================

impl From<crate::easing::BezierCurve> for BezierCurve {
    fn from(curve: crate::easing::BezierCurve) -> Self {
        Self {
            control_points: curve.points.into_iter()
                .map(|point| point.position)
                .collect(),
        }
    }
}

impl From<BezierCurve> for crate::easing::BezierCurve {
    fn from(curve: BezierCurve) -> Self {
        let points = curve.control_points.into_iter()
            .map(|(x, y)| crate::easing::BezierPoint {
                position: (x, y),
                in_handle: (0.0, 0.0),
                out_handle: (0.0, 0.0),
            })
            .collect();
        
        Self { points }
    }
}

// ================== Legacy Adapter ==================

/// Adapter that implements the new interface using the legacy RiveEngine trait
pub struct LegacyRiveEngineAdapter<T: crate::RiveEngine> {
    engine: T,
    event_handler: Option<Box<dyn TimelineEventHandler>>,
    keyframe_counter: std::sync::atomic::AtomicU32,
    tween_counter: std::sync::atomic::AtomicU32,
}

impl<T: crate::RiveEngine> LegacyRiveEngineAdapter<T> {
    pub fn new(engine: T) -> Self {
        Self {
            engine,
            event_handler: None,
            keyframe_counter: std::sync::atomic::AtomicU32::new(0),
            tween_counter: std::sync::atomic::AtomicU32::new(0),
        }
    }
    
    fn next_keyframe_id(&self) -> KeyframeId {
        let _id = self.keyframe_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        KeyframeId::new()
    }
    
    fn next_tween_id(&self) -> TweenId {
        let _id = self.tween_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        TweenId::new()
    }
}

impl<T: crate::RiveEngine> RiveController for LegacyRiveEngineAdapter<T> {
    fn play(&mut self) {
        self.engine.play();
        if let Some(handler) = &mut self.event_handler {
            handler.on_playback_started();
        }
    }
    
    fn pause(&mut self) {
        self.engine.pause();
        if let Some(handler) = &mut self.event_handler {
            handler.on_playback_stopped();
        }
    }
    
    fn stop(&mut self) {
        self.pause(); // Legacy engine doesn't distinguish stop from pause
    }
    
    fn seek_to_frame(&mut self, frame: u32) {
        self.engine.seek(frame);
        if let Some(handler) = &mut self.event_handler {
            handler.on_frame_changed(frame);
        }
    }
    
    fn set_fps(&mut self, _fps: f32) {
        // Legacy engine doesn't support runtime FPS changes
        eprintln!("Warning: Legacy RiveEngine doesn't support runtime FPS changes");
    }
    
    fn add_layer(&mut self, name: &str) -> LayerId {
        self.engine.add_layer(name.to_string(), crate::layer::LayerType::Normal)
    }
    
    fn remove_layer(&mut self, layer_id: LayerId) {
        let layer_id_clone = layer_id.clone();
        self.engine.delete_layer(layer_id);
        if let Some(handler) = &mut self.event_handler {
            handler.on_layer_removed(layer_id_clone);
        }
    }
    
    fn reorder_layers(&mut self, _layer_ids: Vec<LayerId>) {
        eprintln!("Warning: Legacy RiveEngine doesn't support layer reordering");
    }
    
    fn set_layer_visibility(&mut self, layer_id: LayerId, visible: bool) {
        self.engine.set_property(layer_id, 0, "visible", visible);
    }
    
    fn set_layer_locked(&mut self, layer_id: LayerId, locked: bool) {
        self.engine.set_property(layer_id, 0, "locked", locked);
    }
    
    fn rename_layer(&mut self, layer_id: LayerId, new_name: String) {
        self.engine.rename_layer(layer_id, new_name);
    }
    
    fn add_keyframe(&mut self, layer_id: LayerId, frame: u32) -> KeyframeId {
        self.engine.insert_keyframe(layer_id.clone(), frame);
        let keyframe_id = self.next_keyframe_id();
        if let Some(handler) = &mut self.event_handler {
            handler.on_keyframe_added(layer_id, frame, keyframe_id.clone());
        }
        keyframe_id
    }
    
    fn remove_keyframe(&mut self, keyframe_id: KeyframeId) {
        // Legacy engine doesn't track keyframe IDs, so this is a no-op
        eprintln!("Warning: Legacy RiveEngine doesn't support keyframe ID-based removal");
        if let Some(handler) = &mut self.event_handler {
            handler.on_keyframe_removed(keyframe_id);
        }
    }
    
    fn move_keyframe(&mut self, _keyframe_id: KeyframeId, _new_frame: u32) {
        eprintln!("Warning: Legacy RiveEngine doesn't support keyframe ID-based operations");
    }
    
    fn copy_keyframe(&mut self, _keyframe_id: KeyframeId) -> Option<KeyframeData> {
        eprintln!("Warning: Legacy RiveEngine doesn't support keyframe ID-based copy");
        None
    }
    
    fn paste_keyframe(&mut self, layer_id: LayerId, frame: u32, _data: KeyframeData) {
        // Convert to legacy operation
        self.engine.insert_keyframe(layer_id, frame);
    }
    
    fn insert_frame(&mut self, layer_id: LayerId, frame: u32) {
        self.engine.insert_frame(layer_id, frame);
    }
    
    fn remove_frame(&mut self, layer_id: LayerId, frame: u32) {
        self.engine.remove_frame(layer_id, frame);
    }
    
    fn extend_frame(&mut self, layer_id: LayerId, frame: u32) {
        self.engine.insert_frame(layer_id, frame);
    }
    
    fn create_tween(&mut self, layer_id: LayerId, start_frame: u32, _end_frame: u32, tween_type: TweenType) {
        match tween_type {
            TweenType::Motion => self.engine.create_motion_tween(layer_id, start_frame),
            TweenType::Shape => self.engine.create_shape_tween(layer_id, start_frame),
            TweenType::Classic => self.engine.create_motion_tween(layer_id, start_frame), // Fallback
        }
    }
    
    fn set_easing(&mut self, _tween_id: TweenId, _easing: EasingFunction) {
        eprintln!("Warning: Legacy RiveEngine doesn't support easing configuration");
    }
    
    fn remove_tween(&mut self, _tween_id: TweenId) {
        eprintln!("Warning: Legacy RiveEngine doesn't support tween removal");
    }
}

impl<T: crate::RiveEngine> RiveDataProvider for LegacyRiveEngineAdapter<T> {
    fn get_layers(&self) -> Vec<LayerInfo> {
        self.engine.get_layers()
    }
    
    fn get_keyframes(&self, _layer_id: LayerId) -> Vec<KeyframeInfo> {
        // Legacy engine doesn't have a way to enumerate keyframes
        // Return empty for now
        Vec::new()
    }
    
    fn get_frame_data(&self, layer_id: LayerId, frame: u32) -> Option<RiveFrameData> {
        let data = self.engine.get_frame_data(layer_id, frame);
        Some(RiveFrameData {
            id: data.id,
            frame_number: data.frame_number,
            frame_type: match data.frame_type {
                crate::frame::FrameType::Empty => KeyframeType::Empty,
                crate::frame::FrameType::Keyframe => KeyframeType::Keyframe,
                crate::frame::FrameType::Tween => KeyframeType::Keyframe, // Map tween to keyframe
            },
            has_content: data.has_content,
        })
    }
    
    fn get_current_frame(&self) -> u32 {
        self.engine.get_current_frame()
    }
    
    fn get_total_frames(&self) -> u32 {
        self.engine.get_total_frames()
    }
    
    fn get_fps(&self) -> f32 {
        self.engine.get_fps()
    }
    
    fn get_animation_bounds(&self) -> (u32, u32) {
        (0, self.engine.get_total_frames())
    }
    
    fn get_layer_hierarchy(&self) -> Vec<LayerHierarchyNode> {
        // Convert flat layer list to hierarchy
        self.engine.get_layers().into_iter()
            .map(|layer| LayerHierarchyNode {
                layer_info: layer,
                children: Vec::new(),
                depth: 0,
            })
            .collect()
    }
    
    fn get_tweens(&self, _layer_id: LayerId) -> Vec<TweenInfo> {
        // Legacy engine doesn't track tweens separately
        Vec::new()
    }
    
    fn is_playing(&self) -> bool {
        // Legacy engine doesn't track playing state
        false
    }
}

impl<T: crate::RiveEngine> RiveEngine for LegacyRiveEngineAdapter<T> {
    fn set_event_handler(&mut self, handler: Box<dyn TimelineEventHandler>) {
        self.event_handler = Some(handler);
    }
    
    fn event_handler(&self) -> Option<&dyn TimelineEventHandler> {
        self.event_handler.as_ref().map(|h| h.as_ref())
    }
    
    fn initialize(&mut self, _config: RiveConfig) -> Result<(), RiveError> {
        // Legacy engine is always initialized
        Ok(())
    }
    
    fn shutdown(&mut self) {
        // Legacy engine doesn't need shutdown
    }
}

/// Frame data for the Rive integration interface
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiveFrameData {
    pub id: crate::frame::KeyframeId,
    pub frame_number: u32,
    pub frame_type: KeyframeType,
    pub has_content: bool,
}