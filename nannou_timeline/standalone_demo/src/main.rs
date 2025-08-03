//! Standalone demo of the Flash-inspired timeline widget

use eframe::egui::{self, UiBuilder, ComboBox};
use nannou_timeline::{
    timeline_egui::Timeline,
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
use rfd::FileDialog;
use serde_json;

mod rustflash_integration;
mod artboard_renderer;
#[cfg(test)]
mod tests;
use rustflash_integration::RustFlashIntegration;
use artboard_renderer::{ArtboardRenderer, rustflash_types};

// Import our helper modules
mod stage;
mod tools;
mod library;
mod properties;
mod logging;
mod script_templates;
// mod drawing; // Commented out to avoid duplicate draw_stage method
mod widgets;

use stage::{StageItem, StageItemType, ResizeHandle, MarqueeSelection, ContextMenuState, ContextMenuType};
use tools::{Tool, ToolState};
use library::{LibraryTab, LibraryAsset, LibraryAssetType, AssetProperties, LibraryContextMenuState, LibraryContextTarget};
use properties::PropertyTab;
use logging::{LogMessage, LogLevel};

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
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
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
    // Marquee selection state
    marquee_selection: Option<MarqueeSelection>,
    // Resize handle state
    active_resize_handle: Option<(usize, ResizeHandle)>,
    resize_start_size: Option<egui::Vec2>,
    resize_start_pos: egui::Pos2,
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
    // Asset properties dialog
    show_asset_properties_dialog: Option<String>,
    // Tools panel state
    tool_state: ToolState,
    tools_panel_width: f32,
    // Pen tool state
    pen_tool_points: Vec<egui::Pos2>,
    pen_tool_preview: Option<egui::Pos2>,
    // Script editor state
    script_visible: bool,
    script_content: String,
    script_context: Option<ScriptContext>,
    script_error: Option<String>,
    script_panel_height: f32,
    // Curve editor state
    curve_editor: CurveEditorPanel,
    // Artboard renderer for displaying RustFlash content
    artboard_renderer: Option<ArtboardRenderer>,
}

// These types are now imported from our modules

// LogMessage and LogLevel are imported from logging module
// LibraryTab is imported from library module

// Tool enum and ToolState are imported from tools module

// LibraryAsset, LibraryAssetType, AssetProperties, and related types are imported from library module

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
                text_content: "Default Text".to_string(),
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
            // Use RustFlash integration engine
            engine: Box::new(RustFlashIntegration::new()),
            timeline_height: 250.0, // Increased height for timeline
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
            show_asset_properties_dialog: None,
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
            // Pen tool state
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
            // Artboard renderer
            artboard_renderer: None,
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
        
        // Handle dropped files
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    self.handle_dropped_file(path);
                }
            }
        });
        
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
            
            // Calculate timeline position more carefully
            let timeline_y = available_rect.max.y - self.timeline_height - bottom_panels_height;
            let timeline_rect = egui::Rect::from_min_size(
                egui::pos2(available_rect.min.x, timeline_y),
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
            
            // Draw all panels in order, ensuring proper clipping
            
            // 1. Draw timeline FIRST (bottom) to ensure it's not overlapped
            ui.painter().rect_filled(
                timeline_rect,
                0.0,
                egui::Color32::from_gray(30), // Dark background
            );
            
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
            
            // 2. Then draw other panels on top
            self.draw_tools_panel(ui, tools_rect);
            self.draw_stage(ui, stage_rect);
            self.draw_library(ui, library_rect);
            self.draw_properties_panel(ui, properties_rect);
            
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
                                self.script_content = script_templates::LOOP_ANIMATION.to_string();
                                self.log(LogLevel::Info, "Loaded loop animation template");
                                }
                            if ui.selectable_label(false, "Stop at Frame").clicked() {
                                self.script_content = script_templates::STOP_AT_FRAME.to_string();
                                self.log(LogLevel::Info, "Loaded stop at frame template");
                                }
                            if ui.selectable_label(false, "Animate Object").clicked() {
                                self.script_content = script_templates::ANIMATE_OBJECT.to_string();
                                self.log(LogLevel::Info, "Loaded animate object template");
                                }
                            if ui.selectable_label(false, "Create Object").clicked() {
                                self.script_content = script_templates::CREATE_OBJECT.to_string();
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
                            ui.menu_button("⚙", |ui| {
                                if ui.button("📤 Export Library").clicked() {
                                    self.export_library();
                                    ui.close_menu();
                                }
                                if ui.button("📥 Import Library").clicked() {
                                    self.import_library();
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("🔄 Refresh").clicked() {
                                    self.log(LogLevel::Action, "Library refreshed");
                                    ui.close_menu();
                                }
                                if ui.button("🗑️ Clear All").clicked() {
                                    self.library_assets.clear();
                                    self.log(LogLevel::Action, "Library cleared");
                                    ui.close_menu();
                                }
                            });
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
        
        // Handle drag and drop - handled in stage drawing
        
        // Handle context menu
        if let Some(menu_state) = &self.library_context_menu.clone() {
            self.show_library_context_menu(ui, menu_state);
        }
        
        // Handle asset properties dialog
        if let Some(asset_id) = &self.show_asset_properties_dialog.clone() {
            self.show_asset_properties_dialog(ui, asset_id);
        }
    }
    
    fn draw_library_assets_tab(&mut self, ui: &mut egui::Ui) {
        // Drop zone indicator
        let drop_zone_response = ui.allocate_response(
            egui::Vec2::new(ui.available_width(), 30.0),
            egui::Sense::hover()
        );
        
        let is_hovering_with_files = ui.input(|i| !i.raw.hovered_files.is_empty());
        
        if is_hovering_with_files {
            ui.painter().rect_filled(
                drop_zone_response.rect,
                4.0,
                egui::Color32::from_rgba_premultiplied(100, 150, 255, 50)
            );
            ui.painter().text(
                drop_zone_response.rect.center(),
                egui::Align2::CENTER_CENTER,
                "📁 Drop files here to import",
                egui::FontId::default(),
                ui.style().visuals.text_color()
            );
        } else {
            ui.painter().text(
                drop_zone_response.rect.center(),
                egui::Align2::CENTER_CENTER,
                "📁 Drag & drop files to import",
                egui::FontId::default(),
                ui.style().visuals.weak_text_color()
            );
        }
        
        ui.separator();
        
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
                self.import_assets();
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
                // In a real implementation, we'd read the image to get dimensions
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
                        if ui.button("📋 Properties").clicked() {
                            self.log(LogLevel::Action, format!("Show properties for asset: {}", asset_name));
                            self.show_asset_properties_dialog = Some(asset_id.clone());
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
                            self.import_assets_to_folder(folder_name.clone());
                            self.library_context_menu = None;
                            }
                    }
                    LibraryContextTarget::Background => {
                        if ui.button("📁 New Folder").clicked() {
                            self.log(LogLevel::Action, "New root folder");
                            self.library_context_menu = None;
                            }
                        if ui.button("➕ Import Assets").clicked() {
                            self.import_assets();
                            self.library_context_menu = None;
                            }
                    }
                }
            });
    }
    
    fn show_asset_properties_dialog(&mut self, ui: &mut egui::Ui, asset_id: &str) {
        let mut should_close = false;
        
        // Find the asset
        let asset = self.library_assets.iter().find(|a| a.id == asset_id).cloned();
        
        if let Some(asset) = asset {
            egui::Window::new("Asset Properties")
                .collapsible(false)
                .resizable(true)
                .default_width(400.0)
                .default_height(500.0)
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        // Asset icon and name header
                        ui.horizontal(|ui| {
                            let icon = match asset.asset_type {
                                LibraryAssetType::MovieClip => "🎬",
                                LibraryAssetType::Button => "🔘",
                                LibraryAssetType::Graphic => "🖼️",
                                LibraryAssetType::Bitmap => "📷",
                                LibraryAssetType::Sound => "🔊",
                                LibraryAssetType::Video => "📹",
                                LibraryAssetType::Font => "🔤",
                                LibraryAssetType::Folder => "📁",
                            };
                            ui.label(egui::RichText::new(icon).size(24.0));
                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&asset.name).heading());
                                ui.label(format!("Type: {:?}", asset.asset_type));
                            });
                        });
                        
                        ui.separator();
                        
                        // Properties grid
                        egui::Grid::new("asset_properties_grid")
                            .num_columns(2)
                            .spacing([10.0, 8.0])
                            .show(ui, |ui| {
                                // Basic properties
                                ui.label("Asset ID:");
                                ui.label(&asset.id);
                                ui.end_row();
                                
                                ui.label("Folder:");
                                ui.label(&asset.folder);
                                ui.end_row();
                                
                                // Type-specific properties
                                if let Some(file_size) = asset.properties.file_size {
                                    ui.label("File Size:");
                                    ui.label(format!("{} bytes", file_size));
                                    ui.end_row();
                                }
                                
                                if let Some((w, h)) = asset.properties.dimensions {
                                    ui.label("Dimensions:");
                                    ui.label(format!("{} × {} pixels", w, h));
                                    ui.end_row();
                                }
                                
                                if let Some(format) = &asset.properties.format {
                                    ui.label("Format:");
                                    ui.label(format);
                                    ui.end_row();
                                }
                                
                                ui.label("Usage Count:");
                                ui.label(format!("{}", asset.properties.usage_count));
                                ui.end_row();
                                
                                if let Some(linkage) = &asset.properties.linkage_class {
                                    ui.label("Linkage Class:");
                                    ui.label(linkage);
                                    ui.end_row();
                                }
                            });
                        
                        ui.separator();
                        
                        // ActionScript 3 Linkage section (Flash-style)
                        ui.collapsing("ActionScript 3.0 Linkage", |ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut false, "Export for ActionScript");
                                ui.label("Enable this asset for ActionScript access");
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Class:");
                                ui.text_edit_singleline(&mut String::new());
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Base class:");
                                ui.text_edit_singleline(&mut String::new());
                            });
                            
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut false, "Export in frame 1");
                                ui.label("Make available in first frame");
                            });
                        });
                        
                        ui.separator();
                        
                        // Advanced properties
                        ui.collapsing("Advanced", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Compression:");
                                egui::ComboBox::from_label("")
                                    .selected_text("Lossless")
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut 0, 0, "Photo (JPEG)");
                                        ui.selectable_value(&mut 0, 1, "Lossless (PNG/GIF)");
                                        ui.selectable_value(&mut 0, 2, "Custom");
                                    });
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Allow smoothing:");
                                ui.checkbox(&mut true, "");
                            });
                        });
                        
                        ui.add_space(10.0);
                        
                        // Action buttons
                        ui.horizontal(|ui| {
                            if ui.button("OK").clicked() {
                                self.log(LogLevel::Action, format!("Asset properties updated: {}", asset.name));
                                should_close = true;
                            }
                            
                            if ui.button("Cancel").clicked() {
                                should_close = true;
                            }
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Apply").clicked() {
                                    self.log(LogLevel::Action, format!("Asset properties applied: {}", asset.name));
                                }
                            });
                        });
                    });
                });
        } else {
            // Asset not found, close dialog
            should_close = true;
        }
        
        if should_close {
            self.show_asset_properties_dialog = None;
        }
    }
    
    /// Import assets using file dialog
    fn import_assets(&mut self) {
        self.import_assets_to_folder("Graphics".to_string());
    }
    
    /// Import assets to a specific folder
    fn import_assets_to_folder(&mut self, folder: String) {
        // Spawn file dialog in a thread to avoid blocking UI
        let files = FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp"])
            .add_filter("Audio", &["mp3", "wav", "ogg", "m4a", "aac"])
            .add_filter("Video", &["mp4", "mov", "avi", "webm", "flv"])
            .add_filter("Flash Assets", &["swf", "fla"])
            .add_filter("All Files", &["*"])
            .set_title("Import Assets to Library")
            .pick_files();
            
        if let Some(paths) = files {
            for path in paths {
                if let Err(e) = self.import_single_asset(&path, &folder) {
                    self.log(LogLevel::Error, format!("Failed to import {:?}: {}", path, e));
                } else {
                    self.log(LogLevel::Action, format!("Successfully imported: {:?}", path.file_name().unwrap_or_default()));
                }
            }
        }
    }
    
    /// Import a single asset file
    fn import_single_asset(&mut self, path: &std::path::Path, folder: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .ok_or("Invalid file name")?;
            
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
            
        // Determine asset type from extension
        let asset_type = match extension.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "webp" => LibraryAssetType::Bitmap,
            "mp3" | "wav" | "ogg" | "m4a" | "aac" => LibraryAssetType::Sound,
            "mp4" | "mov" | "avi" | "webm" | "flv" => LibraryAssetType::Video,
            "swf" => LibraryAssetType::MovieClip,
            "ttf" | "otf" | "woff" | "woff2" => LibraryAssetType::Font,
            _ => LibraryAssetType::Graphic,
        };
        
        // Get file metadata
        let metadata = std::fs::metadata(path)?;
        let file_size = metadata.len();
        
        // For images, try to get dimensions
        let dimensions = if matches!(asset_type, LibraryAssetType::Bitmap) {
            // Simple dimension detection - would need image crate for real implementation
            Some((800, 600)) // Placeholder dimensions
        } else {
            None
        };
        
        // Create asset ID
        let asset_id = uuid::Uuid::new_v4().to_string();
        
        // Create the asset
        let asset = LibraryAsset {
            id: asset_id,
            name: file_name.to_string(),
            asset_type,
            folder: folder.to_string(),
            properties: AssetProperties {
                file_size: Some(file_size),
                dimensions,
                format: Some(extension.to_uppercase()),
                usage_count: 0,
                linkage_class: None,
            },
        };
        
        // Add to library
        self.library_assets.push(asset);
        
        Ok(())
    }
    
    /// Export library assets to JSON file
    fn export_library(&mut self) {
        let file_path = FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name("library_assets.json")
            .set_title("Export Library Assets")
            .save_file();
            
        if let Some(path) = file_path {
            match self.export_assets_to_file(&path) {
                Ok(()) => {
                    self.log(LogLevel::Action, format!("Library exported to: {:?}", path));
                }
                Err(e) => {
                    self.log(LogLevel::Error, format!("Failed to export library: {}", e));
                }
            }
        }
    }
    
    /// Export assets to a JSON file
    fn export_assets_to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.library_assets)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Import library assets from JSON file
    fn import_library(&mut self) {
        let file_path = FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_title("Import Library Assets")
            .pick_file();
            
        if let Some(path) = file_path {
            match self.import_assets_from_file(&path) {
                Ok(count) => {
                    self.log(LogLevel::Action, format!("Imported {} assets from: {:?}", count, path));
                }
                Err(e) => {
                    self.log(LogLevel::Error, format!("Failed to import library: {}", e));
                }
            }
        }
    }
    
    /// Import assets from a JSON file
    fn import_assets_from_file(&mut self, path: &std::path::Path) -> Result<usize, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let assets: Vec<LibraryAsset> = serde_json::from_str(&content)?;
        let count = assets.len();
        
        // Add imported assets to library (avoiding duplicates by ID)
        for asset in assets {
            if !self.library_assets.iter().any(|a| a.id == asset.id) {
                self.library_assets.push(asset);
            }
        }
        
        Ok(count)
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
                let bottom_panels_height = if self.console_visible { 120.0 } else { 0.0 }; // Developer console height
                let min_stage_height = 100.0; // Reduced minimum height for stage
                let min_timeline_height = 100.0; // Reduced minimum height for timeline
                
                // Calculate the space available for both stage and timeline
                let available_for_both = available_height - bottom_panels_height - self.properties_height - self.splitter_thickness;
                
                // Ensure we have enough space for minimum sizes
                if available_for_both >= min_stage_height + min_timeline_height {
                    // Calculate new timeline height based on drag position
                    let new_timeline_height = available_height - pointer_pos.y + rect.height() / 2.0;
                    
                    // Calculate what the stage height would be
                    let stage_height = available_for_both - new_timeline_height;
                    
                    // Only update if both components meet minimum requirements
                    if stage_height >= min_stage_height && new_timeline_height >= min_timeline_height {
                        self.timeline_height = new_timeline_height;
                    }
                }
            }
        }
        
        // Change cursor on hover
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
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
                                        let _old_x = item.position.x;
                                        if ui.add(egui::DragValue::new(&mut item.position.x).speed(1.0)).changed() {
                                            // Log later to avoid borrow conflict
                                            }
                                        ui.label("Y:");
                                        let _old_y = item.position.y;
                                        if ui.add(egui::DragValue::new(&mut item.position.y).speed(1.0)).changed() {
                                            // Log later to avoid borrow conflict
                                            }
                                        });
                                    
                                    // Size controls
                                    ui.label("Size:");
                                    ui.horizontal(|ui| {
                                        ui.label("W:");
                                        let _old_w = item.size.x;
                                        if ui.add(egui::DragValue::new(&mut item.size.x).speed(1.0).range(1.0..=500.0)).changed() {
                                            // Log later to avoid borrow conflict
                                            }
                                        ui.label("H:");
                                        let _old_h = item.size.y;
                                        if ui.add(egui::DragValue::new(&mut item.size.y).speed(1.0).range(1.0..=500.0)).changed() {
                                            // Log later to avoid borrow conflict
                                            }
                                        });
                                    
                                    // Rotation control
                                    ui.label("Rotation:");
                                    let _old_rotation = item.rotation;
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
    
    /// Draw the stage canvas with artboard rendering
    fn draw_stage(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(30));
            
            // Border
            let border_stroke = egui::Stroke::new(1.0, egui::Color32::DARK_GRAY);
            ui.painter().rect_stroke(rect, 0.0, border_stroke, egui::epaint::StrokeKind::Outside);
            
            // Stage canvas area (leave some padding for controls)
            let canvas_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(10.0, 30.0),
                egui::vec2(rect.width() - 20.0, rect.height() - 40.0)
            );
            
            // Initialize artboard renderer if needed
            if self.artboard_renderer.is_none() {
                self.artboard_renderer = Some(ArtboardRenderer::new().with_debug(true));
            }
            
            // Try to get rendered artboard from RustFlash engine
            let has_artboard = if let Some(rustflash_integration) = self.engine.as_any_mut().downcast_mut::<RustFlashIntegration>() {
                match rustflash_integration.get_renderer_artboard() {
                    Ok(artboard) => {
                        // Render the actual artboard content
                        if let Some(renderer) = &self.artboard_renderer {
                            renderer.render_artboard(ui.painter(), &artboard, canvas_rect);
                        }
                        true
                    }
                    Err(_) => false
                }
            } else {
                false
            };
            
            // Fallback to test pattern if no artboard available
            if !has_artboard {
                if let Some(renderer) = &self.artboard_renderer {
                    let current_frame = self.engine.get_current_frame();
                    renderer.render_test_pattern(ui.painter(), canvas_rect, current_frame);
                }
            }
            
            // Draw stage items (existing demo content)
            self.draw_stage_items(ui, canvas_rect);
            
            // Stage controls header
            let header_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(10.0, 5.0),
                egui::vec2(rect.width() - 20.0, 20.0)
            );
            
            ui.scope_builder(UiBuilder::new().max_rect(header_rect), |ui| {
                ui.horizontal(|ui| {
                    ui.label("🎬 Stage");
                    ui.separator();
                    
                    // Frame info
                    let current_frame = self.engine.get_current_frame();
                    let total_frames = self.engine.get_total_frames();
                    let fps = self.engine.get_fps();
                    
                    ui.label(format!("Frame: {}/{} • {:.1} FPS", current_frame, total_frames, fps));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("🔄 Refresh").clicked() {
                            // Force re-render of artboard
                            if let Some(rustflash_integration) = self.engine.as_any_mut().downcast_mut::<RustFlashIntegration>() {
                                rustflash_integration.mark_dirty();
                            }
                        }
                    });
                });
            });
        });
    }
    
    /// Draw the stage items (existing demo content)
    fn draw_stage_items(&mut self, ui: &mut egui::Ui, canvas_rect: egui::Rect) {
        // This is the existing stage item rendering logic
        // Draw all stage items
        for (index, item) in self.stage_items.iter().enumerate() {
            let is_selected = self.selected_items.contains(&index);
            self.draw_stage_item(ui, item, is_selected, canvas_rect);
        }
        
        // Handle stage interactions (clicking, dragging, etc.)
        self.handle_stage_interactions(ui, canvas_rect);
    }
    
    /// Draw a single stage item
    fn draw_stage_item(&self, ui: &mut egui::Ui, item: &StageItem, is_selected: bool, canvas_rect: egui::Rect) {
        let item_rect = egui::Rect::from_min_size(
            canvas_rect.min + item.position.to_vec2(),
            item.size
        );
        
        // Apply alpha to color
        let item_color = if item.alpha < 1.0 {
            let [r, g, b, _a] = item.color.to_array();
            egui::Color32::from_rgba_unmultiplied(r, g, b, (item.alpha * 255.0) as u8)
        } else {
            item.color
        };
        
        // Draw the item based on type
        match item.item_type {
            StageItemType::Rectangle => {
                ui.painter().rect_filled(item_rect, 5.0, item_color);
                if is_selected {
                    ui.painter().rect_stroke(item_rect, 5.0, egui::Stroke::new(2.0, egui::Color32::YELLOW), egui::epaint::StrokeKind::Outside);
                }
            }
            StageItemType::Circle => {
                let center = item_rect.center();
                let radius = item.size.x.min(item.size.y) / 2.0;
                ui.painter().circle_filled(center, radius, item_color);
                if is_selected {
                    ui.painter().circle_stroke(center, radius, egui::Stroke::new(2.0, egui::Color32::YELLOW));
                }
            }
            StageItemType::Text => {
                ui.painter().text(
                    item_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &item.text_content,
                    egui::FontId::proportional(item.font_size),
                    item_color,
                );
                if is_selected {
                    ui.painter().rect_stroke(item_rect, 0.0, egui::Stroke::new(2.0, egui::Color32::YELLOW), egui::epaint::StrokeKind::Outside);
                }
            }
            StageItemType::MovieClip => {
                // Draw a movieclip representation
                ui.painter().rect_filled(item_rect, 3.0, item_color);
                ui.painter().text(
                    item_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🎬",
                    egui::FontId::proportional(20.0),
                    egui::Color32::WHITE,
                );
                if is_selected {
                    ui.painter().rect_stroke(item_rect, 3.0, egui::Stroke::new(2.0, egui::Color32::YELLOW), egui::epaint::StrokeKind::Outside);
                }
            }
            StageItemType::Path => {
                // Draw a path representation
                ui.painter().rect_stroke(item_rect, 2.0, egui::Stroke::new(2.0, item_color), egui::epaint::StrokeKind::Inside);
                ui.painter().text(
                    item_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✏️",
                    egui::FontId::proportional(16.0),
                    item_color,
                );
                if is_selected {
                    ui.painter().rect_stroke(item_rect, 2.0, egui::Stroke::new(2.0, egui::Color32::YELLOW), egui::epaint::StrokeKind::Outside);
                }
            }
        }
        
        // Draw item name as tooltip on hover
        if let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if item_rect.contains(hover_pos) {
                egui::show_tooltip_text(ui.ctx(), egui::LayerId::new(egui::Order::Tooltip, egui::Id::new(format!("tooltip_{}", item.id))), egui::Id::new(format!("tooltip_{}", item.id)), &item.name);
            }
        }
    }
    
    /// Handle stage interactions (selection, dragging, etc.)
    fn handle_stage_interactions(&mut self, ui: &mut egui::Ui, canvas_rect: egui::Rect) {
        let pointer_pos = ui.input(|i| i.pointer.interact_pos());
        
        if let Some(pos) = pointer_pos {
            if canvas_rect.contains(pos) {
                let relative_pos = pos - canvas_rect.min;
                
                // Check for clicks on items
                if ui.input(|i| i.pointer.primary_clicked()) {
                    let mut clicked_item = None;
                    
                    // Find topmost item under cursor (iterate in reverse for proper z-order)
                    for (index, item) in self.stage_items.iter().enumerate().rev() {
                        let item_rect = egui::Rect::from_min_size(item.position, item.size);
                        if item_rect.contains(egui::Pos2::new(relative_pos.x, relative_pos.y) + canvas_rect.min.to_vec2()) {
                            clicked_item = Some(index);
                            break;
                        }
                    }
                    
                    // Update selection
                    if let Some(item_index) = clicked_item {
                        if !ui.input(|i| i.modifiers.ctrl) {
                            self.selected_items.clear();
                        }
                        if !self.selected_items.contains(&item_index) {
                            self.selected_items.push(item_index);
                        }
                    } else {
                        // Clicked on empty space - clear selection
                        if !ui.input(|i| i.modifiers.ctrl) {
                            self.selected_items.clear();
                        }
                    }
                }
                
                // Handle dragging selected items
                if ui.input(|i| i.pointer.primary_down()) && !self.selected_items.is_empty() {
                    let delta = ui.input(|i| i.pointer.delta());
                    if delta.length() > 0.1 { // Only move if there's significant movement
                        for &item_index in &self.selected_items {
                            if let Some(item) = self.stage_items.get_mut(item_index) {
                                item.position += delta;
                                // Keep items within canvas bounds
                                item.position.x = item.position.x.max(0.0).min(canvas_rect.width() - item.size.x);
                                item.position.y = item.position.y.max(0.0).min(canvas_rect.height() - item.size.y);
                            }
                        }
                    }
                }
            }
        }
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
        Box::new(|cc| {
            // Initialize material icons
            egui_material_icons::initialize(&cc.egui_ctx);
            Ok(Box::new(TimelineApp::default()))
        }),
    )
}