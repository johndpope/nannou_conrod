//! Standalone demo of the Flash-inspired timeline widget

use eframe::egui::{self, UiBuilder, ComboBox};
use nannou_timeline::{
    timeline_egui::Timeline,
    ui::MockRiveEngine, RiveEngine, LayerId,
    layer::LayerType,
    scripting::{ScriptContext, templates},
    CurveEditorPanel,
};
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};

mod rustflash_integration;
use rustflash_integration::RustFlashIntegration;

// Global crash state
static CRASH_OCCURRED: AtomicBool = AtomicBool::new(false);
static CRASH_INFO: Mutex<Option<CrashInfo>> = Mutex::new(None);

#[derive(Clone)]
struct CrashInfo {
    message: String,
    location: String,
    timestamp: String,
    backtrace: String,
}

// Wrapper to capture engine interactions
struct LoggingRiveEngine {
    inner: MockRiveEngine,
    log_sender: Arc<Mutex<Vec<(LogLevel, String)>>>,
}

impl LoggingRiveEngine {
    fn new(log_sender: Arc<Mutex<Vec<(LogLevel, String)>>>) -> Self {
        Self {
            inner: MockRiveEngine::new(),
            log_sender,
        }
    }
    
    fn log(&self, level: LogLevel, msg: String) {
        if let Ok(mut logs) = self.log_sender.lock() {
            logs.push((level, msg));
        }
    }
}

impl RiveEngine for LoggingRiveEngine {
    fn get_layers(&self) -> Vec<nannou_timeline::layer::LayerInfo> {
        self.inner.get_layers()
    }
    
    fn get_frame_data(&self, layer_id: LayerId, frame: u32) -> nannou_timeline::frame::FrameData {
        self.inner.get_frame_data(layer_id, frame)
    }
    
    fn play(&mut self) {
        self.log(LogLevel::Action, "Engine: Play started".to_string());
        self.inner.play()
    }
    
    fn pause(&mut self) {
        self.log(LogLevel::Action, "Engine: Paused".to_string());
        self.inner.pause()
    }
    
    fn seek(&mut self, frame: u32) {
        self.log(LogLevel::Action, format!("Engine: Seek to frame {}", frame));
        self.inner.seek(frame)
    }
    
    fn get_current_frame(&self) -> u32 {
        self.inner.get_current_frame()
    }
    
    fn get_total_frames(&self) -> u32 {
        self.inner.get_total_frames()
    }
    
    fn get_fps(&self) -> f32 {
        self.inner.get_fps()
    }
    
    fn insert_frame(&mut self, layer_id: LayerId, frame: u32) {
        self.log(LogLevel::Action, format!("Insert frame at {} on layer {:?}", frame, layer_id));
        self.inner.insert_frame(layer_id, frame)
    }
    
    fn remove_frame(&mut self, layer_id: LayerId, frame: u32) {
        self.log(LogLevel::Action, format!("Remove frame at {} on layer {:?}", frame, layer_id));
        self.inner.remove_frame(layer_id, frame)
    }
    
    fn insert_keyframe(&mut self, layer_id: LayerId, frame: u32) {
        self.log(LogLevel::Action, format!("Insert keyframe at {} on layer {:?}", frame, layer_id));
        self.inner.insert_keyframe(layer_id, frame)
    }
    
    fn clear_keyframe(&mut self, layer_id: LayerId, frame: u32) {
        self.log(LogLevel::Action, format!("Clear keyframe at {} on layer {:?}", frame, layer_id));
        self.inner.clear_keyframe(layer_id, frame)
    }
    
    fn create_motion_tween(&mut self, layer_id: LayerId, frame: u32) {
        self.log(LogLevel::Action, format!("Create motion tween at {} on layer {:?}", frame, layer_id));
        self.inner.create_motion_tween(layer_id, frame)
    }
    
    fn create_shape_tween(&mut self, layer_id: LayerId, frame: u32) {
        self.log(LogLevel::Action, format!("Create shape tween at {} on layer {:?}", frame, layer_id));
        self.inner.create_shape_tween(layer_id, frame)
    }
    
    fn move_keyframe(&mut self, layer_id: LayerId, from_frame: u32, to_frame: u32) {
        self.log(LogLevel::Action, format!("Move keyframe from {} to {} on layer {:?}", from_frame, to_frame, layer_id));
        self.inner.move_keyframe(layer_id, from_frame, to_frame)
    }
    
    fn copy_keyframe(&mut self, layer_id: LayerId, frame: u32) -> Option<nannou_timeline::frame::FrameData> {
        self.log(LogLevel::Action, format!("Copy keyframe at {} on layer {:?}", frame, layer_id));
        self.inner.copy_keyframe(layer_id, frame)
    }
    
    fn paste_keyframe(&mut self, layer_id: LayerId, frame: u32, data: nannou_timeline::frame::FrameData) {
        self.log(LogLevel::Action, format!("Paste keyframe at {} on layer {:?}", frame, layer_id));
        self.inner.paste_keyframe(layer_id, frame, data)
    }
    
    fn delete_keyframe(&mut self, layer_id: LayerId, frame: u32) {
        self.log(LogLevel::Action, format!("Delete keyframe at {} on layer {:?}", frame, layer_id));
        self.inner.delete_keyframe(layer_id, frame)
    }
    
    fn set_property(&mut self, layer_id: LayerId, frame: u32, property: &str, value: bool) {
        self.log(LogLevel::Action, format!("Set property '{}' to {} at {} on layer {:?}", property, value, frame, layer_id));
        self.inner.set_property(layer_id, frame, property, value)
    }
    
    fn get_property(&self, layer_id: LayerId, frame: u32, property: &str) -> bool {
        self.inner.get_property(layer_id, frame, property)
    }
    
    fn rename_layer(&mut self, layer_id: LayerId, new_name: String) {
        self.log(LogLevel::Action, format!("Renamed layer {:?} to '{}'", layer_id, new_name));
        self.inner.rename_layer(layer_id, new_name)
    }
    
    fn add_layer(&mut self, name: String, layer_type: LayerType) -> LayerId {
        let layer_id = self.inner.add_layer(name.clone(), layer_type);
        self.log(LogLevel::Action, format!("Added new {:?} layer '{}' with id {:?}", layer_type, name, layer_id));
        layer_id
    }
    
    fn delete_layer(&mut self, layer_id: LayerId) {
        self.log(LogLevel::Action, format!("Deleted layer {:?}", layer_id));
        self.inner.delete_layer(layer_id)
    }
    
    fn duplicate_layer(&mut self, layer_id: LayerId) -> LayerId {
        let new_layer_id = self.inner.duplicate_layer(layer_id.clone());
        self.log(LogLevel::Action, format!("Duplicated layer {:?} to {:?}", layer_id, new_layer_id));
        new_layer_id
    }
    
    fn add_folder_layer(&mut self, name: String) -> LayerId {
        let layer_id = self.inner.add_folder_layer(name.clone());
        self.log(LogLevel::Action, format!("Added new folder layer '{}' with id {:?}", name, layer_id));
        layer_id
    }
    
    fn add_motion_guide_layer(&mut self, name: String) -> LayerId {
        let layer_id = self.inner.add_motion_guide_layer(name.clone());
        self.log(LogLevel::Action, format!("Added new motion guide layer '{}' with id {:?}", name, layer_id));
        layer_id
    }
}

struct TimelineApp {
    timeline: Timeline,
    engine: Box<dyn RiveEngine>,
    // State for resizable panels
    timeline_height: f32,
    library_width: f32,
    console_height: f32,
    splitter_thickness: f32,
    // Debug console state
    console_visible: bool,
    log_messages: Vec<LogMessage>,
    auto_scroll: bool,
    // Engine log receiver
    engine_logs: Arc<Mutex<Vec<(LogLevel, String)>>>,
    // Language selection
    selected_language: String,
    // Stage items
    stage_items: Vec<StageItem>,
    selected_items: Vec<usize>,
    // Context menu state
    context_menu: Option<ContextMenuState>,
    // Properties panel state
    properties_height: f32,
    selected_property_tab: PropertyTab,
    // Clipboard for copy/paste operations
    clipboard: Vec<StageItem>,
    // Library panel state
    library_tab: LibraryTab,
    library_assets: Vec<LibraryAsset>,
    library_folders_expanded: Vec<String>,
    selected_library_asset: Option<String>,
    library_search: String,
    // Drag and drop state
    dragging_asset: Option<LibraryAsset>,
    drag_offset: egui::Vec2,
    // Library context menu
    library_context_menu: Option<LibraryContextMenuState>,
    // Tools panel state
    tool_state: ToolState,
    tools_panel_width: f32,
    // Script editor state
    script_visible: bool,
    script_content: String,
    script_context: Option<ScriptContext>,
    script_error: Option<String>,
    script_panel_height: f32,
    // Curve editor state
    curve_editor: CurveEditorPanel,
}

#[derive(Clone)]
struct StageItem {
    id: String,
    name: String,
    item_type: StageItemType,
    position: egui::Pos2,
    size: egui::Vec2,
    color: egui::Color32,
    alpha: f32,  // 0.0 to 1.0
    rotation: f32,
    // Text-specific properties
    text_content: String,
    font_size: f32,
    font_family: String,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum StageItemType {
    Rectangle,
    Circle,
    Text,
    MovieClip,
}

#[derive(Clone)]
struct ContextMenuState {
    position: egui::Pos2,
    menu_type: ContextMenuType,
}

#[derive(Clone)]
enum ContextMenuType {
    Stage(egui::Pos2),
    StageItem(usize),
}

#[derive(Clone, Copy, PartialEq)]
enum PropertyTab {
    Properties,
    Filters,
    ColorEffect,
    Display,
}

#[derive(Clone)]
struct LogMessage {
    timestamp: String,
    level: LogLevel,
    message: String,
}

#[derive(Clone, Copy, PartialEq)]
enum LogLevel {
    Info,
    Action,
    Warning,
    Error,
}

#[derive(Clone, Copy, PartialEq)]
enum LibraryTab {
    Assets,
    Components,
    ActionScript,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Tool {
    // Selection Tools
    Arrow,        // V - Primary selection
    Subselection, // A - Direct selection
    Lasso,        // L - Free-form selection
    
    // Drawing Tools
    Line,         // N - Straight lines
    Pen,          // P - Bezier curves
    Pencil,       // Y - Freehand
    Brush,        // B - Variable width
    Rectangle,    // R - Rectangles
    Oval,         // O - Circles/ellipses
    PolyStar,     // Polygons/stars
    
    // Text and Paint Tools
    Text,         // T - Text objects
    PaintBucket,  // K - Fill areas
    InkBottle,    // S - Apply stroke
    Eyedropper,   // I - Sample colors
    Eraser,       // E - Erase parts
    
    // Transform Tools
    FreeTransform,    // Q - Scale/rotate/skew
    GradientTransform,// F - Adjust gradients
    Zoom,             // Z - Zoom view
    Hand,             // H - Pan view
}

impl Tool {
    fn get_icon(&self) -> &'static str {
        match self {
            Tool::Arrow => "➤",
            Tool::Subselection => "⊡",
            Tool::Lasso => "◉",
            Tool::Line => "╱",
            Tool::Pen => "✒",
            Tool::Pencil => "✏",
            Tool::Brush => "🖌",
            Tool::Rectangle => "▭",
            Tool::Oval => "⭕",
            Tool::PolyStar => "⭐",
            Tool::Text => "T",
            Tool::PaintBucket => "🪣",
            Tool::InkBottle => "🖋",
            Tool::Eyedropper => "💧",
            Tool::Eraser => "🧽",
            Tool::FreeTransform => "⤡",
            Tool::GradientTransform => "🌈",
            Tool::Zoom => "🔍",
            Tool::Hand => "✋",
        }
    }
    
    fn get_name(&self) -> &'static str {
        match self {
            Tool::Arrow => "Selection Tool",
            Tool::Subselection => "Subselection Tool",
            Tool::Lasso => "Lasso Tool",
            Tool::Line => "Line Tool",
            Tool::Pen => "Pen Tool",
            Tool::Pencil => "Pencil Tool",
            Tool::Brush => "Brush Tool",
            Tool::Rectangle => "Rectangle Tool",
            Tool::Oval => "Oval Tool",
            Tool::PolyStar => "PolyStar Tool",
            Tool::Text => "Text Tool",
            Tool::PaintBucket => "Paint Bucket Tool",
            Tool::InkBottle => "Ink Bottle Tool",
            Tool::Eyedropper => "Eyedropper Tool",
            Tool::Eraser => "Eraser Tool",
            Tool::FreeTransform => "Free Transform Tool",
            Tool::GradientTransform => "Gradient Transform Tool",
            Tool::Zoom => "Zoom Tool",
            Tool::Hand => "Hand Tool",
        }
    }
    
    fn get_shortcut(&self) -> Option<char> {
        match self {
            Tool::Arrow => Some('V'),
            Tool::Subselection => Some('A'),
            Tool::Lasso => Some('L'),
            Tool::Line => Some('N'),
            Tool::Pen => Some('P'),
            Tool::Pencil => Some('Y'),
            Tool::Brush => Some('B'),
            Tool::Rectangle => Some('R'),
            Tool::Oval => Some('O'),
            Tool::Text => Some('T'),
            Tool::PaintBucket => Some('K'),
            Tool::InkBottle => Some('S'),
            Tool::Eyedropper => Some('I'),
            Tool::Eraser => Some('E'),
            Tool::FreeTransform => Some('Q'),
            Tool::GradientTransform => Some('F'),
            Tool::Zoom => Some('Z'),
            Tool::Hand => Some('H'),
            _ => None,
        }
    }
}

#[derive(Clone)]
struct ToolState {
    active_tool: Tool,
    stroke_color: egui::Color32,
    fill_color: egui::Color32,
    stroke_width: f32,
    // Tool-specific options
    rectangle_corner_radius: f32,
    star_points: u32,
    star_inner_radius: f32,
    brush_size: f32,
    text_font_size: f32,
    text_font_family: String,
}

#[derive(Clone)]
struct LibraryAsset {
    id: String,
    name: String,
    asset_type: LibraryAssetType,
    folder: String,
    properties: AssetProperties,
}

#[derive(Clone, Copy, PartialEq)]
enum LibraryAssetType {
    MovieClip,
    Button,
    Graphic,
    Bitmap,
    Sound,
    Video,
    Font,
    Folder,
}

#[derive(Clone)]
struct AssetProperties {
    file_size: Option<u64>,
    dimensions: Option<(u32, u32)>,
    format: Option<String>,
    usage_count: u32,
    linkage_class: Option<String>,
}

#[derive(Clone)]
struct LibraryContextMenuState {
    position: egui::Pos2,
    target: LibraryContextTarget,
}

#[derive(Clone)]
enum LibraryContextTarget {
    Asset(String),
    Folder(String),
    Background,
}

impl Default for TimelineApp {
    fn default() -> Self {
        let engine_logs = Arc::new(Mutex::new(Vec::new()));
        
        // Create some initial stage items
        let stage_items = vec![
            StageItem {
                id: "rect1".to_string(),
                name: "Blue Rectangle".to_string(),
                item_type: StageItemType::Rectangle,
                position: egui::Pos2::new(100.0, 100.0),
                size: egui::Vec2::new(120.0, 80.0),
                color: egui::Color32::from_rgb(100, 150, 255),
                alpha: 1.0,
                rotation: 0.0,
                text_content: "Default Text".to_string(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
            },
            StageItem {
                id: "circle1".to_string(),
                name: "Red Circle".to_string(),
                item_type: StageItemType::Circle,
                position: egui::Pos2::new(300.0, 150.0),
                size: egui::Vec2::new(100.0, 100.0),
                color: egui::Color32::from_rgb(255, 100, 100),
                alpha: 1.0,
                rotation: 0.0,
                text_content: "Default Text".to_string(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
            },
            StageItem {
                id: "text1".to_string(),
                name: "Hello Text".to_string(),
                item_type: StageItemType::Text,
                position: egui::Pos2::new(200.0, 250.0),
                size: egui::Vec2::new(150.0, 40.0),
                color: egui::Color32::WHITE,
                alpha: 1.0,
                rotation: 0.0,
                text_content: "Default Text".to_string(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
            },
            StageItem {
                id: "mc1".to_string(),
                name: "MovieClip Instance".to_string(),
                item_type: StageItemType::MovieClip,
                position: egui::Pos2::new(400.0, 300.0),
                size: egui::Vec2::new(80.0, 80.0),
                color: egui::Color32::from_rgb(150, 255, 150),
                alpha: 1.0,
                rotation: 45.0,
                text_content: "Default Text".to_string(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
            },
        ];
        
        let mut timeline = Timeline::new();
        
        // Add test frame comments
        timeline.config.frame_comments.push(nannou_timeline::FrameComment::new(5, "Opening scene starts here"));
        timeline.config.frame_comments.push(nannou_timeline::FrameComment::new(15, "Character enters"));
        timeline.config.frame_comments.push(nannou_timeline::FrameComment::new(30, "Important: Check timing"));
        timeline.config.frame_comments.push(nannou_timeline::FrameComment::new(45, "Background change"));
        
        let mut app = Self {
            timeline,
            // Use RustFlash integration if available, otherwise fall back to mock
            engine: if std::env::var("USE_RUSTFLASH").is_ok() {
                Box::new(RustFlashIntegration::new())
            } else {
                Box::new(LoggingRiveEngine::new(engine_logs.clone()))
            },
            timeline_height: 200.0,
            library_width: 300.0,
            console_height: 150.0,
            splitter_thickness: 4.0,
            console_visible: true,
            log_messages: Vec::new(),
            auto_scroll: true,
            engine_logs,
            selected_language: "en".to_string(),
            stage_items,
            selected_items: Vec::new(),
            context_menu: None,
            properties_height: 200.0,
            selected_property_tab: PropertyTab::Properties,
            clipboard: Vec::new(),
            // Initialize library panel
            library_tab: LibraryTab::Assets,
            library_assets: Self::create_default_library_assets(),
            library_folders_expanded: vec!["Graphics".to_string(), "Sounds".to_string()],
            selected_library_asset: None,
            library_search: String::new(),
            dragging_asset: None,
            drag_offset: egui::Vec2::ZERO,
            library_context_menu: None,
            // Initialize tools panel
            tool_state: ToolState {
                active_tool: Tool::Arrow,
                stroke_color: egui::Color32::BLACK,
                fill_color: egui::Color32::WHITE,
                stroke_width: 1.0,
                rectangle_corner_radius: 0.0,
                star_points: 5,
                star_inner_radius: 0.5,
                brush_size: 10.0,
                text_font_size: 16.0,
                text_font_family: "Arial".to_string(),
            },
            tools_panel_width: 60.0,
            // Script editor
            script_visible: false,
            script_content: templates::LOOP_ANIMATION.to_string(),
            script_context: None,
            script_error: None,
            script_panel_height: 200.0,
            // Curve editor
            curve_editor: CurveEditorPanel::default(),
        };
        app.log(LogLevel::Info, "Timeline application started");
        app.log(LogLevel::Info, "🎮 Keyboard shortcuts:");
        app.log(LogLevel::Info, "  • F12: Toggle debug console");
        app.log(LogLevel::Info, "  • F2: Take screenshot");
        app.log(LogLevel::Info, "  • F9: Toggle script editor");
        app.log(LogLevel::Info, "  • F10: Toggle curve editor");
        app.log(LogLevel::Info, "💡 Hover over timeline elements to see tooltips");
        app.log(LogLevel::Info, "💡 Right-click on layers and frames for context menus");
        app.log(LogLevel::Info, "💡 Click and drag stage items to move them");
        app.log(LogLevel::Info, "💡 Right-click stage items for context menu");
        app
    }
}

impl eframe::App for TimelineApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Collect logs from engine
        let engine_logs: Vec<(LogLevel, String)> = {
            if let Ok(mut logs) = self.engine_logs.lock() {
                logs.drain(..).collect()
            } else {
                Vec::new()
            }
        };
        
        for (level, msg) in engine_logs {
            self.log(level, msg);
        }
        
        // Handle F12 to toggle console
        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            self.console_visible = !self.console_visible;
            self.log(LogLevel::Info, format!("Console {}", if self.console_visible { "shown" } else { "hidden" }));
        }
        
        // Handle F2 for screenshot
        if ctx.input(|i| i.key_pressed(egui::Key::F2)) {
            self.take_screenshot(ctx);
        }
        
        // Handle F9 to toggle script editor
        if ctx.input(|i| i.key_pressed(egui::Key::F9)) {
            self.script_visible = !self.script_visible;
            self.log(LogLevel::Info, format!("Script editor {}", if self.script_visible { "shown" } else { "hidden" }));
            
            // Initialize script context if needed
            if self.script_context.is_none() {
                let _engine_arc = Arc::new(Mutex::new(self.engine.as_ref() as &dyn RiveEngine));
                // Note: This won't work with the current architecture, we'd need a refactor
                // For now, we'll just show the editor without execution capability
            }
        }
        
        // Handle F10 to toggle curve editor
        if ctx.input(|i| i.key_pressed(egui::Key::F10)) {
            self.curve_editor.open = !self.curve_editor.open;
            self.log(LogLevel::Info, format!("Curve editor {}", if self.curve_editor.open { "opened" } else { "closed" }));
        }
        
        // Handle tool keyboard shortcuts
        self.handle_tool_shortcuts(ctx);
        
        // Show crash dialog if a panic occurred
        self.show_crash_dialog(ctx);
        
        // Show curve editor panel if open
        self.curve_editor.show(ctx);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_rect = ui.available_rect_before_wrap();
            
            // Adjust for console and script editor if visible
            let console_space = if self.console_visible { self.console_height } else { 0.0 };
            let script_space = if self.script_visible { self.script_panel_height } else { 0.0 };
            let bottom_panels_height = console_space + script_space;
            
            // Calculate regions with resizable sizes
            let tools_rect = egui::Rect::from_min_size(
                available_rect.min,
                egui::vec2(self.tools_panel_width, available_rect.height() - self.timeline_height - bottom_panels_height),
            );
            
            let library_rect = egui::Rect::from_min_size(
                egui::pos2(available_rect.max.x - self.library_width, available_rect.min.y),
                egui::vec2(self.library_width, available_rect.height() - self.timeline_height - bottom_panels_height),
            );
            
            let timeline_rect = egui::Rect::from_min_size(
                egui::pos2(available_rect.min.x, available_rect.max.y - self.timeline_height - bottom_panels_height),
                egui::vec2(available_rect.width(), self.timeline_height),
            );
            
            let properties_rect = egui::Rect::from_min_size(
                egui::pos2(available_rect.min.x + self.tools_panel_width, available_rect.max.y - self.timeline_height - bottom_panels_height - self.properties_height),
                egui::vec2(available_rect.width() - self.library_width - self.tools_panel_width, self.properties_height),
            );
            
            let stage_rect = egui::Rect::from_min_size(
                egui::pos2(available_rect.min.x + self.tools_panel_width, available_rect.min.y),
                egui::vec2(available_rect.width() - self.library_width - self.tools_panel_width, available_rect.height() - self.timeline_height - bottom_panels_height - self.properties_height),
            );
            
            let mut bottom_y = available_rect.max.y;
            
            let console_rect = if self.console_visible {
                bottom_y -= console_space;
                Some(egui::Rect::from_min_size(
                    egui::pos2(available_rect.min.x, bottom_y),
                    egui::vec2(available_rect.width(), console_space),
                ))
            } else {
                None
            };
            
            let script_rect = if self.script_visible {
                bottom_y -= script_space;
                Some(egui::Rect::from_min_size(
                    egui::pos2(available_rect.min.x, bottom_y),
                    egui::vec2(available_rect.width(), script_space),
                ))
            } else {
                None
            };
            
            // Draw tools panel (left side)
            self.draw_tools_panel(ui, tools_rect);
            
            // Draw stage/canvas (central area)
            self.draw_stage(ui, stage_rect);
            
            // Draw library/hierarchy panel (right side)
            self.draw_library(ui, library_rect);
            
            // Draw properties panel
            self.draw_properties_panel(ui, properties_rect);
            
            // Draw timeline (bottom) - capture any println! from timeline
            ui.scope_builder(UiBuilder::new().max_rect(timeline_rect), |ui| {
                // Intercept timeline interactions by checking before/after
                let prev_frame = self.engine.get_current_frame();
                let prev_zoom = self.timeline.state.zoom_level;
                let prev_playing = self.timeline.state.is_playing;
                
                self.timeline.show(ui, &mut self.engine);
                
                let curr_frame = self.engine.get_current_frame();
                if prev_frame != curr_frame {
                    self.log(LogLevel::Action, format!("Playhead moved to frame {}", curr_frame));
                }
                
                if prev_zoom != self.timeline.state.zoom_level {
                    self.log(LogLevel::Action, format!("Zoom changed to {}%", (self.timeline.state.zoom_level * 100.0) as i32));
                }
                
                if prev_playing != self.timeline.state.is_playing {
                    self.log(LogLevel::Action, format!("Playback {}", if self.timeline.state.is_playing { "started" } else { "stopped" }));
                }
            });
            
            // Add horizontal splitter (between stage and library)
            let h_splitter_rect = egui::Rect::from_min_size(
                egui::pos2(stage_rect.max.x, stage_rect.min.y),
                egui::vec2(self.splitter_thickness, stage_rect.height()),
            );
            self.handle_horizontal_splitter(ui, h_splitter_rect);
            
            // Add vertical splitter (between stage and timeline)
            let v_splitter_rect = egui::Rect::from_min_size(
                egui::pos2(stage_rect.min.x, stage_rect.max.y),
                egui::vec2(stage_rect.width(), self.splitter_thickness),
            );
            self.handle_vertical_splitter(ui, v_splitter_rect);
            
            // Add corner splitter for combined resize
            let corner_rect = egui::Rect::from_min_size(
                egui::pos2(stage_rect.max.x, stage_rect.max.y),
                egui::vec2(self.splitter_thickness, self.splitter_thickness),
            );
            ui.painter().rect_filled(corner_rect, 0.0, egui::Color32::GRAY);
            
            // Draw console if visible
            if let Some(script_rect) = script_rect {
                self.draw_script_editor(ui, script_rect);
                
                // Script editor splitter
                let script_splitter_rect = egui::Rect::from_min_size(
                    egui::pos2(script_rect.min.x, script_rect.min.y),
                    egui::vec2(script_rect.width(), self.splitter_thickness),
                );
                self.handle_script_splitter(ui, script_splitter_rect);
            }
            
            if let Some(console_rect) = console_rect {
                self.draw_console(ui, console_rect);
                
                // Console splitter
                let console_splitter_rect = egui::Rect::from_min_size(
                    egui::pos2(console_rect.min.x, console_rect.min.y),
                    egui::vec2(console_rect.width(), self.splitter_thickness),
                );
                self.handle_console_splitter(ui, console_splitter_rect);
            }
            
            ctx.request_repaint();
        });
    }
}

impl TimelineApp {
    fn create_default_library_assets() -> Vec<LibraryAsset> {
        vec![
            // Graphics folder
            LibraryAsset {
                id: "logo_mc".to_string(),
                name: "Logo".to_string(),
                asset_type: LibraryAssetType::MovieClip,
                folder: "Graphics".to_string(),
                properties: AssetProperties {
                    file_size: None,
                    dimensions: Some((200, 100)),
                    format: None,
                    usage_count: 2,
                    linkage_class: Some("LogoSymbol".to_string()),
                },
            },
            LibraryAsset {
                id: "button_mc".to_string(),
                name: "Button".to_string(),
                asset_type: LibraryAssetType::Button,
                folder: "Graphics".to_string(),
                properties: AssetProperties {
                    file_size: None,
                    dimensions: Some((120, 40)),
                    format: None,
                    usage_count: 5,
                    linkage_class: Some("ButtonSymbol".to_string()),
                },
            },
            LibraryAsset {
                id: "star_graphic".to_string(),
                name: "Star".to_string(),
                asset_type: LibraryAssetType::Graphic,
                folder: "Graphics".to_string(),
                properties: AssetProperties {
                    file_size: None,
                    dimensions: Some((50, 50)),
                    format: None,
                    usage_count: 0,
                    linkage_class: None,
                },
            },
            // Bitmaps folder
            LibraryAsset {
                id: "background_jpg".to_string(),
                name: "Background".to_string(),
                asset_type: LibraryAssetType::Bitmap,
                folder: "Bitmaps".to_string(),
                properties: AssetProperties {
                    file_size: Some(245760),
                    dimensions: Some((1920, 1080)),
                    format: Some("JPEG".to_string()),
                    usage_count: 1,
                    linkage_class: None,
                },
            },
            LibraryAsset {
                id: "icon_png".to_string(),
                name: "Icon".to_string(),
                asset_type: LibraryAssetType::Bitmap,
                folder: "Bitmaps".to_string(),
                properties: AssetProperties {
                    file_size: Some(8192),
                    dimensions: Some((64, 64)),
                    format: Some("PNG".to_string()),
                    usage_count: 3,
                    linkage_class: None,
                },
            },
            // Sounds folder
            LibraryAsset {
                id: "bgm_mp3".to_string(),
                name: "Background Music".to_string(),
                asset_type: LibraryAssetType::Sound,
                folder: "Sounds".to_string(),
                properties: AssetProperties {
                    file_size: Some(3145728),
                    dimensions: None,
                    format: Some("MP3".to_string()),
                    usage_count: 1,
                    linkage_class: Some("BGMSound".to_string()),
                },
            },
            LibraryAsset {
                id: "click_wav".to_string(),
                name: "Click Sound".to_string(),
                asset_type: LibraryAssetType::Sound,
                folder: "Sounds".to_string(),
                properties: AssetProperties {
                    file_size: Some(22050),
                    dimensions: None,
                    format: Some("WAV".to_string()),
                    usage_count: 0,
                    linkage_class: None,
                },
            },
            // Fonts folder
            LibraryAsset {
                id: "arial_font".to_string(),
                name: "Arial".to_string(),
                asset_type: LibraryAssetType::Font,
                folder: "Fonts".to_string(),
                properties: AssetProperties {
                    file_size: Some(367112),
                    dimensions: None,
                    format: Some("TTF".to_string()),
                    usage_count: 10,
                    linkage_class: None,
                },
            },
        ]
    }
    
    fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        self.log_messages.push(LogMessage {
            timestamp,
            level,
            message: message.into(),
        });
        
        // Keep only last 1000 messages
        if self.log_messages.len() > 1000 {
            self.log_messages.drain(0..100);
        }
    }
    
    fn take_screenshot(&mut self, _ctx: &egui::Context) {
        // Create timestamp for filename
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("timeline_demo_screenshot_{}.png", timestamp);
        
        // Get the Downloads folder path
        let downloads_path = dirs::download_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let filepath = downloads_path.join(&filename);
        
        // Use macOS screencapture command to capture the window
        let result = Command::new("screencapture")
            .arg("-iW") // Interactive window capture
            .arg(&filepath)
            .spawn();
        
        match result {
            Ok(mut child) => {
                // Spawn a thread to wait for the screenshot and log the result
                let log_sender = self.engine_logs.clone();
                let filepath_str = filepath.to_string_lossy().to_string();
                std::thread::spawn(move || {
                    match child.wait() {
                        Ok(status) if status.success() => {
                            if let Ok(mut logs) = log_sender.lock() {
                                logs.push((LogLevel::Info, format!("📸 Screenshot saved to: {}", filepath_str)));
                                }
                            }
                        Ok(_) => {
                            if let Ok(mut logs) = log_sender.lock() {
                                logs.push((LogLevel::Info, "Screenshot cancelled".to_string()));
                                }
                            }
                        Err(e) => {
                            if let Ok(mut logs) = log_sender.lock() {
                                logs.push((LogLevel::Action, format!("Screenshot error: {}", e)));
                                }
                            }
                    }
                });
                
                self.log(LogLevel::Info, "📸 Click on the Timeline Demo window to capture screenshot (F2)");
            }
            Err(e) => {
                self.log(LogLevel::Action, format!("Failed to start screenshot: {}", e));
            }
        }
    }
    
    fn draw_console(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(20));
            
            // Border
            let border_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(60));
            ui.painter().line_segment([rect.left_top(), rect.right_top()], border_stroke);
            ui.painter().line_segment([rect.right_top(), rect.right_bottom()], border_stroke);
            ui.painter().line_segment([rect.right_bottom(), rect.left_bottom()], border_stroke);
            ui.painter().line_segment([rect.left_bottom(), rect.left_top()], border_stroke);
            
            // Console header
            let header_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 25.0));
            ui.scope_builder(UiBuilder::new().max_rect(header_rect), |ui| {
                ui.horizontal(|ui| {
                    ui.label("🖥️ Developer Console");
                    ui.separator();
                    if ui.button("Clear").clicked() {
                        self.log_messages.clear();
                        self.log(LogLevel::Info, "Console cleared");
                    }
                    ui.separator();
                    ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                    ui.separator();
                    ui.label(format!("{} messages", self.log_messages.len()));
                    
                    // Test panic button (only in debug mode)
                    #[cfg(debug_assertions)]
                    ui.separator();
                    #[cfg(debug_assertions)]
                    if ui.button("💥 Test Crash").clicked() {
                        panic!("Test panic: User clicked the test crash button!");
                    }
                    
                    // Language selector
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("Language:");
                        let current_lang = self.selected_language.clone();
                        ComboBox::from_label("")
                            .selected_text(match current_lang.as_str() {
                                "en" => "English",
                                "es" => "Español",
                                "ja" => "日本語",
                                "zh" => "中文",
                                _ => "English",
                                })
                            .show_ui(ui, |ui| {
                                if ui.selectable_value(&mut self.selected_language, "en".to_string(), "English").clicked() {
                                    self.timeline.i18n.set_language("en");
                                    self.log(LogLevel::Info, "Language changed to English");
                                    }
                                if ui.selectable_value(&mut self.selected_language, "es".to_string(), "Español").clicked() {
                                    self.timeline.i18n.set_language("es");
                                    self.log(LogLevel::Info, "Idioma cambiado a Español");
                                    }
                                if ui.selectable_value(&mut self.selected_language, "ja".to_string(), "日本語").clicked() {
                                    self.timeline.i18n.set_language("ja");
                                    self.log(LogLevel::Info, "言語を日本語に変更しました");
                                    }
                                if ui.selectable_value(&mut self.selected_language, "zh".to_string(), "中文").clicked() {
                                    self.timeline.i18n.set_language("zh");
                                    self.log(LogLevel::Info, "语言已更改为中文");
                                    }
                                });
                    });
                });
            });
            
            // Console content
            let content_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(0.0, 25.0),
                egui::vec2(rect.width(), rect.height() - 25.0),
            );
            
            ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(self.auto_scroll)
                    .show(ui, |ui| {
                        for msg in &self.log_messages {
                            let color = match msg.level {
                                LogLevel::Info => egui::Color32::from_gray(180),
                                LogLevel::Action => egui::Color32::from_rgb(100, 200, 100),
                                LogLevel::Warning => egui::Color32::from_rgb(255, 200, 100),
                                LogLevel::Error => egui::Color32::from_rgb(255, 100, 100),
                                };
                            
                            ui.horizontal(|ui| {
                                ui.colored_label(egui::Color32::from_gray(120), &msg.timestamp);
                                ui.colored_label(color, &msg.message);
                                });
                            }
                    });
            });
        });
    }
    
    fn handle_console_splitter(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        
        let color = if response.hovered() {
            egui::Color32::from_gray(100)
        } else {
            egui::Color32::from_gray(70)
        };
        ui.painter().rect_filled(rect, 0.0, color);
        
        if response.dragged() {
            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let available_height = ui.available_height();
                self.console_height = (available_height - pointer_pos.y + rect.height() / 2.0)
                    .clamp(50.0, available_height - 300.0);
            }
        }
        
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
    }
    
    fn draw_script_editor(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(25));
            
            // Border
            let border_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(60));
            ui.painter().rect_stroke(rect, 0.0, border_stroke, egui::epaint::StrokeKind::Outside);
            
            // Script editor header
            let header_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 30.0));
            ui.scope_builder(UiBuilder::new().max_rect(header_rect), |ui| {
                ui.horizontal(|ui| {
                    ui.label("📜 Script Editor");
                    ui.separator();
                    
                    if ui.button("▶ Run").clicked() {
                        self.log(LogLevel::Action, "Executing script...");
                        // For demo purposes, just log the script
                        self.log(LogLevel::Info, format!("Script content: {}", self.script_content.lines().next().unwrap_or("(empty)")));
                        self.script_error = None;
                    }
                    
                    if ui.button("⏹ Stop").clicked() {
                        self.log(LogLevel::Action, "Script execution stopped");
                    }
                    
                    if ui.button("🗑 Clear").clicked() {
                        self.script_content.clear();
                        self.script_error = None;
                        self.log(LogLevel::Action, "Script editor cleared");
                    }
                    
                    ui.separator();
                    
                    // Script templates dropdown
                    ComboBox::from_label("Templates")
                        .selected_text("Select template...")
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(false, "Loop Animation").clicked() {
                                self.script_content = templates::LOOP_ANIMATION.to_string();
                                self.log(LogLevel::Info, "Loaded loop animation template");
                                }
                            if ui.selectable_label(false, "Stop at Frame").clicked() {
                                self.script_content = templates::STOP_AT_FRAME.to_string();
                                self.log(LogLevel::Info, "Loaded stop at frame template");
                                }
                            if ui.selectable_label(false, "Animate Object").clicked() {
                                self.script_content = templates::ANIMATE_OBJECT.to_string();
                                self.log(LogLevel::Info, "Loaded animate object template");
                                }
                            if ui.selectable_label(false, "Create Object").clicked() {
                                self.script_content = templates::CREATE_OBJECT.to_string();
                                self.log(LogLevel::Info, "Loaded create object template");
                                }
                            });
                    
                    // Error indicator
                    if let Some(error) = &self.script_error {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", error));
                            });
                    }
                });
            });
            
            // Script content area
            let content_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(0.0, 30.0),
                egui::vec2(rect.width(), rect.height() - 30.0),
            );
            
            ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Script editor with syntax highlighting (basic)
                        let mut layouter = |ui: &egui::Ui, text_buffer: &dyn egui::TextBuffer, _wrap_width: f32| {
                            let mut job = egui::text::LayoutJob::default();
                            let string = text_buffer.as_str();
                            
                            // Basic syntax highlighting
                            for line in string.lines() {
                                if line.trim().starts_with("//") {
                                    // Comments in green
                                    job.append(line, 0.0, egui::TextFormat {
                                        color: egui::Color32::from_rgb(120, 200, 120),
                                        ..Default::default()
                                        });
                                    } else if line.contains("timeline.") || line.contains("stage.") {
                                    // API calls in blue
                                    job.append(line, 0.0, egui::TextFormat {
                                        color: egui::Color32::from_rgb(120, 160, 255),
                                        ..Default::default()
                                        });
                                    } else if line.contains("if") || line.contains("let") || line.contains("fn") {
                                    // Keywords in orange
                                    job.append(line, 0.0, egui::TextFormat {
                                        color: egui::Color32::from_rgb(255, 180, 100),
                                        ..Default::default()
                                        });
                                    } else {
                                    // Default text
                                    job.append(line, 0.0, egui::TextFormat::default());
                                    }
                                job.append("\n", 0.0, egui::TextFormat::default());
                                }
                            
                            job.wrap.max_width = f32::INFINITY;
                            ui.fonts(|f| f.layout_job(job))
                            };
                        
                        let response = ui.add(
                            egui::TextEdit::multiline(&mut self.script_content)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .desired_rows(20)
                                .layouter(&mut layouter)
                        );
                        
                        if response.changed() {
                            self.script_error = None;
                            }
                    });
            });
        });
    }
    
    fn handle_script_splitter(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        
        let color = if response.hovered() {
            egui::Color32::from_gray(100)
        } else {
            egui::Color32::from_gray(70)
        };
        ui.painter().rect_filled(rect, 0.0, color);
        
        if response.dragged() {
            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let available_height = ui.available_height();
                self.script_panel_height = (available_height - pointer_pos.y + rect.height() / 2.0)
                    .clamp(100.0, available_height - 300.0);
            }
        }
        
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
    }
    
    fn draw_stage(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(30));
            
            // Border
            let border_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(60));
            ui.painter().line_segment([rect.left_top(), rect.right_top()], border_stroke);
            ui.painter().line_segment([rect.right_top(), rect.right_bottom()], border_stroke);
            ui.painter().line_segment([rect.right_bottom(), rect.left_bottom()], border_stroke);
            ui.painter().line_segment([rect.left_bottom(), rect.left_top()], border_stroke);
            
            // Grid pattern for visual reference
            let grid_size = 50.0;
            let grid_color = egui::Color32::from_gray(35);
            
            // Vertical lines
            let mut x = rect.left() + grid_size;
            while x < rect.right() {
                ui.painter().line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    egui::Stroke::new(0.5, grid_color),
                );
                x += grid_size;
            }
            
            // Horizontal lines
            let mut y = rect.top() + grid_size;
            while y < rect.bottom() {
                ui.painter().line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    egui::Stroke::new(0.5, grid_color),
                );
                y += grid_size;
            }
            
            // Draw stage items
            let mut clicked_item = None;
            let mut right_clicked_item = None;
            let mut hovered_item = None;
            let mut drag_info = None;
            
            for (index, item) in self.stage_items.iter().enumerate() {
                let item_rect = egui::Rect::from_center_size(
                    rect.min + item.position.to_vec2(),
                    item.size
                );
                
                // Check if item is visible in stage
                if !rect.intersects(item_rect) {
                    continue;
                }
                
                // Item interaction - use allocate_rect to ensure proper event handling
                let response = ui.allocate_rect(item_rect, egui::Sense::click_and_drag());
                
                // Handle hover
                if response.hovered() {
                    hovered_item = Some(index);
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                
                // Handle selection
                let is_selected = self.selected_items.contains(&index);
                
                // Handle dragging
                if response.dragged() && is_selected {
                    let delta = response.drag_delta();
                    drag_info = Some((index, delta));
                }
                
                // Handle clicks
                if response.clicked() {
                    clicked_item = Some(index);
                }
                
                // Handle right-click
                if response.secondary_clicked() {
                    right_clicked_item = Some(index);
                }
                
                // Draw the item based on type
                match item.item_type {
                    StageItemType::Rectangle => {
                        // Apply rotation if needed
                        if item.rotation != 0.0 {
                            // For simplicity, we'll just draw without rotation for now
                            // Full rotation would require transform matrix
                            }
                        
                        let color_with_alpha = egui::Color32::from_rgba_premultiplied(
                            item.color.r(),
                            item.color.g(), 
                            item.color.b(),
                            (item.alpha * 255.0) as u8
                        );
                        ui.painter().rect_filled(item_rect, 5.0, color_with_alpha);
                        
                        // Draw selection border if selected
                        if is_selected {
                            let stroke = egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE);
                            // Draw rect border using line segments for egui 0.32
                            let r = item_rect;
                            ui.painter().line_segment([r.left_top(), r.right_top()], stroke);
                            ui.painter().line_segment([r.right_top(), r.right_bottom()], stroke);
                            ui.painter().line_segment([r.right_bottom(), r.left_bottom()], stroke);
                            ui.painter().line_segment([r.left_bottom(), r.left_top()], stroke);
                            }
                    },
                    StageItemType::Circle => {
                        let center = item_rect.center();
                        let radius = item.size.x.min(item.size.y) / 2.0;
                        let color_with_alpha = egui::Color32::from_rgba_premultiplied(
                            item.color.r(),
                            item.color.g(), 
                            item.color.b(),
                            (item.alpha * 255.0) as u8
                        );
                        ui.painter().circle_filled(center, radius, color_with_alpha);
                        
                        // Draw selection border if selected
                        if is_selected {
                            ui.painter().circle_stroke(
                                center,
                                radius + 2.0,
                                egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE)
                            );
                            }
                    },
                    StageItemType::Text => {
                        let color_with_alpha = egui::Color32::from_rgba_premultiplied(
                            item.color.r(),
                            item.color.g(), 
                            item.color.b(),
                            (item.alpha * 255.0) as u8
                        );
                        ui.painter().text(
                            item_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &item.text_content,
                            egui::FontId::proportional(item.font_size),
                            color_with_alpha,
                        );
                        
                        // Draw selection rect if selected
                        if is_selected {
                            let stroke = egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE);
                            // Draw rect border using line segments for egui 0.32
                            let r = item_rect;
                            ui.painter().line_segment([r.left_top(), r.right_top()], stroke);
                            ui.painter().line_segment([r.right_top(), r.right_bottom()], stroke);
                            ui.painter().line_segment([r.right_bottom(), r.left_bottom()], stroke);
                            ui.painter().line_segment([r.left_bottom(), r.left_top()], stroke);
                            }
                    },
                    StageItemType::MovieClip => {
                        // Draw as a rounded rectangle with icon
                        let color_with_alpha = egui::Color32::from_rgba_premultiplied(
                            item.color.r(),
                            item.color.g(), 
                            item.color.b(),
                            (item.alpha * 255.0) as u8
                        );
                        ui.painter().rect_filled(item_rect, 10.0, color_with_alpha);
                        ui.painter().text(
                            item_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "🎬",
                            egui::FontId::proportional(24.0),
                            egui::Color32::BLACK,
                        );
                        
                        // Draw selection border if selected
                        if is_selected {
                            let stroke = egui::Stroke::new(2.0, egui::Color32::LIGHT_BLUE);
                            // Draw rounded rect border using line segments for egui 0.32
                            let r = item_rect;
                            ui.painter().line_segment([r.left_top(), r.right_top()], stroke);
                            ui.painter().line_segment([r.right_top(), r.right_bottom()], stroke);
                            ui.painter().line_segment([r.right_bottom(), r.left_bottom()], stroke);
                            ui.painter().line_segment([r.left_bottom(), r.left_top()], stroke);
                            }
                    },
                }
                
                // Draw item name when hovered
                if hovered_item == Some(index) {
                    let name_pos = item_rect.center_bottom() + egui::vec2(0.0, 5.0);
                    ui.painter().text(
                        name_pos,
                        egui::Align2::CENTER_TOP,
                        &item.name,
                        egui::FontId::proportional(12.0),
                        egui::Color32::WHITE,
                    );
                }
            }
            
            // Handle stage background interactions (only if no item was clicked)
            if clicked_item.is_none() && right_clicked_item.is_none() {
                let stage_response = ui.interact(rect, ui.id().with("stage_bg"), egui::Sense::click());
                
                if stage_response.clicked() {
                    if let Some(pos) = stage_response.interact_pointer_pos() {
                        let stage_pos = pos - rect.min.to_vec2();
                        
                        // Handle tool-specific actions
                        match self.tool_state.active_tool {
                            Tool::Arrow => {
                            // Arrow tool - deselect when clicking empty space
                            self.selected_items.clear();
                            self.log(LogLevel::Action, format!("Stage clicked at ({:.1}, {:.1})", 
                                stage_pos.x, stage_pos.y));
                            }
                            Tool::Rectangle => {
                            // Rectangle tool - create a rectangle
                            let new_rect = StageItem {
                                id: format!("rect_{}", self.stage_items.len() + 1),
                                name: format!("Rectangle {}", self.stage_items.len() + 1),
                                item_type: StageItemType::Rectangle,
                                position: stage_pos,
                                size: egui::Vec2::new(100.0, 60.0),
                                color: self.tool_state.fill_color,
                                alpha: 1.0,
                                rotation: 0.0,
                                text_content: String::new(),
                                font_size: 16.0,
                                font_family: "Arial".to_string(),
                                };
                            self.stage_items.push(new_rect.clone());
                            self.log(LogLevel::Action, format!("Created {} with Rectangle tool", new_rect.name));
                            }
                            Tool::Oval => {
                            // Oval tool - create a circle/oval
                            let new_oval = StageItem {
                                id: format!("oval_{}", self.stage_items.len() + 1),
                                name: format!("Circle {}", self.stage_items.len() + 1),
                                item_type: StageItemType::Circle,
                                position: stage_pos,
                                size: egui::Vec2::splat(80.0),
                                color: self.tool_state.fill_color,
                                alpha: 1.0,
                                rotation: 0.0,
                                text_content: String::new(),
                                font_size: 16.0,
                                font_family: "Arial".to_string(),
                                };
                            self.stage_items.push(new_oval.clone());
                            self.log(LogLevel::Action, format!("Created {} with Oval tool", new_oval.name));
                            }
                            Tool::Text => {
                            // Text tool - create a text object
                            let new_text = StageItem {
                                id: format!("text_{}", self.stage_items.len() + 1),
                                name: format!("Text {}", self.stage_items.len() + 1),
                                item_type: StageItemType::Text,
                                position: stage_pos,
                                size: egui::Vec2::new(120.0, 30.0),
                                color: self.tool_state.fill_color,
                                alpha: 1.0,
                                rotation: 0.0,
                                text_content: "New Text".to_string(),
                                font_size: self.tool_state.text_font_size,
                                font_family: self.tool_state.text_font_family.clone(),
                                };
                            self.stage_items.push(new_text.clone());
                            self.log(LogLevel::Action, format!("Created {} with Text tool", new_text.name));
                            }
                            _ => {
                            // Other tools - just log the click for now
                            self.log(LogLevel::Action, format!("{} clicked at ({:.1}, {:.1})", 
                                self.tool_state.active_tool.get_name(), stage_pos.x, stage_pos.y));
                            }
                    }
                    }
                }
                
                if stage_response.secondary_clicked() {
                    // Right-clicked on empty stage
                    if let Some(pos) = stage_response.interact_pointer_pos() {
                        self.context_menu = Some(ContextMenuState {
                            position: pos,
                            menu_type: ContextMenuType::Stage(pos - rect.min.to_vec2()),
                            });
                    }
                }
            }
            
            // Handle item clicks after drawing
            if let Some(index) = clicked_item {
                // Handle multi-selection with Ctrl/Cmd
                let modifiers = ui.input(|i| i.modifiers);
                if modifiers.ctrl || modifiers.command {
                    // Toggle selection
                    if self.selected_items.contains(&index) {
                        self.selected_items.retain(|&i| i != index);
                        self.log(LogLevel::Action, format!("Deselected: {}", self.stage_items[index].name));
                    } else {
                        self.selected_items.push(index);
                        self.log(LogLevel::Action, format!("Added to selection: {}", self.stage_items[index].name));
                    }
                } else {
                    // Single selection
                    self.selected_items.clear();
                    self.selected_items.push(index);
                    self.log(LogLevel::Action, format!("Selected: {}", self.stage_items[index].name));
                }
            }
            
            if let Some(index) = right_clicked_item {
                // Ensure right-clicked item is selected
                if !self.selected_items.contains(&index) {
                    self.selected_items.clear();
                    self.selected_items.push(index);
                }
                if let Some(pos) = ui.ctx().pointer_interact_pos() {
                    self.context_menu = Some(ContextMenuState {
                        position: pos,
                        menu_type: ContextMenuType::StageItem(index),
                    });
                }
            }
            
            // Apply drag movement after the loop to avoid borrowing issues
            if let Some((index, delta)) = drag_info {
                if let Some(item) = self.stage_items.get_mut(index) {
                    item.position += delta;
                    let name = item.name.clone();
                    let pos = item.position;
                    self.log(LogLevel::Action, format!("Moving {} to ({:.1}, {:.1})", 
                        name, pos.x, pos.y));
                }
            }
            
            // Show context menu if active
            if let Some(menu_state) = &self.context_menu.clone() {
                self.show_context_menu(ui, menu_state, rect);
            }
        });
    }
    
    fn draw_library(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(40));
            
            // Border
            let border_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(60));
            ui.painter().line_segment([rect.left_top(), rect.right_top()], border_stroke);
            ui.painter().line_segment([rect.right_top(), rect.right_bottom()], border_stroke);
            ui.painter().line_segment([rect.right_bottom(), rect.left_bottom()], border_stroke);
            ui.painter().line_segment([rect.left_bottom(), rect.left_top()], border_stroke);
            
            // Content with padding
            let padded_rect = rect.shrink(10.0);
            ui.scope_builder(UiBuilder::new().max_rect(padded_rect), |ui| {
                ui.vertical(|ui| {
                    // Header with options
                    ui.horizontal(|ui| {
                        ui.heading("📚 Library");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("⚙").on_hover_text("Library Options").clicked() {
                                self.log(LogLevel::Action, "Library options clicked");
                                }
                            });
                    });
                    ui.separator();
                    
                    // Tabs
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.library_tab == LibraryTab::Assets, "Assets").clicked() {
                            self.library_tab = LibraryTab::Assets;
                            self.log(LogLevel::Action, "Library tab: Assets selected");
                            }
                        if ui.selectable_label(self.library_tab == LibraryTab::Components, "Components").clicked() {
                            self.library_tab = LibraryTab::Components;
                            self.log(LogLevel::Action, "Library tab: Components selected");
                            }
                        if ui.selectable_label(self.library_tab == LibraryTab::ActionScript, "AS Linkage").clicked() {
                            self.library_tab = LibraryTab::ActionScript;
                            self.log(LogLevel::Action, "Library tab: ActionScript selected");
                            }
                    });
                    
                    ui.separator();
                    
                    // Tab content
                    match self.library_tab {
                        LibraryTab::Assets => self.draw_library_assets_tab(ui),
                        LibraryTab::Components => self.draw_library_components_tab(ui),
                        LibraryTab::ActionScript => self.draw_library_actionscript_tab(ui),
                    }
                });
            });
        });
        
        // Handle drag and drop
        self.handle_library_drag_drop(ui, rect);
        
        // Handle context menu
        if let Some(menu_state) = &self.library_context_menu.clone() {
            self.show_library_context_menu(ui, menu_state);
        }
    }
    
    fn draw_library_assets_tab(&mut self, ui: &mut egui::Ui) {
        // Search bar
        ui.horizontal(|ui| {
            ui.label("🔍");
            let search_response = ui.text_edit_singleline(&mut self.library_search);
            if search_response.changed() {
                self.log(LogLevel::Action, format!("Library search: '{}'", self.library_search));
            }
        });
        
        ui.separator();
        
        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("➕ Import").on_hover_text("Import assets from file").clicked() {
                self.log(LogLevel::Action, "Import assets clicked");
            }
            if ui.button("🆕 New Symbol").on_hover_text("Create new symbol").clicked() {
                self.log(LogLevel::Action, "New symbol clicked");
            }
            if ui.button("📁 New Folder").on_hover_text("Create new folder").clicked() {
                self.log(LogLevel::Action, "New folder clicked");
            }
        });
        
        ui.separator();
        
        // Asset tree view
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Group assets by folder
            let mut folders: std::collections::HashMap<String, Vec<LibraryAsset>> = std::collections::HashMap::new();
            folders.insert("Graphics".to_string(), Vec::new());
            folders.insert("Bitmaps".to_string(), Vec::new());
            folders.insert("Sounds".to_string(), Vec::new());
            folders.insert("Fonts".to_string(), Vec::new());
            
            let search_term = self.library_search.to_lowercase();
            for asset in &self.library_assets {
                if self.library_search.is_empty() || 
                   asset.name.to_lowercase().contains(&search_term) {
                    folders.entry(asset.folder.clone()).or_default().push(asset.clone());
                }
            }
            
            // Draw folders
            let mut folders_to_expand = Vec::new();
            let mut folders_to_collapse = Vec::new();
            let mut log_messages = Vec::new();
            
            // Define folder order for consistent display
            let folder_order = ["Sounds", "Fonts", "Bitmaps", "Graphics"];
            let mut sorted_folders: Vec<_> = Vec::new();
            
            // Add folders in defined order
            for &folder_name in &folder_order {
                if let Some(assets) = folders.get(folder_name) {
                    sorted_folders.push((folder_name, assets));
                }
            }
            
            // Add any remaining folders not in the predefined order
            for (folder_name, assets) in folders.iter() {
                if !folder_order.contains(&folder_name.as_str()) {
                    sorted_folders.push((folder_name.as_str(), assets));
                }
            }
            
            for (folder_name, assets) in sorted_folders {
                let folder_id = egui::Id::new(format!("library_folder_{}", folder_name));
                let is_expanded = self.library_folders_expanded.contains(&folder_name.to_string());
                
                ui.horizontal(|ui| {
                    // Folder toggle
                    if ui.small_button(if is_expanded { "▼" } else { "▶" }).clicked() {
                        if is_expanded {
                            folders_to_collapse.push(folder_name.to_string());
                            } else {
                            folders_to_expand.push(folder_name.to_string());
                            }
                        log_messages.push((LogLevel::Action, format!("Folder '{}' {}", 
                            folder_name, if is_expanded { "collapsed" } else { "expanded" })));
                    }
                    
                    // Folder icon and name
                    ui.label("📁");
                    if ui.selectable_label(false, folder_name).clicked() {
                        log_messages.push((LogLevel::Action, format!("Folder '{}' clicked", folder_name)));
                    }
                    
                    // Right-click context menu
                    ui.interact(ui.min_rect(), folder_id, egui::Sense::click())
                        .context_menu(|ui| {
                            if ui.button("📁 New Folder").clicked() {
                                log_messages.push((LogLevel::Action, format!("New folder in '{}'", folder_name)));
                                ui.close();
                                }
                            if ui.button("➕ Import to Folder").clicked() {
                                log_messages.push((LogLevel::Action, format!("Import to '{}'", folder_name)));
                                ui.close();
                                }
                            ui.separator();
                            if ui.button("✏️ Rename").clicked() {
                                log_messages.push((LogLevel::Action, format!("Rename folder '{}'", folder_name)));
                                ui.close();
                                }
                            if ui.button("🗑️ Delete").clicked() {
                                log_messages.push((LogLevel::Action, format!("Delete folder '{}'", folder_name)));
                                ui.close();
                                }
                            });
                });
                
                // Draw assets in folder if expanded
                if is_expanded {
                    ui.indent(folder_id, |ui| {
                        for asset in assets {
                            self.draw_library_asset(ui, asset);
                            }
                    });
                }
            }
            
            // Apply folder state changes
            for folder in folders_to_expand {
                self.library_folders_expanded.push(folder);
            }
            for folder in folders_to_collapse {
                self.library_folders_expanded.retain(|f| f != &folder);
            }
            
            // Log messages
            for (level, msg) in log_messages {
                self.log(level, msg);
            }
        });
    }
    
    fn draw_library_asset(&mut self, ui: &mut egui::Ui, asset: &LibraryAsset) {
        let is_selected = self.selected_library_asset.as_ref() == Some(&asset.id);
        
        ui.horizontal(|ui| {
            // Asset icon
            let icon = match asset.asset_type {
                LibraryAssetType::MovieClip => "🎭",
                LibraryAssetType::Button => "🔘",
                LibraryAssetType::Graphic => "🎨",
                LibraryAssetType::Bitmap => "🖼️",
                LibraryAssetType::Sound => "🔊",
                LibraryAssetType::Video => "🎬",
                LibraryAssetType::Font => "🔤",
                LibraryAssetType::Folder => "📁",
            };
            ui.label(icon);
            
            // Asset name with selection
            let response = ui.selectable_label(is_selected, &asset.name);
            
            // Type label
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.weak(match asset.asset_type {
                    LibraryAssetType::MovieClip => "MovieClip",
                    LibraryAssetType::Button => "Button",
                    LibraryAssetType::Graphic => "Graphic",
                    LibraryAssetType::Bitmap => "Bitmap",
                    LibraryAssetType::Sound => "Sound",
                    LibraryAssetType::Video => "Video",
                    LibraryAssetType::Font => "Font",
                    LibraryAssetType::Folder => "Folder",
                });
            });
            
            // Handle selection
            if response.clicked() {
                self.selected_library_asset = Some(asset.id.clone());
                self.log(LogLevel::Action, format!("Selected asset: {}", asset.name));
            }
            
            // Handle drag start
            if response.drag_started() {
                self.dragging_asset = Some(asset.clone());
                self.drag_offset = response.interact_pointer_pos()
                    .map(|p| p - response.rect.center())
                    .unwrap_or(egui::Vec2::ZERO);
                self.log(LogLevel::Action, format!("Started dragging: {}", asset.name));
            }
            
            // Right-click context menu
            response.context_menu(|ui| {
                ui.label(&asset.name);
                ui.separator();
                
                if ui.button("✏️ Rename").clicked() {
                    self.log(LogLevel::Action, format!("Rename asset: {}", asset.name));
                    ui.close();
                }
                if ui.button("📑 Duplicate").clicked() {
                    self.log(LogLevel::Action, format!("Duplicate asset: {}", asset.name));
                    ui.close();
                }
                if ui.button("🗑️ Delete").clicked() {
                    self.log(LogLevel::Action, format!("Delete asset: {}", asset.name));
                    ui.close();
                }
                ui.separator();
                
                if ui.button("ℹ️ Properties").clicked() {
                    self.log(LogLevel::Action, format!("Show properties: {}", asset.name));
                    ui.close();
                }
                if ui.button("✏️ Edit").clicked() {
                    self.log(LogLevel::Action, format!("Edit asset: {}", asset.name));
                    ui.close();
                }
                
                if asset.properties.linkage_class.is_some() {
                    ui.separator();
                    if ui.button("🔗 Edit Linkage").clicked() {
                        self.log(LogLevel::Action, format!("Edit linkage: {}", asset.name));
                        ui.close();
                    }
                }
            });
        });
    }
    
    fn draw_library_components_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Component library - organize reusable UI components");
        ui.separator();
        ui.weak("No components available");
    }
    
    fn draw_library_actionscript_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("ActionScript Linkage - Export symbols for code access");
        ui.separator();
        
        // Show assets with linkage
        for asset in &self.library_assets {
            if let Some(linkage_class) = &asset.properties.linkage_class {
                ui.horizontal(|ui| {
                    ui.label(&asset.name);
                    ui.label("→");
                    ui.code(linkage_class);
                });
            }
        }
    }
    
    fn handle_library_drag_drop(&mut self, ui: &mut egui::Ui, library_rect: egui::Rect) {
        if let Some(asset) = self.dragging_asset.clone() {
            if ui.input(|i| i.pointer.any_released()) {
                // Check if dropped on stage
                if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                    if !library_rect.contains(pointer_pos) {
                        // Dropped outside library - create instance on stage
                        let stage_pos = pointer_pos - self.drag_offset;
                        self.create_stage_instance_from_asset(&asset, stage_pos);
                    }
                }
                self.dragging_asset = None;
                self.drag_offset = egui::Vec2::ZERO;
            } else if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                // Draw drag preview
                let preview_pos = pointer_pos - self.drag_offset;
                ui.painter().text(
                    preview_pos,
                    egui::Align2::CENTER_CENTER,
                    &asset.name,
                    egui::FontId::default(),
                    ui.style().visuals.text_color(),
                );
            }
        }
    }
    
    fn create_stage_instance_from_asset(&mut self, asset: &LibraryAsset, position: egui::Pos2) {
        let new_item = match asset.asset_type {
            LibraryAssetType::MovieClip | LibraryAssetType::Button | LibraryAssetType::Graphic => {
                StageItem {
                    id: format!("{}_{}", asset.id, self.stage_items.len() + 1),
                    name: format!("{} Instance", asset.name),
                    item_type: StageItemType::MovieClip,
                    position,
                    size: egui::Vec2::new(100.0, 100.0),
                    color: egui::Color32::from_rgb(150, 255, 150),
                    alpha: 1.0,
                    rotation: 0.0,
                    text_content: String::new(),
                    font_size: 16.0,
                    font_family: "Arial".to_string(),
                }
            }
            LibraryAssetType::Bitmap => {
                let size = asset.properties.dimensions
                    .map(|(w, h)| egui::Vec2::new(w as f32, h as f32))
                    .unwrap_or(egui::Vec2::new(100.0, 100.0));
                StageItem {
                    id: format!("{}_{}", asset.id, self.stage_items.len() + 1),
                    name: format!("{} Instance", asset.name),
                    item_type: StageItemType::Rectangle, // Using rectangle as placeholder for bitmap
                    position,
                    size,
                    color: egui::Color32::WHITE,
                    alpha: 1.0,
                    rotation: 0.0,
                    text_content: String::new(),
                    font_size: 16.0,
                    font_family: "Arial".to_string(),
                }
            }
            _ => return, // Don't create instances for other types
        };
        
        self.stage_items.push(new_item.clone());
        self.log(LogLevel::Action, format!("Created {} from library at ({:.1}, {:.1})", 
            new_item.name, position.x, position.y));
    }
    
    fn show_library_context_menu(&mut self, ui: &mut egui::Ui, menu_state: &LibraryContextMenuState) {
        egui::Window::new("library_context_menu")
            .fixed_pos(menu_state.position)
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                match &menu_state.target {
                    LibraryContextTarget::Asset(asset_id) => {
                        let asset_name = self.library_assets.iter()
                            .find(|a| a.id == *asset_id)
                            .map(|a| a.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());
                        
                        ui.label(&asset_name);
                        ui.separator();
                        
                        if ui.button("✏️ Rename").clicked() {
                            self.log(LogLevel::Action, format!("Rename asset: {}", asset_name));
                            self.library_context_menu = None;
                            }
                        if ui.button("📑 Duplicate").clicked() {
                            self.log(LogLevel::Action, format!("Duplicate asset: {}", asset_name));
                            self.library_context_menu = None;
                            }
                        if ui.button("🗑️ Delete").clicked() {
                            self.log(LogLevel::Action, format!("Delete asset: {}", asset_name));
                            self.library_context_menu = None;
                            }
                    }
                    LibraryContextTarget::Folder(folder_name) => {
                        ui.label(folder_name);
                        ui.separator();
                        
                        if ui.button("📁 New Folder").clicked() {
                            self.log(LogLevel::Action, format!("New folder in '{}'", folder_name));
                            self.library_context_menu = None;
                            }
                        if ui.button("➕ Import").clicked() {
                            self.log(LogLevel::Action, format!("Import to '{}'", folder_name));
                            self.library_context_menu = None;
                            }
                    }
                    LibraryContextTarget::Background => {
                        if ui.button("📁 New Folder").clicked() {
                            self.log(LogLevel::Action, "New root folder");
                            self.library_context_menu = None;
                            }
                        if ui.button("➕ Import Assets").clicked() {
                            self.log(LogLevel::Action, "Import assets");
                            self.library_context_menu = None;
                            }
                    }
                }
            });
    }
    
    fn draw_tools_panel(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(45));
            
            // Border
            let border_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(60));
            ui.painter().line_segment([rect.right_top(), rect.right_bottom()], border_stroke);
            
            // Content with padding
            let padded_rect = rect.shrink(5.0);
            ui.scope_builder(UiBuilder::new().max_rect(padded_rect), |ui| {
                ui.vertical(|ui| {
                    // Selection Tools Section
                    ui.label("Selection");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        self.draw_tool_button(ui, Tool::Arrow);
                        self.draw_tool_button(ui, Tool::Subselection);
                    });
                    ui.horizontal(|ui| {
                        self.draw_tool_button(ui, Tool::Lasso);
                        ui.add_space(24.0); // Empty space
                    });
                    
                    ui.add_space(10.0);
                    
                    // Drawing Tools Section
                    ui.label("Drawing");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        self.draw_tool_button(ui, Tool::Line);
                        self.draw_tool_button(ui, Tool::Pen);
                    });
                    ui.horizontal(|ui| {
                        self.draw_tool_button(ui, Tool::Pencil);
                        self.draw_tool_button(ui, Tool::Brush);
                    });
                    ui.horizontal(|ui| {
                        self.draw_tool_button(ui, Tool::Rectangle);
                        self.draw_tool_button(ui, Tool::Oval);
                    });
                    ui.horizontal(|ui| {
                        self.draw_tool_button(ui, Tool::PolyStar);
                        ui.add_space(24.0);
                    });
                    
                    ui.add_space(10.0);
                    
                    // Text & Paint Tools Section
                    ui.label("Text & Paint");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        self.draw_tool_button(ui, Tool::Text);
                        self.draw_tool_button(ui, Tool::PaintBucket);
                    });
                    ui.horizontal(|ui| {
                        self.draw_tool_button(ui, Tool::InkBottle);
                        self.draw_tool_button(ui, Tool::Eyedropper);
                    });
                    ui.horizontal(|ui| {
                        self.draw_tool_button(ui, Tool::Eraser);
                        ui.add_space(24.0);
                    });
                    
                    ui.add_space(10.0);
                    
                    // Transform Tools Section
                    ui.label("Transform");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        self.draw_tool_button(ui, Tool::FreeTransform);
                        self.draw_tool_button(ui, Tool::GradientTransform);
                    });
                    ui.horizontal(|ui| {
                        self.draw_tool_button(ui, Tool::Zoom);
                        self.draw_tool_button(ui, Tool::Hand);
                    });
                    
                    ui.add_space(20.0);
                    
                    // Color Controls
                    ui.label("Colors");
                    ui.separator();
                    
                    // Stroke and Fill color swatches
                    ui.horizontal(|ui| {
                        // Stroke color
                        ui.vertical(|ui| {
                            ui.label("Stroke");
                            let stroke_response = ui.add_sized(
                                egui::vec2(30.0, 30.0),
                                egui::Button::new("")
                                    .fill(self.tool_state.stroke_color)
                            );
                            if stroke_response.clicked() {
                                self.log(LogLevel::Action, "Stroke color picker opened");
                                }
                            });
                        
                        // Fill color
                        ui.vertical(|ui| {
                            ui.label("Fill");
                            let fill_response = ui.add_sized(
                                egui::vec2(30.0, 30.0),
                                egui::Button::new("")
                                    .fill(self.tool_state.fill_color)
                            );
                            if fill_response.clicked() {
                                self.log(LogLevel::Action, "Fill color picker opened");
                                }
                            });
                    });
                    
                    // Swap colors button
                    if ui.button("⇄ Swap").clicked() {
                        std::mem::swap(&mut self.tool_state.stroke_color, &mut self.tool_state.fill_color);
                        self.log(LogLevel::Action, "Swapped stroke and fill colors");
                    }
                    
                    // Default colors button
                    if ui.button("⬜⬛ Default").clicked() {
                        self.tool_state.stroke_color = egui::Color32::BLACK;
                        self.tool_state.fill_color = egui::Color32::WHITE;
                        self.log(LogLevel::Action, "Reset to default colors");
                    }
                });
            });
        });
    }
    
    fn draw_tool_button(&mut self, ui: &mut egui::Ui, tool: Tool) {
        let is_active = self.tool_state.active_tool == tool;
        let button_size = egui::vec2(24.0, 24.0);
        
        let mut button = egui::Button::new(tool.get_icon())
            .min_size(button_size);
            
        if is_active {
            button = button.fill(ui.style().visuals.selection.bg_fill);
        }
        
        let response = ui.add(button);
        
        // Tooltip with shortcut
        let mut tooltip = tool.get_name().to_string();
        if let Some(shortcut) = tool.get_shortcut() {
            tooltip.push_str(&format!(" ({})", shortcut));
        }
        
        // Handle click and hover
        if response.clicked() {
            self.tool_state.active_tool = tool;
            self.log(LogLevel::Action, format!("Selected tool: {}", tool.get_name()));
            
            // Update cursor based on tool
            match tool {
                Tool::Hand => ui.ctx().set_cursor_icon(egui::CursorIcon::Grab),
                Tool::Zoom => ui.ctx().set_cursor_icon(egui::CursorIcon::ZoomIn),
                Tool::Text => ui.ctx().set_cursor_icon(egui::CursorIcon::Text),
                _ => ui.ctx().set_cursor_icon(egui::CursorIcon::Default),
            }
        }
        
        response.on_hover_text(tooltip);
    }
    
    fn handle_horizontal_splitter(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        
        // Visual feedback
        let color = if response.hovered() {
            egui::Color32::from_gray(100)
        } else {
            egui::Color32::from_gray(70)
        };
        ui.painter().rect_filled(rect, 0.0, color);
        
        // Handle dragging
        if response.dragged() {
            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let available_width = ui.available_width();
                self.library_width = (available_width - pointer_pos.x + rect.width() / 2.0)
                    .clamp(150.0, available_width - 200.0);
            }
        }
        
        // Change cursor on hover
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
    }
    
    fn handle_vertical_splitter(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        
        // Visual feedback
        let color = if response.hovered() {
            egui::Color32::from_gray(100)
        } else {
            egui::Color32::from_gray(70)
        };
        ui.painter().rect_filled(rect, 0.0, color);
        
        // Handle dragging
        if response.dragged() {
            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let available_height = ui.available_height();
                self.timeline_height = (available_height - pointer_pos.y + rect.height() / 2.0)
                    .clamp(100.0, available_height - 200.0);
            }
        }
        
        // Change cursor on hover
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
    }
    
    fn show_context_menu(&mut self, ui: &mut egui::Ui, menu_state: &ContextMenuState, _stage_rect: egui::Rect) {
        // Create a window for the context menu
        egui::Window::new("context_menu")
            .fixed_pos(menu_state.position)
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                match &menu_state.menu_type {
                    ContextMenuType::Stage(stage_pos) => {
                        ui.label("Stage Context Menu");
                        ui.separator();
                        
                        if ui.button("➕ Add Rectangle").clicked() {
                            let new_item = StageItem {
                                id: format!("rect_{}", self.stage_items.len() + 1),
                                name: format!("Rectangle {}", self.stage_items.len() + 1),
                                item_type: StageItemType::Rectangle,
                                position: *stage_pos,
                                size: egui::Vec2::new(100.0, 60.0),
                                color: egui::Color32::from_rgb(150, 150, 255),
                                alpha: 1.0,
                                rotation: 0.0,
                                text_content: "Default Text".to_string(),
                                font_size: 16.0,
                                font_family: "Arial".to_string(),
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
                                position: *stage_pos,
                                size: egui::Vec2::splat(80.0),
                                color: egui::Color32::from_rgb(255, 150, 150),
                                alpha: 1.0,
                                rotation: 0.0,
                                text_content: "Default Text".to_string(),
                                font_size: 16.0,
                                font_family: "Arial".to_string(),
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
                                position: *stage_pos,
                                size: egui::Vec2::new(120.0, 30.0),
                                color: egui::Color32::WHITE,
                                alpha: 1.0,
                                rotation: 0.0,
                                text_content: "Default Text".to_string(),
                                font_size: 16.0,
                                font_family: "Arial".to_string(),
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
                                position: *stage_pos,
                                size: egui::Vec2::splat(100.0),
                                color: egui::Color32::from_rgb(150, 255, 150),
                                alpha: 1.0,
                                rotation: 0.0,
                                text_content: "Default Text".to_string(),
                                font_size: 16.0,
                                font_family: "Arial".to_string(),
                                };
                            self.stage_items.push(new_item.clone());
                            self.log(LogLevel::Action, format!("Added {} at ({:.1}, {:.1})", 
                                new_item.name, stage_pos.x, stage_pos.y));
                            self.context_menu = None;
                            }
                        
                        ui.separator();
                        
                        // Paste options
                        let can_paste = !self.clipboard.is_empty();
                        if ui.add_enabled(can_paste, egui::Button::new("📄 Paste in Place")).clicked() {
                            let clipboard_items = self.clipboard.clone();
                            for clipboard_item in clipboard_items {
                                let mut new_item = clipboard_item;
                                new_item.id = format!("{}_paste_{}", new_item.id, self.stage_items.len());
                                new_item.name = format!("{} (Copy)", new_item.name);
                                let name = new_item.name.clone();
                                self.stage_items.push(new_item);
                                self.log(LogLevel::Action, format!("Pasted {} in place", name));
                                }
                            self.context_menu = None;
                            }
                        
                        if ui.add_enabled(can_paste, egui::Button::new("📍 Paste at Position")).clicked() {
                            let clipboard_items = self.clipboard.clone();
                            for clipboard_item in clipboard_items {
                                let mut new_item = clipboard_item;
                                new_item.id = format!("{}_paste_{}", new_item.id, self.stage_items.len());
                                new_item.name = format!("{} (Copy)", new_item.name);
                                new_item.position = *stage_pos;
                                let name = new_item.name.clone();
                                self.stage_items.push(new_item);
                                self.log(LogLevel::Action, format!("Pasted {} at cursor", name));
                                }
                            self.context_menu = None;
                            }
                        
                        // Select All option
                        if ui.button("📋 Select All").clicked() {
                            self.selected_items = (0..self.stage_items.len()).collect();
                            self.log(LogLevel::Action, format!("Selected all {} items", self.stage_items.len()));
                            self.context_menu = None;
                            }
                        
                        ui.separator();
                        if ui.button("Cancel").clicked() {
                            self.context_menu = None;
                            }
                    },
                    ContextMenuType::StageItem(index) => {
                        if let Some(item) = self.stage_items.get(*index).cloned() {
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
                                new_item.position += egui::Vec2::splat(20.0);
                                let name = new_item.name.clone();
                                self.stage_items.push(new_item);
                                self.log(LogLevel::Action, format!("Duplicated {}", name));
                                self.context_menu = None;
                                }
                            
                            ui.separator();
                            
                            if ui.button("⬆️ Bring to Front").clicked() {
                                if *index < self.stage_items.len() - 1 {
                                    let item = self.stage_items.remove(*index);
                                    self.stage_items.push(item);
                                    let new_index = self.stage_items.len() - 1;
                                    self.selected_items.clear();
                                    self.selected_items.push(new_index);
                                    self.log(LogLevel::Action, "Brought to front");
                                    }
                                self.context_menu = None;
                                }
                            
                            if ui.button("⬇️ Send to Back").clicked() {
                                if *index > 0 {
                                    let item = self.stage_items.remove(*index);
                                    self.stage_items.insert(0, item);
                                    self.selected_items.clear();
                                    self.selected_items.push(0);
                                    self.log(LogLevel::Action, "Sent to back");
                                    }
                                self.context_menu = None;
                                }
                            
                            ui.separator();
                            
                            if ui.button("🔄 Rotate 45°").clicked() {
                                if let Some(item) = self.stage_items.get_mut(*index) {
                                    item.rotation = (item.rotation + 45.0) % 360.0;
                                    let rotation = item.rotation;
                                    self.log(LogLevel::Action, format!("Rotated to {}°", rotation));
                                    }
                                self.context_menu = None;
                                }
                            
                            ui.separator();
                            
                            if ui.button("🗑️ Delete").clicked() {
                                let removed = self.stage_items.remove(*index);
                                self.log(LogLevel::Action, format!("Deleted {}", removed.name));
                                self.selected_items.clear();
                                self.context_menu = None;
                                }
                            
                            ui.separator();
                            if ui.button("Cancel").clicked() {
                                self.context_menu = None;
                                }
                            }
                    },
                }
            });
        
        // Close context menu if clicked outside
        if ui.ctx().input(|i| i.pointer.any_click()) {
            if let Some(pos) = ui.ctx().pointer_interact_pos() {
                // Check if click is outside the menu area (rough estimate)
                if (pos - menu_state.position).length() > 200.0 {
                    self.context_menu = None;
                }
            }
        }
    }
    fn handle_tool_shortcuts(&mut self, ctx: &egui::Context) {
        // Check for single-key tool shortcuts
        ctx.input(|i| {
            // Only process shortcuts if no text field is focused
            if i.focused {
                return;
            }
            
            let mut new_tool = None;
            
            if i.key_pressed(egui::Key::V) {
                new_tool = Some(Tool::Arrow);
            } else if i.key_pressed(egui::Key::A) {
                new_tool = Some(Tool::Subselection);
            } else if i.key_pressed(egui::Key::L) {
                new_tool = Some(Tool::Lasso);
            } else if i.key_pressed(egui::Key::N) {
                new_tool = Some(Tool::Line);
            } else if i.key_pressed(egui::Key::P) {
                new_tool = Some(Tool::Pen);
            } else if i.key_pressed(egui::Key::Y) {
                new_tool = Some(Tool::Pencil);
            } else if i.key_pressed(egui::Key::B) {
                new_tool = Some(Tool::Brush);
            } else if i.key_pressed(egui::Key::R) {
                new_tool = Some(Tool::Rectangle);
            } else if i.key_pressed(egui::Key::O) {
                new_tool = Some(Tool::Oval);
            } else if i.key_pressed(egui::Key::T) {
                new_tool = Some(Tool::Text);
            } else if i.key_pressed(egui::Key::K) {
                new_tool = Some(Tool::PaintBucket);
            } else if i.key_pressed(egui::Key::S) {
                new_tool = Some(Tool::InkBottle);
            } else if i.key_pressed(egui::Key::I) {
                new_tool = Some(Tool::Eyedropper);
            } else if i.key_pressed(egui::Key::E) {
                new_tool = Some(Tool::Eraser);
            } else if i.key_pressed(egui::Key::Q) {
                new_tool = Some(Tool::FreeTransform);
            } else if i.key_pressed(egui::Key::F) {
                new_tool = Some(Tool::GradientTransform);
            } else if i.key_pressed(egui::Key::Z) {
                new_tool = Some(Tool::Zoom);
            } else if i.key_pressed(egui::Key::H) {
                new_tool = Some(Tool::Hand);
            }
            
            if let Some(tool) = new_tool {
                self.tool_state.active_tool = tool;
                self.log(LogLevel::Action, format!("Selected tool: {} (keyboard)", tool.get_name()));
            }
        });
    }
    
    fn show_crash_dialog(&self, ctx: &egui::Context) {
        // Check if a crash occurred
        if !CRASH_OCCURRED.load(Ordering::SeqCst) {
            return;
        }
        
        let crash_info = CRASH_INFO.lock().unwrap().clone();
        if let Some(info) = crash_info {
            egui::Window::new("Application Error")
                .resizable(false)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    // Error icon and title
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("❌").size(32.0));
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Timeline Demo has stopped working").size(16.0).strong());
                            ui.label("A problem caused the program to stop working correctly.");
                            });
                    });
                    
                    ui.separator();
                    
                    // Error details
                    ui.label(egui::RichText::new("Problem signature:").strong());
                    ui.monospace(format!("Error Type: Panic"));
                    ui.monospace(format!("Time: {}", info.timestamp));
                    ui.monospace(format!("Location: {}", info.location));
                    
                    ui.add_space(10.0);
                    
                    // Collapsible details
                    ui.collapsing("View problem details", |ui| {
                        ui.label(egui::RichText::new("Error Message:").strong());
                        ui.monospace(&info.message);
                        
                        ui.add_space(5.0);
                        
                        ui.label(egui::RichText::new("Stack Trace:").strong());
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                ui.monospace(&info.backtrace);
                                });
                    });
                    
                    ui.separator();
                    
                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("📋 Copy Error Details").clicked() {
                            let error_text = format!(
                                "Timeline Demo Error Report\n\
                                Time: {}\n\
                                Location: {}\n\
                                Message: {}\n\n\
                                Stack Trace:\n{}",
                                info.timestamp, info.location, info.message, info.backtrace
                            );
                            ui.ctx().copy_text(error_text);
                            }
                        
                        if ui.button("🔄 Restart Application").clicked() {
                            // Restart the application
                            if let Ok(exe) = std::env::current_exe() {
                                Command::new(exe).spawn().ok();
                                }
                            std::process::exit(1);
                            }
                        
                        if ui.button("❌ Close Program").clicked() {
                            std::process::exit(1);
                            }
                    });
                });
        }
    }
    
    fn draw_properties_panel(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(45));
            
            // Border
            let border_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(60));
            ui.painter().line_segment([rect.left_top(), rect.right_top()], border_stroke);
            ui.painter().line_segment([rect.right_top(), rect.right_bottom()], border_stroke);
            ui.painter().line_segment([rect.right_bottom(), rect.left_bottom()], border_stroke);
            ui.painter().line_segment([rect.left_bottom(), rect.left_top()], border_stroke);
            
            // Content with padding
            let padded_rect = rect.shrink(10.0);
            ui.scope_builder(UiBuilder::new().max_rect(padded_rect), |ui| {
                ui.vertical(|ui| {
                    ui.heading("🔧 Properties");
                    ui.separator();
                    
                    // Tab bar
                    ui.horizontal(|ui| {
                        if ui.selectable_label(
                            self.selected_property_tab == PropertyTab::Properties, 
                            "Properties"
                        ).clicked() {
                            self.selected_property_tab = PropertyTab::Properties;
                            }
                        
                        if ui.selectable_label(
                            self.selected_property_tab == PropertyTab::Filters, 
                            "Filters"
                        ).clicked() {
                            self.selected_property_tab = PropertyTab::Filters;
                            }
                        
                        if ui.selectable_label(
                            self.selected_property_tab == PropertyTab::ColorEffect, 
                            "Color Effect"
                        ).clicked() {
                            self.selected_property_tab = PropertyTab::ColorEffect;
                            }
                        
                        if ui.selectable_label(
                            self.selected_property_tab == PropertyTab::Display, 
                            "Display"
                        ).clicked() {
                            self.selected_property_tab = PropertyTab::Display;
                            }
                    });
                    
                    ui.separator();
                    
                    // Property content based on selected item
                    if let Some(&item_index) = self.selected_items.first() {
                        if let Some(item) = self.stage_items.get_mut(item_index) {
                            let item_name = item.name.clone();
                            
                            match self.selected_property_tab {
                                PropertyTab::Properties => {
                                    ui.label(format!("Selected: {}", item_name));
                                    ui.separator();
                                    
                                    // Position controls
                                    ui.label("Position:");
                                    ui.horizontal(|ui| {
                                        ui.label("X:");
                                        let old_x = item.position.x;
                                        if ui.add(egui::DragValue::new(&mut item.position.x).speed(1.0)).changed() {
                                            // Log later to avoid borrow conflict
                                            }
                                        ui.label("Y:");
                                        let old_y = item.position.y;
                                        if ui.add(egui::DragValue::new(&mut item.position.y).speed(1.0)).changed() {
                                            // Log later to avoid borrow conflict
                                            }
                                        });
                                    
                                    // Size controls
                                    ui.label("Size:");
                                    ui.horizontal(|ui| {
                                        ui.label("W:");
                                        let old_w = item.size.x;
                                        if ui.add(egui::DragValue::new(&mut item.size.x).speed(1.0).range(1.0..=500.0)).changed() {
                                            // Log later to avoid borrow conflict
                                            }
                                        ui.label("H:");
                                        let old_h = item.size.y;
                                        if ui.add(egui::DragValue::new(&mut item.size.y).speed(1.0).range(1.0..=500.0)).changed() {
                                            // Log later to avoid borrow conflict
                                            }
                                        });
                                    
                                    // Rotation control
                                    ui.label("Rotation:");
                                    let old_rotation = item.rotation;
                                    if ui.add(egui::DragValue::new(&mut item.rotation).speed(1.0).suffix("°")).changed() {
                                        item.rotation = item.rotation % 360.0;
                                        // Log later to avoid borrow conflict
                                        }
                                    
                                    // Color control
                                    ui.label("Color:");
                                    let mut color = [
                                        item.color.r() as f32 / 255.0,
                                        item.color.g() as f32 / 255.0,
                                        item.color.b() as f32 / 255.0,
                                    ];
                                    if ui.color_edit_button_rgb(&mut color).changed() {
                                        item.color = egui::Color32::from_rgb(
                                            (color[0] * 255.0) as u8,
                                            (color[1] * 255.0) as u8,
                                            (color[2] * 255.0) as u8,
                                        );
                                        // Log later to avoid borrow conflict
                                        }
                                    
                                    // Alpha control  
                                    ui.label("Alpha (Transparency):");
                                    ui.horizontal(|ui| {
                                        let mut alpha_percent = item.alpha * 100.0;
                                        if ui.add(egui::Slider::new(&mut alpha_percent, 0.0..=100.0).suffix("%")).changed() {
                                            item.alpha = alpha_percent / 100.0;
                                            // Log later to avoid borrow conflict
                                            }
                                        ui.label(format!("{:.0}%", alpha_percent));
                                        });
                                    
                                    // Text-specific properties for text items
                                    if item.item_type == StageItemType::Text {
                                        ui.separator();
                                        ui.label("Text Properties:");
                                        
                                        // Text content
                                        ui.label("Text Content:");
                                        if ui.text_edit_singleline(&mut item.text_content).changed() {
                                            // Log later to avoid borrow conflict
                                            }
                                        
                                        // Font size
                                        ui.label("Font Size:");
                                        if ui.add(egui::DragValue::new(&mut item.font_size).speed(1.0).range(8.0..=72.0).suffix("pt")).changed() {
                                            // Log later to avoid borrow conflict  
                                            }
                                        
                                        // Font family
                                        ui.label("Font Family:");
                                        ComboBox::from_label("")
                                            .selected_text(&item.font_family)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_value(&mut item.font_family, "Arial".to_string(), "Arial").clicked() {
                                                    // Font changed
                                                    }
                                                if ui.selectable_value(&mut item.font_family, "Times New Roman".to_string(), "Times New Roman").clicked() {
                                                    // Font changed
                                                    }
                                                if ui.selectable_value(&mut item.font_family, "Courier New".to_string(), "Courier New").clicked() {
                                                    // Font changed
                                                    }
                                                if ui.selectable_value(&mut item.font_family, "Helvetica".to_string(), "Helvetica").clicked() {
                                                    // Font changed
                                                    }
                                                });
                                        }
                                    
                                    // Item type info
                                    ui.separator();
                                    ui.label(format!("Type: {:?}", item.item_type));
                                    ui.label(format!("ID: {}", item.id));
                                    },
                                PropertyTab::Filters => {
                                    ui.label("🎨 Filters");
                                    ui.separator();
                                    ui.label("Drop Shadow");
                                    ui.checkbox(&mut false, "Enable Drop Shadow");
                                    ui.label("Blur");
                                    ui.checkbox(&mut false, "Enable Blur");
                                    ui.label("Glow");
                                    ui.checkbox(&mut false, "Enable Glow");
                                    ui.label("Bevel and Emboss");
                                    ui.checkbox(&mut false, "Enable Bevel");
                                    },
                                PropertyTab::ColorEffect => {
                                    ui.label("🌈 Color Effect");
                                    ui.separator();
                                    ui.label("Style:");
                                    ComboBox::from_label("")
                                        .selected_text("None")
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut 0, 0, "None");
                                            ui.selectable_value(&mut 0, 1, "Brightness");
                                            ui.selectable_value(&mut 0, 2, "Tint");
                                            ui.selectable_value(&mut 0, 3, "Alpha");
                                            ui.selectable_value(&mut 0, 4, "Advanced");
                                            });
                                    },
                                PropertyTab::Display => {
                                    ui.label("📺 Display");
                                    ui.separator();
                                    ui.label("Blend mode:");
                                    ComboBox::from_label("")
                                        .selected_text("Normal")
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut 0, 0, "Normal");
                                            ui.selectable_value(&mut 0, 1, "Multiply");
                                            ui.selectable_value(&mut 0, 2, "Screen");
                                            ui.selectable_value(&mut 0, 3, "Overlay");
                                            ui.selectable_value(&mut 0, 4, "Hard Light");
                                            });
                                    
                                    ui.checkbox(&mut true, "Visible");
                                    ui.checkbox(&mut false, "Cache as Bitmap");
                                    },
                                }
                            }
                    } else {
                        // No item selected
                        ui.label("No object selected");
                        ui.separator();
                        ui.label("Select an object on the stage to view and edit its properties.");
                    }
                });
            });
        });
    }
}

fn setup_panic_handler() {
    let default_panic = panic::take_hook();
    
    panic::set_hook(Box::new(move |panic_info| {
        // Extract panic information
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        
        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "Unknown location".to_string()
        };
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // Get backtrace
        let backtrace = std::backtrace::Backtrace::capture().to_string();
        
        // Store crash info
        if let Ok(mut info) = CRASH_INFO.lock() {
            *info = Some(CrashInfo {
                message,
                location,
                timestamp,
                backtrace,
            });
        }
        
        // Set crash flag
        CRASH_OCCURRED.store(true, Ordering::SeqCst);
        
        // Call the default panic handler (prints to stderr)
        default_panic(panic_info);
    }));
}

fn main() -> Result<(), eframe::Error> {
    // Set up panic handler
    setup_panic_handler();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Flash-Style Timeline Demo"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Timeline Demo",
        options,
        Box::new(|_cc| Ok(Box::new(TimelineApp::default()))),
    )
}