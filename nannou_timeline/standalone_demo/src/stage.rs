//! Stage-related types and functionality

use egui::{self, Pos2, Vec2, Color32};

#[derive(Clone)]
pub struct StageItem {
    pub id: String,
    pub name: String,
    pub item_type: StageItemType,
    pub position: Pos2,
    pub size: Vec2,
    pub color: Color32,
    pub alpha: f32,  // 0.0 to 1.0
    pub rotation: f32,
    // Text-specific properties
    pub text_content: String,
    pub font_size: f32,
    pub font_family: String,
    // Path-specific properties
    pub path_points: Vec<Pos2>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StageItemType {
    Rectangle,
    Circle,
    Text,
    MovieClip,
    Path,
}

impl StageItem {
    pub fn new_rectangle(id: String, name: String, position: Pos2, size: Vec2, color: Color32) -> Self {
        Self {
            id,
            name,
            item_type: StageItemType::Rectangle,
            position,
            size,
            color,
            alpha: 1.0,
            rotation: 0.0,
            text_content: String::new(),
            font_size: 16.0,
            font_family: "Arial".to_string(),
            path_points: Vec::new(),
        }
    }

    pub fn new_circle(id: String, name: String, position: Pos2, radius: f32, color: Color32) -> Self {
        Self {
            id,
            name,
            item_type: StageItemType::Circle,
            position,
            size: Vec2::splat(radius * 2.0),
            color,
            alpha: 1.0,
            rotation: 0.0,
            text_content: String::new(),
            font_size: 16.0,
            font_family: "Arial".to_string(),
            path_points: Vec::new(),
        }
    }

    pub fn new_text(id: String, name: String, position: Pos2, text: String, font_size: f32, color: Color32) -> Self {
        Self {
            id,
            name,
            item_type: StageItemType::Text,
            position,
            size: Vec2::new(120.0, 30.0), // Default size, adjust based on text
            color,
            alpha: 1.0,
            rotation: 0.0,
            text_content: text,
            font_size,
            font_family: "Arial".to_string(),
            path_points: Vec::new(),
        }
    }

    pub fn new_movieclip(id: String, name: String, position: Pos2, size: Vec2, color: Color32) -> Self {
        Self {
            id,
            name,
            item_type: StageItemType::MovieClip,
            position,
            size,
            color,
            alpha: 1.0,
            rotation: 0.0,
            text_content: String::new(),
            font_size: 16.0,
            font_family: "Arial".to_string(),
            path_points: Vec::new(),
        }
    }

    pub fn new_path(id: String, name: String, points: Vec<Pos2>, color: Color32) -> Self {
        let (min_x, min_y, max_x, max_y) = points.iter().fold(
            (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
            |(min_x, min_y, max_x, max_y), point| {
                (min_x.min(point.x), min_y.min(point.y), max_x.max(point.x), max_y.max(point.y))
            }
        );
        
        Self {
            id,
            name,
            item_type: StageItemType::Path,
            position: Pos2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0),
            size: Vec2::new(max_x - min_x, max_y - min_y),
            color,
            alpha: 1.0,
            rotation: 0.0,
            text_content: String::new(),
            font_size: 16.0,
            font_family: "Arial".to_string(),
            path_points: points,
        }
    }

    pub fn get_rect(&self) -> egui::Rect {
        egui::Rect::from_center_size(self.position, self.size)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct MarqueeSelection {
    pub start_pos: Pos2,
    pub current_pos: Pos2,
    pub is_dragging: bool,
}

impl MarqueeSelection {
    pub fn new(start_pos: Pos2) -> Self {
        Self {
            start_pos,
            current_pos: start_pos,
            is_dragging: true,
        }
    }
    
    pub fn get_rect(&self) -> egui::Rect {
        egui::Rect::from_two_pos(self.start_pos, self.current_pos)
    }
}

#[derive(Clone)]
pub struct ContextMenuState {
    pub position: Pos2,
    pub menu_type: ContextMenuType,
}

#[derive(Clone)]
pub enum ContextMenuType {
    Stage(Pos2),
    StageItem(usize),
}