//! Standalone demo using egui_dock for panel management

use eframe::egui::{self, ComboBox};
use nannou_timeline::{
    Timeline, DockManager, FlashTabViewer,
    ui::MockRiveEngine, RiveEngine, LayerId,
    layer::LayerType,
};
use std::sync::{Arc, Mutex};
use std::process::Command;

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
    // Console state
    console_visible: bool,
    console_height: f32,
    log_messages: Vec<LogMessage>,
    auto_scroll: bool,
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
        
        Self {
            timeline: Timeline::new(),
            engine: Box::new(LoggingRiveEngine::new(engine_logs.clone())),
            dock_manager: DockManager::new(),
            engine_logs,
            selected_language: "en".to_string(),
            selected_layer: None,
            selected_frame: None,
            console_visible: false,
            console_height: 150.0,
            log_messages: Vec::new(),
            auto_scroll: true,
        }
    }
}

impl TimelineApp {
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
        
        // Handle keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::F12)) {
            self.console_visible = !self.console_visible;
            self.log(LogLevel::Info, format!("Console {}", if self.console_visible { "shown" } else { "hidden" }));
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::F2)) {
            self.take_screenshot(ctx);
        }
        
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
        Box::new(|_cc| Ok(Box::new(TimelineApp::default()))),
    )
}