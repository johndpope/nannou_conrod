//! Library panel types and functionality

use egui::Pos2;

#[derive(Clone, Copy, PartialEq)]
pub enum LibraryTab {
    Assets,
    Components,
    ActionScript,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LibraryAssetType {
    MovieClip,
    Button,
    Graphic,
    Bitmap,
    Sound,
    Video,
    Font,
    Folder,
}

#[derive(Clone)]
pub struct LibraryAsset {
    pub id: String,
    pub name: String,
    pub asset_type: LibraryAssetType,
    pub folder: String,
    pub properties: AssetProperties,
}

#[derive(Clone)]
pub struct AssetProperties {
    pub file_size: Option<u64>,
    pub dimensions: Option<(u32, u32)>,
    pub format: Option<String>,
    pub usage_count: u32,
    pub linkage_class: Option<String>,
}

#[derive(Clone)]
pub struct LibraryContextMenuState {
    pub position: Pos2,
    pub target: LibraryContextTarget,
}

#[derive(Clone)]
pub enum LibraryContextTarget {
    Asset(String),
    Folder(String),
    Background,
}

impl LibraryAsset {
    pub fn new_movieclip(id: String, name: String, folder: String) -> Self {
        Self {
            id,
            name,
            asset_type: LibraryAssetType::MovieClip,
            folder,
            properties: AssetProperties {
                file_size: None,
                dimensions: None,
                format: None,
                usage_count: 0,
                linkage_class: None,
            },
        }
    }

    pub fn new_graphic(id: String, name: String, folder: String) -> Self {
        Self {
            id,
            name,
            asset_type: LibraryAssetType::Graphic,
            folder,
            properties: AssetProperties {
                file_size: None,
                dimensions: None,
                format: None,
                usage_count: 0,
                linkage_class: None,
            },
        }
    }

    pub fn new_bitmap(id: String, name: String, folder: String, dimensions: (u32, u32), file_size: u64) -> Self {
        Self {
            id,
            name,
            asset_type: LibraryAssetType::Bitmap,
            folder,
            properties: AssetProperties {
                file_size: Some(file_size),
                dimensions: Some(dimensions),
                format: Some("PNG".to_string()),
                usage_count: 0,
                linkage_class: None,
            },
        }
    }
}