//! Rhai scripting integration for timeline animations
//! 
//! This module provides a scripting layer that allows users to control
//! timeline animations and display objects using Rhai scripts.

use rhai::{Engine, Scope, AST, Dynamic, EvalAltResult};
use std::sync::{Arc, Mutex};
use crate::{RiveEngine, LayerId};

/// Timeline controller that scripts can access
#[derive(Clone)]
pub struct TimelineController {
    pub timeline_engine: Arc<Mutex<Box<dyn RiveEngine>>>,
}

impl TimelineController {
    pub fn new(engine: Arc<Mutex<Box<dyn RiveEngine>>>) -> Self {
        Self {
            timeline_engine: engine,
        }
    }
    
    pub fn play(&self) {
        if let Ok(mut engine) = self.timeline_engine.lock() {
            engine.play();
        }
    }
    
    pub fn pause(&self) {
        if let Ok(mut engine) = self.timeline_engine.lock() {
            engine.pause();
        }
    }
    
    pub fn stop(&self) {
        if let Ok(mut engine) = self.timeline_engine.lock() {
            engine.pause();
            engine.seek(0);
        }
    }
    
    pub fn goto_and_play(&self, frame: i64) {
        if let Ok(mut engine) = self.timeline_engine.lock() {
            engine.seek(frame as u32);
            engine.play();
        }
    }
    
    pub fn goto_and_stop(&self, frame: i64) {
        if let Ok(mut engine) = self.timeline_engine.lock() {
            engine.seek(frame as u32);
            engine.pause();
        }
    }
    
    pub fn get_current_frame(&self) -> i64 {
        if let Ok(engine) = self.timeline_engine.lock() {
            engine.get_current_frame() as i64
        } else {
            0
        }
    }
    
    pub fn get_total_frames(&self) -> i64 {
        if let Ok(engine) = self.timeline_engine.lock() {
            engine.get_total_frames() as i64
        } else {
            0
        }
    }
}

/// Stage object for script access
#[derive(Clone)]
pub struct ScriptStage {
    pub width: f32,
    pub height: f32,
    pub frame_rate: f32,
    display_objects: Arc<Mutex<Vec<ScriptDisplayObject>>>,
}

impl ScriptStage {
    pub fn new(width: f32, height: f32, frame_rate: f32) -> Self {
        Self {
            width,
            height,
            frame_rate,
            display_objects: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get child by name
    pub fn get_child_by_name(&self, name: String) -> Dynamic {
        if let Ok(objects) = self.display_objects.lock() {
            if let Some(obj) = objects.iter().find(|obj| obj.name == name) {
                return Dynamic::from(obj.clone());
            }
        }
        Dynamic::UNIT
    }
    
    /// Add a display object
    pub fn add_child(&self, child: ScriptDisplayObject) {
        if let Ok(mut objects) = self.display_objects.lock() {
            objects.push(child);
        }
    }
    
    /// Remove a display object
    pub fn remove_child(&self, name: String) -> bool {
        if let Ok(mut objects) = self.display_objects.lock() {
            if let Some(pos) = objects.iter().position(|obj| obj.name == name) {
                objects.remove(pos);
                return true;
            }
        }
        false
    }
}

/// Display object for script manipulation
#[derive(Clone, Debug)]
pub struct ScriptDisplayObject {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub alpha: f32,
    pub visible: bool,
}

impl ScriptDisplayObject {
    pub fn new(name: String) -> Self {
        Self {
            name,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            alpha: 1.0,
            visible: true,
        }
    }
}

/// Script execution context
pub struct ScriptContext {
    engine: Engine,
    scope: Scope<'static>,
    timeline_controller: TimelineController,
    stage: ScriptStage,
}

impl ScriptContext {
    /// Create a new script context with timeline bindings
    pub fn new(timeline_engine: Arc<Mutex<Box<dyn RiveEngine>>>) -> Self {
        let mut engine = Engine::new();
        let scope = Scope::new();
        
        // Create controllers
        let timeline_controller = TimelineController::new(timeline_engine);
        let stage = ScriptStage::new(800.0, 600.0, 24.0);
        
        // Register types
        engine.register_type::<TimelineController>()
            .register_fn("play", TimelineController::play)
            .register_fn("pause", TimelineController::pause)
            .register_fn("stop", TimelineController::stop)
            .register_fn("gotoAndPlay", TimelineController::goto_and_play)
            .register_fn("gotoAndStop", TimelineController::goto_and_stop)
            .register_get("currentFrame", |ctrl: &mut TimelineController| ctrl.get_current_frame())
            .register_get("totalFrames", |ctrl: &mut TimelineController| ctrl.get_total_frames());
        
        engine.register_type::<ScriptStage>()
            .register_get("width", |s: &mut ScriptStage| s.width as i64)
            .register_get("height", |s: &mut ScriptStage| s.height as i64)
            .register_get("frameRate", |s: &mut ScriptStage| s.frame_rate as i64)
            .register_fn("getChildByName", ScriptStage::get_child_by_name)
            .register_fn("addChild", ScriptStage::add_child)
            .register_fn("removeChild", ScriptStage::remove_child);
        
        engine.register_type::<ScriptDisplayObject>()
            .register_get_set("x", 
                |obj: &mut ScriptDisplayObject| obj.x as f64,
                |obj: &mut ScriptDisplayObject, val: f64| obj.x = val as f32
            )
            .register_get_set("y",
                |obj: &mut ScriptDisplayObject| obj.y as f64,
                |obj: &mut ScriptDisplayObject, val: f64| obj.y = val as f32
            )
            .register_get_set("rotation",
                |obj: &mut ScriptDisplayObject| obj.rotation as f64,
                |obj: &mut ScriptDisplayObject, val: f64| obj.rotation = val as f32
            )
            .register_get_set("scaleX",
                |obj: &mut ScriptDisplayObject| obj.scale_x as f64,
                |obj: &mut ScriptDisplayObject, val: f64| obj.scale_x = val as f32
            )
            .register_get_set("scaleY",
                |obj: &mut ScriptDisplayObject| obj.scale_y as f64,
                |obj: &mut ScriptDisplayObject, val: f64| obj.scale_y = val as f32
            )
            .register_get_set("alpha",
                |obj: &mut ScriptDisplayObject| obj.alpha as f64,
                |obj: &mut ScriptDisplayObject, val: f64| obj.alpha = val as f32
            )
            .register_get_set("visible",
                |obj: &mut ScriptDisplayObject| obj.visible,
                |obj: &mut ScriptDisplayObject, val: bool| obj.visible = val
            );
        
        // Register global functions
        engine.register_fn("print", |s: &str| {
            println!("Script: {}", s);
        });
        
        Self {
            engine,
            scope,
            timeline_controller,
            stage,
        }
    }
    
    /// Execute a script in the context
    pub fn execute_script(&mut self, script: &str) -> Result<Dynamic, Box<EvalAltResult>> {
        // Add objects to scope
        self.scope.clear();
        self.scope.push("timeline", self.timeline_controller.clone());
        self.scope.push("stage", self.stage.clone());
        
        // Execute the script
        self.engine.eval_with_scope::<Dynamic>(&mut self.scope, script)
    }
    
    /// Compile a script for repeated execution
    pub fn compile_script(&self, script: &str) -> Result<AST, Box<EvalAltResult>> {
        self.engine.compile(script).map_err(|e| e.into())
    }
    
    /// Execute a compiled script
    pub fn execute_ast(&mut self, ast: &AST) -> Result<Dynamic, Box<EvalAltResult>> {
        // Add objects to scope
        self.scope.clear();
        self.scope.push("timeline", self.timeline_controller.clone());
        self.scope.push("stage", self.stage.clone());
        
        // Execute the AST
        self.engine.eval_ast_with_scope::<Dynamic>(&mut self.scope, ast)
    }
    
    /// Add a display object to the stage
    pub fn add_display_object(&mut self, name: String) -> ScriptDisplayObject {
        let obj = ScriptDisplayObject::new(name);
        self.stage.add_child(obj.clone());
        obj
    }
}

/// Event types that scripts can handle
#[derive(Clone, Debug)]
pub enum ScriptEvent {
    EnterFrame,
    MouseClick { x: f32, y: f32 },
    MouseOver { x: f32, y: f32 },
    MouseOut,
    KeyDown { key: String },
    KeyUp { key: String },
}

/// Script manager that handles all scripts in the timeline
pub struct ScriptManager {
    contexts: Vec<ScriptContext>,
    frame_scripts: std::collections::HashMap<u32, Vec<String>>,
    layer_scripts: std::collections::HashMap<LayerId, Vec<String>>,
    global_scripts: Vec<String>,
}

impl ScriptManager {
    pub fn new() -> Self {
        Self {
            contexts: Vec::new(),
            frame_scripts: std::collections::HashMap::new(),
            layer_scripts: std::collections::HashMap::new(),
            global_scripts: Vec::new(),
        }
    }
    
    /// Add a script to a specific frame
    pub fn add_frame_script(&mut self, frame: u32, script: String) {
        self.frame_scripts.entry(frame).or_insert_with(Vec::new).push(script);
    }
    
    /// Add a script to a layer
    pub fn add_layer_script(&mut self, layer_id: LayerId, script: String) {
        self.layer_scripts.entry(layer_id).or_insert_with(Vec::new).push(script);
    }
    
    /// Add a global script
    pub fn add_global_script(&mut self, script: String) {
        self.global_scripts.push(script);
    }
    
    /// Execute scripts for the current frame
    pub fn execute_frame_scripts(&mut self, frame: u32) {
        // Execute frame-specific scripts
        if let Some(scripts) = self.frame_scripts.get(&frame) {
            for script in scripts {
                println!("Executing frame {} script: {}", frame, script);
            }
        }
        
        // Execute global scripts
        for script in &self.global_scripts {
            println!("Executing global script: {}", script);
        }
    }
    
    /// Handle an event
    pub fn handle_event(&mut self, event: ScriptEvent) {
        match event {
            ScriptEvent::EnterFrame => {
                println!("Handling EnterFrame event");
            }
            ScriptEvent::MouseClick { x, y } => {
                println!("Handling MouseClick at ({}, {})", x, y);
            }
            _ => {}
        }
    }
}

/// Example scripts for common tasks
pub mod templates {
    pub const LOOP_ANIMATION: &str = r#"
// Loop animation when reaching the end
if timeline.currentFrame >= timeline.totalFrames - 1 {
    timeline.gotoAndPlay(0);
}
"#;
    
    pub const STOP_AT_FRAME: &str = r#"
// Stop at a specific frame
if timeline.currentFrame >= 60 {
    timeline.stop();
}
"#;
    
    pub const ANIMATE_OBJECT: &str = r#"
// Animate an object across the stage
let obj = stage.getChildByName("myObject");
if obj != () {
    obj.x = obj.x + 5.0;
    if obj.x > stage.width {
        obj.x = -50.0;
    }
}
"#;
    
    pub const CREATE_OBJECT: &str = r#"
// Create a new display object
let circle = #{
    name: "circle1",
    x: 100.0,
    y: 100.0,
    rotation: 0.0,
    scaleX: 1.0,
    scaleY: 1.0,
    alpha: 1.0,
    visible: true
};
stage.addChild(circle);
print("Created circle at (100, 100)");
"#;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_script_execution() {
        // Create a mock engine
        let engine = Arc::new(Mutex::new(Box::new(crate::ui::MockRiveEngine::new()) as Box<dyn RiveEngine>));
        let mut context = ScriptContext::new(engine);
        
        // Test basic script
        let script = r#"
            print("Testing script execution");
            timeline.currentFrame
        "#;
        
        let result = context.execute_script(script);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_timeline_control() {
        let engine = Arc::new(Mutex::new(Box::new(crate::ui::MockRiveEngine::new()) as Box<dyn RiveEngine>));
        let mut context = ScriptContext::new(engine);
        
        let script = r#"
            timeline.gotoAndStop(10);
            timeline.currentFrame
        "#;
        
        let result = context.execute_script(script);
        assert!(result.is_ok());
    }
}