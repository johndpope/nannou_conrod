//! Standalone demo using egui_dock for panel management

use eframe::egui::{self, UiBuilder, ComboBox};
use nannou_timeline::{
    timeline_egui::Timeline, DockManager, FlashTabViewer,
    ui::MockRiveEngine, RiveEngine, LayerId,
    layer::LayerType,
    scripting::ScriptContext,
    CurveEditorPanel,
};
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono;

mod rustflash_integration;
use rustflash_integration::RustFlashIntegration;

// Import our helper modules
mod stage;
mod tools;
mod library;
mod properties;
mod logging;
mod script_templates;
mod drawing;
mod widgets;

use stage::{StageItem, StageItemType, ResizeHandle, MarqueeSelection, ContextMenuState};
use tools::{Tool, ToolState};
use library::{LibraryTab, LibraryAsset, LibraryAssetType, AssetProperties, LibraryContextMenuState};
use properties::PropertyTab;
use logging::{LogMessage, LogLevel};

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
        self.log(LogLevel::Action, format!("Set property {} = {} at frame {} on layer {:?}", property, value, frame, layer_id));
        self.inner.set_property(layer_id, frame, property, value)
    }
    
    fn get_property(&self, layer_id: LayerId, frame: u32, property: &str) -> bool {
        self.inner.get_property(layer_id, frame, property)
    }
    
    fn rename_layer(&mut self, layer_id: LayerId, new_name: String) {
        self.log(LogLevel::Action, format!("Rename layer {:?} to '{}'", layer_id, new_name));
        self.inner.rename_layer(layer_id, new_name)
    }
    
    fn add_layer(&mut self, name: String, layer_type: LayerType) -> LayerId {
        let id = self.inner.add_layer(name.clone(), layer_type);
        self.log(LogLevel::Action, format!("Add layer '{}' of type {:?}", name, layer_type));
        id
    }
    
    fn delete_layer(&mut self, layer_id: LayerId) {
        self.log(LogLevel::Action, format!("Delete layer {:?}", layer_id));
        self.inner.delete_layer(layer_id)
    }
    
    fn duplicate_layer(&mut self, layer_id: LayerId) -> LayerId {
        let layer_id_str = format!("{:?}", layer_id);
        let new_id = self.inner.duplicate_layer(layer_id);
        self.log(LogLevel::Action, format!("Duplicate layer {} -> {:?}", layer_id_str, new_id));
        new_id
    }
    
    fn add_folder_layer(&mut self, name: String) -> LayerId {
        let id = self.inner.add_folder_layer(name.clone());
        self.log(LogLevel::Action, format!("Add folder layer '{}'", name));
        id
    }
    
    fn add_motion_guide_layer(&mut self, name: String) -> LayerId {
        let id = self.inner.add_motion_guide_layer(name.clone());
        self.log(LogLevel::Action, format!("Add motion guide layer '{}'", name));
        id
    }
}

struct TimelineApp {
    timeline: Timeline,
    engine: Box<dyn RiveEngine>,
    dock_manager: DockManager,
    engine_logs: Arc<Mutex<Vec<(LogLevel, String)>>>,
    selected_language: String,
    selected_layer: Option<LayerId>,
    selected_frame: Option<u32>,
    
    // Stage items
    stage_items: Vec<StageItem>,
    selected_items: Vec<usize>,
    
    // Marquee selection state
    marquee_selection: Option<MarqueeSelection>,
    
    // Resize handle state
    active_resize_handle: Option<(usize, ResizeHandle)>,
    resize_start_size: Option<egui::Vec2>,
    resize_start_pos: egui::Pos2,
    
    // Context menu state
    context_menu: Option<ContextMenuState>,
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
    
    // Scene management
    scene_manager: nannou_timeline::SceneManager,
    scene_tab_state: nannou_timeline::SceneTabState,
    
    // Tools panel state
    tool_state: ToolState,
    tools_panel_width: f32,
    
    // Pen tool state
    pen_tool_points: Vec<egui::Pos2>,
    pen_tool_preview: Option<egui::Pos2>,
    
    // Script editor
    script_visible: bool,
    script_content: String,
    script_context: Option<ScriptContext>,
    script_error: Option<String>,
    script_panel_height: f32,
    
    // Curve editor
    curve_editor: CurveEditorPanel,
    
    // Console state
    console_visible: bool,
    console_height: f32,
    log_messages: Vec<LogMessage>,
    auto_scroll: bool,
}

// Global crash state
static CRASH_OCCURRED: AtomicBool = AtomicBool::new(false);
static CRASH_INFO: Mutex<Option<CrashInfo>> = Mutex::new(None);

#[derive(Clone)]
struct CrashInfo {
    message: String,
    location: String,
    timestamp: String,
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
                size: egui::Vec2::new(150.0, 100.0),
                color: egui::Color32::from_rgb(100, 150, 255),
                alpha: 1.0,
                rotation: 0.0,
                text_content: "Default Text".to_string(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
                path_points: Vec::new(),
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
                path_points: Vec::new(),
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
                text_content: "Hello World!".to_string(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
                path_points: Vec::new(),
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
                path_points: Vec::new(),
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
            engine: Box::new(LoggingRiveEngine::new(engine_logs.clone())),
            dock_manager: DockManager::new(),
            engine_logs,
            selected_language: "en".to_string(),
            selected_layer: None,
            selected_frame: None,
            
            stage_items,
            selected_items: Vec::new(),
            marquee_selection: None,
            active_resize_handle: None,
            resize_start_size: None,
            resize_start_pos: egui::Pos2::ZERO,
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
            
            // Initialize scene management
            scene_manager: nannou_timeline::SceneManager::new(),
            scene_tab_state: nannou_timeline::SceneTabState::default(),
            
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
            tools_panel_width: 48.0,
            pen_tool_points: Vec::new(),
            pen_tool_preview: None,
            
            // Script editor
            script_visible: false,
            script_content: script_templates::LOOP_ANIMATION.to_string(),
            script_context: None,
            script_error: None,
            script_panel_height: 200.0,
            
            // Curve editor
            curve_editor: CurveEditorPanel::default(),
            
            console_visible: false,
            console_height: 150.0,
            log_messages: Vec::new(),
            auto_scroll: true,
        };
        
        app.log(LogLevel::Info, "Timeline dock application started".to_string());
        app.log(LogLevel::Info, "🎮 Keyboard shortcuts:".to_string());
        app.log(LogLevel::Info, "  • F12: Toggle debug console".to_string());
        app.log(LogLevel::Info, "  • F2: Take screenshot".to_string());
        app.log(LogLevel::Info, "  • F9: Toggle script editor".to_string());
        app.log(LogLevel::Info, "  • F10: Toggle curve editor".to_string());
        app.log(LogLevel::Info, "💡 Hover over timeline elements to see tooltips".to_string());
        app.log(LogLevel::Info, "💡 Right-click on layers and frames for context menus".to_string());
        app.log(LogLevel::Info, "💡 Click and drag stage items to move them".to_string());
        app.log(LogLevel::Info, "💡 Right-click stage items for context menu".to_string());
        app
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
                id: "texture_png".to_string(),
                name: "Texture".to_string(),
                asset_type: LibraryAssetType::Bitmap,
                folder: "Bitmaps".to_string(),
                properties: AssetProperties {
                    file_size: Some(81920),
                    dimensions: Some((512, 512)),
                    format: Some("PNG".to_string()),
                    usage_count: 3,
                    linkage_class: None,
                },
            },
            // Sounds folder
            LibraryAsset {
                id: "music_mp3".to_string(),
                name: "Background Music".to_string(),
                asset_type: LibraryAssetType::Sound,
                folder: "Sounds".to_string(),
                properties: AssetProperties {
                    file_size: Some(3145728),
                    dimensions: None,
                    format: Some("MP3".to_string()),
                    usage_count: 1,
                    linkage_class: None,
                },
            },
            LibraryAsset {
                id: "click_wav".to_string(),
                name: "Click Sound".to_string(),
                asset_type: LibraryAssetType::Sound,
                folder: "Sounds".to_string(),
                properties: AssetProperties {
                    file_size: Some(32768),
                    dimensions: None,
                    format: Some("WAV".to_string()),
                    usage_count: 8,
                    linkage_class: Some("ClickSound".to_string()),
                },
            },
        ]
    }
    
    fn handle_dropped_file(&mut self, path: &std::path::Path) {
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");
        
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        // Determine asset type based on file extension
        let (asset_type, mime_type) = match extension.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" => (LibraryAssetType::Bitmap, "image/*"),
            "mp3" | "wav" | "ogg" | "aac" => (LibraryAssetType::Sound, "audio/*"),
            "mp4" | "mov" | "avi" | "webm" => (LibraryAssetType::Video, "video/*"),
            "ttf" | "otf" | "woff" | "woff2" => (LibraryAssetType::Font, "font/*"),
            _ => {
                self.log(LogLevel::Warning, format!("Unsupported file type: .{}", extension));
                return;
            }
        };
        
        // Create a new library asset
        let asset_id = format!("asset_{}", self.library_assets.len() + 1);
        let file_size = path.metadata().ok().map(|m| m.len()).unwrap_or(0);
        
        let new_asset = match asset_type {
            LibraryAssetType::Bitmap => {
                LibraryAsset::new_bitmap(
                    asset_id.clone(),
                    filename.to_string(),
                    "Imported".to_string(),
                    (100, 100), // Placeholder dimensions
                    file_size
                )
            }
            _ => {
                LibraryAsset {
                    id: asset_id.clone(),
                    name: filename.to_string(),
                    asset_type,
                    folder: "Imported".to_string(),
                    properties: AssetProperties {
                        file_size: Some(file_size),
                        dimensions: None,
                        format: Some(extension.to_uppercase()),
                        usage_count: 0,
                        linkage_class: None,
                    },
                }
            }
        };
        
        self.library_assets.push(new_asset);
        self.log(LogLevel::Action, format!("Imported {} as {} asset", filename, mime_type));
        
        // Optionally, auto-create an instance on the stage
        if asset_type == LibraryAssetType::Bitmap {
            if let Some(asset) = self.library_assets.iter().find(|a| a.id == asset_id).cloned() {
                let stage_center = egui::Pos2::new(400.0, 300.0); // Center of typical stage
                self.create_stage_instance_from_asset(&asset, stage_center);
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
                    path_points: Vec::new(),
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
                    path_points: Vec::new(),
                }
            }
            _ => return, // Don't create instances for other types
        };
        
        self.stage_items.push(new_item.clone());
        self.log(LogLevel::Action, format!("Created {} from library at ({:.1}, {:.1})", 
            new_item.name, position.x, position.y));
    }
    
    fn take_screenshot(&mut self, _ctx: &egui::Context) {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("timeline_demo_screenshot_{}.png", timestamp);
        
        let downloads_path = dirs::download_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        let filepath = downloads_path.join(&filename);
        
        let result = Command::new("screencapture")
            .arg("-iW")
            .arg(&filepath)
            .spawn();
        
        match result {
            Ok(mut child) => {
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
                
                self.log(LogLevel::Info, "📸 Click on the Timeline Demo window to capture screenshot (F2)".to_string());
            }
            Err(e) => {
                self.log(LogLevel::Action, format!("Failed to start screenshot: {}", e));
            }
        }
    }
    
    fn log(&mut self, level: LogLevel, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.log_messages.push(LogMessage {
            timestamp,
            level,
            message,
        });
        
        // Keep log size reasonable
        if self.log_messages.len() > 1000 {
            self.log_messages.drain(0..100);
        }
    }
    
    fn handle_scene_events(&mut self, events: Vec<nannou_timeline::SceneTabEvent>) {
        for event in events {
            match event {
                nannou_timeline::SceneTabEvent::SwitchToScene(scene_id) => {
                    if let Err(e) = self.scene_manager.switch_to_scene(&scene_id) {
                        self.log(LogLevel::Error, format!("Failed to switch to scene: {}", e));
                    } else {
                        self.log(LogLevel::Info, format!("Switched to scene: {}", scene_id));
                    }
                }
                nannou_timeline::SceneTabEvent::AddScene => {
                    let scene_count = self.scene_manager.scene_count();
                    let scene_name = format!("Scene {}", scene_count + 1);
                    let scene_id = self.scene_manager.create_scene(&scene_name);
                    self.log(LogLevel::Info, format!("Created new scene: {} ({})", scene_name, scene_id));
                }
                nannou_timeline::SceneTabEvent::CloseScene(scene_id) => {
                    if let Err(e) = self.scene_manager.remove_scene(&scene_id) {
                        self.log(LogLevel::Error, format!("Failed to remove scene: {}", e));
                    } else {
                        self.log(LogLevel::Info, format!("Removed scene: {}", scene_id));
                    }
                }
                nannou_timeline::SceneTabEvent::RenameScene(scene_id) => {
                    // This is handled by the scene tabs widget directly
                    self.log(LogLevel::Info, format!("Started renaming scene: {}", scene_id));
                }
                nannou_timeline::SceneTabEvent::ConfirmRename(scene_id, new_name) => {
                    if let Err(e) = self.scene_manager.rename_scene(&scene_id, &new_name) {
                        self.log(LogLevel::Error, format!("Failed to rename scene: {}", e));
                    } else {
                        self.log(LogLevel::Info, format!("Renamed scene {} to: {}", scene_id, new_name));
                    }
                }
                nannou_timeline::SceneTabEvent::CancelRename => {
                    self.log(LogLevel::Info, "Cancelled scene rename".to_string());
                }
                nannou_timeline::SceneTabEvent::ContextMenu(scene_id, _pos) => {
                    self.log(LogLevel::Info, format!("Opened context menu for scene: {}", scene_id));
                }
            }
        }
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
        
        // Handle dropped files
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    self.handle_dropped_file(path);
                }
            }
        });
        
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
        }
        
        // Handle F10 to toggle curve editor
        if ctx.input(|i| i.key_pressed(egui::Key::F10)) {
            self.curve_editor.open = !self.curve_editor.open;
            self.log(LogLevel::Info, format!("Curve editor {}", if self.curve_editor.open { "opened" } else { "closed" }));
        }
        
        // Handle scene navigation shortcuts
        let navigation = nannou_timeline::SceneNavigation::new(&self.scene_manager);
        let navigation_events = navigation.handle_shortcuts(ctx);
        self.handle_scene_events(navigation_events);
        
        // Show console if visible
        if self.console_visible {
            egui::TopBottomPanel::bottom("console")
                .resizable(true)
                .default_height(self.console_height)
                .show(ctx, |ui| {
                    ui.heading("🖥️ Developer Console");
                    
                    ui.horizontal(|ui| {
                        if ui.button("Clear").clicked() {
                            self.log_messages.clear();
                            self.log(LogLevel::Info, "Console cleared".to_string());
                        }
                        ui.separator();
                        ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                        ui.separator();
                        ui.label(format!("{} messages", self.log_messages.len()));
                        
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
                                        self.log(LogLevel::Info, "Language changed to English".to_string());
                                    }
                                    if ui.selectable_value(&mut self.selected_language, "es".to_string(), "Español").clicked() {
                                        self.timeline.i18n.set_language("es");
                                        self.log(LogLevel::Info, "Idioma cambiado a Español".to_string());
                                    }
                                    if ui.selectable_value(&mut self.selected_language, "ja".to_string(), "日本語").clicked() {
                                        self.timeline.i18n.set_language("ja");
                                        self.log(LogLevel::Info, "言語を日本語に変更しました".to_string());
                                    }
                                    if ui.selectable_value(&mut self.selected_language, "zh".to_string(), "中文").clicked() {
                                        self.timeline.i18n.set_language("zh");
                                        self.log(LogLevel::Info, "语言已更改为中文".to_string());
                                    }
                                });
                        });
                    });
                    
                    ui.separator();
                    
                    // Console content
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
        }
        
        // Show scene tabs at the top
        let scene_events = egui::TopBottomPanel::top("scene_tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Scenes:");
                ui.separator();
                
                let scene_tabs = nannou_timeline::SceneTabs::new(&self.scene_manager, &mut self.scene_tab_state);
                scene_tabs.show(ui)
            }).inner
        }).inner;
        
        // Handle scene events
        self.handle_scene_events(scene_events);
        
        // Create tab viewer
        let mut tab_viewer = FlashTabViewer {
            timeline: &mut self.timeline,
            engine: &mut self.engine,
            selected_layer: self.selected_layer.clone(),
            selected_frame: self.selected_frame,
        };
        
        // Show dock manager
        self.dock_manager.show(ctx, &mut tab_viewer);
        
        // Update selection from tab viewer
        self.selected_layer = tab_viewer.selected_layer;
        self.selected_frame = tab_viewer.selected_frame;
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Flash-Style Timeline Demo (egui_dock)"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Timeline Demo",
        options,
        Box::new(|cc| {
            // Initialize material icons
            egui_material_icons::initialize(&cc.egui_ctx);
            Ok(Box::new(TimelineApp::default()))
        }),
    )
}