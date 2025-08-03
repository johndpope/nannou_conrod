//! Resize handle drawing functionality

use crate::{
    stage::ResizeHandle,
    TimelineApp,
};
use egui::{self, Pos2, Vec2, Color32, Rect, Stroke, Sense};

impl TimelineApp {
    pub fn draw_resize_handles(&self, ui: &mut egui::Ui, rect: Rect) -> Option<ResizeHandle> {
        let handle_size = 8.0;
        let handle_color = Color32::from_rgb(100, 150, 255);
        let handle_stroke = Stroke::new(1.0, Color32::WHITE);
        
        let handles = [
            (ResizeHandle::TopLeft, rect.left_top()),
            (ResizeHandle::TopRight, rect.right_top()),
            (ResizeHandle::BottomLeft, rect.left_bottom()),
            (ResizeHandle::BottomRight, rect.right_bottom()),
            (ResizeHandle::Top, Pos2::new((rect.left() + rect.right()) / 2.0, rect.top())),
            (ResizeHandle::Bottom, Pos2::new((rect.left() + rect.right()) / 2.0, rect.bottom())),
            (ResizeHandle::Left, Pos2::new(rect.left(), (rect.top() + rect.bottom()) / 2.0)),
            (ResizeHandle::Right, Pos2::new(rect.right(), (rect.top() + rect.bottom()) / 2.0)),
        ];
        
        let mut hit_handle = None;
        
        for (handle_type, pos) in handles {
            let handle_rect = Rect::from_center_size(pos, Vec2::splat(handle_size));
            let response = ui.allocate_rect(handle_rect, Sense::hover());
            
            // Draw the handle
            ui.painter().rect_filled(handle_rect, 2.0, handle_color);
            
            // Check for hover/click
            if response.hovered() {
                ui.painter().rect_filled(handle_rect, 2.0, handle_color.gamma_multiply(1.2));
                
                // Set appropriate cursor
                let cursor = match handle_type {
                    ResizeHandle::Left | ResizeHandle::Right => egui::CursorIcon::ResizeHorizontal,
                    ResizeHandle::Top | ResizeHandle::Bottom => egui::CursorIcon::ResizeVertical,
                    ResizeHandle::TopLeft | ResizeHandle::BottomRight => egui::CursorIcon::ResizeNeSw,
                    ResizeHandle::TopRight | ResizeHandle::BottomLeft => egui::CursorIcon::ResizeNwSe,
                };
                ui.ctx().set_cursor_icon(cursor);
                
                if ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary)) {
                    hit_handle = Some(handle_type);
                }
            }
        }
        
        hit_handle
    }
}