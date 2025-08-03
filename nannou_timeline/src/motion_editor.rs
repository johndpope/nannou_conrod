//! Motion Editor window for bezier curve easing control

use egui::{Pos2, Rect, Color32, Stroke};
use std::collections::HashMap;
use crate::easing::{BezierCurve, EasingPreset, PropertyId};

/// Main Motion Editor window for editing easing curves
#[derive(Clone, Debug)]
pub struct MotionEditor {
    /// Whether the editor is open
    pub open: bool,
    /// Curves for each property
    pub curves: HashMap<PropertyId, BezierCurve>,
    /// Currently selected property
    pub selected_property: Option<PropertyId>,
    /// Preview time position (0-1)
    pub preview_time: f32,
    /// Which control point is being dragged
    pub dragging_point: Option<usize>,
    /// Whether we're dragging a handle (not the point itself)
    pub dragging_handle: Option<(usize, bool)>, // (point_index, is_out_handle)
}

impl Default for MotionEditor {
    fn default() -> Self {
        let mut curves = HashMap::new();
        
        // Initialize with linear curves for common properties
        for property in PropertyId::all_properties() {
            curves.insert(property, BezierCurve::linear());
        }
        
        Self {
            open: false,
            curves,
            selected_property: Some(PropertyId::PositionX),
            preview_time: 0.0,
            dragging_point: None,
            dragging_handle: None,
        }
    }
}

impl MotionEditor {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Open the Motion Editor window
    pub fn open(&mut self) {
        self.open = true;
    }
    
    /// Close the Motion Editor window  
    pub fn close(&mut self) {
        self.open = false;
    }
    
    /// Show the Motion Editor UI
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        
        let mut open = self.open;
        let response = egui::Window::new("Motion Editor")
            .default_size([800.0, 600.0])
            .resizable(true)
            .open(&mut open)
            .show(ctx, |ui| {
                self.draw_editor_ui(ui);
            });
            
        self.open = open;
        
        // Close window if user clicked X
        if response.is_none() {
            self.open = false;
        }
    }
    
    /// Draw the main editor UI
    fn draw_editor_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Left panel: Property list
            ui.vertical(|ui| {
                ui.set_width(200.0);
                self.draw_property_list(ui);
            });
            
            ui.separator();
            
            // Right panel: Curve editor
            ui.vertical(|ui| {
                self.draw_curve_editor(ui);
            });
        });
        
        ui.separator();
        
        // Bottom: Preview controls
        self.draw_preview_controls(ui);
    }
    
    /// Draw the property list panel
    fn draw_property_list(&mut self, ui: &mut egui::Ui) {
        ui.label("Animatable Properties");
        ui.separator();
        
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for property in PropertyId::all_properties() {
                    let is_selected = self.selected_property.as_ref() == Some(&property);
                    
                    if ui.selectable_label(is_selected, property.name()).clicked() {
                        self.selected_property = Some(property.clone());
                    }
                    
                    // Show mini curve preview
                    if let Some(curve) = self.curves.get(&property) {
                        self.draw_mini_curve_preview(ui, curve);
                    }
                }
            });
    }
    
    /// Draw a small curve preview thumbnail
    fn draw_mini_curve_preview(&self, ui: &mut egui::Ui, curve: &BezierCurve) {
        let size = egui::Vec2::new(150.0, 30.0);
        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
        
        if curve.points.len() >= 2 {
            let mut path_points = Vec::new();
            
            // Sample the curve at regular intervals
            for i in 0..=20 {
                let t = i as f32 / 20.0;
                let value = curve.evaluate(t);
                let x = rect.min.x + t * rect.width();
                let y = rect.max.y - value * rect.height();
                path_points.push(Pos2::new(x, y));
            }
            
            // Draw background
            ui.painter().rect_filled(rect, 2.0, Color32::from_gray(30));
            
            // Draw curve
            if path_points.len() > 1 {
                for i in 0..path_points.len() - 1 {
                    ui.painter().line_segment(
                        [path_points[i], path_points[i + 1]],
                        Stroke::new(1.0, Color32::from_rgb(100, 150, 255)),
                    );
                }
            }
        }
    }
    
    /// Draw the main curve editor
    fn draw_curve_editor(&mut self, ui: &mut egui::Ui) {
        if let Some(ref property) = self.selected_property.clone() {
            ui.label(format!("Easing Curve: {}", property.name()));
            
            // Preset picker
            self.draw_preset_picker(ui, property);
            
            ui.separator();
            
            // Main curve plot
            self.draw_curve_plot(ui, property);
        } else {
            ui.label("Select a property to edit its easing curve");
        }
    }
    
    /// Draw preset easing function picker
    fn draw_preset_picker(&mut self, ui: &mut egui::Ui, property: &PropertyId) {
        ui.label("Presets:");
        ui.horizontal_wrapped(|ui| {
            for preset in EasingPreset::all_presets() {
                if ui.button(preset.name()).clicked() {
                    let curve = preset.to_curve();
                    self.curves.insert(property.clone(), curve);
                    println!("Applied {} preset to {}", preset.name(), property.name());
                }
            }
        });
    }
    
    /// Draw the interactive curve plot
    fn draw_curve_plot(&mut self, ui: &mut egui::Ui, property: &PropertyId) {
        let curve = self.curves.get(property).cloned().unwrap_or_default();
        
        let size = ui.available_size_before_wrap();
        let plot_size = egui::Vec2::new(size.x, (size.y - 100.0).max(300.0));
        let (rect, response) = ui.allocate_exact_size(plot_size, egui::Sense::click_and_drag());
        
        // Draw background
        ui.painter().rect_filled(rect, 4.0, Color32::from_gray(25));
        
        // Draw grid
        self.draw_grid(ui, rect);
        
        // Draw curve
        self.draw_curve(ui, rect, &curve);
        
        // Draw control points and handles
        self.draw_control_points(ui, rect, &curve);
        
        // Handle interactions
        self.handle_curve_interactions(ui, rect, response, property);
    }
    
    /// Draw grid background
    fn draw_grid(&self, ui: &mut egui::Ui, rect: Rect) {
        let grid_color = Color32::from_gray(40);
        let grid_stroke = Stroke::new(1.0, grid_color);
        
        // Vertical lines (time)
        for i in 0..=10 {
            let x = rect.min.x + (i as f32 / 10.0) * rect.width();
            ui.painter().line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                grid_stroke,
            );
        }
        
        // Horizontal lines (value)
        for i in 0..=10 {
            let y = rect.max.y - (i as f32 / 10.0) * rect.height();
            ui.painter().line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                grid_stroke,
            );
        }
    }
    
    /// Draw the bezier curve
    fn draw_curve(&self, ui: &mut egui::Ui, rect: Rect, curve: &BezierCurve) {
        if curve.points.len() < 2 {
            return;
        }
        
        let mut path_points = Vec::new();
        
        // Sample the curve at high resolution for smooth rendering
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let value = curve.evaluate(t);
            let x = rect.min.x + t * rect.width();
            let y = rect.max.y - value * rect.height();
            path_points.push(Pos2::new(x, y));
        }
        
        // Draw the curve as connected line segments
        for i in 0..path_points.len() - 1 {
            ui.painter().line_segment(
                [path_points[i], path_points[i + 1]],
                Stroke::new(2.0, Color32::WHITE),
            );
        }
    }
    
    /// Draw control points and bezier handles
    fn draw_control_points(&self, ui: &mut egui::Ui, rect: Rect, curve: &BezierCurve) {
        let point_color = Color32::from_rgb(255, 255, 100);
        let handle_color = Color32::from_rgb(100, 255, 100);
        let handle_stroke = Stroke::new(1.0, handle_color);
        
        for (_i, point) in curve.points.iter().enumerate() {
            // Convert normalized coordinates to screen coordinates
            let screen_pos = Pos2::new(
                rect.min.x + point.position.0 * rect.width(),
                rect.max.y - point.position.1 * rect.height(),
            );
            
            // Draw handles
            if point.in_handle != (0.0, 0.0) {
                let in_handle_pos = Pos2::new(
                    screen_pos.x + point.in_handle.0 * rect.width(),
                    screen_pos.y - point.in_handle.1 * rect.height(),
                );
                
                // Handle line
                ui.painter().line_segment([screen_pos, in_handle_pos], handle_stroke);
                // Handle point
                ui.painter().circle_filled(in_handle_pos, 4.0, handle_color);
            }
            
            if point.out_handle != (0.0, 0.0) {
                let out_handle_pos = Pos2::new(
                    screen_pos.x + point.out_handle.0 * rect.width(),
                    screen_pos.y - point.out_handle.1 * rect.height(),
                );
                
                // Handle line
                ui.painter().line_segment([screen_pos, out_handle_pos], handle_stroke);
                // Handle point
                ui.painter().circle_filled(out_handle_pos, 4.0, handle_color);
            }
            
            // Draw main control point
            ui.painter().circle_filled(screen_pos, 6.0, point_color);
        }
    }
    
    /// Handle mouse interactions with the curve
    fn handle_curve_interactions(&mut self, _ui: &mut egui::Ui, rect: Rect, response: egui::Response, _property: &PropertyId) {
        // TODO: Implement dragging of control points and handles
        if response.clicked() {
            if let Some(cursor_pos) = response.interact_pointer_pos() {
                // Convert screen coordinates to normalized coordinates
                let normalized_x = (cursor_pos.x - rect.min.x) / rect.width();
                let normalized_y = (rect.max.y - cursor_pos.y) / rect.height();
                
                println!("Clicked curve at normalized position: ({:.2}, {:.2})", normalized_x, normalized_y);
            }
        }
    }
    
    /// Draw preview controls at bottom
    fn draw_preview_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Preview:");
            
            // Time scrubber
            ui.add(egui::Slider::new(&mut self.preview_time, 0.0..=1.0)
                .text("Time")
                .show_value(true));
            
            // Show evaluated value
            if let Some(ref property) = self.selected_property {
                if let Some(curve) = self.curves.get(property) {
                    let value = curve.evaluate(self.preview_time);
                    ui.label(format!("Value: {:.3}", value));
                }
            }
            
            // Reset button
            if ui.button("Reset Time").clicked() {
                self.preview_time = 0.0;
            }
        });
    }
}