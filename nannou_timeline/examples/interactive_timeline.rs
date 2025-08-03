//! Interactive Timeline Example - Shows the timeline with Flash-like controls
//! 
//! Run with: cargo run --example interactive_timeline

use eframe::egui::{self, Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Button};
use nannou_timeline::{Timeline, TimelineConfig, ui::MockRiveEngine, RiveEngine, LayerId};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 600.0])
            .with_title("Interactive Flash-style Timeline"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Interactive Timeline",
        options,
        Box::new(|_cc| Ok(Box::new(TimelineApp::default()))),
    )
}

struct TimelineApp {
    timeline: Timeline,
    engine: Box<dyn RiveEngine>,
    // UI state
    show_layer_controls: bool,
    selected_layer_index: Option<usize>,
    layer_visibility: Vec<bool>,
    layer_locked: Vec<bool>,
    // Context menu state
    show_context_menu: bool,
    context_menu_pos: Pos2,
    context_menu_type: ContextMenuType,
}

#[derive(Clone, Copy, PartialEq)]
enum ContextMenuType {
    None,
    Layer(usize),
    Frame(usize, u32), // layer_index, frame_number
}

impl Default for TimelineApp {
    fn default() -> Self {
        let engine = Box::new(MockRiveEngine::new());
        let layer_count = engine.get_layers().len();
        
        Self {
            timeline: Timeline::new(),
            engine,
            show_layer_controls: true,
            selected_layer_index: None,
            layer_visibility: vec![true; layer_count],
            layer_locked: vec![false; layer_count],
            show_context_menu: false,
            context_menu_pos: Pos2::ZERO,
            context_menu_type: ContextMenuType::None,
        }
    }
}

impl eframe::App for TimelineApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle right-click context menu
        ctx.input(|i| {
            if i.pointer.secondary_clicked() {
                if let Some(pos) = i.pointer.interact_pos() {
                    self.show_context_menu = true;
                    self.context_menu_pos = pos;
                }
            }
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎬 Flash-style Timeline with Interactive Controls");
            ui.separator();
            
            // Instructions
            ui.horizontal(|ui| {
                ui.label("💡 Instructions:");
                ui.label("• Click layers to select");
                ui.label("• Right-click for context menus");
                ui.label("• Use buttons to add/remove layers");
                ui.label("• Toggle visibility with 👁 icon");
                ui.label("• Press Space to play/pause");
            });
            ui.separator();
            
            // Main timeline area
            let available_rect = ui.available_rect_before_wrap();
            let timeline_rect = Rect::from_min_size(
                available_rect.min,
                Vec2::new(available_rect.width(), available_rect.height() - 100.0)
            );
            
            // Custom timeline rendering with enhanced layer panel
            self.draw_enhanced_timeline(ui, timeline_rect);
            
            // Show context menu if active
            if self.show_context_menu {
                self.show_context_menu(ui, ctx);
            }
            
            // Flash-style properties panel at bottom
            ui.separator();
            self.draw_properties_panel(ui);
        });
    }
}

impl TimelineApp {
    fn draw_enhanced_timeline(&mut self, ui: &mut Ui, rect: Rect) {
        // Split the timeline area
        let layer_panel_width = 200.0;
        let layer_panel_rect = Rect::from_min_size(rect.min, Vec2::new(layer_panel_width, rect.height()));
        let timeline_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + layer_panel_width, rect.min.y),
            Vec2::new(rect.width() - layer_panel_width, rect.height())
        );
        
        // Draw enhanced layer panel
        self.draw_flash_layer_panel(ui, layer_panel_rect);
        
        // Draw timeline
        ui.scope_builder(egui::UiBuilder::new().max_rect(timeline_rect), |ui| {
            self.timeline.show(ui, &mut self.engine);
        });
    }
    
    fn draw_flash_layer_panel(&mut self, ui: &mut Ui, rect: Rect) {
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(45));
            
            // Controls area at bottom
            let controls_height = 35.0;
            let controls_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, rect.max.y - controls_height),
                Vec2::new(rect.width(), controls_height)
            );
            
            // Layer list area
            let layers_rect = Rect::from_min_size(
                rect.min,
                Vec2::new(rect.width(), rect.height() - controls_height)
            );
            
            // Draw layer controls (Flash-style buttons)
            ui.scope_builder(egui::UiBuilder::new().max_rect(controls_rect), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().button_padding = Vec2::new(4.0, 4.0);
                    
                    // Add layer button
                    if ui.button("➕").on_hover_text("Add Layer (Ctrl+Alt+L)").clicked() {
                        self.add_layer();
                    }
                    
                    // Add folder button
                    if ui.button("📁+").on_hover_text("Add Folder").clicked() {
                        self.add_folder();
                    }
                    
                    // Delete layer button
                    let delete_enabled = self.selected_layer_index.is_some();
                    if ui.add_enabled(delete_enabled, Button::new("🗑")).on_hover_text("Delete Layer").clicked() {
                        if let Some(index) = self.selected_layer_index {
                            self.delete_layer(index);
                        }
                    }
                    
                    // Duplicate layer button
                    if ui.add_enabled(delete_enabled, Button::new("📋")).on_hover_text("Duplicate Layer").clicked() {
                        if let Some(index) = self.selected_layer_index {
                            self.duplicate_layer(index);
                        }
                    }
                });
            });
            
            // Draw layers
            let layers = self.engine.get_layers();
            let mut y_offset = 5.0;
            
            egui::ScrollArea::vertical()
                .id_salt("layer_scroll")
                .max_height(layers_rect.height())
                .show(ui, |ui| {
                    for (index, layer) in layers.iter().enumerate() {
                        let layer_rect = Rect::from_min_size(
                            Pos2::new(layers_rect.min.x + 5.0, layers_rect.min.y + y_offset),
                            Vec2::new(layers_rect.width() - 10.0, 25.0)
                        );
                        
                        // Check if selected
                        let is_selected = self.selected_layer_index == Some(index);
                        
                        // Draw layer background
                        let bg_color = if is_selected {
                            Color32::from_rgb(70, 130, 180)
                        } else {
                            Color32::from_gray(35)
                        };
                        ui.painter().rect_filled(layer_rect, 3.0, bg_color);
                        
                        // Layer interaction
                        let response = ui.interact(layer_rect, ui.id().with(index), Sense::click());
                        if response.clicked() {
                            self.selected_layer_index = Some(index);
                        }
                        
                        // Right-click context menu
                        if response.secondary_clicked() {
                            self.show_context_menu = true;
                            self.context_menu_pos = response.hover_pos().unwrap_or(layer_rect.center());
                            self.context_menu_type = ContextMenuType::Layer(index);
                        }
                        
                        // Draw layer controls (visibility, lock, etc.)
                        let icon_size = 16.0;
                        let mut x_offset = layer_rect.min.x + 5.0;
                        
                        // Visibility toggle
                        let vis_rect = Rect::from_min_size(
                            Pos2::new(x_offset, layer_rect.min.y + 4.0),
                            Vec2::splat(icon_size)
                        );
                        let vis_response = ui.interact(vis_rect, ui.id().with((index, "vis")), Sense::click());
                        if vis_response.clicked() {
                            self.layer_visibility[index] = !self.layer_visibility[index];
                        }
                        ui.painter().text(
                            vis_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            if self.layer_visibility[index] { "👁" } else { "⚫" },
                            egui::FontId::proportional(14.0),
                            Color32::WHITE,
                        );
                        x_offset += icon_size + 5.0;
                        
                        // Lock toggle
                        let lock_rect = Rect::from_min_size(
                            Pos2::new(x_offset, layer_rect.min.y + 4.0),
                            Vec2::splat(icon_size)
                        );
                        let lock_response = ui.interact(lock_rect, ui.id().with((index, "lock")), Sense::click());
                        if lock_response.clicked() {
                            self.layer_locked[index] = !self.layer_locked[index];
                        }
                        ui.painter().text(
                            lock_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            if self.layer_locked[index] { "🔒" } else { "🔓" },
                            egui::FontId::proportional(14.0),
                            Color32::WHITE,
                        );
                        x_offset += icon_size + 10.0;
                        
                        // Layer name
                        ui.painter().text(
                            Pos2::new(x_offset, layer_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &layer.name,
                            egui::FontId::proportional(14.0),
                            Color32::WHITE,
                        );
                        
                        y_offset += 30.0;
                    }
                });
        });
    }
    
    fn show_context_menu(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        // Create context menu
        egui::Area::new(ui.id().with("context_menu"))
            .order(egui::Order::Foreground)
            .fixed_pos(self.context_menu_pos)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    match self.context_menu_type {
                        ContextMenuType::Layer(index) => {
                            ui.label(format!("Layer {}", index + 1));
                            ui.separator();
                            
                            if ui.button("📋 Duplicate Layer").clicked() {
                                self.duplicate_layer(index);
                                self.show_context_menu = false;
                            }
                            if ui.button("🗑 Delete Layer").clicked() {
                                self.delete_layer(index);
                                self.show_context_menu = false;
                            }
                            if ui.button("✏️ Rename Layer").clicked() {
                                // TODO: Show rename dialog
                                self.show_context_menu = false;
                            }
                            ui.separator();
                            if ui.button("👁 Show/Hide").clicked() {
                                self.layer_visibility[index] = !self.layer_visibility[index];
                                self.show_context_menu = false;
                            }
                            if ui.button("🔒 Lock/Unlock").clicked() {
                                self.layer_locked[index] = !self.layer_locked[index];
                                self.show_context_menu = false;
                            }
                            ui.separator();
                            if ui.button("➡️ Create Motion Tween").clicked() {
                                // TODO: Create tween
                                self.show_context_menu = false;
                            }
                        }
                        ContextMenuType::Frame(layer, frame) => {
                            ui.label(format!("Frame {} on Layer {}", frame, layer + 1));
                            ui.separator();
                            
                            if ui.button("🔑 Insert Keyframe (F6)").clicked() {
                                // TODO: Insert keyframe
                                self.show_context_menu = false;
                            }
                            if ui.button("⬜ Insert Blank Keyframe (F7)").clicked() {
                                // TODO: Insert blank keyframe
                                self.show_context_menu = false;
                            }
                            if ui.button("❌ Clear Keyframe (Shift+F6)").clicked() {
                                // TODO: Clear keyframe
                                self.show_context_menu = false;
                            }
                            ui.separator();
                            if ui.button("📋 Copy Frames").clicked() {
                                // TODO: Copy frames
                                self.show_context_menu = false;
                            }
                            if ui.button("📄 Paste Frames").clicked() {
                                // TODO: Paste frames
                                self.show_context_menu = false;
                            }
                        }
                        ContextMenuType::None => {}
                    }
                });
            });
        
        // Close menu on click outside
        if ui.input(|i| i.pointer.primary_clicked()) {
            self.show_context_menu = false;
        }
    }
    
    fn draw_properties_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Properties:");
            
            if let Some(layer_index) = self.selected_layer_index {
                let layers = self.engine.get_layers();
                if let Some(layer) = layers.get(layer_index) {
                    ui.label(format!("Layer: {}", layer.name));
                    ui.separator();
                    ui.label(format!("Type: {:?}", layer.layer_type));
                    ui.separator();
                    ui.label(format!("Visible: {}", self.layer_visibility[layer_index]));
                    ui.label(format!("Locked: {}", self.layer_locked[layer_index]));
                }
            } else {
                ui.label("No layer selected");
            }
        });
    }
    
    // Action methods
    fn add_layer(&mut self) {
        println!("Adding new layer");
        // TODO: Actually add layer to engine
        self.layer_visibility.push(true);
        self.layer_locked.push(false);
    }
    
    fn add_folder(&mut self) {
        println!("Adding new folder");
        // TODO: Implement folder support
    }
    
    fn delete_layer(&mut self, index: usize) {
        println!("Deleting layer {}", index);
        // TODO: Actually delete from engine
        if index < self.layer_visibility.len() {
            self.layer_visibility.remove(index);
            self.layer_locked.remove(index);
        }
        self.selected_layer_index = None;
    }
    
    fn duplicate_layer(&mut self, index: usize) {
        println!("Duplicating layer {}", index);
        // TODO: Actually duplicate in engine
        if index < self.layer_visibility.len() {
            self.layer_visibility.push(self.layer_visibility[index]);
            self.layer_locked.push(self.layer_locked[index]);
        }
    }
}