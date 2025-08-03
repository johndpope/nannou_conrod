//! Workspace management for saving and loading panel layouts

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::{Panel, PanelId};

/// A saved workspace layout
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceLayout {
    pub name: String,
    pub panels: Vec<Panel>,
    pub saved_at: String, // ISO timestamp
}

/// Manages multiple workspace layouts
#[derive(Default)]
pub struct WorkspaceManager {
    pub layouts: HashMap<String, WorkspaceLayout>,
    pub active_workspace: String,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        let mut manager = Self {
            layouts: HashMap::new(),
            active_workspace: "Default".to_string(),
        };
        
        // Add default workspaces
        manager.add_default_workspaces();
        manager
    }
    
    /// Add built-in workspace layouts
    fn add_default_workspaces(&mut self) {
        // Essentials workspace
        let essentials = WorkspaceLayout {
            name: "Essentials".to_string(),
            panels: Vec::new(), // Will be populated by WindowManager
            saved_at: chrono::Utc::now().to_rfc3339(),
        };
        self.layouts.insert("Essentials".to_string(), essentials);
        
        // Advanced workspace
        let advanced = WorkspaceLayout {
            name: "Advanced".to_string(),
            panels: Vec::new(),
            saved_at: chrono::Utc::now().to_rfc3339(),
        };
        self.layouts.insert("Advanced".to_string(), advanced);
        
        // Animation workspace
        let animation = WorkspaceLayout {
            name: "Animation".to_string(),
            panels: Vec::new(),
            saved_at: chrono::Utc::now().to_rfc3339(),
        };
        self.layouts.insert("Animation".to_string(), animation);
    }
    
    /// Save current panel layout
    pub fn save_current(&mut self, name: String, panels: &HashMap<PanelId, Panel>) {
        let layout = WorkspaceLayout {
            name: name.clone(),
            panels: panels.values().cloned().collect(),
            saved_at: chrono::Utc::now().to_rfc3339(),
        };
        
        self.layouts.insert(name.clone(), layout);
        self.active_workspace = name;
    }
    
    /// Load a workspace layout
    pub fn load(&mut self, name: &str) -> Option<Vec<Panel>> {
        if let Some(layout) = self.layouts.get(name) {
            self.active_workspace = name.to_string();
            Some(layout.panels.clone())
        } else {
            None
        }
    }
    
    /// Get list of available workspaces
    pub fn list_workspaces(&self) -> Vec<String> {
        let mut names: Vec<_> = self.layouts.keys().cloned().collect();
        names.sort();
        names
    }
    
    /// Export workspace to JSON
    pub fn export_json(&self, name: &str) -> Option<String> {
        self.layouts.get(name)
            .and_then(|layout| serde_json::to_string_pretty(layout).ok())
    }
    
    /// Import workspace from JSON
    pub fn import_json(&mut self, json: &str) -> Result<String, String> {
        match serde_json::from_str::<WorkspaceLayout>(json) {
            Ok(layout) => {
                let name = layout.name.clone();
                self.layouts.insert(name.clone(), layout);
                Ok(name)
            },
            Err(e) => Err(format!("Failed to import workspace: {}", e)),
        }
    }
}