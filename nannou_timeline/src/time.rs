//! Frame-based time system for Flash-style timeline

use serde::{Deserialize, Serialize};

/// Frame-based time representation
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrameTime {
    /// Current frame number (0-based)
    pub frame: u32,
    /// Frames per second
    pub fps: f32,
}

impl FrameTime {
    /// Create a new FrameTime
    pub fn new(frame: u32, fps: f32) -> Self {
        Self { frame, fps }
    }

    /// Convert to seconds
    pub fn to_seconds(&self) -> f32 {
        self.frame as f32 / self.fps
    }

    /// Create from seconds
    pub fn from_seconds(seconds: f32, fps: f32) -> Self {
        Self {
            frame: (seconds * fps).round() as u32,
            fps,
        }
    }

    /// Format as timecode (HH:MM:SS:FF)
    pub fn to_timecode(&self) -> String {
        let total_seconds = self.to_seconds();
        let hours = (total_seconds / 3600.0).floor() as u32;
        let minutes = ((total_seconds % 3600.0) / 60.0).floor() as u32;
        let seconds = (total_seconds % 60.0).floor() as u32;
        let frames = self.frame % self.fps.round() as u32;
        
        format!("{:02}:{:02}:{:02}:{:02}", hours, minutes, seconds, frames)
    }

    /// Format as seconds
    pub fn to_seconds_string(&self) -> String {
        format!("{:.3}s", self.to_seconds())
    }

    /// Format as frame number
    pub fn to_frame_string(&self) -> String {
        format!("Frame {}", self.frame)
    }
}

/// Common FPS presets
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum FpsPreset {
    /// Film (24 fps)
    Film,
    /// PAL (25 fps)
    Pal,
    /// NTSC (29.97 fps)
    Ntsc,
    /// Web (30 fps)
    Web,
    /// High frame rate (60 fps)
    High,
    /// Custom FPS
    Custom(f32),
}

impl FpsPreset {
    pub fn to_fps(&self) -> f32 {
        match self {
            FpsPreset::Film => 24.0,
            FpsPreset::Pal => 25.0,
            FpsPreset::Ntsc => 29.97,
            FpsPreset::Web => 30.0,
            FpsPreset::High => 60.0,
            FpsPreset::Custom(fps) => *fps,
        }
    }

    pub fn label(&self) -> String {
        match self {
            FpsPreset::Film => "24 fps (Film)".to_string(),
            FpsPreset::Pal => "25 fps (PAL)".to_string(),
            FpsPreset::Ntsc => "29.97 fps (NTSC)".to_string(),
            FpsPreset::Web => "30 fps (Web)".to_string(),
            FpsPreset::High => "60 fps (High)".to_string(),
            FpsPreset::Custom(fps) => format!("{} fps (Custom)", fps),
        }
    }

    pub fn all_presets() -> Vec<FpsPreset> {
        vec![
            FpsPreset::Film,
            FpsPreset::Pal,
            FpsPreset::Ntsc,
            FpsPreset::Web,
            FpsPreset::High,
        ]
    }
}

impl Default for FpsPreset {
    fn default() -> Self {
        FpsPreset::Film // 24 fps like Flash default
    }
}

/// Frame label for marking important points
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameLabel {
    pub frame: u32,
    pub label: String,
    #[serde(skip)]
    pub color: Option<egui::Color32>,
}

impl FrameLabel {
    pub fn new(frame: u32, label: impl Into<String>) -> Self {
        Self {
            frame,
            label: label.into(),
            color: None,
        }
    }

    pub fn with_color(mut self, color: egui::Color32) -> Self {
        self.color = Some(color);
        self
    }
}

/// Frame comment for non-functional notes and organization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrameComment {
    pub frame: u32,
    pub comment: String,
    pub author: Option<String>,
    pub timestamp: Option<String>,
    #[serde(skip)]
    pub color: Option<egui::Color32>,
}

impl FrameComment {
    pub fn new(frame: u32, comment: impl Into<String>) -> Self {
        Self {
            frame,
            comment: comment.into(),
            author: None,
            timestamp: None,
            color: Some(egui::Color32::from_rgb(100, 150, 255)), // Light blue for comments
        }
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    pub fn with_color(mut self, color: egui::Color32) -> Self {
        self.color = Some(color);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_time_conversion() {
        let ft = FrameTime::new(24, 24.0);
        assert_eq!(ft.to_seconds(), 1.0);
        
        let ft2 = FrameTime::from_seconds(2.5, 24.0);
        assert_eq!(ft2.frame, 60);
    }

    #[test]
    fn test_timecode_formatting() {
        let ft = FrameTime::new(3661, 24.0); // 152.54 seconds
        let timecode = ft.to_timecode();
        assert!(timecode.starts_with("00:02:32:")); // 2 minutes, 32 seconds
    }

    #[test]
    fn test_fps_presets() {
        assert_eq!(FpsPreset::Film.to_fps(), 24.0);
        assert_eq!(FpsPreset::Ntsc.to_fps(), 29.97);
        assert_eq!(FpsPreset::Custom(12.0).to_fps(), 12.0);
    }
}