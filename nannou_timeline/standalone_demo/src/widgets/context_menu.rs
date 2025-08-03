//! Context menu functionality for stage and stage items

use crate::{
    stage::{StageItem, StageItemType, ContextMenuState, ContextMenuType},
    TimelineApp,
    LogLevel,
};
use egui::{self, Pos2, Vec2, Color32};

impl TimelineApp {
    pub fn show_context_menu(&mut self, ui: &mut egui::Ui, menu_state: &ContextMenuState, _stage_rect: egui::Rect) {
        // Create a window for the context menu
        egui::Window::new("context_menu")
            .fixed_pos(menu_state.position)
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                match &menu_state.menu_type {
                    ContextMenuType::Stage(stage_pos) => {
                        self.show_stage_context_menu(ui, *stage_pos);
                    },
                    ContextMenuType::StageItem(index) => {
                        self.show_stage_item_context_menu(ui, *index);
                    }
                }
            });
    }
    
    fn show_stage_context_menu(&mut self, ui: &mut egui::Ui, stage_pos: Pos2) {
        ui.label("Stage Context Menu");
        ui.separator();
        
        if ui.button("➕ Add Rectangle").clicked() {
            let new_item = StageItem {
                id: format!("rect_{}", self.stage_items.len() + 1),
                name: format!("Rectangle {}", self.stage_items.len() + 1),
                item_type: StageItemType::Rectangle,
                position: stage_pos,
                size: Vec2::new(100.0, 60.0),
                color: Color32::from_rgb(150, 150, 255),
                alpha: 1.0,
                rotation: 0.0,
                text_content: "Default Text".to_string(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
                path_points: Vec::new(),
            };
            self.stage_items.push(new_item.clone());
            self.log(LogLevel::Action, format!("Added {} at ({:.1}, {:.1})", 
                new_item.name, stage_pos.x, stage_pos.y));
            self.context_menu = None;
        }
        
        if ui.button("⭕ Add Circle").clicked() {
            let new_item = StageItem {
                id: format!("circle_{}", self.stage_items.len() + 1),
                name: format!("Circle {}", self.stage_items.len() + 1),
                item_type: StageItemType::Circle,
                position: stage_pos,
                size: Vec2::splat(80.0),
                color: Color32::from_rgb(255, 150, 150),
                alpha: 1.0,
                rotation: 0.0,
                text_content: "Default Text".to_string(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
                path_points: Vec::new(),
            };
            self.stage_items.push(new_item.clone());
            self.log(LogLevel::Action, format!("Added {} at ({:.1}, {:.1})", 
                new_item.name, stage_pos.x, stage_pos.y));
            self.context_menu = None;
        }
        
        if ui.button("📝 Add Text").clicked() {
            let new_item = StageItem {
                id: format!("text_{}", self.stage_items.len() + 1),
                name: format!("Text {}", self.stage_items.len() + 1),
                item_type: StageItemType::Text,
                position: stage_pos,
                size: Vec2::new(120.0, 30.0),
                color: Color32::from_gray(20),
                alpha: 1.0,
                rotation: 0.0,
                text_content: "Sample Text".to_string(),
                font_size: 18.0,
                font_family: "Arial".to_string(),
                path_points: Vec::new(),
            };
            self.stage_items.push(new_item.clone());
            self.log(LogLevel::Action, format!("Added {} at ({:.1}, {:.1})", 
                new_item.name, stage_pos.x, stage_pos.y));
            self.context_menu = None;
        }
        
        if ui.button("🎬 Add MovieClip").clicked() {
            let new_item = StageItem {
                id: format!("mc_{}", self.stage_items.len() + 1),
                name: format!("MovieClip {}", self.stage_items.len() + 1),
                item_type: StageItemType::MovieClip,
                position: stage_pos,
                size: Vec2::new(120.0, 80.0),
                color: Color32::from_rgb(150, 255, 150),
                alpha: 1.0,
                rotation: 0.0,
                text_content: "Default Text".to_string(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
                path_points: Vec::new(),
            };
            self.stage_items.push(new_item.clone());
            self.log(LogLevel::Action, format!("Added {} at ({:.1}, {:.1})", 
                new_item.name, stage_pos.x, stage_pos.y));
            self.context_menu = None;
        }
        
        ui.separator();
        
        if !self.clipboard.is_empty() && ui.button("📋 Paste").clicked() {
            // Calculate center of clipboard items
            let clipboard_items = self.clipboard.clone(); // Clone to avoid borrow issues
            let center = clipboard_items.iter()
                .fold(Pos2::ZERO, |acc, item| acc + item.position.to_vec2()) / clipboard_items.len() as f32;
                
            // Paste all items at the clicked position, maintaining relative positions
            for clipboard_item in &clipboard_items {
                let mut new_item = clipboard_item.clone();
                new_item.id = format!("{}_pasted_{}", new_item.id, self.stage_items.len());
                new_item.name = format!("{} (Pasted)", new_item.name);
                // Maintain relative position
                let offset = clipboard_item.position - center;
                new_item.position = stage_pos + offset;
                let new_item_name = new_item.name.clone();
                let new_item_pos = new_item.position;
                self.stage_items.push(new_item);
                self.log(LogLevel::Action, format!("Pasted {} at ({:.1}, {:.1})", 
                    new_item_name, new_item_pos.x, new_item_pos.y));
            }
            self.context_menu = None;
        }
        
        ui.separator();
        
        if ui.button("❌ Cancel").clicked() {
            self.context_menu = None;
        }
    }
    
    fn show_stage_item_context_menu(&mut self, ui: &mut egui::Ui, index: usize) {
        if let Some(item) = self.stage_items.get(index).cloned() {
            ui.label(format!("📌 {}", item.name));
            ui.separator();
            
            if ui.button("✏️ Rename").clicked() {
                self.log(LogLevel::Action, format!("Rename {} (not implemented)", item.name));
                self.context_menu = None;
            }
            
            ui.separator();
            
            // Copy selected items to clipboard
            if ui.button("📄 Copy").clicked() {
                self.clipboard.clear();
                for &selected_index in &self.selected_items {
                    if let Some(selected_item) = self.stage_items.get(selected_index) {
                        self.clipboard.push(selected_item.clone());
                    }
                }
                self.log(LogLevel::Action, format!("Copied {} item(s) to clipboard", self.clipboard.len()));
                self.context_menu = None;
            }
            
            // Cut selected items (copy and delete)
            if ui.button("✂️ Cut").clicked() {
                self.clipboard.clear();
                let mut items_to_remove = Vec::new();
                for &selected_index in &self.selected_items {
                    if let Some(selected_item) = self.stage_items.get(selected_index) {
                        self.clipboard.push(selected_item.clone());
                        items_to_remove.push(selected_index);
                    }
                }
                // Remove items in reverse order to maintain valid indices
                items_to_remove.sort_by(|a, b| b.cmp(a));
                for index in items_to_remove {
                    if index < self.stage_items.len() {
                        let removed_item = self.stage_items.remove(index);
                        self.log(LogLevel::Action, format!("Cut {} to clipboard", removed_item.name));
                    }
                }
                self.selected_items.clear();
                self.context_menu = None;
            }
            
            if ui.button("📋 Duplicate").clicked() {
                let mut new_item = item.clone();
                new_item.id = format!("{}_copy", new_item.id);
                new_item.name = format!("{} Copy", new_item.name);
                new_item.position += Vec2::splat(20.0);
                let new_index = self.stage_items.len();
                self.stage_items.push(new_item.clone());
                self.selected_items = vec![new_index];
                self.log(LogLevel::Action, format!("Duplicated {}", item.name));
                self.context_menu = None;
            }
            
            ui.separator();
            
            if ui.button("⬆️ Bring to Front").clicked() {
                if index < self.stage_items.len() {
                    let item = self.stage_items.remove(index);
                    self.stage_items.push(item.clone());
                    // Update selection index
                    self.selected_items = vec![self.stage_items.len() - 1];
                    self.log(LogLevel::Action, format!("Brought {} to front", item.name));
                    self.context_menu = None;
                }
            }
            
            if ui.button("⬇️ Send to Back").clicked() {
                if index < self.stage_items.len() {
                    let item = self.stage_items.remove(index);
                    self.stage_items.insert(0, item.clone());
                    // Update selection index
                    self.selected_items = vec![0];
                    self.log(LogLevel::Action, format!("Sent {} to back", item.name));
                    self.context_menu = None;
                }
            }
            
            ui.separator();
            
            if ui.button("🔄 Rotate 90° CW").clicked() {
                if let Some(item) = self.stage_items.get_mut(index) {
                    item.rotation = (item.rotation + 90.0) % 360.0;
                    let item_name = item.name.clone();
                    let rotation = item.rotation;
                    self.log(LogLevel::Action, format!("Rotated {} to {:.0}°", item_name, rotation));
                }
                self.context_menu = None;
            }
            
            if ui.button("🔄 Rotate 90° CCW").clicked() {
                if let Some(item) = self.stage_items.get_mut(index) {
                    item.rotation = (item.rotation - 90.0 + 360.0) % 360.0;
                    let item_name = item.name.clone();
                    let rotation = item.rotation;
                    self.log(LogLevel::Action, format!("Rotated {} to {:.0}°", item_name, rotation));
                }
                self.context_menu = None;
            }
            
            ui.separator();
            
            if ui.button("🗑️ Delete").clicked() {
                if index < self.stage_items.len() {
                    let removed_item = self.stage_items.remove(index);
                    self.selected_items.retain(|&i| i != index);
                    // Adjust selected indices for items after the removed one
                    for selected_index in &mut self.selected_items {
                        if *selected_index > index {
                            *selected_index -= 1;
                        }
                    }
                    self.log(LogLevel::Action, format!("Deleted {}", removed_item.name));
                    self.context_menu = None;
                }
            }
            
            ui.separator();
            
            if ui.button("❌ Cancel").clicked() {
                self.context_menu = None;
            }
        }
    }
}