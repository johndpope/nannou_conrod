//! Scene management for multi-scene animation projects

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use crate::layer::LayerId;

/// Unique identifier for a scene
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SceneId(pub String);

impl SceneId {
    pub fn new() -> Self {
        Self(format!("scene_{}", Uuid::new_v4()))
    }

    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl Default for SceneId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SceneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Scene configuration and properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneProperties {
    /// Scene name/title
    pub name: String,
    /// Optional scene description
    pub description: String,
    /// Frame rate for this scene (can override global)
    pub frame_rate: Option<f32>,
    /// Stage dimensions for this scene
    pub stage_size: Option<(u32, u32)>,
    /// Background color (RGBA)
    pub background_color: Option<[f32; 4]>,
    /// Total frame count in this scene
    pub frame_count: u32,
    /// Whether this scene has unsaved changes
    pub modified: bool,
}

impl Default for SceneProperties {
    fn default() -> Self {
        Self {
            name: "Scene 1".to_string(),
            description: String::new(),
            frame_rate: None, // Use global default
            stage_size: None, // Use global default
            background_color: None, // Use global default
            frame_count: 100,
            modified: false,
        }
    }
}

/// A scene containing timeline, layers, and assets
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
    /// Unique scene identifier
    pub id: SceneId,
    /// Scene properties and configuration
    pub properties: SceneProperties,
    /// Layers in this scene
    pub layers: Vec<LayerId>,
    /// Scene-specific assets (not shared)
    pub local_assets: Vec<String>, // Asset IDs
    /// Current playhead position in this scene
    pub current_frame: u32,
    /// Selected layers in this scene
    pub selected_layers: Vec<LayerId>,
    /// Scene creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

impl Scene {
    /// Create a new scene with default properties
    pub fn new(name: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: SceneId::new(),
            properties: SceneProperties {
                name: name.into(),
                ..Default::default()
            },
            layers: Vec::new(),
            local_assets: Vec::new(),
            current_frame: 0,
            selected_layers: Vec::new(),
            created_at: now,
            modified_at: now,
        }
    }

    /// Create a duplicate of this scene
    pub fn duplicate(&self, new_name: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: SceneId::new(),
            properties: SceneProperties {
                name: new_name.into(),
                modified: true, // Mark as modified since it's a new scene
                ..self.properties.clone()
            },
            layers: self.layers.clone(),
            local_assets: self.local_assets.clone(),
            current_frame: self.current_frame,
            selected_layers: Vec::new(), // Clear selection in duplicate
            created_at: now,
            modified_at: now,
        }
    }

    /// Mark scene as modified
    pub fn mark_modified(&mut self) {
        self.properties.modified = true;
        self.modified_at = chrono::Utc::now();
    }

    /// Mark scene as saved
    pub fn mark_saved(&mut self) {
        self.properties.modified = false;
    }

    /// Get display name for the scene
    pub fn display_name(&self) -> String {
        if self.properties.modified {
            format!("{}*", self.properties.name)
        } else {
            self.properties.name.clone()
        }
    }

    /// Get scene summary information
    pub fn summary(&self) -> SceneSummary {
        SceneSummary {
            id: self.id.clone(),
            name: self.properties.name.clone(),
            layer_count: self.layers.len(),
            frame_count: self.properties.frame_count as usize,
            modified: self.properties.modified,
        }
    }
}

/// Scene summary for UI display
#[derive(Clone, Debug)]
pub struct SceneSummary {
    pub id: SceneId,
    pub name: String,
    pub layer_count: usize,
    pub frame_count: usize,
    pub modified: bool,
}

/// Scene manager for handling multiple scenes
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SceneManager {
    /// All scenes in the project
    scenes: HashMap<SceneId, Scene>,
    /// Currently active scene
    active_scene_id: Option<SceneId>,
    /// Scene order for tab display
    scene_order: Vec<SceneId>,
    /// Recently accessed scenes (for history)
    recent_scenes: Vec<SceneId>,
}

impl SceneManager {
    /// Create a new scene manager with a default scene
    pub fn new() -> Self {
        let mut manager = Self {
            scenes: HashMap::new(),
            active_scene_id: None,
            scene_order: Vec::new(),
            recent_scenes: Vec::new(),
        };

        // Create the first default scene
        let scene = Scene::new("Scene 1");
        manager.add_scene(scene);
        
        manager
    }

    /// Add a new scene
    pub fn add_scene(&mut self, scene: Scene) -> SceneId {
        let scene_id = scene.id.clone();
        
        // If this is the first scene, make it active
        if self.active_scene_id.is_none() {
            self.active_scene_id = Some(scene_id.clone());
        }
        
        self.scenes.insert(scene_id.clone(), scene);
        self.scene_order.push(scene_id.clone());
        
        scene_id
    }

    /// Create a new scene with given name
    pub fn create_scene(&mut self, name: impl Into<String>) -> SceneId {
        let scene = Scene::new(name);
        self.add_scene(scene)
    }

    /// Remove a scene
    pub fn remove_scene(&mut self, scene_id: &SceneId) -> Result<(), String> {
        if self.scenes.len() <= 1 {
            return Err("Cannot delete the last scene".to_string());
        }

        if let Some(_scene) = self.scenes.remove(scene_id) {
            // Remove from order
            self.scene_order.retain(|id| id != scene_id);
            
            // If this was the active scene, switch to another
            if self.active_scene_id.as_ref() == Some(scene_id) {
                self.active_scene_id = self.scene_order.first().cloned();
            }
            
            // Remove from recent list
            self.recent_scenes.retain(|id| id != scene_id);
            
            Ok(())
        } else {
            Err("Scene not found".to_string())
        }
    }

    /// Duplicate a scene
    pub fn duplicate_scene(&mut self, scene_id: &SceneId, new_name: impl Into<String>) -> Result<SceneId, String> {
        if let Some(scene) = self.scenes.get(scene_id) {
            let duplicate = scene.duplicate(new_name);
            let new_id = duplicate.id.clone();
            
            // Insert after the original scene
            if let Some(pos) = self.scene_order.iter().position(|id| id == scene_id) {
                self.scene_order.insert(pos + 1, new_id.clone());
            } else {
                self.scene_order.push(new_id.clone());
            }
            
            self.scenes.insert(new_id.clone(), duplicate);
            Ok(new_id)
        } else {
            Err("Scene not found".to_string())
        }
    }

    /// Switch to a different scene
    pub fn switch_to_scene(&mut self, scene_id: &SceneId) -> Result<(), String> {
        if self.scenes.contains_key(scene_id) {
            // Add to recent list
            if let Some(current_id) = &self.active_scene_id {
                self.recent_scenes.retain(|id| id != current_id);
                self.recent_scenes.insert(0, current_id.clone());
                
                // Keep only last 10 recent scenes
                if self.recent_scenes.len() > 10 {
                    self.recent_scenes.truncate(10);
                }
            }
            
            self.active_scene_id = Some(scene_id.clone());
            Ok(())
        } else {
            Err("Scene not found".to_string())
        }
    }

    /// Get the currently active scene
    pub fn get_active_scene(&self) -> Option<&Scene> {
        self.active_scene_id.as_ref()
            .and_then(|id| self.scenes.get(id))
    }

    /// Get the currently active scene mutably
    pub fn get_active_scene_mut(&mut self) -> Option<&mut Scene> {
        self.active_scene_id.clone()
            .and_then(move |id| self.scenes.get_mut(&id))
    }

    /// Get a scene by ID
    pub fn get_scene(&self, scene_id: &SceneId) -> Option<&Scene> {
        self.scenes.get(scene_id)
    }

    /// Get a scene by ID mutably
    pub fn get_scene_mut(&mut self, scene_id: &SceneId) -> Option<&mut Scene> {
        self.scenes.get_mut(scene_id)
    }

    /// Get all scenes in display order
    pub fn get_scenes_ordered(&self) -> Vec<&Scene> {
        self.scene_order.iter()
            .filter_map(|id| self.scenes.get(id))
            .collect()
    }

    /// Get scene summaries for UI display
    pub fn get_scene_summaries(&self) -> Vec<SceneSummary> {
        self.scene_order.iter()
            .filter_map(|id| self.scenes.get(id))
            .map(|scene| scene.summary())
            .collect()
    }

    /// Rename a scene
    pub fn rename_scene(&mut self, scene_id: &SceneId, new_name: impl Into<String>) -> Result<(), String> {
        if let Some(scene) = self.scenes.get_mut(scene_id) {
            scene.properties.name = new_name.into();
            scene.mark_modified();
            Ok(())
        } else {
            Err("Scene not found".to_string())
        }
    }

    /// Move scene to different position
    pub fn move_scene(&mut self, scene_id: &SceneId, new_index: usize) -> Result<(), String> {
        if let Some(current_pos) = self.scene_order.iter().position(|id| id == scene_id) {
            let scene_id = self.scene_order.remove(current_pos);
            let insert_pos = new_index.min(self.scene_order.len());
            self.scene_order.insert(insert_pos, scene_id);
            Ok(())
        } else {
            Err("Scene not found".to_string())
        }
    }

    /// Get active scene ID
    pub fn get_active_scene_id(&self) -> Option<&SceneId> {
        self.active_scene_id.as_ref()
    }

    /// Check if there are any modified scenes
    pub fn has_unsaved_changes(&self) -> bool {
        self.scenes.values().any(|scene| scene.properties.modified)
    }

    /// Get count of scenes
    pub fn scene_count(&self) -> usize {
        self.scenes.len()
    }
}