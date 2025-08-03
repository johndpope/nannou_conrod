//! Properties panel types and functionality

#[derive(Clone, Copy, PartialEq)]
pub enum PropertyTab {
    Properties,
    Filters,
    ColorEffect,
    Display,
}

impl PropertyTab {
    pub fn get_name(&self) -> &'static str {
        match self {
            PropertyTab::Properties => "Properties",
            PropertyTab::Filters => "Filters",
            PropertyTab::ColorEffect => "Color Effect",
            PropertyTab::Display => "Display",
        }
    }
}

// Properties that can be edited for stage items
#[derive(Clone)]
pub struct ItemProperties {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub alpha: f32,
    pub visible: bool,
    pub name: String,
    pub instance_name: String,
}

impl ItemProperties {
    pub fn from_stage_item(item: &crate::stage::StageItem) -> Self {
        Self {
            x: item.position.x,
            y: item.position.y,
            width: item.size.x,
            height: item.size.y,
            rotation: item.rotation,
            alpha: item.alpha,
            visible: true,
            name: item.name.clone(),
            instance_name: item.id.clone(),
        }
    }
}