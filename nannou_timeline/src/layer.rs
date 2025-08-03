//! Layer management for the timeline

use serde::{Deserialize, Serialize};

/// Unique identifier for a layer
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(pub String);

impl LayerId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// Type of layer
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerType {
    Normal,
    Mask,
    Guide,
    Folder,
    Audio,
}

/// Information about a layer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayerInfo {
    pub id: LayerId,
    pub name: String,
    pub layer_type: LayerType,
    pub visible: bool,
    pub locked: bool,
    pub parent_id: Option<LayerId>,
    pub children: Vec<LayerId>,
}

impl LayerInfo {
    /// Create a new layer with default values
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            id: LayerId::new(format!("layer_{}", uuid::Uuid::new_v4())),
            name,
            layer_type: LayerType::Normal,
            visible: true,
            locked: false,
            parent_id: None,
            children: vec![],
        }
    }

    /// Create a folder layer
    pub fn new_folder(name: impl Into<String>) -> Self {
        let mut layer = Self::new(name);
        layer.layer_type = LayerType::Folder;
        layer
    }
    
    /// Create an audio layer
    pub fn new_audio(name: impl Into<String>) -> Self {
        let mut layer = Self::new(name);
        layer.layer_type = LayerType::Audio;
        layer
    }
}

/// Layer structure for internal use
#[derive(Clone, Debug)]
pub struct Layer {
    pub info: LayerInfo,
    pub frames: std::collections::HashMap<u32, crate::frame::Frame>,
}

impl Layer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            info: LayerInfo::new(name),
            frames: std::collections::HashMap::new(),
        }
    }

    /// Add a keyframe at the specified frame number
    pub fn add_keyframe(&mut self, frame_number: u32) {
        self.frames.insert(frame_number, crate::frame::Frame::new_keyframe());
    }

    /// Remove a keyframe
    pub fn remove_keyframe(&mut self, frame_number: u32) {
        self.frames.remove(&frame_number);
    }

    /// Check if a frame has a keyframe
    pub fn has_keyframe(&self, frame_number: u32) -> bool {
        self.frames.contains_key(&frame_number)
    }
}

/// Mock layer data for testing
pub fn create_mock_layers() -> Vec<LayerInfo> {
    vec![
        LayerInfo {
            id: LayerId::new("layer1"),
            name: "Background".to_string(),
            layer_type: LayerType::Normal,
            visible: true,
            locked: false,
            parent_id: None,
            children: vec![],
        },
        LayerInfo {
            id: LayerId::new("layer2"),
            name: "Character".to_string(),
            layer_type: LayerType::Normal,
            visible: true,
            locked: false,
            parent_id: None,
            children: vec![],
        },
        LayerInfo {
            id: LayerId::new("layer3"),
            name: "Effects".to_string(),
            layer_type: LayerType::Folder,
            visible: true,
            locked: false,
            parent_id: None,
            children: vec![LayerId::new("layer4"), LayerId::new("layer5")],
        },
        LayerInfo {
            id: LayerId::new("layer4"),
            name: "Glow".to_string(),
            layer_type: LayerType::Normal,
            visible: false,
            locked: false,
            parent_id: Some(LayerId::new("layer3")),
            children: vec![],
        },
        LayerInfo {
            id: LayerId::new("layer5"),
            name: "Particles".to_string(),
            layer_type: LayerType::Normal,
            visible: true,
            locked: true,
            parent_id: Some(LayerId::new("layer3")),
            children: vec![],
        },
        LayerInfo {
            id: LayerId::new("layer6"),
            name: "Background Music".to_string(),
            layer_type: LayerType::Audio,
            visible: true,
            locked: false,
            parent_id: None,
            children: vec![],
        },
        LayerInfo {
            id: LayerId::new("layer7"),
            name: "Sound Effects".to_string(),
            layer_type: LayerType::Audio,
            visible: true,
            locked: false,
            parent_id: None,
            children: vec![],
        },
    ]
}

// Add uuid dependency for generating unique IDs
use uuid;