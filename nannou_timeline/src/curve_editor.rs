//! Standalone Curve Editor Component for egui
//! 
//! A reusable curve editor widget that can be integrated into any egui application
//! for editing animation curves, easing functions, or any other parametric curves.

use egui::{Response, Ui, Widget, Vec2, Pos2, Rect, Color32, Stroke, Sense, Id};
use crate::easing::{BezierCurve, BezierPoint, EasingPreset};

/// A reusable curve editor widget for egui
pub struct CurveEditor<'a> {
    /// The curve data to edit
    curve: &'a mut BezierCurve,
    /// Unique ID for this editor instance
    id: Id,
    /// Width of the editor widget
    width: f32,
    /// Height of the editor widget
    height: f32,
    /// Whether to show grid lines
    show_grid: bool,
    /// Whether to show tangent handles
    show_handles: bool,
    /// Grid divisions
    grid_divisions: usize,
    /// Callback when curve changes
    on_change: Option<Box<dyn FnMut(&BezierCurve) + 'a>>,
}

impl<'a> CurveEditor<'a> {
    /// Create a new curve editor for the given curve
    pub fn new(id_source: impl std::hash::Hash, curve: &'a mut BezierCurve) -> Self {
        Self {
            curve,
            id: Id::new(id_source),
            width: 300.0,
            height: 200.0,
            show_grid: true,
            show_handles: true,
            grid_divisions: 10,
            on_change: None,
        }
    }
    
    /// Set the size of the editor
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    
    /// Set whether to show grid lines
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }
    
    /// Set whether to show bezier handles
    pub fn show_handles(mut self, show: bool) -> Self {
        self.show_handles = show;
        self
    }
    
    /// Set grid divisions
    pub fn grid_divisions(mut self, divisions: usize) -> Self {
        self.grid_divisions = divisions;
        self
    }
    
    /// Set callback for when curve changes
    pub fn on_change(mut self, callback: impl FnMut(&BezierCurve) + 'a) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }
    
    /// Apply an easing preset to the curve
    pub fn apply_preset(&mut self, preset: EasingPreset) {
        *self.curve = match preset {
            EasingPreset::Linear => BezierCurve::linear(),
            EasingPreset::EaseIn => BezierCurve::ease_in(),
            EasingPreset::EaseOut => BezierCurve::ease_out(),
            EasingPreset::EaseInOut => BezierCurve::ease_in_out(),
            _ => BezierCurve::linear(), // Fallback for unsupported presets
        };
        if let Some(ref mut callback) = self.on_change {
            callback(self.curve);
        }
    }
}

impl<'a> Widget for CurveEditor<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(self.width, self.height);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
        
        if ui.is_rect_visible(rect) {
            // Draw background
            ui.painter().rect_filled(rect, 4.0, Color32::from_gray(30));
            
            // Draw border
            ui.painter().rect_stroke(
                rect, 
                4.0, 
                Stroke::new(1.0, Color32::from_gray(60)),
                egui::epaint::StrokeKind::Outside
            );
            
            // Draw grid
            if self.show_grid {
                self.draw_grid(ui, rect);
            }
            
            // Draw curve
            self.draw_curve(ui, rect);
            
            // Draw control points and handles
            if self.show_handles {
                self.draw_control_points(ui, rect);
            }
            
            // Handle interactions
            self.handle_interactions(ui, rect, &response);
            
            // Draw axes labels
            self.draw_axes_labels(ui, rect);
        }
        
        response
    }
}

impl<'a> CurveEditor<'a> {
    /// Draw grid lines
    fn draw_grid(&self, ui: &mut Ui, rect: Rect) {
        let grid_color = Color32::from_gray(40);
        let grid_stroke = Stroke::new(0.5, grid_color);
        
        // Vertical lines
        for i in 0..=self.grid_divisions {
            let t = i as f32 / self.grid_divisions as f32;
            let x = rect.min.x + t * rect.width();
            ui.painter().line_segment(
                [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                grid_stroke,
            );
        }
        
        // Horizontal lines
        for i in 0..=self.grid_divisions {
            let t = i as f32 / self.grid_divisions as f32;
            let y = rect.max.y - t * rect.height();
            ui.painter().line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                grid_stroke,
            );
        }
        
        // Highlight center lines
        let center_stroke = Stroke::new(1.0, Color32::from_gray(50));
        
        // X=0.5 line
        let center_x = rect.min.x + rect.width() * 0.5;
        ui.painter().line_segment(
            [Pos2::new(center_x, rect.min.y), Pos2::new(center_x, rect.max.y)],
            center_stroke,
        );
        
        // Y=0.5 line
        let center_y = rect.min.y + rect.height() * 0.5;
        ui.painter().line_segment(
            [Pos2::new(rect.min.x, center_y), Pos2::new(rect.max.x, center_y)],
            center_stroke,
        );
    }
    
    /// Draw the bezier curve
    fn draw_curve(&self, ui: &mut Ui, rect: Rect) {
        if self.curve.points.len() < 2 {
            return;
        }
        
        let mut path_points = Vec::new();
        let samples = 100;
        
        // Sample the curve
        for i in 0..=samples {
            let t = i as f32 / samples as f32;
            let value = self.curve.evaluate(t);
            let x = rect.min.x + t * rect.width();
            let y = rect.max.y - value * rect.height();
            path_points.push(Pos2::new(x, y));
        }
        
        // Draw curve as line segments
        let curve_stroke = Stroke::new(2.0, Color32::from_rgb(100, 200, 255));
        for i in 0..path_points.len() - 1 {
            ui.painter().line_segment(
                [path_points[i], path_points[i + 1]],
                curve_stroke,
            );
        }
    }
    
    /// Draw control points and bezier handles
    fn draw_control_points(&self, ui: &mut Ui, rect: Rect) {
        let point_radius = 6.0;
        let handle_radius = 4.0;
        let point_color = Color32::from_rgb(255, 200, 100);
        let handle_color = Color32::from_rgb(100, 255, 100);
        let handle_stroke = Stroke::new(1.0, Color32::from_gray(150));
        
        for point in &self.curve.points {
            // Convert normalized coordinates to screen coordinates
            let screen_pos = self.norm_to_screen(point.position, rect);
            
            // Draw bezier handles
            if point.in_handle != (0.0, 0.0) {
                let in_handle_pos = Pos2::new(
                    screen_pos.x + point.in_handle.0 * rect.width(),
                    screen_pos.y - point.in_handle.1 * rect.height(),
                );
                
                // Handle line
                ui.painter().line_segment([screen_pos, in_handle_pos], handle_stroke);
                // Handle point
                ui.painter().circle_filled(in_handle_pos, handle_radius, handle_color);
            }
            
            if point.out_handle != (0.0, 0.0) {
                let out_handle_pos = Pos2::new(
                    screen_pos.x + point.out_handle.0 * rect.width(),
                    screen_pos.y - point.out_handle.1 * rect.height(),
                );
                
                // Handle line
                ui.painter().line_segment([screen_pos, out_handle_pos], handle_stroke);
                // Handle point
                ui.painter().circle_filled(out_handle_pos, handle_radius, handle_color);
            }
            
            // Draw control point (on top)
            ui.painter().circle_filled(screen_pos, point_radius, point_color);
        }
    }
    
    /// Handle mouse interactions
    fn handle_interactions(&mut self, ui: &mut Ui, rect: Rect, response: &Response) {
        if let Some(pos) = response.interact_pointer_pos() {
            if rect.contains(pos) {
                // Check if we're clicking on a control point or handle
                let click_threshold = 10.0;
                
                if response.clicked() {
                    // Check for clicking on existing points
                    let mut clicked_point = None;
                    
                    for (i, point) in self.curve.points.iter().enumerate() {
                        let screen_pos = self.norm_to_screen(point.position, rect);
                        if screen_pos.distance(pos) < click_threshold {
                            clicked_point = Some(i);
                            break;
                        }
                    }
                    
                    if clicked_point.is_none() && ui.input(|i| !i.modifiers.shift) {
                        // Add new point at click position
                        let norm_pos = self.screen_to_norm(pos, rect);
                        let new_point = BezierPoint {
                            position: norm_pos,
                            in_handle: (0.0, 0.0),
                            out_handle: (0.0, 0.0),
                        };
                        
                        // Insert at appropriate position to maintain x-order
                        let insert_index = self.curve.points
                            .iter()
                            .position(|p| p.position.0 > norm_pos.0)
                            .unwrap_or(self.curve.points.len());
                        
                        self.curve.points.insert(insert_index, new_point);
                        
                        if let Some(ref mut callback) = self.on_change {
                            callback(&self.curve);
                        }
                    }
                }
                
                // Handle right-click to delete points
                if response.secondary_clicked() {
                    for (i, point) in self.curve.points.iter().enumerate() {
                        let screen_pos = self.norm_to_screen(point.position, rect);
                        if screen_pos.distance(pos) < click_threshold && self.curve.points.len() > 2 {
                            self.curve.points.remove(i);
                            if let Some(ref mut callback) = self.on_change {
                                callback(&self.curve);
                            }
                            break;
                        }
                    }
                }
                
                // Handle dragging
                if response.dragged() {
                    let delta = response.drag_delta();
                    let norm_delta = (
                        delta.x / rect.width(),
                        -delta.y / rect.height()
                    );
                    
                    // Find what we're dragging
                    let mut dragged_point_index = None;
                    let mut dragged_handle = None; // None, Some("in"), Some("out")
                    
                    // First pass: find what we're dragging
                    for (i, point) in self.curve.points.iter().enumerate() {
                        let screen_pos = self.norm_to_screen(point.position, rect);
                        
                        // Check main point
                        if screen_pos.distance(pos - delta) < click_threshold {
                            dragged_point_index = Some(i);
                            break;
                        }
                        
                        // Check handles
                        if self.show_handles {
                            let in_handle_pos = Pos2::new(
                                screen_pos.x + point.in_handle.0 * rect.width(),
                                screen_pos.y - point.in_handle.1 * rect.height(),
                            );
                            let out_handle_pos = Pos2::new(
                                screen_pos.x + point.out_handle.0 * rect.width(),
                                screen_pos.y - point.out_handle.1 * rect.height(),
                            );
                            
                            if in_handle_pos.distance(pos - delta) < click_threshold {
                                dragged_point_index = Some(i);
                                dragged_handle = Some("in");
                                break;
                            }
                            
                            if out_handle_pos.distance(pos - delta) < click_threshold {
                                dragged_point_index = Some(i);
                                dragged_handle = Some("out");
                                break;
                            }
                        }
                    }
                    
                    // Second pass: apply the drag
                    if let Some(i) = dragged_point_index {
                        let mut changed = false;
                        
                        match dragged_handle {
                            None => {
                                // Dragging main point
                                self.curve.points[i].position.0 = (self.curve.points[i].position.0 + norm_delta.0).clamp(0.0, 1.0);
                                self.curve.points[i].position.1 = (self.curve.points[i].position.1 + norm_delta.1).clamp(0.0, 1.0);
                                
                                // Keep x-order (don't allow dragging past neighbors)
                                if i > 0 {
                                    self.curve.points[i].position.0 = self.curve.points[i].position.0.max(self.curve.points[i-1].position.0 + 0.01);
                                }
                                if i < self.curve.points.len() - 1 {
                                    self.curve.points[i].position.0 = self.curve.points[i].position.0.min(self.curve.points[i+1].position.0 - 0.01);
                                }
                                changed = true;
                            }
                            Some("in") => {
                                // Dragging in handle
                                self.curve.points[i].in_handle.0 += norm_delta.0;
                                self.curve.points[i].in_handle.1 += norm_delta.1;
                                changed = true;
                            }
                            Some("out") => {
                                // Dragging out handle
                                self.curve.points[i].out_handle.0 += norm_delta.0;
                                self.curve.points[i].out_handle.1 += norm_delta.1;
                                changed = true;
                            }
                            _ => {}
                        }
                        
                        if changed {
                            if let Some(ref mut callback) = self.on_change {
                                callback(&self.curve);
                            }
                        }
                    }
                }
            }
        }
        
        // Show tooltip (return response without consuming it)
        response.clone().on_hover_text("Left-click to add point, right-click to delete, drag to move");
    }
    
    /// Draw axes labels
    fn draw_axes_labels(&self, ui: &mut Ui, rect: Rect) {
        let text_color = Color32::from_gray(150);
        let font_id = egui::FontId::proportional(10.0);
        
        // X-axis label (Time)
        ui.painter().text(
            Pos2::new(rect.center().x, rect.max.y + 10.0),
            egui::Align2::CENTER_TOP,
            "Time",
            font_id.clone(),
            text_color,
        );
        
        // Y-axis label (Value)
        ui.painter().text(
            Pos2::new(rect.min.x - 30.0, rect.center().y),
            egui::Align2::CENTER_CENTER,
            "Value",
            font_id,
            text_color,
        );
        
        // Corner values
        let corner_font = egui::FontId::proportional(8.0);
        ui.painter().text(
            Pos2::new(rect.min.x - 5.0, rect.max.y + 5.0),
            egui::Align2::RIGHT_TOP,
            "0",
            corner_font.clone(),
            text_color,
        );
        ui.painter().text(
            Pos2::new(rect.max.x + 5.0, rect.max.y + 5.0),
            egui::Align2::LEFT_TOP,
            "1",
            corner_font.clone(),
            text_color,
        );
        ui.painter().text(
            Pos2::new(rect.min.x - 5.0, rect.min.y - 5.0),
            egui::Align2::RIGHT_BOTTOM,
            "1",
            corner_font,
            text_color,
        );
    }
    
    /// Convert normalized coordinates (0-1) to screen coordinates
    fn norm_to_screen(&self, norm_pos: (f32, f32), rect: Rect) -> Pos2 {
        Pos2::new(
            rect.min.x + norm_pos.0 * rect.width(),
            rect.max.y - norm_pos.1 * rect.height(),
        )
    }
    
    /// Convert screen coordinates to normalized coordinates (0-1)
    fn screen_to_norm(&self, screen_pos: Pos2, rect: Rect) -> (f32, f32) {
        (
            ((screen_pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0),
            ((rect.max.y - screen_pos.y) / rect.height()).clamp(0.0, 1.0),
        )
    }
}

/// Standalone curve editor panel that can be docked or shown as a window
pub struct CurveEditorPanel {
    /// The curve being edited
    pub curve: BezierCurve,
    /// Whether the panel is open
    pub open: bool,
    /// Selected preset (if any)
    pub selected_preset: Option<EasingPreset>,
}

impl Default for CurveEditorPanel {
    fn default() -> Self {
        Self {
            curve: BezierCurve::linear(),
            open: false,
            selected_preset: Some(EasingPreset::Linear),
        }
    }
}

impl CurveEditorPanel {
    /// Show the curve editor panel
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        
        let mut open = self.open;
        
        egui::Window::new("Curve Editor")
            .open(&mut open)
            .default_size([400.0, 500.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.ui(ui);
            });
            
        self.open = open;
    }
    
    /// Draw the panel UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Toolbar
        ui.horizontal(|ui| {
            ui.label("Preset:");
            
            let presets = [
                ("Linear", EasingPreset::Linear),
                ("Ease In", EasingPreset::EaseIn),
                ("Ease Out", EasingPreset::EaseOut),
                ("Ease In Out", EasingPreset::EaseInOut),
                ("Ease In Quad", EasingPreset::EaseInQuad),
                ("Ease Out Quad", EasingPreset::EaseOutQuad),
                ("Ease In Out Quad", EasingPreset::EaseInOutQuad),
                ("Ease In Cubic", EasingPreset::EaseInCubic),
                ("Ease Out Cubic", EasingPreset::EaseOutCubic),
                ("Ease In Out Cubic", EasingPreset::EaseInOutCubic),
            ];
            
            for (name, preset) in presets {
                if ui.selectable_label(
                    self.selected_preset == Some(preset.clone()), 
                    name
                ).clicked() {
                    self.selected_preset = Some(preset.clone());
                    self.curve = preset.to_curve();
                }
            }
        });
        
        ui.separator();
        
        // Main curve editor
        let mut curve_changed = false;
        ui.add(
            CurveEditor::new("main_curve_editor", &mut self.curve)
                .size(ui.available_width(), 300.0)
                .on_change(|_| {
                    curve_changed = true;
                })
        );
        
        if curve_changed {
            // Clear preset selection when manually editing
            self.selected_preset = None;
        }
        
        ui.separator();
        
        // Control points list
        ui.label("Control Points:");
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                let mut points_changed = false;
                
                for (i, point) in self.curve.points.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Point {}:", i + 1));
                        
                        ui.label("X:");
                        if ui.add(
                            egui::DragValue::new(&mut point.position.0)
                                .speed(0.01)
                                .range(0.0..=1.0)
                        ).changed() {
                            points_changed = true;
                        }
                        
                        ui.label("Y:");
                        if ui.add(
                            egui::DragValue::new(&mut point.position.1)
                                .speed(0.01)
                                .range(0.0..=1.0)
                        ).changed() {
                            points_changed = true;
                        }
                    });
                }
                
                if points_changed {
                    self.selected_preset = None;
                }
            });
        
        ui.separator();
        
        // Preview value at time
        ui.horizontal(|ui| {
            ui.label("Preview:");
            
            let mut preview_time = 0.5;
            ui.label("Time:");
            ui.add(
                egui::DragValue::new(&mut preview_time)
                    .speed(0.01)
                    .range(0.0..=1.0)
            );
            
            let value = self.curve.evaluate(preview_time);
            ui.label(format!("Value: {:.3}", value));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_curve_editor_creation() {
        let mut curve = BezierCurve::linear();
        let editor = CurveEditor::new("test", &mut curve);
        assert_eq!(editor.width, 300.0);
        assert_eq!(editor.height, 200.0);
    }
    
    #[test]
    fn test_coordinate_conversion() {
        let mut curve = BezierCurve::linear();
        let editor = CurveEditor::new("test", &mut curve);
        
        let rect = Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::new(100.0, 100.0));
        
        // Test norm to screen
        let screen = editor.norm_to_screen((0.5, 0.5), rect);
        assert_eq!(screen.x, 60.0); // 10 + 0.5 * 100
        assert_eq!(screen.y, 60.0); // 110 - 0.5 * 100
        
        // Test screen to norm
        let norm = editor.screen_to_norm(Pos2::new(60.0, 60.0), rect);
        assert!((norm.0 - 0.5).abs() < 0.001);
        assert!((norm.1 - 0.5).abs() < 0.001);
    }
}