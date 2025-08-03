//! Frame and keyframe management

use serde::{Deserialize, Serialize};
use crate::LayerId;

/// Unique identifier for a keyframe
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyframeId(pub String);

impl KeyframeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

/// Type of frame
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameType {
    Empty,
    Keyframe,
    Tween,
}

/// Frame data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    pub frame_type: FrameType,
    pub keyframe_id: Option<KeyframeId>,
    pub tween_info: Option<TweenInfo>,
}

impl Frame {
    pub fn new_empty() -> Self {
        Self {
            frame_type: FrameType::Empty,
            keyframe_id: None,
            tween_info: None,
        }
    }

    pub fn new_keyframe() -> Self {
        Self {
            frame_type: FrameType::Keyframe,
            keyframe_id: Some(KeyframeId::new()),
            tween_info: None,
        }
    }

    pub fn new_tween(tween_info: TweenInfo) -> Self {
        Self {
            frame_type: FrameType::Tween,
            keyframe_id: None,
            tween_info: Some(tween_info),
        }
    }
}

/// Information about a tween
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TweenInfo {
    pub tween_type: TweenType,
    pub easing: EasingFunction,
    pub start_frame: u32,
    pub end_frame: u32,
}

/// Type of tween
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TweenType {
    Motion,
    Shape,
    Classic,
}

/// Easing function for tweens
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Custom(Vec<(f32, f32)>), // Bezier control points
}

/// Frame data returned by the engine
#[derive(Clone, Debug)]
pub struct FrameData {
    pub frame_number: u32,
    pub frame_type: FrameType,
    pub has_content: bool,
    pub id: KeyframeId,
}

impl Default for FrameData {
    fn default() -> Self {
        Self {
            frame_number: 0,
            frame_type: FrameType::Empty,
            has_content: false,
            id: KeyframeId::new(),
        }
    }
}

/// Mock implementation for frame data
pub fn create_mock_frame_data(layer_id: &LayerId, frame: u32) -> FrameData {
    // Create a more visible pattern for testing
    let frame_type = match (layer_id.0.as_str(), frame) {
        // Audio layers: only keyframes at specific points
        (id, f) if id.contains("layer6") || id.contains("layer7") => {
            if f % 15 == 0 && f > 0 { FrameType::Keyframe } else { FrameType::Empty }
        }
        // Normal layers: more frequent keyframes for visibility
        (_, f) if f % 5 == 0 && f > 0 => FrameType::Keyframe,
        (_, f) if f % 5 < 3 && f % 5 > 0 => FrameType::Tween,
        _ => FrameType::Empty,
    };

    FrameData {
        frame_number: frame,
        frame_type,
        has_content: !matches!(frame_type, FrameType::Empty),
        id: KeyframeId::new(),
    }
}

use uuid;