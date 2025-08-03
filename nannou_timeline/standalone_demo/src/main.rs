//! Standalone demo of the Flash-inspired timeline widget

use eframe::egui;
use nannou_timeline::{Timeline, ui::MockRiveEngine, RiveEngine, LayerId};
use std::sync::{Arc, Mutex};

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
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 600.0])
            .with_title("Flash-style Timeline Demo"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Timeline Demo",
        options,
        Box::new(|_cc| Box::new(TimelineApp::default())),
    )
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
        };
        app.log(LogLevel::Info, "Timeline application started");
        app.log(LogLevel::Info, "Press F12 to toggle debug console");
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
            
            let stage_rect = egui::Rect::from_min_size(
                available_rect.min,
                egui::vec2(available_rect.width() - self.library_width, available_rect.height() - self.timeline_height - console_space),
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
            
            // Draw timeline (bottom) - capture any println! from timeline
            ui.allocate_ui_at_rect(timeline_rect, |ui| {
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
    
    fn draw_console(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.allocate_ui_at_rect(rect, |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(20));
            
            // Border
            ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_gray(60)));
            
            // Console header
            let header_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 25.0));
            ui.allocate_ui_at_rect(header_rect, |ui| {
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
                });
            });
            
            // Console content
            let content_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(0.0, 25.0),
                egui::vec2(rect.width(), rect.height() - 25.0),
            );
            
            ui.allocate_ui_at_rect(content_rect, |ui| {
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
        ui.allocate_ui_at_rect(rect, |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(30));
            
            // Border
            ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_gray(60)));
            
            // Center content
            let center = rect.center();
            ui.painter().text(
                center,
                egui::Align2::CENTER_CENTER,
                "Stage / Canvas",
                egui::FontId::proportional(24.0),
                egui::Color32::from_gray(100),
            );
            
            // Stage dimensions
            ui.painter().text(
                center + egui::vec2(0.0, 30.0),
                egui::Align2::CENTER_CENTER,
                format!("{}x{}", rect.width() as i32, rect.height() as i32),
                egui::FontId::proportional(14.0),
                egui::Color32::from_gray(80),
            );
            
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
            
            // Log stage interactions
            let response = ui.interact(rect, ui.id().with("stage"), egui::Sense::click());
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    self.log(LogLevel::Action, format!("Stage clicked at ({:.1}, {:.1})", pos.x - rect.min.x, pos.y - rect.min.y));
                }
            }
        });
    }
    
    fn draw_library(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        ui.allocate_ui_at_rect(rect, |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_gray(40));
            
            // Border
            ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::from_gray(60)));
            
            // Content with padding
            let padded_rect = rect.shrink(10.0);
            ui.allocate_ui_at_rect(padded_rect, |ui| {
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
}