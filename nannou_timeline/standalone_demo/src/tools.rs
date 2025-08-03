//! Tools panel types and functionality

use egui::Color32;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tool {
    // Selection Tools
    Arrow,        // V - Primary selection
    Subselection, // A - Direct selection
    Lasso,        // L - Free-form selection
    
    // Drawing Tools
    Line,         // N - Straight lines
    Pen,          // P - Bezier curves
    Pencil,       // Y - Freehand
    Brush,        // B - Variable width
    Rectangle,    // R - Rectangles
    Oval,         // O - Circles/ellipses
    PolyStar,     // Polygons/stars
    
    // Text and Paint Tools
    Text,         // T - Text objects
    PaintBucket,  // K - Fill areas
    InkBottle,    // S - Apply stroke
    Eyedropper,   // I - Sample colors
    Eraser,       // E - Erase parts
    
    // Transform Tools
    FreeTransform,    // Q - Scale/rotate/skew
    GradientTransform,// F - Adjust gradients
    Zoom,             // Z - Zoom view
    Hand,             // H - Pan view
}

impl Tool {
    pub fn get_icon(&self) -> &'static str {
        match self {
            Tool::Arrow => "↖",         // Better arrow for selection
            Tool::Subselection => "◇",   // Direct selection
            Tool::Lasso => "⟡",         // Lasso selection
            Tool::Line => "╱",          // Line tool
            Tool::Pen => "⌐",           // Pen tool
            Tool::Pencil => "✎",        // Pencil
            Tool::Brush => "🖌",        // Brush
            Tool::Rectangle => "▭",     // Rectangle
            Tool::Oval => "○",          // Oval/Circle
            Tool::PolyStar => "⬟",      // Star/Polygon
            Tool::Text => "A",          // Text tool
            Tool::PaintBucket => "▣",   // Paint bucket
            Tool::InkBottle => "🖋",    // Ink bottle
            Tool::Eyedropper => "🔍",   // Eyedropper
            Tool::Eraser => "▤",        // Eraser
            Tool::FreeTransform => "⤢", // Free transform
            Tool::GradientTransform => "◐", // Gradient transform
            Tool::Zoom => "⊕",         // Zoom
            Tool::Hand => "✋",         // Hand
        }
    }
    
    pub fn get_name(&self) -> &'static str {
        match self {
            Tool::Arrow => "Selection Tool",
            Tool::Subselection => "Subselection Tool",
            Tool::Lasso => "Lasso Tool",
            Tool::Line => "Line Tool",
            Tool::Pen => "Pen Tool",
            Tool::Pencil => "Pencil Tool",
            Tool::Brush => "Brush Tool",
            Tool::Rectangle => "Rectangle Tool",
            Tool::Oval => "Oval Tool",
            Tool::PolyStar => "PolyStar Tool",
            Tool::Text => "Text Tool",
            Tool::PaintBucket => "Paint Bucket Tool",
            Tool::InkBottle => "Ink Bottle Tool",
            Tool::Eyedropper => "Eyedropper Tool",
            Tool::Eraser => "Eraser Tool",
            Tool::FreeTransform => "Free Transform Tool",
            Tool::GradientTransform => "Gradient Transform Tool",
            Tool::Zoom => "Zoom Tool",
            Tool::Hand => "Hand Tool",
        }
    }
    
    pub fn get_shortcut(&self) -> Option<char> {
        match self {
            Tool::Arrow => Some('V'),
            Tool::Subselection => Some('A'),
            Tool::Lasso => Some('L'),
            Tool::Line => Some('N'),
            Tool::Pen => Some('P'),
            Tool::Pencil => Some('Y'),
            Tool::Brush => Some('B'),
            Tool::Rectangle => Some('R'),
            Tool::Oval => Some('O'),
            Tool::Text => Some('T'),
            Tool::PaintBucket => Some('K'),
            Tool::InkBottle => Some('S'),
            Tool::Eyedropper => Some('I'),
            Tool::Eraser => Some('E'),
            Tool::FreeTransform => Some('Q'),
            Tool::GradientTransform => Some('F'),
            Tool::Zoom => Some('Z'),
            Tool::Hand => Some('H'),
            _ => None,
        }
    }
    
    pub fn get_cursor(&self) -> egui::CursorIcon {
        match self {
            Tool::Arrow => egui::CursorIcon::Default,
            Tool::Hand => egui::CursorIcon::Grab,
            Tool::Zoom => egui::CursorIcon::ZoomIn,
            Tool::Text => egui::CursorIcon::Text,
            Tool::Eraser => egui::CursorIcon::NotAllowed,
            Tool::Eyedropper => egui::CursorIcon::Crosshair,
            Tool::PaintBucket => egui::CursorIcon::Crosshair,
            _ => egui::CursorIcon::Crosshair, // Drawing tools get crosshair
        }
    }
}

#[derive(Clone)]
pub struct ToolState {
    pub active_tool: Tool,
    pub stroke_color: Color32,
    pub fill_color: Color32,
    pub stroke_width: f32,
    // Tool-specific options
    pub rectangle_corner_radius: f32,
    pub star_points: u32,
    pub star_inner_radius: f32,
    pub brush_size: f32,
    pub text_font_size: f32,
    pub text_font_family: String,
}

impl Default for ToolState {
    fn default() -> Self {
        Self {
            active_tool: Tool::Arrow,
            stroke_color: Color32::BLACK,
            fill_color: Color32::WHITE,
            stroke_width: 1.0,
            rectangle_corner_radius: 0.0,
            star_points: 5,
            star_inner_radius: 0.5,
            brush_size: 10.0,
            text_font_size: 16.0,
            text_font_family: "Arial".to_string(),
        }
    }
}