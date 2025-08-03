//! Standalone demo of the Flash-inspired timeline widget

use eframe::egui::{self, UiBuilder, ComboBox};
use nannou_timeline::{
    timeline_egui::Timeline,
    ui::MockRiveEngine, RiveEngine, LayerId,
    layer::LayerType,
};
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};

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
        
        let mut app = Self {
            timeline: Timeline::new(),
            engine: Box::new(LoggingRiveEngine::new(engine_logs.clone())),
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
        };
        app.log(LogLevel::Info, "Timeline application started");
        app.log(LogLevel::Info, "🎮 Keyboard shortcuts:");
        app.log(LogLevel::Info, "  • F12: Toggle debug console");
        app.log(LogLevel::Info, "  • F2: Take screenshot");
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
        
        // Show crash dialog if a panic occurred
        self.show_crash_dialog(ctx);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_rect = ui.available_rect_before_wrap();
            
            // Adjust for console if visible
            let console_space = if self.console_visible { self.console_height } else { 0.0 };
            
            // Calculate regions with resizable sizes
            let library_rect = egui::Rect::from_min_size(
                egui::pos2(available_rect.max.x - self.library_width, available_rect.min.y),
                egui::vec2(self.library_width, available_rect.height() - self.timeline_height - console_space),
            );
            
            let timeline_rect = egui::Rect::from_min_size(
                egui::pos2(available_rect.min.x, available_rect.max.y - self.timeline_height - console_space),
                egui::vec2(available_rect.width() - self.library_width, self.timeline_height),
            );
            
            let properties_rect = egui::Rect::from_min_size(
                egui::pos2(available_rect.min.x, available_rect.max.y - self.timeline_height - console_space - self.properties_height),
                egui::vec2(available_rect.width() - self.library_width, self.properties_height),
            );
            
            let stage_rect = egui::Rect::from_min_size(
                available_rect.min,
                egui::vec2(available_rect.width() - self.library_width, available_rect.height() - self.timeline_height - console_space - self.properties_height),
            );
            
            let console_rect = if self.console_visible {
                Some(egui::Rect::from_min_size(
                    egui::pos2(available_rect.min.x, available_rect.max.y - console_space),
                    egui::vec2(available_rect.width(), console_space),
                ))
            } else {
                None
            };
            
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
                
                // Item interaction
                let item_id = ui.id().with(format!("stage_item_{}", item.id));
                let response = ui.interact(item_rect, item_id, egui::Sense::click_and_drag());
                
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
            
            // Handle stage background interactions
            let stage_response = ui.interact(rect, ui.id().with("stage_bg"), egui::Sense::click());
            
            if stage_response.clicked() && clicked_item.is_none() {
                // Clicked on empty stage - deselect
                self.selected_items.clear();
                if let Some(pos) = stage_response.interact_pointer_pos() {
                    self.log(LogLevel::Action, format!("Stage clicked at ({:.1}, {:.1})", 
                        pos.x - rect.min.x, pos.y - rect.min.y));
                }
            }
            
            if stage_response.secondary_clicked() && right_clicked_item.is_none() {
                // Right-clicked on empty stage
                if let Some(pos) = stage_response.interact_pointer_pos() {
                    self.context_menu = Some(ContextMenuState {
                        position: pos,
                        menu_type: ContextMenuType::Stage(pos - rect.min.to_vec2()),
                    });
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
                    ui.heading("📚 Library");
                    ui.separator();
                    
                    // Tabs
                    ui.horizontal(|ui| {
                        if ui.selectable_label(true, "Assets").clicked() {
                            self.log(LogLevel::Action, "Library tab: Assets selected");
                        }
                        if ui.selectable_label(false, "Properties").clicked() {
                            self.log(LogLevel::Action, "Library tab: Properties selected");
                        }
                    });
                    
                    ui.separator();
                    
                    // Scrollable asset list
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label("🎭 Symbols:");
                        for i in 1..=5 {
                            ui.horizontal(|ui| {
                                if ui.button(format!("Symbol_{}", i)).clicked() {
                                    self.log(LogLevel::Action, format!("Selected Symbol_{}", i));
                                }
                            });
                        }
                        
                        ui.add_space(10.0);
                        ui.label("🖼️ Bitmaps:");
                        for i in 1..=3 {
                            ui.horizontal(|ui| {
                                if ui.button(format!("Image_{}", i)).clicked() {
                                    self.log(LogLevel::Action, format!("Selected Image_{}", i));
                                }
                            });
                        }
                        
                        ui.add_space(10.0);
                        ui.label("🔊 Sounds:");
                        for i in 1..=2 {
                            ui.horizontal(|ui| {
                                if ui.button(format!("Sound_{}", i)).clicked() {
                                    self.log(LogLevel::Action, format!("Selected Sound_{}", i));
                                }
                            });
                        }
                    });
                });
            });
        });
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