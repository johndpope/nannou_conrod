//! UI helper utilities for the timeline

use egui::{*, self};

/// Helper to create consistent button styles
pub fn timeline_button(ui: &mut Ui, text: &str) -> Response {
    ui.add(Button::new(text).min_size(vec2(30.0, 20.0)))
}

/// Helper to create icon buttons
pub fn icon_button(ui: &mut Ui, icon: &str, size: f32) -> Response {
    ui.add(Button::new(icon).min_size(vec2(size, size)))
}

/// Helper to draw a separator line
pub fn separator_line(ui: &mut Ui, vertical: bool) {
    let rect = ui.available_rect_before_wrap();
    let stroke = ui.style().visuals.widgets.noninteractive.bg_stroke;
    
    if vertical {
        ui.painter().line_segment(
            [rect.center_top(), rect.center_bottom()],
            stroke,
        );
    } else {
        ui.painter().line_segment(
            [rect.left_center(), rect.right_center()],
            stroke,
        );
    }
}

/// Mock Rive engine for testing
pub struct MockRiveEngine {
    layers: Vec<crate::layer::LayerInfo>,
    current_frame: u32,
    total_frames: u32,
    fps: f32,
    is_playing: bool,
}

impl MockRiveEngine {
    pub fn new() -> Self {
        Self {
            layers: crate::layer::create_mock_layers(),
            current_frame: 0,
            total_frames: 100,
            fps: 24.0,
            is_playing: false,
        }
    }
}

impl crate::RiveEngine for MockRiveEngine {
    fn get_layers(&self) -> Vec<crate::layer::LayerInfo> {
        self.layers.clone()
    }

    fn get_frame_data(&self, layer_id: crate::LayerId, frame: u32) -> crate::frame::FrameData {
        crate::frame::create_mock_frame_data(&layer_id, frame)
    }

    fn play(&mut self) {
        self.is_playing = true;
        println!("MockRiveEngine: Playing");
    }

    fn pause(&mut self) {
        self.is_playing = false;
        println!("MockRiveEngine: Paused");
    }

    fn seek(&mut self, frame: u32) {
        self.current_frame = frame.min(self.total_frames);
        println!("MockRiveEngine: Seeking to frame {}", self.current_frame);
    }

    fn get_current_frame(&self) -> u32 {
        self.current_frame
    }

    fn get_total_frames(&self) -> u32 {
        self.total_frames
    }

    fn get_fps(&self) -> f32 {
        self.fps
    }
    
    fn insert_frame(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Inserting frame at {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would modify the timeline data
    }
    
    fn remove_frame(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Removing frame at {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would modify the timeline data
    }
    
    fn insert_keyframe(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Inserting keyframe at {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would create a new keyframe
    }
    
    fn clear_keyframe(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Clearing keyframe at {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would remove the keyframe
    }
    
    fn create_motion_tween(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Creating motion tween at {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would create a motion tween between keyframes
    }
    
    fn create_shape_tween(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Creating shape tween at {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would create a shape tween between keyframes
    }
    
    // New keyframe manipulation methods
    fn move_keyframe(&mut self, layer_id: crate::LayerId, from_frame: u32, to_frame: u32) {
        println!("MockRiveEngine: Moving keyframe from frame {} to frame {} on layer {:?}", from_frame, to_frame, layer_id);
        // In a real implementation, this would move keyframe data in the timeline
    }
    
    fn copy_keyframe(&mut self, layer_id: crate::LayerId, frame: u32) -> Option<crate::frame::FrameData> {
        println!("MockRiveEngine: Copying keyframe at frame {} on layer {:?}", frame, layer_id);
        // Return mock frame data for copy operation
        Some(crate::frame::create_mock_frame_data(&layer_id, frame))
    }
    
    fn paste_keyframe(&mut self, layer_id: crate::LayerId, frame: u32, data: crate::frame::FrameData) {
        println!("MockRiveEngine: Pasting keyframe at frame {} on layer {:?} with data {:?}", frame, layer_id, data);
        // In a real implementation, this would paste the frame data
    }
    
    fn delete_keyframe(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Deleting keyframe at frame {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would remove the keyframe
    }
    
    // Property manipulation methods
    fn set_property(&mut self, layer_id: crate::LayerId, frame: u32, property: &str, value: bool) {
        println!("MockRiveEngine: Setting property '{}' to {} at frame {} on layer {:?}", property, value, frame, layer_id);
        // In a real implementation, this would set layer properties at specific frames
    }
    
    fn get_property(&self, layer_id: crate::LayerId, frame: u32, property: &str) -> bool {
        println!("MockRiveEngine: Getting property '{}' at frame {} on layer {:?}", property, frame, layer_id);
        // Return mock property value
        match property {
            "visible" => true,
            "locked" => false,
            _ => false,
        }
    }
}