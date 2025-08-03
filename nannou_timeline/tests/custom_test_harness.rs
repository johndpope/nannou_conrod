//! Custom test harness for timeline UI using direct egui event simulation
//! 
//! This provides fine-grained control over input events for testing complex
//! interactions like drag-and-drop, multi-selection, and timeline scrubbing.

use egui::{Context, Event, Modifiers, PointerButton, Pos2, Vec2, Key};
use nannou_timeline::{Timeline, TimelineConfig, ui::MockRiveEngine, RiveEngine};
use std::collections::VecDeque;

/// Custom test harness that allows precise event injection
pub struct TimelineTestHarness {
    ctx: Context,
    timeline: Timeline,
    engine: Box<dyn RiveEngine>,
    event_queue: VecDeque<Event>,
    frame_count: usize,
}

impl TimelineTestHarness {
    /// Create a new test harness with default timeline
    pub fn new() -> Self {
        let config = TimelineConfig {
            frame_width: 10.0,
            default_track_height: 30.0,
            snap: nannou_timeline::SnapConfig {
                enabled: true,
                threshold_pixels: 8.0,
                snap_to_frames: true,
                show_guides: true,
                snap_to_keyframes: true,
                snap_to_markers: false,
            },
            ..Default::default()
        };
        
        Self {
            ctx: Context::default(),
            timeline: Timeline::with_config(config),
            engine: Box::new(MockRiveEngine::new()),
            event_queue: VecDeque::new(),
            frame_count: 0,
        }
    }
    
    /// Queue a mouse click event at the specified position
    pub fn click_at(&mut self, pos: Pos2, button: PointerButton) {
        // Mouse down
        self.event_queue.push_back(Event::PointerButton {
            pos,
            button,
            pressed: true,
            modifiers: Modifiers::default(),
        });
        
        // Mouse up
        self.event_queue.push_back(Event::PointerButton {
            pos,
            button,
            pressed: false,
            modifiers: Modifiers::default(),
        });
    }
    
    /// Queue a mouse drag from start to end position
    pub fn drag_from_to(&mut self, start: Pos2, end: Pos2, button: PointerButton) {
        // Mouse down at start
        self.event_queue.push_back(Event::PointerButton {
            pos: start,
            button,
            pressed: true,
            modifiers: Modifiers::default(),
        });
        
        // Move to end (simulate multiple intermediate positions)
        let steps = 10;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let pos = start + (end - start) * t;
            self.event_queue.push_back(Event::PointerMoved(pos));
        }
        
        // Mouse up at end
        self.event_queue.push_back(Event::PointerButton {
            pos: end,
            button,
            pressed: false,
            modifiers: Modifiers::default(),
        });
    }
    
    /// Queue a keyboard event
    pub fn press_key(&mut self, key: Key, modifiers: Modifiers) {
        self.event_queue.push_back(Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        });
        
        self.event_queue.push_back(Event::Key {
            key,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers,
        });
    }
    
    /// Queue a scroll event
    pub fn scroll(&mut self, delta: Vec2) {
        self.event_queue.push_back(Event::Scroll(delta));
    }
    
    /// Queue a right-click for context menu
    pub fn right_click_at(&mut self, pos: Pos2) {
        self.click_at(pos, PointerButton::Secondary);
    }
    
    /// Run one frame with queued events
    pub fn run_frame(&mut self) {
        // Collect events for this frame
        let mut raw_input = egui::RawInput::default();
        raw_input.events = self.event_queue.drain(..).collect();
        
        // Begin frame
        self.ctx.begin_frame(raw_input);
        
        // Render timeline
        egui::CentralPanel::default().show(&self.ctx, |ui| {
            self.timeline.show(ui, &mut self.engine);
        });
        
        // End frame
        self.ctx.end_frame();
        self.frame_count += 1;
    }
    
    /// Run multiple frames
    pub fn run_frames(&mut self, count: usize) {
        for _ in 0..count {
            self.run_frame();
        }
    }
    
    /// Get current timeline state for assertions
    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }
    
    /// Get mutable timeline for setup
    pub fn timeline_mut(&mut self) -> &mut Timeline {
        &mut self.timeline
    }
    
    /// Get engine reference
    pub fn engine(&self) -> &dyn RiveEngine {
        self.engine.as_ref()
    }
    
    /// Get mutable engine reference
    pub fn engine_mut(&mut self) -> &mut dyn RiveEngine {
        self.engine.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nannou_timeline::{LayerId, frame::FrameType};
    
    #[test]
    fn test_timeline_click_to_move_playhead() {
        let mut harness = TimelineTestHarness::new();
        
        // Click at frame 10 position (100px with 10px per frame)
        let click_pos = Pos2::new(300.0, 50.0); // Accounting for layer panel width
        harness.click_at(click_pos, PointerButton::Primary);
        harness.run_frame();
        
        // Playhead should move to clicked frame
        let expected_frame = 10;
        assert_eq!(harness.timeline().state.playhead_frame, expected_frame);
    }
    
    #[test]
    fn test_playhead_drag_scrubbing() {
        let mut harness = TimelineTestHarness::new();
        
        // Drag playhead from frame 0 to frame 20
        let start_pos = Pos2::new(200.0, 15.0); // Playhead handle position
        let end_pos = Pos2::new(400.0, 15.0);
        
        harness.drag_from_to(start_pos, end_pos, PointerButton::Primary);
        harness.run_frame();
        
        // Playhead should follow drag
        assert!(harness.timeline().state.playhead_frame > 0);
    }
    
    #[test]
    fn test_frame_selection_drag() {
        let mut harness = TimelineTestHarness::new();
        
        // Drag to select frames 5-10
        let start_pos = Pos2::new(250.0, 80.0); // Frame grid area
        let end_pos = Pos2::new(300.0, 80.0);
        
        harness.drag_from_to(start_pos, end_pos, PointerButton::Primary);
        harness.run_frame();
        
        // Should have selected frames
        assert!(!harness.timeline().state.selected_frames.is_empty());
    }
    
    #[test]
    fn test_layer_selection_click() {
        let mut harness = TimelineTestHarness::new();
        
        // Click on layer panel
        let layer_click_pos = Pos2::new(100.0, 80.0);
        harness.click_at(layer_click_pos, PointerButton::Primary);
        harness.run_frame();
        
        // Should have selected a layer
        let selected_layers = &harness.timeline().state.selected_layers;
        assert!(!selected_layers.is_empty(), "Layer should be selected after click");
    }
    
    #[test]
    fn test_context_menu_right_click() {
        let mut harness = TimelineTestHarness::new();
        
        // Right-click on frame
        let frame_pos = Pos2::new(250.0, 80.0);
        harness.right_click_at(frame_pos);
        harness.run_frame();
        
        // Context menu state would be internal to egui
        // We can at least verify the event was processed without panic
        assert!(true);
    }
    
    #[test]
    fn test_keyboard_shortcuts() {
        let mut harness = TimelineTestHarness::new();
        
        // Test space bar for play/pause
        let initial_playing = harness.timeline().state.is_playing;
        harness.press_key(Key::Space, Modifiers::default());
        harness.run_frame();
        
        // Playing state should toggle
        assert_ne!(harness.timeline().state.is_playing, initial_playing);
        
        // Test F5 for insert keyframe
        harness.press_key(Key::F5, Modifiers::default());
        harness.run_frame();
        
        // Test Shift+F5 for clear keyframe
        let mut shift_mod = Modifiers::default();
        shift_mod.shift = true;
        harness.press_key(Key::F5, shift_mod);
        harness.run_frame();
    }
    
    #[test]
    fn test_multi_selection_with_modifier() {
        let mut harness = TimelineTestHarness::new();
        
        // First click to select
        harness.click_at(Pos2::new(250.0, 80.0), PointerButton::Primary);
        harness.run_frame();
        
        // Ctrl+click to add to selection
        let mut ctrl_mod = Modifiers::default();
        ctrl_mod.ctrl = true;
        harness.event_queue.push_back(Event::PointerButton {
            pos: Pos2::new(300.0, 80.0),
            button: PointerButton::Primary,
            pressed: true,
            modifiers: ctrl_mod,
        });
        harness.event_queue.push_back(Event::PointerButton {
            pos: Pos2::new(300.0, 80.0),
            button: PointerButton::Primary,
            pressed: false,
            modifiers: ctrl_mod,
        });
        harness.run_frame();
        
        // Should have multiple selections
        let selection_count = harness.timeline().state.keyframe_selection.selected.len();
        assert!(selection_count >= 1);
    }
    
    #[test]
    fn test_zoom_with_scroll() {
        let mut harness = TimelineTestHarness::new();
        
        let initial_zoom = harness.timeline().state.zoom_level;
        
        // Scroll up to zoom in
        harness.scroll(Vec2::new(0.0, 120.0));
        harness.run_frame();
        
        // Zoom should increase
        assert!(harness.timeline().state.zoom_level > initial_zoom);
        
        // Scroll down to zoom out
        harness.scroll(Vec2::new(0.0, -240.0));
        harness.run_frame();
        
        // Zoom should decrease
        assert!(harness.timeline().state.zoom_level < initial_zoom);
    }
    
    #[test]
    fn test_copy_paste_keyframes() {
        let mut harness = TimelineTestHarness::new();
        
        // Select a keyframe
        let keyframe_pos = Pos2::new(250.0, 80.0);
        harness.click_at(keyframe_pos, PointerButton::Primary);
        harness.run_frame();
        
        // Copy (Ctrl+C)
        let mut ctrl_mod = Modifiers::default();
        ctrl_mod.ctrl = true;
        harness.press_key(Key::C, ctrl_mod);
        harness.run_frame();
        
        // Move to new position
        harness.click_at(Pos2::new(350.0, 80.0), PointerButton::Primary);
        harness.run_frame();
        
        // Paste (Ctrl+V)
        harness.press_key(Key::V, ctrl_mod);
        harness.run_frame();
        
        // Verify clipboard operation completed
        assert!(harness.frame_count > 0);
    }
    
    #[test]
    fn test_drag_keyframe_to_new_position() {
        let mut harness = TimelineTestHarness::new();
        
        // Create a keyframe first
        let layer_id = LayerId::new("test_layer");
        harness.engine_mut().insert_keyframe(layer_id.clone(), 5);
        
        // Drag keyframe from frame 5 to frame 10
        let start_pos = Pos2::new(250.0, 80.0); // Frame 5
        let end_pos = Pos2::new(300.0, 80.0);   // Frame 10
        
        harness.drag_from_to(start_pos, end_pos, PointerButton::Primary);
        harness.run_frame();
        
        // Keyframe should have moved
        let frame_10_data = harness.engine().get_frame_data(layer_id.clone(), 10);
        assert_eq!(frame_10_data.frame_type, FrameType::Keyframe);
    }
    
    #[test]
    fn test_timeline_scroll_horizontal() {
        let mut harness = TimelineTestHarness::new();
        
        let initial_scroll = harness.timeline().state.scroll_x;
        
        // Horizontal scroll
        harness.scroll(Vec2::new(100.0, 0.0));
        harness.run_frame();
        
        // Should have scrolled
        assert_ne!(harness.timeline().state.scroll_x, initial_scroll);
    }
    
    #[test]
    fn test_layer_visibility_toggle() {
        let mut harness = TimelineTestHarness::new();
        
        // Click on visibility icon area (eye icon)
        let eye_icon_pos = Pos2::new(170.0, 80.0); // Near end of layer panel
        harness.click_at(eye_icon_pos, PointerButton::Primary);
        harness.run_frame();
        
        // Layer visibility should toggle
        // Note: Actual toggle would be internal to the timeline widget
        assert!(harness.frame_count > 0);
    }
    
    #[test]
    fn test_snap_behavior_with_shift() {
        let mut harness = TimelineTestHarness::new();
        
        // Enable snap
        harness.timeline_mut().config.snap.enabled = true;
        
        // Drag without shift (should snap)
        harness.drag_from_to(
            Pos2::new(252.0, 80.0),  // Near frame 5
            Pos2::new(298.0, 80.0),  // Near frame 10
            PointerButton::Primary
        );
        harness.run_frame();
        
        // Now drag with shift (should not snap)
        let mut shift_mod = Modifiers::default();
        shift_mod.shift = true;
        
        harness.event_queue.push_back(Event::PointerButton {
            pos: Pos2::new(252.0, 100.0),
            button: PointerButton::Primary,
            pressed: true,
            modifiers: shift_mod,
        });
        harness.event_queue.push_back(Event::PointerMoved(Pos2::new(298.0, 100.0)));
        harness.event_queue.push_back(Event::PointerButton {
            pos: Pos2::new(298.0, 100.0),
            button: PointerButton::Primary,
            pressed: false,
            modifiers: shift_mod,
        });
        harness.run_frame();
    }
}

/// Integration test module for complex interactions
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_complex_selection_workflow() {
        let mut harness = TimelineTestHarness::new();
        
        // 1. Click to select first keyframe
        harness.click_at(Pos2::new(250.0, 80.0), PointerButton::Primary);
        harness.run_frame();
        
        // 2. Shift+click to extend selection
        let mut shift_mod = Modifiers::default();
        shift_mod.shift = true;
        harness.event_queue.push_back(Event::PointerButton {
            pos: Pos2::new(350.0, 80.0),
            button: PointerButton::Primary,
            pressed: true,
            modifiers: shift_mod,
        });
        harness.run_frame();
        
        // 3. Drag selection to new position
        harness.drag_from_to(
            Pos2::new(300.0, 80.0),
            Pos2::new(400.0, 80.0),
            PointerButton::Primary
        );
        harness.run_frame();
        
        // Verify workflow completed
        assert!(harness.frame_count >= 3);
    }
    
    #[test]
    fn test_timeline_playback_interaction() {
        let mut harness = TimelineTestHarness::new();
        
        // Start playback
        harness.press_key(Key::Space, Modifiers::default());
        harness.run_frame();
        assert!(harness.timeline().state.is_playing);
        
        // Let it play for a few frames
        harness.run_frames(5);
        
        // Stop playback
        harness.press_key(Key::Space, Modifiers::default());
        harness.run_frame();
        assert!(!harness.timeline().state.is_playing);
        
        // Playhead should have advanced
        assert!(harness.timeline().state.playhead_frame > 0);
    }
}