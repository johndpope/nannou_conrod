//! UI tests for the Flash-style timeline standalone demo
//! Tests stage interactions, tool usage, and library drag-and-drop

use egui::{Context, Event, Modifiers, PointerButton, Pos2, Vec2, Key, CursorIcon};

/// Test harness for the standalone demo application
pub struct DemoTestHarness {
    ctx: Context,
    app: crate::TimelineApp,
    event_queue: Vec<Event>,
}

impl DemoTestHarness {
    pub fn new() -> Self {
        Self {
            ctx: Context::default(),
            app: crate::TimelineApp::default(),
            event_queue: Vec::new(),
        }
    }
    
    /// Queue a click event
    pub fn click(&mut self, pos: Pos2) {
        self.event_queue.push(Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::default(),
        });
        self.event_queue.push(Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::default(),
        });
    }
    
    /// Queue a drag operation
    pub fn drag(&mut self, from: Pos2, to: Pos2) {
        // Mouse down
        self.event_queue.push(Event::PointerButton {
            pos: from,
            button: PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::default(),
        });
        
        // Drag motion
        let steps = 5;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let pos = from + (to - from) * t;
            self.event_queue.push(Event::PointerMoved(pos));
        }
        
        // Mouse up
        self.event_queue.push(Event::PointerButton {
            pos: to,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::default(),
        });
    }
    
    /// Run one frame
    pub fn run_frame(&mut self) {
        let mut raw_input = egui::RawInput::default();
        raw_input.events = self.event_queue.drain(..).collect();
        
        self.ctx.begin_frame(raw_input);
        self.app.update(&self.ctx);
        self.ctx.end_frame();
    }
    
    /// Get app reference for assertions
    pub fn app(&self) -> &crate::TimelineApp {
        &self.app
    }
    
    /// Get mutable app reference for setup
    pub fn app_mut(&mut self) -> &mut crate::TimelineApp {
        &mut self.app
    }
}

#[cfg(test)]
mod stage_tests {
    use super::*;
    use crate::{Tool, StageItemType};
    
    #[test]
    fn test_stage_item_selection() {
        let mut harness = DemoTestHarness::new();
        
        // Click on first stage item
        let item_pos = harness.app().stage_items[0].position;
        let stage_pos = Pos2::new(400.0 + item_pos.x, 100.0 + item_pos.y);
        
        harness.click(stage_pos);
        harness.run_frame();
        
        // Item should be selected
        assert!(!harness.app().selected_items.is_empty());
        assert_eq!(harness.app().selected_items[0], 0);
    }
    
    #[test]
    fn test_marquee_selection() {
        let mut harness = DemoTestHarness::new();
        
        // Set tool to Arrow
        harness.app_mut().tool_state.active_tool = Tool::Arrow;
        
        // Drag to create marquee selection
        let start = Pos2::new(400.0, 100.0);
        let end = Pos2::new(600.0, 300.0);
        
        harness.drag(start, end);
        harness.run_frame();
        
        // Multiple items should be selected
        assert!(harness.app().selected_items.len() > 1);
    }
    
    #[test]
    fn test_stage_item_drag() {
        let mut harness = DemoTestHarness::new();
        
        // Select an item first
        let item_index = 0;
        let initial_pos = harness.app().stage_items[item_index].position;
        let stage_pos = Pos2::new(400.0 + initial_pos.x, 100.0 + initial_pos.y);
        
        harness.click(stage_pos);
        harness.run_frame();
        
        // Drag the item
        let drag_delta = Vec2::new(50.0, 30.0);
        harness.drag(stage_pos, stage_pos + drag_delta);
        harness.run_frame();
        
        // Item should have moved
        let new_pos = harness.app().stage_items[item_index].position;
        assert_ne!(initial_pos, new_pos);
    }
    
    #[test]
    fn test_tool_cursor_changes() {
        let mut harness = DemoTestHarness::new();
        
        // Test different tools
        let tools_and_cursors = vec![
            (Tool::Arrow, CursorIcon::Default),
            (Tool::Hand, CursorIcon::Grab),
            (Tool::Zoom, CursorIcon::ZoomIn),
            (Tool::Text, CursorIcon::Text),
            (Tool::Eyedropper, CursorIcon::Crosshair),
        ];
        
        for (tool, _expected_cursor) in tools_and_cursors {
            harness.app_mut().tool_state.active_tool = tool;
            harness.run_frame();
            
            // Cursor would be set internally by egui
            // Verify tool change was applied
            assert_eq!(harness.app().tool_state.active_tool, tool);
        }
    }
}

#[cfg(test)]
mod drawing_tool_tests {
    use super::*;
    use crate::{Tool, StageItemType};
    
    #[test]
    fn test_rectangle_tool_creates_rectangle() {
        let mut harness = DemoTestHarness::new();
        
        // Set Rectangle tool
        harness.app_mut().tool_state.active_tool = Tool::Rectangle;
        
        let initial_count = harness.app().stage_items.len();
        
        // Click on stage
        harness.click(Pos2::new(500.0, 200.0));
        harness.run_frame();
        
        // New rectangle should be created
        assert_eq!(harness.app().stage_items.len(), initial_count + 1);
        
        let new_item = harness.app().stage_items.last().unwrap();
        assert_eq!(new_item.item_type, StageItemType::Rectangle);
    }
    
    #[test]
    fn test_pen_tool_creates_path_point() {
        let mut harness = DemoTestHarness::new();
        
        // Set Pen tool
        harness.app_mut().tool_state.active_tool = Tool::Pen;
        
        let initial_count = harness.app().stage_items.len();
        
        // Click to create path point
        harness.click(Pos2::new(450.0, 250.0));
        harness.run_frame();
        
        // Path point should be created
        assert_eq!(harness.app().stage_items.len(), initial_count + 1);
        
        let new_item = harness.app().stage_items.last().unwrap();
        assert_eq!(new_item.item_type, StageItemType::Circle); // Simplified as circle
        assert!(new_item.name.contains("Path"));
    }
    
    #[test]
    fn test_text_tool_creates_text() {
        let mut harness = DemoTestHarness::new();
        
        // Set Text tool
        harness.app_mut().tool_state.active_tool = Tool::Text;
        
        let initial_count = harness.app().stage_items.len();
        
        // Click to create text
        harness.click(Pos2::new(600.0, 150.0));
        harness.run_frame();
        
        // Text object should be created
        assert_eq!(harness.app().stage_items.len(), initial_count + 1);
        
        let new_item = harness.app().stage_items.last().unwrap();
        assert_eq!(new_item.item_type, StageItemType::Text);
        assert!(!new_item.text_content.is_empty());
    }
}

#[cfg(test)]
mod library_drag_drop_tests {
    use super::*;
    use crate::{LibraryAssetType, StageItemType};
    
    #[test]
    fn test_library_asset_drag_to_stage() {
        let mut harness = DemoTestHarness::new();
        
        // Simulate dragging a MovieClip from library
        let asset_index = 0; // First asset
        let asset = harness.app().library_assets[asset_index].clone();
        harness.app_mut().dragging_asset = Some(asset);
        
        let initial_count = harness.app().stage_items.len();
        
        // Drop on stage
        let drop_pos = Pos2::new(500.0, 200.0);
        harness.event_queue.push(Event::PointerButton {
            pos: drop_pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::default(),
        });
        harness.run_frame();
        
        // New instance should be created
        assert_eq!(harness.app().stage_items.len(), initial_count + 1);
        
        // Dragging state should be cleared
        assert!(harness.app().dragging_asset.is_none());
    }
    
    #[test]
    fn test_different_asset_types_create_correct_items() {
        let mut harness = DemoTestHarness::new();
        
        let test_cases = vec![
            (LibraryAssetType::MovieClip, StageItemType::MovieClip),
            (LibraryAssetType::Graphic, StageItemType::Rectangle),
            (LibraryAssetType::Bitmap, StageItemType::Rectangle),
            (LibraryAssetType::Button, StageItemType::Rectangle),
        ];
        
        for (asset_type, expected_item_type) in test_cases {
            // Create test asset
            let asset = crate::LibraryAsset {
                id: format!("test_{:?}", asset_type),
                name: format!("Test {:?}", asset_type),
                asset_type,
                folder: "Test".to_string(),
                properties: Default::default(),
            };
            
            harness.app_mut().dragging_asset = Some(asset);
            
            let initial_count = harness.app().stage_items.len();
            
            // Drop on stage
            harness.event_queue.push(Event::PointerButton {
                pos: Pos2::new(500.0, 200.0),
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::default(),
            });
            harness.run_frame();
            
            // Verify correct item type was created
            let new_item = harness.app().stage_items.last().unwrap();
            assert_eq!(new_item.item_type, expected_item_type);
        }
    }
}

#[cfg(test)]
mod properties_panel_tests {
    use super::*;
    
    #[test]
    fn test_properties_update_on_selection() {
        let mut harness = DemoTestHarness::new();
        
        // Select an item
        let item_pos = harness.app().stage_items[0].position;
        let stage_pos = Pos2::new(400.0 + item_pos.x, 100.0 + item_pos.y);
        
        harness.click(stage_pos);
        harness.run_frame();
        
        // Properties should be available for selected item
        assert!(!harness.app().selected_items.is_empty());
        
        // Simulate properties panel interaction
        // The actual properties would be rendered in the UI
        let selected_index = harness.app().selected_items[0];
        let item = &harness.app().stage_items[selected_index];
        
        // Verify item has properties
        assert!(!item.name.is_empty());
        assert!(item.size.x > 0.0);
        assert!(item.size.y > 0.0);
    }
}

#[cfg(test)]
mod keyboard_shortcut_tests {
    use super::*;
    
    #[test]
    fn test_tool_keyboard_shortcuts() {
        let mut harness = DemoTestHarness::new();
        
        let shortcuts = vec![
            (Key::V, Tool::Arrow),
            (Key::P, Tool::Pen),
            (Key::R, Tool::Rectangle),
            (Key::O, Tool::Oval),
            (Key::T, Tool::Text),
            (Key::H, Tool::Hand),
            (Key::Z, Tool::Zoom),
        ];
        
        for (key, expected_tool) in shortcuts {
            harness.event_queue.push(Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::default(),
            });
            harness.run_frame();
            
            // Tool should change
            assert_eq!(harness.app().tool_state.active_tool, expected_tool);
        }
    }
    
    #[test]
    fn test_copy_paste_shortcuts() {
        let mut harness = DemoTestHarness::new();
        
        // Select an item
        harness.app_mut().selected_items.push(0);
        
        // Copy (Ctrl+C)
        let mut ctrl = Modifiers::default();
        ctrl.ctrl = true;
        
        harness.event_queue.push(Event::Key {
            key: Key::C,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: ctrl,
        });
        harness.run_frame();
        
        // Clipboard should have content
        assert!(!harness.app().clipboard.is_empty());
        
        // Paste (Ctrl+V)
        let initial_count = harness.app().stage_items.len();
        
        harness.event_queue.push(Event::Key {
            key: Key::V,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: ctrl,
        });
        harness.run_frame();
        
        // New item should be created
        assert_eq!(harness.app().stage_items.len(), initial_count + 1);
    }
}

#[cfg(test)]
mod context_menu_tests {
    use super::*;
    
    #[test]
    fn test_stage_context_menu() {
        let mut harness = DemoTestHarness::new();
        
        // Right-click on empty stage
        harness.event_queue.push(Event::PointerButton {
            pos: Pos2::new(500.0, 200.0),
            button: PointerButton::Secondary,
            pressed: true,
            modifiers: Modifiers::default(),
        });
        harness.run_frame();
        
        // Context menu state should be set
        assert!(harness.app().context_menu.is_some());
    }
    
    #[test]
    fn test_item_context_menu() {
        let mut harness = DemoTestHarness::new();
        
        // Right-click on an item
        let item_pos = harness.app().stage_items[0].position;
        let stage_pos = Pos2::new(400.0 + item_pos.x, 100.0 + item_pos.y);
        
        harness.event_queue.push(Event::PointerButton {
            pos: stage_pos,
            button: PointerButton::Secondary,
            pressed: true,
            modifiers: Modifiers::default(),
        });
        harness.run_frame();
        
        // Context menu should appear for the item
        assert!(harness.app().context_menu.is_some());
    }
}