//! Stage drawing methods

use crate::{
    stage::{StageItem, StageItemType, ResizeHandle, MarqueeSelection, ContextMenuState, ContextMenuType},
    library::{LibraryAsset, LibraryAssetType},
    tools::{Tool, ToolState},
    logging::LogLevel,
    TimelineApp,
};
use egui::{self, Pos2, Vec2, Color32, Rect, Stroke, Sense, FontId, Align2, UiBuilder};

impl TimelineApp {
    pub fn draw_stage(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            // Background
            ui.painter().rect_filled(rect, 0.0, Color32::from_gray(30));
            
            // Border
            let border_stroke = Stroke::new(1.0, Color32::from_gray(60));
            ui.painter().line_segment([rect.left_top(), rect.right_top()], border_stroke);
            ui.painter().line_segment([rect.right_top(), rect.right_bottom()], border_stroke);
            ui.painter().line_segment([rect.right_bottom(), rect.left_bottom()], border_stroke);
            ui.painter().line_segment([rect.left_bottom(), rect.left_top()], border_stroke);
            
            // Grid pattern for visual reference
            let grid_size = 50.0;
            let grid_color = Color32::from_gray(35);
            
            // Vertical lines
            let mut x = rect.left() + grid_size;
            while x < rect.right() {
                ui.painter().line_segment(
                    [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                    Stroke::new(0.5, grid_color),
                );
                x += grid_size;
            }
            
            // Horizontal lines
            let mut y = rect.top() + grid_size;
            while y < rect.bottom() {
                ui.painter().line_segment(
                    [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                    Stroke::new(0.5, grid_color),
                );
                y += grid_size;
            }
            
            // Draw stage items
            let mut clicked_item = None;
            let mut right_clicked_item = None;
            let mut hovered_item = None;
            let mut drag_info = None;
            let mut resize_handle_hit = None;
            
            for (index, item) in self.stage_items.iter().enumerate() {
                let item_rect = Rect::from_center_size(
                    rect.min + item.position.to_vec2(),
                    item.size
                );
                
                // Check if item is visible in stage
                if !rect.intersects(item_rect) {
                    continue;
                }
                
                // Item interaction - use allocate_rect to ensure proper event handling
                let response = ui.allocate_rect(item_rect, Sense::click_and_drag());
                
                // Handle hover - set cursor based on active tool
                if response.hovered() {
                    hovered_item = Some(index);
                    // Set cursor based on active tool when hovering over items
                    match self.tool_state.active_tool {
                        Tool::Arrow => ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand),
                        Tool::Hand => ui.ctx().set_cursor_icon(egui::CursorIcon::Grab),
                        Tool::Zoom => ui.ctx().set_cursor_icon(egui::CursorIcon::ZoomIn),
                        Tool::Text => ui.ctx().set_cursor_icon(egui::CursorIcon::Text),
                        Tool::Eraser => ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed),
                        Tool::Eyedropper => ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair),
                        Tool::PaintBucket => ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair),
                        _ => ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair), // Drawing tools get crosshair
                    }
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
                
                // Draw the item
                self.draw_stage_item(ui, item, item_rect, is_selected);
                
                // Draw item name when hovered
                if hovered_item == Some(index) {
                    let name_pos = item_rect.center_bottom() + Vec2::new(0.0, 5.0);
                    ui.painter().text(
                        name_pos,
                        Align2::CENTER_TOP,
                        &item.name,
                        FontId::proportional(12.0),
                        Color32::WHITE,
                    );
                }
            }
            
            // Draw resize handles for selected items
            if self.tool_state.active_tool == Tool::Arrow {
                for &index in &self.selected_items {
                    if let Some(item) = self.stage_items.get(index) {
                        let item_rect = Rect::from_center_size(
                            rect.min + item.position.to_vec2(),
                            item.size
                        );
                        
                        // Draw resize handles and check for hits
                        if let Some(handle) = self.draw_resize_handles(ui, item_rect) {
                            resize_handle_hit = Some((index, handle));
                        }
                    }
                }
            }
            
            // Handle stage background interactions
            self.handle_stage_background_interactions(
                ui, rect, &mut clicked_item, &mut right_clicked_item
            );
            
            // Handle item clicks after drawing
            self.handle_item_clicks(ui, clicked_item, right_clicked_item);
            
            // Handle resize handle detection and dragging
            self.handle_resize_dragging(ui, rect, resize_handle_hit);
            
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
            
            // Draw marquee selection rectangle if active
            if let Some(marquee) = &self.marquee_selection {
                self.draw_marquee_selection(ui, marquee);
            }
            
            // Show context menu if active
            if let Some(menu_state) = &self.context_menu.clone() {
                self.show_context_menu(ui, menu_state, rect);
            }
        });
    }

    fn draw_stage_item(&self, ui: &mut egui::Ui, item: &StageItem, item_rect: Rect, is_selected: bool) {
        match item.item_type {
            StageItemType::Rectangle => {
                let color_with_alpha = Color32::from_rgba_premultiplied(
                    item.color.r(),
                    item.color.g(), 
                    item.color.b(),
                    (item.alpha * 255.0) as u8
                );
                ui.painter().rect_filled(item_rect, 5.0, color_with_alpha);
                
                // Draw selection border if selected
                if is_selected {
                    let stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
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
                let color_with_alpha = Color32::from_rgba_premultiplied(
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
                        Stroke::new(2.0, Color32::LIGHT_BLUE)
                    );
                }
            },
            StageItemType::Text => {
                let color_with_alpha = Color32::from_rgba_premultiplied(
                    item.color.r(),
                    item.color.g(), 
                    item.color.b(),
                    (item.alpha * 255.0) as u8
                );
                ui.painter().text(
                    item_rect.center(),
                    Align2::CENTER_CENTER,
                    &item.text_content,
                    FontId::proportional(item.font_size),
                    color_with_alpha,
                );
                
                // Draw selection rect if selected
                if is_selected {
                    let stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
                    let r = item_rect;
                    ui.painter().line_segment([r.left_top(), r.right_top()], stroke);
                    ui.painter().line_segment([r.right_top(), r.right_bottom()], stroke);
                    ui.painter().line_segment([r.right_bottom(), r.left_bottom()], stroke);
                    ui.painter().line_segment([r.left_bottom(), r.left_top()], stroke);
                }
            },
            StageItemType::MovieClip => {
                // Draw as a rounded rectangle with icon
                let color_with_alpha = Color32::from_rgba_premultiplied(
                    item.color.r(),
                    item.color.g(), 
                    item.color.b(),
                    (item.alpha * 255.0) as u8
                );
                ui.painter().rect_filled(item_rect, 10.0, color_with_alpha);
                ui.painter().text(
                    item_rect.center(),
                    Align2::CENTER_CENTER,
                    "🎬",
                    FontId::proportional(24.0),
                    Color32::BLACK,
                );
                
                // Draw selection border if selected
                if is_selected {
                    let stroke = Stroke::new(2.0, Color32::LIGHT_BLUE);
                    let r = item_rect;
                    ui.painter().line_segment([r.left_top(), r.right_top()], stroke);
                    ui.painter().line_segment([r.right_top(), r.right_bottom()], stroke);
                    ui.painter().line_segment([r.right_bottom(), r.left_bottom()], stroke);
                    ui.painter().line_segment([r.left_bottom(), r.left_top()], stroke);
                }
            },
            StageItemType::Path => {
                // Draw path using the path_points
                if item.path_points.len() >= 2 {
                    let color_with_alpha = Color32::from_rgba_premultiplied(
                        item.color.r(),
                        item.color.g(), 
                        item.color.b(),
                        (item.alpha * 255.0) as u8
                    );
                    
                    // Draw lines between consecutive points
                    for window in item.path_points.windows(2) {
                        ui.painter().line_segment(
                            [window[0], window[1]], 
                            Stroke::new(2.0, color_with_alpha)
                        );
                    }
                    
                    // Draw selection if selected
                    if is_selected {
                        // Draw points
                        for &point in &item.path_points {
                            ui.painter().circle_filled(point, 3.0, Color32::LIGHT_BLUE);
                        }
                    }
                }
            },
        }
    }

    fn handle_stage_background_interactions(
        &mut self, 
        ui: &mut egui::Ui, 
        rect: Rect,
        clicked_item: &mut Option<usize>,
        right_clicked_item: &mut Option<usize>
    ) {
        if clicked_item.is_none() && right_clicked_item.is_none() {
            let stage_response = ui.interact(rect, ui.id().with("stage_bg"), Sense::click_and_drag());
            
            if stage_response.clicked() {
                if let Some(pos) = stage_response.interact_pointer_pos() {
                    let stage_pos = pos - rect.min.to_vec2();
                    self.handle_tool_click_on_stage(stage_pos);
                }
            }
            
            if stage_response.secondary_clicked() {
                // Right-clicked on empty stage
                if let Some(pos) = stage_response.interact_pointer_pos() {
                    self.context_menu = Some(ContextMenuState {
                        position: pos,
                        menu_type: ContextMenuType::Stage(pos - rect.min.to_vec2()),
                    });
                }
            }
            
            // Handle marquee selection for Arrow tool
            if self.tool_state.active_tool == Tool::Arrow {
                self.handle_marquee_selection(ui, &stage_response, rect);
            }
            
            // Handle drag-and-drop from library to stage
            self.handle_library_drag_drop(ui, &stage_response, rect);
            
            // Set cursor for empty stage area based on active tool
            if stage_response.hovered() {
                ui.ctx().set_cursor_icon(self.tool_state.active_tool.get_cursor());
            }
        }
    }

    fn handle_tool_click_on_stage(&mut self, stage_pos: Pos2) {
        match self.tool_state.active_tool {
            Tool::Arrow => {
                // Arrow tool - deselect when clicking empty space
                self.selected_items.clear();
                self.log(LogLevel::Action, format!("Stage clicked at ({:.1}, {:.1})", 
                    stage_pos.x, stage_pos.y));
            }
            Tool::Rectangle => {
                self.create_rectangle_at(stage_pos);
            }
            Tool::Oval => {
                self.create_oval_at(stage_pos);
            }
            Tool::Text => {
                self.create_text_at(stage_pos);
            }
            Tool::Line => {
                self.create_line_at(stage_pos);
            }
            Tool::Pen => {
                self.create_pen_point_at(stage_pos);
            }
            Tool::Pencil => {
                self.create_pencil_mark_at(stage_pos);
            }
            Tool::Brush => {
                self.create_brush_stroke_at(stage_pos);
            }
            Tool::Hand => {
                // Hand tool - pan/scroll functionality (log for now)
                self.log(LogLevel::Action, format!("Hand tool panning at ({:.1}, {:.1})", 
                    stage_pos.x, stage_pos.y));
            }
            Tool::Zoom => {
                // Zoom tool - zoom in/out functionality (log for now)
                self.log(LogLevel::Action, format!("Zoom tool clicked at ({:.1}, {:.1})", 
                    stage_pos.x, stage_pos.y));
            }
            _ => {
                // Other tools - just log the click for now
                self.log(LogLevel::Action, format!("{} clicked at ({:.1}, {:.1})", 
                    self.tool_state.active_tool.get_name(), stage_pos.x, stage_pos.y));
            }
        }
    }

    fn handle_marquee_selection(&mut self, ui: &mut egui::Ui, stage_response: &egui::Response, rect: Rect) {
        if stage_response.drag_started() {
            if let Some(start_pos) = stage_response.interact_pointer_pos() {
                self.marquee_selection = Some(MarqueeSelection::new(start_pos));
                self.log(LogLevel::Action, "Started marquee selection".to_string());
            }
        }
        
        if let Some(ref mut marquee) = &mut self.marquee_selection {
            if stage_response.dragged() {
                if let Some(current_pos) = stage_response.interact_pointer_pos() {
                    marquee.current_pos = current_pos;
                }
            }
            
            if stage_response.drag_stopped() {
                // Complete marquee selection
                let selection_rect = marquee.get_rect();
                let mut selected_count = 0;
                
                // Clear current selection unless Ctrl/Cmd is held
                let modifiers = ui.input(|i| i.modifiers);
                if !modifiers.ctrl && !modifiers.command {
                    self.selected_items.clear();
                }
                
                // Check which items intersect with the marquee rectangle
                for (index, item) in self.stage_items.iter().enumerate() {
                    let item_rect = Rect::from_center_size(
                        rect.min + item.position.to_vec2(),
                        item.size
                    );
                    
                    if selection_rect.intersects(item_rect) {
                        if !self.selected_items.contains(&index) {
                            self.selected_items.push(index);
                            selected_count += 1;
                        }
                    }
                }
                
                self.log(LogLevel::Action, format!("Marquee selection completed: {} items selected", selected_count));
                self.marquee_selection = None;
            }
        }
    }

    pub fn handle_library_drag_drop(&mut self, ui: &mut egui::Ui, stage_response: &egui::Response, rect: Rect) {
        if let Some(ref dragging_asset) = self.dragging_asset {
            // Visual feedback - show dragging asset under cursor
            if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                ui.painter().text(
                    pointer_pos + Vec2::new(10.0, -10.0),
                    Align2::LEFT_BOTTOM,
                    &dragging_asset.name,
                    FontId::proportional(12.0),
                    Color32::WHITE,
                );
            }
            
            // Check for drop on stage
            if stage_response.hovered() && ui.input(|i| i.pointer.any_released()) {
                if let Some(drop_pos) = stage_response.interact_pointer_pos() {
                    let stage_pos = drop_pos - rect.min.to_vec2();
                    let asset_clone = dragging_asset.clone();
                    self.create_item_from_library_asset(&asset_clone, stage_pos);
                    self.dragging_asset = None;
                }
            }
        }
        
        // Clear drag state if mouse released anywhere
        if ui.input(|i| i.pointer.any_released()) && self.dragging_asset.is_some() {
            self.log(LogLevel::Action, "Drag cancelled".to_string());
            self.dragging_asset = None;
        }
    }

    fn handle_item_clicks(&mut self, ui: &mut egui::Ui, clicked_item: Option<usize>, right_clicked_item: Option<usize>) {
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
    }

    fn handle_resize_dragging(&mut self, ui: &mut egui::Ui, rect: Rect, resize_handle_hit: Option<(usize, ResizeHandle)>) {
        // Handle resize handle detection and dragging
        if let Some((item_index, handle)) = resize_handle_hit {
            if ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary)) {
                // Start resize
                self.active_resize_handle = Some((item_index, handle));
                if let Some(item) = self.stage_items.get(item_index) {
                    self.resize_start_size = Some(item.size);
                    self.resize_start_pos = item.position;
                }
            }
        }
        
        // Handle active resize dragging
        if let Some((item_index, handle)) = self.active_resize_handle {
            if ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary)) {
                if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                    if let Some(start_size) = self.resize_start_size {
                        let stage_pos = pointer_pos - rect.min.to_vec2();
                        let item_center = self.resize_start_pos;
                        
                        // Apply resize directly here to avoid double borrow
                        if let Some(item) = self.stage_items.get_mut(item_index) {
                            match handle {
                                ResizeHandle::Right => {
                                    item.size.x = (stage_pos.x - item_center.x) * 2.0;
                                    item.size.x = item.size.x.max(10.0);
                                }
                                ResizeHandle::Left => {
                                    let new_width = (item_center.x - stage_pos.x) * 2.0;
                                    if new_width > 10.0 {
                                        item.size.x = new_width;
                                        item.position.x = self.resize_start_pos.x - (new_width - start_size.x) / 2.0;
                                    }
                                }
                                ResizeHandle::Bottom => {
                                    item.size.y = (stage_pos.y - item_center.y) * 2.0;
                                    item.size.y = item.size.y.max(10.0);
                                }
                                ResizeHandle::Top => {
                                    let new_height = (item_center.y - stage_pos.y) * 2.0;
                                    if new_height > 10.0 {
                                        item.size.y = new_height;
                                        item.position.y = self.resize_start_pos.y - (new_height - start_size.y) / 2.0;
                                    }
                                }
                                ResizeHandle::BottomRight => {
                                    item.size.x = (stage_pos.x - item_center.x) * 2.0;
                                    item.size.y = (stage_pos.y - item_center.y) * 2.0;
                                    item.size.x = item.size.x.max(10.0);
                                    item.size.y = item.size.y.max(10.0);
                                }
                                ResizeHandle::TopLeft => {
                                    let new_width = (item_center.x - stage_pos.x) * 2.0;
                                    let new_height = (item_center.y - stage_pos.y) * 2.0;
                                    if new_width > 10.0 && new_height > 10.0 {
                                        item.size.x = new_width;
                                        item.size.y = new_height;
                                        item.position.x = self.resize_start_pos.x - (new_width - start_size.x) / 2.0;
                                        item.position.y = self.resize_start_pos.y - (new_height - start_size.y) / 2.0;
                                    }
                                }
                                _ => {
                                    // Handle other resize handles as needed
                                }
                            }
                        }
                    }
                }
            } else {
                // Mouse released - stop resizing
                if let Some(item) = self.stage_items.get(item_index) {
                    self.log(LogLevel::Action, format!("Resized {} to {:.1}x{:.1}", 
                        item.name, item.size.x, item.size.y));
                }
                self.active_resize_handle = None;
                self.resize_start_size = None;
            }
        }
    }

    fn create_item_from_library_asset(&mut self, asset: &LibraryAsset, stage_pos: Pos2) {
        let new_item = match asset.asset_type {
            LibraryAssetType::MovieClip => StageItem {
                id: format!("instance_{}", self.stage_items.len() + 1),
                name: format!("{}_instance", asset.name),
                item_type: StageItemType::MovieClip,
                position: stage_pos,
                size: Vec2::new(80.0, 60.0),
                color: Color32::LIGHT_BLUE,
                alpha: 1.0,
                rotation: 0.0,
                text_content: String::new(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
                path_points: Vec::new(),
            },
            LibraryAssetType::Graphic => StageItem {
                id: format!("graphic_{}", self.stage_items.len() + 1),
                name: format!("{}_graphic", asset.name),
                item_type: StageItemType::Rectangle,
                position: stage_pos,
                size: Vec2::new(100.0, 60.0),
                color: Color32::GREEN,
                alpha: 1.0,
                rotation: 0.0,
                text_content: String::new(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
                path_points: Vec::new(),
            },
            LibraryAssetType::Bitmap => StageItem {
                id: format!("bitmap_{}", self.stage_items.len() + 1),
                name: format!("{}_bitmap", asset.name),
                item_type: StageItemType::Rectangle,
                position: stage_pos,
                size: Vec2::new(120.0, 80.0),
                color: Color32::YELLOW,
                alpha: 1.0,
                rotation: 0.0,
                text_content: String::new(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
                path_points: Vec::new(),
            },
            LibraryAssetType::Button => StageItem {
                id: format!("button_{}", self.stage_items.len() + 1),
                name: format!("{}_button", asset.name),
                item_type: StageItemType::Rectangle,
                position: stage_pos,
                size: Vec2::new(100.0, 30.0),
                color: Color32::LIGHT_GRAY,
                alpha: 1.0,
                rotation: 0.0,
                text_content: "Button".to_string(),
                font_size: 14.0,
                font_family: "Arial".to_string(),
                path_points: Vec::new(),
            },
            _ => StageItem {
                id: format!("asset_{}", self.stage_items.len() + 1),
                name: format!("{}_asset", asset.name),
                item_type: StageItemType::Rectangle,
                position: stage_pos,
                size: Vec2::new(60.0, 60.0),
                color: Color32::GRAY,
                alpha: 1.0,
                rotation: 0.0,
                text_content: String::new(),
                font_size: 16.0,
                font_family: "Arial".to_string(),
                path_points: Vec::new(),
            },
        };
        
        self.stage_items.push(new_item.clone());
        self.log(LogLevel::Action, format!("Dropped '{}' onto stage, created '{}'", 
            asset.name, new_item.name));
    }
    
    fn draw_marquee_selection(&self, ui: &mut egui::Ui, marquee: &MarqueeSelection) {
        let selection_rect = marquee.get_rect();
        
        // Draw selection rectangle with border and semi-transparent fill
        ui.painter().rect_filled(
            selection_rect,
            0.0,
            Color32::from_rgba_premultiplied(100, 150, 255, 50), // Light blue fill
        );
        
        // Draw border using line segments
        let stroke = egui::Stroke::new(1.0, Color32::from_rgb(100, 150, 255));
        ui.painter().line_segment([selection_rect.left_top(), selection_rect.right_top()], stroke);
        ui.painter().line_segment([selection_rect.right_top(), selection_rect.right_bottom()], stroke);
        ui.painter().line_segment([selection_rect.right_bottom(), selection_rect.left_bottom()], stroke);
        ui.painter().line_segment([selection_rect.left_bottom(), selection_rect.left_top()], stroke);
        
        // Draw corner indicators for better visibility
        let corners = [
            selection_rect.left_top(),
            selection_rect.right_top(),
            selection_rect.right_bottom(),
            selection_rect.left_bottom(),
        ];
        
        for corner in corners {
            ui.painter().circle_filled(
                corner,
                3.0,
                Color32::from_rgb(100, 150, 255),
            );
        }
    }
    
    fn create_rectangle_at(&mut self, stage_pos: Pos2) {
        let new_rect = StageItem {
            id: format!("rect_{}", self.stage_items.len() + 1),
            name: format!("Rectangle {}", self.stage_items.len() + 1),
            item_type: StageItemType::Rectangle,
            position: stage_pos,
            size: Vec2::new(100.0, 60.0),
            color: self.tool_state.fill_color,
            alpha: 1.0,
            rotation: 0.0,
            text_content: String::new(),
            font_size: 16.0,
            font_family: "Arial".to_string(),
            path_points: Vec::new(),
        };
        self.stage_items.push(new_rect.clone());
        self.log(LogLevel::Action, format!("Created {} with Rectangle tool", new_rect.name));
    }
    
    fn create_oval_at(&mut self, stage_pos: Pos2) {
        let new_oval = StageItem {
            id: format!("oval_{}", self.stage_items.len() + 1),
            name: format!("Circle {}", self.stage_items.len() + 1),
            item_type: StageItemType::Circle,
            position: stage_pos,
            size: Vec2::splat(80.0),
            color: self.tool_state.fill_color,
            alpha: 1.0,
            rotation: 0.0,
            text_content: String::new(),
            font_size: 16.0,
            font_family: "Arial".to_string(),
            path_points: Vec::new(),
        };
        self.stage_items.push(new_oval.clone());
        self.log(LogLevel::Action, format!("Created {} with Oval tool", new_oval.name));
    }
    
    fn create_text_at(&mut self, stage_pos: Pos2) {
        let new_text = StageItem {
            id: format!("text_{}", self.stage_items.len() + 1),
            name: format!("Text {}", self.stage_items.len() + 1),
            item_type: StageItemType::Text,
            position: stage_pos,
            size: Vec2::new(120.0, 30.0),
            color: self.tool_state.fill_color,
            alpha: 1.0,
            rotation: 0.0,
            text_content: "New Text".to_string(),
            font_size: self.tool_state.text_font_size,
            font_family: self.tool_state.text_font_family.clone(),
            path_points: Vec::new(),
        };
        self.stage_items.push(new_text.clone());
        self.log(LogLevel::Action, format!("Created {} with Text tool", new_text.name));
    }
    
    fn create_line_at(&mut self, stage_pos: Pos2) {
        let new_line = StageItem {
            id: format!("line_{}", self.stage_items.len() + 1),
            name: format!("Line {}", self.stage_items.len() + 1),
            item_type: StageItemType::Rectangle, // Use rectangle for line representation
            position: stage_pos,
            size: Vec2::new(100.0, 2.0), // Thin rectangle as line
            color: self.tool_state.stroke_color,
            alpha: 1.0,
            rotation: 0.0,
            text_content: String::new(),
            font_size: 16.0,
            font_family: "Arial".to_string(),
            path_points: Vec::new(),
        };
        self.stage_items.push(new_line.clone());
        self.log(LogLevel::Action, format!("Created {} with Line tool", new_line.name));
    }
    
    fn create_pen_point_at(&mut self, stage_pos: Pos2) {
        let new_path_point = StageItem {
            id: format!("path_{}", self.stage_items.len() + 1),
            name: format!("Path {}", self.stage_items.len() + 1),
            item_type: StageItemType::Circle,
            position: stage_pos,
            size: Vec2::splat(8.0), // Small circle for path point
            color: self.tool_state.stroke_color,
            alpha: 1.0,
            rotation: 0.0,
            text_content: String::new(),
            font_size: 16.0,
            font_family: "Arial".to_string(),
            path_points: Vec::new(),
        };
        self.stage_items.push(new_path_point.clone());
        self.log(LogLevel::Action, format!("Created {} with Pen tool", new_path_point.name));
    }
    
    fn create_pencil_mark_at(&mut self, stage_pos: Pos2) {
        let new_pencil_mark = StageItem {
            id: format!("pencil_{}", self.stage_items.len() + 1),
            name: format!("Pencil Mark {}", self.stage_items.len() + 1),
            item_type: StageItemType::Circle,
            position: stage_pos,
            size: Vec2::splat(4.0), // Small mark
            color: self.tool_state.stroke_color,
            alpha: 1.0,
            rotation: 0.0,
            text_content: String::new(),
            font_size: 16.0,
            font_family: "Arial".to_string(),
            path_points: Vec::new(),
        };
        self.stage_items.push(new_pencil_mark.clone());
        self.log(LogLevel::Action, format!("Created {} with Pencil tool", new_pencil_mark.name));
    }
    
    fn create_brush_stroke_at(&mut self, stage_pos: Pos2) {
        let new_brush_stroke = StageItem {
            id: format!("brush_{}", self.stage_items.len() + 1),
            name: format!("Brush Stroke {}", self.stage_items.len() + 1),
            item_type: StageItemType::Circle,
            position: stage_pos,
            size: Vec2::splat(self.tool_state.brush_size),
            color: self.tool_state.stroke_color,
            alpha: 1.0,
            rotation: 0.0,
            text_content: String::new(),
            font_size: 16.0,
            font_family: "Arial".to_string(),
            path_points: Vec::new(),
        };
        self.stage_items.push(new_brush_stroke.clone());
        self.log(LogLevel::Action, format!("Created {} with Brush tool", new_brush_stroke.name));
    }
}