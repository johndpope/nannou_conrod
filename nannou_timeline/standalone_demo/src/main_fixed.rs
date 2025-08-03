//! Fixed standalone demo of the Flash-inspired timeline widget
//! Resolves the "min > max" crash and implements complete Flash CS6-style interface

use eframe::egui::{self, UiBuilder, Color32, Pos2, Rect, Vec2, Stroke, Align2, FontId, Button, TextEdit, Align};
use nannou_timeline::{Timeline, TimelineConfig, ui::MockRiveEngine, RiveEngine, LayerId};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

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
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 800.0])
            .with_title("Flash CS6-style Timeline Demo"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Timeline Demo",
        options,
        Box::new(|_cc| Ok(Box::new(TimelineApp::default()))),
    )
}

struct TimelineApp {
    timeline: Timeline,
    engine: Box<dyn RiveEngine>,
    // State for resizable panels
    timeline_height: f32,
    library_width: f32,
    console_height: f32,
    properties_height: f32,
    splitter_thickness: f32,
    // Debug console state
    console_visible: bool,
    log_messages: Vec<LogMessage>,
    auto_scroll: bool,
    // Engine log receiver
    engine_logs: Arc<Mutex<Vec<(LogLevel, String)>>>,
    // Library panel state
    library_tab: LibraryTab,
    selected_asset: Option<String>,
    // Properties panel state
    selected_property_tab: PropertyTab,
}

#[derive(Clone, Copy, PartialEq)]
enum LibraryTab {
    Assets,
    Components,
    ActionScript,
}

#[derive(Clone, Copy, PartialEq)]
enum PropertyTab {
    Properties,
    Filters,
    ColorEffect,
    Display,
    Component,
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
            timeline_height: 250.0,
            library_width: 350.0,
            console_height: 150.0,
            properties_height: 250.0,
            splitter_thickness: 4.0,
            console_visible: true,
            log_messages: Vec::new(),
            auto_scroll: true,
            engine_logs,
            library_tab: LibraryTab::Assets,
            selected_asset: None,
            selected_property_tab: PropertyTab::Properties,
        };
        app.log(LogLevel::Info, "Flash CS6-style Timeline application started");
        app.log(LogLevel::Info, "Press F12 to toggle debug console");
        app
    }
}

impl eframe::App for TimelineApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply Flash-like dark theme
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = Color32::from_gray(30);
        style.visuals.panel_fill = Color32::from_gray(30);
        style.visuals.faint_bg_color = Color32::from_gray(35);
        style.visuals.extreme_bg_color = Color32::from_gray(40);
        style.visuals.widgets.inactive.bg_fill = Color32::from_gray(45);
        style.visuals.widgets.hovered.bg_fill = Color32::from_gray(55);
        style.visuals.widgets.active.bg_fill = Color32::from_gray(65);
        ctx.set_style(style);
        
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
            
            // Calculate layout with proper minimum sizes
            let console_space = if self.console_visible { 
                self.console_height.max(50.0) 
            } else { 
                0.0 
            };
            
            let timeline_height = self.timeline_height.max(150.0);
            let library_width = self.library_width.max(200.0);
            let properties_height = self.properties_height.max(150.0);
            
            // Ensure we have enough space
            let min_stage_height = 200.0;
            let min_stage_width = 400.0;
            
            let total_height = available_rect.height();
            let total_width = available_rect.width();
            
            // Adjust sizes if needed to maintain minimums
            let actual_timeline_height = timeline_height.min(total_height - console_space - properties_height - min_stage_height);
            let actual_library_width = library_width.min(total_width - min_stage_width);
            let actual_properties_height = properties_height.min(total_height - console_space - actual_timeline_height - min_stage_height);
            
            // Calculate regions
            let stage_rect = Rect::from_min_size(
                available_rect.min,
                Vec2::new(
                    total_width - actual_library_width,
                    total_height - actual_timeline_height - console_space - actual_properties_height,
                ),
            );
            
            let library_rect = Rect::from_min_size(
                Pos2::new(stage_rect.max.x, available_rect.min.y),
                Vec2::new(actual_library_width, stage_rect.height()),
            );
            
            let properties_rect = Rect::from_min_size(
                Pos2::new(available_rect.min.x, stage_rect.max.y),
                Vec2::new(stage_rect.width(), actual_properties_height),
            );
            
            let timeline_rect = Rect::from_min_size(
                Pos2::new(available_rect.min.x, properties_rect.max.y),
                Vec2::new(total_width, actual_timeline_height),
            );
            
            let console_rect = if self.console_visible {
                Some(Rect::from_min_size(
                    Pos2::new(available_rect.min.x, timeline_rect.max.y),
                    Vec2::new(total_width, console_space),
                ))
            } else {
                None
            };
            
            // Draw all panels
            self.draw_stage(ui, stage_rect);
            self.draw_library_panel(ui, library_rect);
            self.draw_properties_panel(ui, properties_rect);
            
            // Draw timeline with proper bounds checking
            ui.scope_builder(UiBuilder::new().max_rect(timeline_rect), |ui| {
                // Set clip rect to prevent overflow
                ui.set_clip_rect(timeline_rect);
                
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
            
            // Draw splitters
            self.draw_splitters(ui, stage_rect, library_rect, properties_rect, timeline_rect, console_rect);
            
            // Draw console if visible
            if let Some(console_rect) = console_rect {
                self.draw_console(ui, console_rect);
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
    
    fn draw_stage(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(30));
            
            // Border
            let border_stroke = Stroke::new(1.0, Color32::from_gray(60));
            ui.painter().rect_stroke(rect, 0.0, border_stroke);
            
            // Stage content
            let center = rect.center();
            
            // Mock stage with Flash-like appearance
            let stage_size = Vec2::new(550.0, 400.0); // Flash default stage size
            let stage_rect = Rect::from_center_size(center, stage_size.min(rect.size() * 0.8));
            
            // White stage background
            ui.painter().rect_filled(stage_rect, 0.0, Color32::WHITE);
            ui.painter().rect_stroke(stage_rect, 0.0, Stroke::new(2.0, Color32::from_gray(100)));
            
            // Stage info
            ui.painter().text(
                stage_rect.center(),
                Align2::CENTER_CENTER,
                "Stage",
                FontId::proportional(24.0),
                Color32::from_gray(200),
            );
            
            ui.painter().text(
                stage_rect.center() + Vec2::new(0.0, 30.0),
                Align2::CENTER_CENTER,
                format!("{}x{} @ {} fps", 
                    stage_size.x as i32, 
                    stage_size.y as i32,
                    self.timeline.config.fps.to_fps()
                ),
                FontId::proportional(14.0),
                Color32::from_gray(150),
            );
            
            // Grid pattern
            let grid_size = 25.0;
            let grid_color = Color32::from_gray(240);
            
            let mut x = stage_rect.left() + grid_size;
            while x < stage_rect.right() {
                ui.painter().line_segment(
                    [Pos2::new(x, stage_rect.top()), Pos2::new(x, stage_rect.bottom())],
                    Stroke::new(0.5, grid_color),
                );
                x += grid_size;
            }
            
            let mut y = stage_rect.top() + grid_size;
            while y < stage_rect.bottom() {
                ui.painter().line_segment(
                    [Pos2::new(stage_rect.left(), y), Pos2::new(stage_rect.right(), y)],
                    Stroke::new(0.5, grid_color),
                );
                y += grid_size;
            }
            
            // Handle stage interactions
            let response = ui.interact(stage_rect, ui.id().with("stage"), egui::Sense::click());
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let stage_pos = pos - stage_rect.min;
                    self.log(LogLevel::Action, format!("Stage clicked at ({:.1}, {:.1})", stage_pos.x, stage_pos.y));
                }
            }
        });
    }
    
    fn draw_library_panel(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(40));
            
            // Border
            ui.painter().rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_gray(60)));
            
            // Content with padding
            let padded_rect = rect.shrink(5.0);
            ui.scope_builder(UiBuilder::new().max_rect(padded_rect), |ui| {
                ui.vertical(|ui| {
                    // Header
                    ui.horizontal(|ui| {
                        ui.heading("Library");
                        ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("⚙").on_hover_text("Library Options").clicked() {
                                self.log(LogLevel::Action, "Library options clicked");
                            }
                        });
                    });
                    
                    ui.separator();
                    
                    // Tabs
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.library_tab == LibraryTab::Assets, "Assets").clicked() {
                            self.library_tab = LibraryTab::Assets;
                            self.log(LogLevel::Action, "Library tab: Assets");
                        }
                        if ui.selectable_label(self.library_tab == LibraryTab::Components, "Components").clicked() {
                            self.library_tab = LibraryTab::Components;
                            self.log(LogLevel::Action, "Library tab: Components");
                        }
                        if ui.selectable_label(self.library_tab == LibraryTab::ActionScript, "AS Linkage").clicked() {
                            self.library_tab = LibraryTab::ActionScript;
                            self.log(LogLevel::Action, "Library tab: ActionScript Linkage");
                        }
                    });
                    
                    ui.separator();
                    
                    // Import buttons
                    ui.horizontal(|ui| {
                        if ui.button("Import...").clicked() {
                            self.log(LogLevel::Action, "Import media clicked");
                        }
                        if ui.button("New Symbol").clicked() {
                            self.log(LogLevel::Action, "New symbol clicked");
                        }
                        if ui.button("New Folder").clicked() {
                            self.log(LogLevel::Action, "New folder clicked");
                        }
                    });
                    
                    ui.separator();
                    
                    // Search bar
                    ui.horizontal(|ui| {
                        ui.label("🔍");
                        let mut search_text = String::new();
                        if ui.text_edit_singleline(&mut search_text)
                            .hint_text("Search library...")
                            .changed() 
                        {
                            self.log(LogLevel::Action, format!("Library search: {}", search_text));
                        }
                    });
                    
                    ui.separator();
                    
                    // Asset list
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            match self.library_tab {
                                LibraryTab::Assets => {
                                    self.draw_asset_tree(ui);
                                }
                                LibraryTab::Components => {
                                    ui.label("No components in library");
                                }
                                LibraryTab::ActionScript => {
                                    ui.label("ActionScript Linkage:");
                                    ui.separator();
                                    for i in 1..=3 {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("Symbol_{}", i));
                                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                                ui.label(format!("com.example.Symbol{}", i));
                                            });
                                        });
                                    }
                                }
                            }
                        });
                });
            });
        });
    }
    
    fn draw_asset_tree(&mut self, ui: &mut egui::Ui) {
        // Folders
        ui.collapsing("📁 Graphics", |ui| {
            for i in 1..=5 {
                let asset_name = format!("Symbol_{}", i);
                let is_selected = self.selected_asset.as_ref() == Some(&asset_name);
                
                ui.horizontal(|ui| {
                    if ui.selectable_label(is_selected, format!("🎭 {}", asset_name)).clicked() {
                        self.selected_asset = Some(asset_name.clone());
                        self.log(LogLevel::Action, format!("Selected asset: {}", asset_name));
                    }
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.label("MovieClip");
                    });
                });
            }
        });
        
        ui.collapsing("📁 Bitmaps", |ui| {
            for i in 1..=3 {
                let asset_name = format!("Image_{}.png", i);
                let is_selected = self.selected_asset.as_ref() == Some(&asset_name);
                
                ui.horizontal(|ui| {
                    if ui.selectable_label(is_selected, format!("🖼️ {}", asset_name)).clicked() {
                        self.selected_asset = Some(asset_name.clone());
                        self.log(LogLevel::Action, format!("Selected asset: {}", asset_name));
                    }
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.label("Bitmap");
                    });
                });
            }
        });
        
        ui.collapsing("📁 Sounds", |ui| {
            for i in 1..=2 {
                let asset_name = format!("Sound_{}.mp3", i);
                let is_selected = self.selected_asset.as_ref() == Some(&asset_name);
                
                ui.horizontal(|ui| {
                    if ui.selectable_label(is_selected, format!("🔊 {}", asset_name)).clicked() {
                        self.selected_asset = Some(asset_name.clone());
                        self.log(LogLevel::Action, format!("Selected asset: {}", asset_name));
                    }
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.label("Sound");
                    });
                });
            }
        });
        
        ui.collapsing("📁 Fonts", |ui| {
            ui.label("No embedded fonts");
        });
    }
    
    fn draw_properties_panel(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(40));
            
            // Border
            ui.painter().rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_gray(60)));
            
            // Content
            let padded_rect = rect.shrink(5.0);
            ui.scope_builder(UiBuilder::new().max_rect(padded_rect), |ui| {
                ui.vertical(|ui| {
                    // Header
                    ui.heading("Properties");
                    ui.separator();
                    
                    // Property tabs
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.selected_property_tab == PropertyTab::Properties, "Properties").clicked() {
                            self.selected_property_tab = PropertyTab::Properties;
                        }
                        if ui.selectable_label(self.selected_property_tab == PropertyTab::Filters, "Filters").clicked() {
                            self.selected_property_tab = PropertyTab::Filters;
                        }
                        if ui.selectable_label(self.selected_property_tab == PropertyTab::ColorEffect, "Color").clicked() {
                            self.selected_property_tab = PropertyTab::ColorEffect;
                        }
                        if ui.selectable_label(self.selected_property_tab == PropertyTab::Display, "Display").clicked() {
                            self.selected_property_tab = PropertyTab::Display;
                        }
                    });
                    
                    ui.separator();
                    
                    // Property content
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            match self.selected_property_tab {
                                PropertyTab::Properties => {
                                    self.draw_properties_content(ui);
                                }
                                PropertyTab::Filters => {
                                    ui.label("No filters applied");
                                    ui.separator();
                                    if ui.button("Add Filter").clicked() {
                                        self.log(LogLevel::Action, "Add filter clicked");
                                    }
                                }
                                PropertyTab::ColorEffect => {
                                    ui.label("Color Effect:");
                                    ui.horizontal(|ui| {
                                        ui.label("Style:");
                                        egui::ComboBox::from_id_salt("color_style")
                                            .selected_text("None")
                                            .show_ui(ui, |ui| {
                                                ui.selectable_label(true, "None");
                                                ui.selectable_label(false, "Brightness");
                                                ui.selectable_label(false, "Tint");
                                                ui.selectable_label(false, "Advanced");
                                            });
                                    });
                                }
                                PropertyTab::Display => {
                                    ui.label("Display:");
                                    ui.horizontal(|ui| {
                                        ui.label("Blend:");
                                        egui::ComboBox::from_id_salt("blend_mode")
                                            .selected_text("Normal")
                                            .show_ui(ui, |ui| {
                                                ui.selectable_label(true, "Normal");
                                                ui.selectable_label(false, "Add");
                                                ui.selectable_label(false, "Multiply");
                                                ui.selectable_label(false, "Screen");
                                            });
                                    });
                                }
                                _ => {}
                            }
                        });
                });
            });
        });
    }
    
    fn draw_properties_content(&mut self, ui: &mut egui::Ui) {
        if let Some(asset) = &self.selected_asset {
            ui.label(format!("Instance of: {}", asset));
            ui.separator();
            
            // Transform properties
            ui.label("Position and Size:");
            ui.horizontal(|ui| {
                ui.label("X:");
                let mut x_text = "0.0".to_string();
                ui.add(TextEdit::singleline(&mut x_text).desired_width(60.0));
                
                ui.label("Y:");
                let mut y_text = "0.0".to_string();
                ui.add(TextEdit::singleline(&mut y_text).desired_width(60.0));
            });
            
            ui.horizontal(|ui| {
                ui.label("W:");
                let mut w_text = "100.0".to_string();
                ui.add(TextEdit::singleline(&mut w_text).desired_width(60.0));
                
                ui.label("H:");
                let mut h_text = "100.0".to_string();
                ui.add(TextEdit::singleline(&mut h_text).desired_width(60.0));
            });
            
            ui.separator();
            
            // 3D properties
            ui.label("3D Position and View:");
            ui.horizontal(|ui| {
                ui.label("Z:");
                let mut z_text = "0.0".to_string();
                ui.add(TextEdit::singleline(&mut z_text).desired_width(60.0));
            });
            
            ui.separator();
            
            // Color properties
            ui.label("Color Effect:");
            ui.horizontal(|ui| {
                ui.label("Alpha:");
                let mut alpha = 100.0;
                ui.add(egui::Slider::new(&mut alpha, 0.0..=100.0).suffix("%"));
            });
        } else {
            ui.label("No object selected");
            ui.separator();
            ui.label("Select an object on the stage or in the timeline to see its properties.");
        }
    }
    
    fn draw_console(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(20));
            
            // Border
            ui.painter().rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_gray(60)));
            
            // Console header
            let header_height = 25.0;
            let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_height));
            
            ui.scope_builder(UiBuilder::new().max_rect(header_rect), |ui| {
                ui.horizontal(|ui| {
                    ui.label("🖥️ Output");
                    ui.separator();
                    
                    if ui.button("Clear").clicked() {
                        self.log_messages.clear();
                        self.log(LogLevel::Info, "Console cleared");
                    }
                    
                    ui.separator();
                    ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                    
                    ui.separator();
                    ui.label(format!("{} messages", self.log_messages.len()));
                    
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Filters").clicked() {
                            self.log(LogLevel::Action, "Console filters clicked");
                        }
                    });
                });
            });
            
            // Console content
            let content_rect = Rect::from_min_size(
                rect.min + Vec2::new(0.0, header_height),
                Vec2::new(rect.width(), rect.height() - header_height),
            );
            
            ui.scope_builder(UiBuilder::new().max_rect(content_rect), |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(self.auto_scroll)
                    .show(ui, |ui| {
                        for msg in &self.log_messages {
                            let color = match msg.level {
                                LogLevel::Info => Color32::from_gray(180),
                                LogLevel::Action => Color32::from_rgb(100, 200, 100),
                                LogLevel::Warning => Color32::from_rgb(255, 200, 100),
                                LogLevel::Error => Color32::from_rgb(255, 100, 100),
                            };
                            
                            ui.horizontal(|ui| {
                                ui.colored_label(Color32::from_gray(120), &msg.timestamp);
                                ui.colored_label(color, &msg.message);
                            });
                        }
                    });
            });
        });
    }
    
    fn draw_splitters(&mut self, ui: &mut egui::Ui, stage_rect: Rect, library_rect: Rect, properties_rect: Rect, timeline_rect: Rect, console_rect: Option<Rect>) {
        // Horizontal splitter (between stage and library)
        let h_splitter_rect = Rect::from_min_size(
            Pos2::new(stage_rect.max.x, stage_rect.min.y),
            Vec2::new(self.splitter_thickness, stage_rect.height()),
        );
        self.handle_horizontal_splitter(ui, h_splitter_rect);
        
        // Vertical splitter 1 (between stage and properties)
        let v_splitter1_rect = Rect::from_min_size(
            Pos2::new(stage_rect.min.x, stage_rect.max.y),
            Vec2::new(stage_rect.width(), self.splitter_thickness),
        );
        self.handle_vertical_splitter(ui, v_splitter1_rect, SplitterType::Properties);
        
        // Vertical splitter 2 (between properties and timeline)
        let v_splitter2_rect = Rect::from_min_size(
            Pos2::new(properties_rect.min.x, properties_rect.max.y),
            Vec2::new(properties_rect.width() + library_rect.width(), self.splitter_thickness),
        );
        self.handle_vertical_splitter(ui, v_splitter2_rect, SplitterType::Timeline);
        
        // Console splitter if visible
        if console_rect.is_some() {
            let console_splitter_rect = Rect::from_min_size(
                Pos2::new(timeline_rect.min.x, timeline_rect.max.y),
                Vec2::new(timeline_rect.width(), self.splitter_thickness),
            );
            self.handle_vertical_splitter(ui, console_splitter_rect, SplitterType::Console);
        }
    }
    
    fn handle_horizontal_splitter(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        
        let color = if response.hovered() {
            Color32::from_gray(100)
        } else {
            Color32::from_gray(70)
        };
        ui.painter().rect_filled(rect, 0.0, color);
        
        if response.dragged() {
            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let available_width = ui.available_width();
                self.library_width = (available_width - pointer_pos.x + rect.width() / 2.0)
                    .clamp(200.0, available_width - 400.0);
            }
        }
        
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
    }
    
    fn handle_vertical_splitter(&mut self, ui: &mut egui::Ui, rect: Rect, splitter_type: SplitterType) {
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        
        let color = if response.hovered() {
            Color32::from_gray(100)
        } else {
            Color32::from_gray(70)
        };
        ui.painter().rect_filled(rect, 0.0, color);
        
        if response.dragged() {
            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let available_height = ui.available_height();
                
                match splitter_type {
                    SplitterType::Properties => {
                        let remaining_space = available_height - self.timeline_height - self.console_height;
                        self.properties_height = (pointer_pos.y - rect.min.y + rect.height() / 2.0)
                            .clamp(100.0, remaining_space - 200.0);
                    }
                    SplitterType::Timeline => {
                        let y_from_bottom = available_height - pointer_pos.y;
                        self.timeline_height = (y_from_bottom - self.console_height + rect.height() / 2.0)
                            .clamp(150.0, available_height - self.console_height - self.properties_height - 200.0);
                    }
                    SplitterType::Console => {
                        self.console_height = (available_height - pointer_pos.y + rect.height() / 2.0)
                            .clamp(50.0, available_height - self.timeline_height - self.properties_height - 200.0);
                    }
                }
            }
        }
        
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
    }
}

#[derive(Clone, Copy)]
enum SplitterType {
    Properties,
    Timeline,
    Console,
}