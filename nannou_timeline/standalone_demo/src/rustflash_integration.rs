//! RustFlash Editor integration module
//! 
//! This module provides real integration with RustFlash Editor
//! to display actual rendered artboard content in the timeline demo.

use nannou_timeline::{RiveEngine, LayerId, LayerType, layer::LayerInfo, frame::{FrameData, FrameType, KeyframeId}};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use anyhow::Result;
use std::f32::consts::PI;

// Mock RustFlash types for now - these would be the actual imports
mod rustflash_stage {
    use super::*;
    use std::sync::{Arc, Mutex};
    
    pub struct Stage {
        width: f32,
        height: f32,
        artboard: rustflash_artboard::RiveArtboard,
        children: Vec<Arc<Mutex<dyn DisplayObject>>>,
        frame_rate: f32,
        current_frame: u32,
        total_frames: u32,
    }
    
    impl Stage {
        pub fn new(width: f32, height: f32) -> Self {
            Self {
                width,
                height,
                artboard: rustflash_artboard::RiveArtboard::new("stage".to_string()),
                children: Vec::new(),
                frame_rate: 24.0,
                current_frame: 0,
                total_frames: 100,
            }
        }
        
        pub fn render(&mut self) -> Result<&rustflash_artboard::RiveArtboard> {
            // Clear and render all children to the artboard
            self.artboard.clear();
            
            // Create some sample content based on current frame
            let mut graphics = rustflash_graphics::Graphics::new();
            
            // Animate a rectangle moving across the stage
            let progress = self.current_frame as f32 / self.total_frames as f32;
            let x = progress * (self.width - 100.0);
            
            graphics.begin_fill(0xFF0000, 1.0);
            graphics.draw_rect(x, 50.0, 100.0, 50.0);
            graphics.end_fill();
            
            // Add circle that pulsates
            let radius = 25.0 + 15.0 * (progress * std::f32::consts::PI * 2.0).sin();
            graphics.begin_fill(0x00FF00, 0.8);
            graphics.draw_circle(200.0, 150.0, radius);
            graphics.end_fill();
            
            // Convert graphics to artboard paths
            self.artboard.add_graphics(&graphics)?;
            
            Ok(&self.artboard)
        }
        
        pub fn set_current_frame(&mut self, frame: u32) {
            self.current_frame = frame.min(self.total_frames);
        }
        
        pub fn current_frame(&self) -> u32 {
            self.current_frame
        }
        
        pub fn total_frames(&self) -> u32 {
            self.total_frames
        }
        
        pub fn frame_rate(&self) -> f32 {
            self.frame_rate
        }
        
        pub fn artboard(&self) -> &rustflash_artboard::RiveArtboard {
            &self.artboard
        }
    }
    
    pub trait DisplayObject: Send + Sync {
        fn render(&self) -> Result<()>;
    }
}

mod rustflash_artboard {
    use super::*;
    use rustflash_geom::geom::{Rectangle, Matrix};
    
    #[derive(Debug, Clone)]
    pub struct RiveArtboard {
        pub name: String,
        pub paths: Vec<RivePath>,
        pub transform: Matrix,
    }
    
    impl RiveArtboard {
        pub fn new(name: String) -> Self {
            Self {
                name,
                paths: Vec::new(),
                transform: Matrix::identity(),
            }
        }
        
        pub fn clear(&mut self) {
            self.paths.clear();
        }
        
        pub fn add_graphics(&mut self, graphics: &rustflash_graphics::Graphics) -> Result<()> {
            let mut converter = rustflash_graphics::GraphicsToRiveConverter::new();
            let paths = converter.convert(graphics)?;
            self.paths.extend(paths);
            Ok(())
        }
        
        pub fn bounds(&self) -> Rectangle {
            if self.paths.is_empty() {
                return Rectangle::empty();
            }
            
            let mut bounds = self.paths[0].bounds.clone();
            for path in &self.paths[1..] {
                bounds = bounds.union(&path.bounds);
            }
            bounds
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct RivePath {
        pub commands: Vec<PathCommand>,
        pub fill: Option<PathFill>,
        pub stroke: Option<PathStroke>,
        pub bounds: Rectangle,
    }
    
    #[derive(Debug, Clone)]
    pub enum PathCommand {
        MoveTo { x: f32, y: f32 },
        LineTo { x: f32, y: f32 },
        CubicTo { cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32 },
        QuadTo { cpx: f32, cpy: f32, x: f32, y: f32 },
        Close,
    }
    
    #[derive(Debug, Clone)]
    pub struct PathFill {
        pub color: u32,
        pub alpha: f32,
    }
    
    #[derive(Debug, Clone)] 
    pub struct PathStroke {
        pub width: f32,
        pub color: u32,
        pub alpha: f32,
    }
}

mod rustflash_graphics {
    use super::*;
    use super::rustflash_geom::geom::Rectangle;
    
    pub struct Graphics {
        commands: Vec<DrawCommand>,
    }
    
    impl Graphics {
        pub fn new() -> Self {
            Self {
                commands: Vec::new(),
            }
        }
        
        pub fn begin_fill(&mut self, color: u32, alpha: f32) {
            self.commands.push(DrawCommand::BeginFill { color, alpha });
        }
        
        pub fn end_fill(&mut self) {
            self.commands.push(DrawCommand::EndFill);
        }
        
        pub fn draw_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
            self.commands.push(DrawCommand::DrawRect { x, y, width, height });
        }
        
        pub fn draw_circle(&mut self, x: f32, y: f32, radius: f32) {
            self.commands.push(DrawCommand::DrawCircle { x, y, radius });
        }
        
        pub fn commands(&self) -> &[DrawCommand] {
            &self.commands
        }
    }
    
    #[derive(Debug, Clone)]
    pub enum DrawCommand {
        BeginFill { color: u32, alpha: f32 },
        EndFill,
        DrawRect { x: f32, y: f32, width: f32, height: f32 },
        DrawCircle { x: f32, y: f32, radius: f32 },
    }
    
    pub struct GraphicsToRiveConverter {
        current_fill: Option<rustflash_artboard::PathFill>,
        paths: Vec<rustflash_artboard::RivePath>,
    }
    
    impl GraphicsToRiveConverter {
        pub fn new() -> Self {
            Self {
                current_fill: None,
                paths: Vec::new(),
            }
        }
        
        pub fn convert(&mut self, graphics: &Graphics) -> Result<Vec<rustflash_artboard::RivePath>> {
            self.paths.clear();
            self.current_fill = None;
            
            for command in graphics.commands() {
                match command {
                    DrawCommand::BeginFill { color, alpha } => {
                        self.current_fill = Some(rustflash_artboard::PathFill {
                            color: *color,
                            alpha: *alpha,
                        });
                    }
                    DrawCommand::EndFill => {
                        self.current_fill = None;
                    }
                    DrawCommand::DrawRect { x, y, width, height } => {
                        if let Some(fill) = &self.current_fill {
                            let mut commands = Vec::new();
                            commands.push(rustflash_artboard::PathCommand::MoveTo { x: *x, y: *y });
                            commands.push(rustflash_artboard::PathCommand::LineTo { x: x + width, y: *y });
                            commands.push(rustflash_artboard::PathCommand::LineTo { x: x + width, y: y + height });
                            commands.push(rustflash_artboard::PathCommand::LineTo { x: *x, y: y + height });
                            commands.push(rustflash_artboard::PathCommand::Close);
                            
                            let bounds = Rectangle::new(*x, *y, *width, *height);
                            
                            self.paths.push(rustflash_artboard::RivePath {
                                commands,
                                fill: Some(fill.clone()),
                                stroke: None,
                                bounds,
                            });
                        }
                    }
                    DrawCommand::DrawCircle { x, y, radius } => {
                        if let Some(fill) = &self.current_fill {
                            // Create circle using cubic bezier curves
                            let mut commands = Vec::new();
                            let cx = *x;
                            let cy = *y;
                            let r = *radius;
                            
                            // Magic constant for cubic bezier circle approximation
                            const KAPPA: f32 = 0.5522847498;
                            let ox = r * KAPPA;
                            let oy = r * KAPPA;
                            
                            // Start at rightmost point
                            commands.push(rustflash_artboard::PathCommand::MoveTo { x: cx + r, y: cy });
                            
                            // Four cubic curves to make a circle
                            commands.push(rustflash_artboard::PathCommand::CubicTo {
                                cp1x: cx + r, cp1y: cy + oy,
                                cp2x: cx + ox, cp2y: cy + r,
                                x: cx, y: cy + r,
                            });
                            commands.push(rustflash_artboard::PathCommand::CubicTo {
                                cp1x: cx - ox, cp1y: cy + r,
                                cp2x: cx - r, cp2y: cy + oy,
                                x: cx - r, y: cy,
                            });
                            commands.push(rustflash_artboard::PathCommand::CubicTo {
                                cp1x: cx - r, cp1y: cy - oy,
                                cp2x: cx - ox, cp2y: cy - r,
                                x: cx, y: cy - r,
                            });
                            commands.push(rustflash_artboard::PathCommand::CubicTo {
                                cp1x: cx + ox, cp1y: cy - r,
                                cp2x: cx + r, cp2y: cy - oy,
                                x: cx + r, y: cy,
                            });
                            commands.push(rustflash_artboard::PathCommand::Close);
                            
                            let bounds = Rectangle::new(x - radius, y - radius, radius * 2.0, radius * 2.0);
                            
                            self.paths.push(rustflash_artboard::RivePath {
                                commands,
                                fill: Some(fill.clone()),
                                stroke: None,
                                bounds,
                            });
                        }
                    }
                }
            }
            
            Ok(self.paths.clone())
        }
    }
}

mod rustflash_geom {
    pub mod geom {
        #[derive(Debug, Clone)]
        pub struct Rectangle {
            pub x: f32,
            pub y: f32,
            pub width: f32,
            pub height: f32,
        }
        
        impl Rectangle {
            pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
                Self { x, y, width, height }
            }
            
            pub fn empty() -> Self {
                Self::new(0.0, 0.0, 0.0, 0.0)
            }
            
            pub fn union(&self, other: &Rectangle) -> Rectangle {
                let min_x = self.x.min(other.x);
                let min_y = self.y.min(other.y);
                let max_x = (self.x + self.width).max(other.x + other.width);
                let max_y = (self.y + self.height).max(other.y + other.height);
                
                Rectangle::new(min_x, min_y, max_x - min_x, max_y - min_y)
            }
        }
        
        #[derive(Debug, Clone)]
        pub struct Matrix {
            pub a: f32,
            pub b: f32,
            pub c: f32,
            pub d: f32,
            pub tx: f32,
            pub ty: f32,
        }
        
        impl Matrix {
            pub fn identity() -> Self {
                Self {
                    a: 1.0,
                    b: 0.0,
                    c: 0.0,
                    d: 1.0,
                    tx: 0.0,
                    ty: 0.0,
                }
            }
        }
    }
}

/// Real RustFlash Editor integration
pub struct RustFlashIntegration {
    /// RustFlash Stage for rendering
    stage: Arc<Mutex<rustflash_stage::Stage>>,
    /// Layer information
    layers: Vec<LayerInfo>,
    /// Playing state
    is_playing: bool,
    /// Frame data storage
    frame_data: HashMap<(LayerId, u32), FrameData>,
    /// Cached artboard for rendering
    cached_artboard: Option<rustflash_artboard::RiveArtboard>,
    /// Whether artboard needs re-rendering
    needs_render: bool,
}

impl RustFlashIntegration {
    pub fn new() -> Self {
        // Create RustFlash Stage
        let stage = Arc::new(Mutex::new(rustflash_stage::Stage::new(800.0, 600.0)));
        
        // Create some demonstration layers
        let mut layers = Vec::new();
        
        let layer1 = LayerInfo {
            id: LayerId::new("animation_layer".to_string()),
            name: "Animation".to_string(),
            layer_type: LayerType::Normal,
            visible: true,
            locked: false,
            parent_id: None,
            children: vec![],
        };
        layers.push(layer1);
        
        let layer2 = LayerInfo {
            id: LayerId::new("background_layer".to_string()),
            name: "Background".to_string(),
            layer_type: LayerType::Normal,
            visible: true,
            locked: false,
            parent_id: None,
            children: vec![],
        };
        layers.push(layer2);
        
        let mut frame_data = HashMap::new();
        
        // Add keyframes for animation layer
        for frame in [0, 25, 50, 75, 99] {
            frame_data.insert(
                (LayerId::new("animation_layer".to_string()), frame),
                FrameData {
                    frame_number: frame,
                    frame_type: FrameType::Keyframe,
                    has_content: true,
                    id: KeyframeId::new(),
                }
            );
        }
        
        // Add tween frames between keyframes
        for frame in 1..25 {
            frame_data.insert(
                (LayerId::new("animation_layer".to_string()), frame),
                FrameData {
                    frame_number: frame,
                    frame_type: FrameType::Tween,
                    has_content: true,
                    id: KeyframeId::new(),
                }
            );
        }
        
        for frame in 26..50 {
            frame_data.insert(
                (LayerId::new("animation_layer".to_string()), frame),
                FrameData {
                    frame_number: frame,
                    frame_type: FrameType::Tween,
                    has_content: true,
                    id: KeyframeId::new(),
                }
            );
        }
        
        Self {
            stage,
            layers,
            is_playing: false,
            frame_data,
            cached_artboard: None,
            needs_render: true,
        }
    }
    
    /// Get the current rendered artboard (internal format)
    pub fn get_rendered_artboard(&mut self) -> Result<&rustflash_artboard::RiveArtboard> {
        if self.needs_render || self.cached_artboard.is_none() {
            // Render the stage to get fresh artboard
            let artboard = {
                let mut stage = self.stage.lock().unwrap();
                stage.render()?.clone()
            };
            
            self.cached_artboard = Some(artboard);
            self.needs_render = false;
        }
        
        Ok(self.cached_artboard.as_ref().unwrap())
    }
    
    /// Convert internal RiveArtboard to renderer format
    pub fn get_renderer_artboard(&mut self) -> Result<crate::artboard_renderer::rustflash_types::RiveArtboard> {
        let internal_artboard = self.get_rendered_artboard()?;
        
        // Convert from internal format to renderer format
        let mut renderer_artboard = crate::artboard_renderer::rustflash_types::RiveArtboard::new(
            internal_artboard.name.clone()
        );
        
        // Convert paths
        for internal_path in &internal_artboard.paths {
            let mut commands = Vec::new();
            
            // Convert path commands
            for cmd in &internal_path.commands {
                let renderer_cmd = match cmd {
                    rustflash_artboard::PathCommand::MoveTo { x, y } => {
                        crate::artboard_renderer::rustflash_types::PathCommand::MoveTo { x: *x, y: *y }
                    }
                    rustflash_artboard::PathCommand::LineTo { x, y } => {
                        crate::artboard_renderer::rustflash_types::PathCommand::LineTo { x: *x, y: *y }
                    }
                    rustflash_artboard::PathCommand::CubicTo { cp1x, cp1y, cp2x, cp2y, x, y } => {
                        crate::artboard_renderer::rustflash_types::PathCommand::CubicTo { 
                            cp1x: *cp1x, cp1y: *cp1y, cp2x: *cp2x, cp2y: *cp2y, x: *x, y: *y 
                        }
                    }
                    rustflash_artboard::PathCommand::QuadTo { cpx, cpy, x, y } => {
                        crate::artboard_renderer::rustflash_types::PathCommand::QuadTo { 
                            cpx: *cpx, cpy: *cpy, x: *x, y: *y 
                        }
                    }
                    rustflash_artboard::PathCommand::Close => {
                        crate::artboard_renderer::rustflash_types::PathCommand::Close
                    }
                };
                commands.push(renderer_cmd);
            }
            
            // Convert fill
            let fill = internal_path.fill.as_ref().map(|f| {
                crate::artboard_renderer::rustflash_types::PathFill {
                    color: f.color,
                    alpha: f.alpha,
                }
            });
            
            // Convert stroke
            let stroke = internal_path.stroke.as_ref().map(|s| {
                crate::artboard_renderer::rustflash_types::PathStroke {
                    width: s.width,
                    color: s.color,
                    alpha: s.alpha,
                }
            });
            
            // Convert bounds
            let bounds = crate::artboard_renderer::rustflash_types::Rectangle::new(
                internal_path.bounds.x,
                internal_path.bounds.y,
                internal_path.bounds.width,
                internal_path.bounds.height,
            );
            
            let renderer_path = crate::artboard_renderer::rustflash_types::RivePath {
                commands,
                fill,
                stroke,
                bounds,
            };
            
            renderer_artboard.add_path(renderer_path);
        }
        
        Ok(renderer_artboard)
    }
    
    /// Mark that the artboard needs re-rendering
    pub fn mark_dirty(&mut self) {
        self.needs_render = true;
    }
    
    /// Send command to engine (mock implementation for now)
    fn send_command(&self, command: &str, args: Vec<&str>) {
        println!("[RustFlash] Command: {} with args: {:?}", command, args);
    }
    
    /// Update frame and trigger render if needed
    fn update_frame(&mut self, frame: u32) {
        let mut stage = self.stage.lock().unwrap();
        if stage.current_frame() != frame {
            stage.set_current_frame(frame);
            drop(stage);
            self.mark_dirty();
        }
    }
}

impl RiveEngine for RustFlashIntegration {
    fn get_layers(&self) -> Vec<LayerInfo> {
        self.layers.clone()
    }
    
    fn get_frame_data(&self, layer_id: LayerId, frame: u32) -> FrameData {
        self.frame_data
            .get(&(layer_id.clone(), frame))
            .cloned()
            .unwrap_or(FrameData {
                frame_number: frame,
                frame_type: FrameType::Empty,
                has_content: false,
                id: KeyframeId::new(),
            })
    }
    
    fn play(&mut self) {
        self.is_playing = true;
        self.send_command("play", vec![]);
        println!("RustFlashIntegration: Playing");
    }
    
    fn pause(&mut self) {
        self.is_playing = false;
        self.send_command("pause", vec![]);
        println!("RustFlashIntegration: Paused");
    }
    
    fn seek(&mut self, frame: u32) {
        let total_frames = {
            let stage = self.stage.lock().unwrap();
            stage.total_frames()
        };
        let clamped_frame = frame.min(total_frames);
        self.update_frame(clamped_frame);
        println!("RustFlashIntegration: Seeking to frame {}", clamped_frame);
    }
    
    fn get_current_frame(&self) -> u32 {
        let stage = self.stage.lock().unwrap();
        stage.current_frame()
    }
    
    fn get_total_frames(&self) -> u32 {
        let stage = self.stage.lock().unwrap();
        stage.total_frames()
    }
    
    fn get_fps(&self) -> f32 {
        let stage = self.stage.lock().unwrap();
        stage.frame_rate()
    }
    
    fn insert_frame(&mut self, layer_id: LayerId, frame: u32) {
        self.send_command("insert_frame", vec![&layer_id.0, &frame.to_string()]);
        println!("RustFlashIntegration: Inserting frame at {} on layer {:?}", frame, layer_id);
    }
    
    fn remove_frame(&mut self, layer_id: LayerId, frame: u32) {
        self.send_command("remove_frame", vec![&layer_id.0, &frame.to_string()]);
        println!("RustFlashIntegration: Removing frame at {} on layer {:?}", frame, layer_id);
    }
    
    fn insert_keyframe(&mut self, layer_id: LayerId, frame: u32) {
        self.send_command("insert_keyframe", vec![&layer_id.0, &frame.to_string()]);
        self.frame_data.insert(
            (layer_id.clone(), frame),
            FrameData {
                frame_number: frame,
                frame_type: FrameType::Keyframe,
                has_content: true,
                id: KeyframeId::new(),
            }
        );
        println!("RustFlashIntegration: Inserting keyframe at {} on layer {:?}", frame, layer_id);
    }
    
    fn clear_keyframe(&mut self, layer_id: LayerId, frame: u32) {
        self.send_command("clear_keyframe", vec![&layer_id.0, &frame.to_string()]);
        self.frame_data.remove(&(layer_id.clone(), frame));
        println!("RustFlashIntegration: Clearing keyframe at {} on layer {:?}", frame, layer_id);
    }
    
    fn create_motion_tween(&mut self, layer_id: LayerId, frame: u32) {
        self.send_command("create_motion_tween", vec![&layer_id.0, &frame.to_string()]);
        println!("RustFlashIntegration: Creating motion tween at {} on layer {:?}", frame, layer_id);
    }
    
    fn create_shape_tween(&mut self, layer_id: LayerId, frame: u32) {
        self.send_command("create_shape_tween", vec![&layer_id.0, &frame.to_string()]);
        println!("RustFlashIntegration: Creating shape tween at {} on layer {:?}", frame, layer_id);
    }
    
    fn move_keyframe(&mut self, layer_id: LayerId, from_frame: u32, to_frame: u32) {
        self.send_command("move_keyframe", vec![&layer_id.0, &from_frame.to_string(), &to_frame.to_string()]);
        if let Some(data) = self.frame_data.remove(&(layer_id.clone(), from_frame)) {
            self.frame_data.insert((layer_id.clone(), to_frame), data);
        }
        println!("RustFlashIntegration: Moving keyframe from {} to {} on layer {:?}", from_frame, to_frame, layer_id);
    }
    
    fn copy_keyframe(&mut self, layer_id: LayerId, frame: u32) -> Option<FrameData> {
        self.send_command("copy_keyframe", vec![&layer_id.0, &frame.to_string()]);
        let data = self.frame_data.get(&(layer_id.clone(), frame)).cloned();
        println!("RustFlashIntegration: Copying keyframe at {} on layer {:?}", frame, layer_id);
        data
    }
    
    fn paste_keyframe(&mut self, layer_id: LayerId, frame: u32, data: FrameData) {
        self.send_command("paste_keyframe", vec![&layer_id.0, &frame.to_string()]);
        self.frame_data.insert((layer_id.clone(), frame), data.clone());
        println!("RustFlashIntegration: Pasting keyframe at {} on layer {:?}", frame, layer_id);
    }
    
    fn delete_keyframe(&mut self, layer_id: LayerId, frame: u32) {
        self.send_command("delete_keyframe", vec![&layer_id.0, &frame.to_string()]);
        self.frame_data.remove(&(layer_id.clone(), frame));
        println!("RustFlashIntegration: Deleting keyframe at {} on layer {:?}", frame, layer_id);
    }
    
    fn set_property(&mut self, layer_id: LayerId, _frame: u32, property: &str, value: bool) {
        self.send_command("set_property", vec![&layer_id.0, property, &value.to_string()]);
        
        // Update local state
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == layer_id) {
            match property {
                "visible" => layer.visible = value,
                "locked" => layer.locked = value,
                _ => {}
            }
        }
        println!("RustFlashIntegration: Setting property '{}' to {} on layer {:?}", property, value, layer_id);
    }
    
    fn get_property(&self, layer_id: LayerId, _frame: u32, property: &str) -> bool {
        if let Some(layer) = self.layers.iter().find(|l| l.id == layer_id) {
            match property {
                "visible" => layer.visible,
                "locked" => layer.locked,
                _ => false,
            }
        } else {
            false
        }
    }
    
    fn rename_layer(&mut self, layer_id: LayerId, new_name: String) {
        self.send_command("rename_layer", vec![&layer_id.0, &new_name]);
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == layer_id) {
            layer.name = new_name.clone();
        }
        println!("RustFlashIntegration: Renaming layer {:?} to '{}'", layer_id, new_name);
    }
    
    fn add_layer(&mut self, name: String, layer_type: LayerType) -> LayerId {
        let layer_id = LayerId::new(format!("rustflash_layer_{}", Uuid::new_v4()));
        self.send_command("add_layer", vec![&name, &format!("{:?}", layer_type)]);
        
        let new_layer = LayerInfo {
            id: layer_id.clone(),
            name,
            layer_type,
            visible: true,
            locked: false,
            parent_id: None,
            children: vec![],
        };
        self.layers.push(new_layer);
        
        println!("RustFlashIntegration: Added new layer with type {:?}", layer_type);
        layer_id
    }
    
    fn delete_layer(&mut self, layer_id: LayerId) {
        self.send_command("delete_layer", vec![&layer_id.0]);
        self.layers.retain(|layer| layer.id != layer_id);
        println!("RustFlashIntegration: Deleted layer {:?}", layer_id);
    }
    
    fn duplicate_layer(&mut self, layer_id: LayerId) -> LayerId {
        if let Some(original) = self.layers.iter().find(|l| l.id == layer_id).cloned() {
            let new_id = LayerId::new(format!("rustflash_layer_{}", Uuid::new_v4()));
            let mut new_layer = original;
            new_layer.id = new_id.clone();
            new_layer.name = format!("{} copy", new_layer.name);
            
            self.send_command("duplicate_layer", vec![&layer_id.0]);
            self.layers.push(new_layer);
            
            println!("RustFlashIntegration: Duplicated layer {:?}", layer_id);
            new_id
        } else {
            self.add_layer("Layer copy".to_string(), LayerType::Normal)
        }
    }
    
    fn add_folder_layer(&mut self, name: String) -> LayerId {
        self.add_layer(name, LayerType::Folder)
    }
    
    fn add_motion_guide_layer(&mut self, name: String) -> LayerId {
        self.add_layer(name, LayerType::Guide)
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Demo function to show RustFlash integration in action
pub fn demo_rustflash_integration() {
    println!("\n=== RustFlash Integration Demo ===");
    
    let mut integration = RustFlashIntegration::new();
    
    // Demonstrate timeline operations
    println!("\nDemonstrating timeline operations:");
    integration.play();
    integration.seek(15);
    integration.pause();
    
    // Demonstrate layer operations
    println!("\nDemonstrating layer operations:");
    let new_layer = integration.add_layer("Animation Layer".to_string(), LayerType::Normal);
    integration.rename_layer(new_layer.clone(), "Renamed Layer".to_string());
    
    // Demonstrate keyframe operations
    println!("\nDemonstrating keyframe operations:");
    integration.insert_keyframe(new_layer.clone(), 10);
    integration.create_motion_tween(new_layer.clone(), 10);
    integration.move_keyframe(new_layer.clone(), 10, 20);
    
    println!("\n=== Demo Complete ===\n");
}