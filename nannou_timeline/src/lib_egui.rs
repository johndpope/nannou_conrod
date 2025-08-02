//! A Flash-inspired timeline widget for nannou using egui.
//!
//! The primary type is **Timeline** - an egui widget that mimics Adobe Flash's
//! timeline interface with layers, keyframes, and playback controls.

use egui::{Context, Ui, Response, Vec2, Rect, Color32, Stroke, FontId};
use std::collections::HashMap;

pub use playhead::Playhead;
pub use ruler::Ruler;
pub use timeline::{Timeline, TimelineState};
pub use layer::{Layer, LayerId, LayerType};
pub use frame::{Frame, FrameType, KeyframeId};
pub use track::Track;

pub mod playhead;
pub mod ruler;
pub mod timeline;
pub mod layer;
pub mod frame;
pub mod track;
pub mod ui;

// Re-export time calculation utilities
pub use time_calc as time;

/// Mock interface for Rive integration
pub trait RiveEngine: Send + Sync {
    fn get_layers(&self) -> Vec<layer::LayerInfo>;
    fn get_frame_data(&self, layer_id: LayerId, frame: u32) -> frame::FrameData;
    fn play(&mut self);
    fn pause(&mut self);
    fn seek(&mut self, frame: u32);
    fn get_current_frame(&self) -> u32;
    fn get_total_frames(&self) -> u32;
    fn get_fps(&self) -> f32;
}

/// Timeline configuration
#[derive(Clone, Debug)]
pub struct TimelineConfig {
    /// Width of the layer panel (left side)
    pub layer_panel_width: f32,
    /// Height of the ruler
    pub ruler_height: f32,
    /// Height of the playback controls
    pub controls_height: f32,
    /// Default track height
    pub default_track_height: f32,
    /// Frame width (for zoom)
    pub frame_width: f32,
    /// Colors and styling
    pub style: TimelineStyle,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            layer_panel_width: 200.0,
            ruler_height: 30.0,
            controls_height: 40.0,
            default_track_height: 30.0,
            frame_width: 10.0,
            style: TimelineStyle::default(),
        }
    }
}

/// Visual styling for the timeline
#[derive(Clone, Debug)]
pub struct TimelineStyle {
    pub background_color: Color32,
    pub grid_color: Color32,
    pub layer_background: Color32,
    pub layer_selected: Color32,
    pub frame_empty: Color32,
    pub frame_keyframe: Color32,
    pub frame_tween: Color32,
    pub playhead_color: Color32,
    pub border_color: Color32,
    pub text_color: Color32,
}

impl Default for TimelineStyle {
    fn default() -> Self {
        Self {
            background_color: Color32::from_gray(40),
            grid_color: Color32::from_gray(60),
            layer_background: Color32::from_gray(50),
            layer_selected: Color32::from_rgb(70, 130, 180),
            frame_empty: Color32::from_gray(45),
            frame_keyframe: Color32::from_gray(20),
            frame_tween: Color32::from_rgb(100, 100, 150),
            playhead_color: Color32::from_rgb(255, 0, 0),
            border_color: Color32::from_gray(80),
            text_color: Color32::from_gray(220),
        }
    }
}

/// The duration of a sequence of bars in ticks (preserved from original)
pub fn bars_duration_ticks<I>(bars: I, ppqn: time::Ppqn) -> time::Ticks
where
    I: IntoIterator<Item = time::TimeSig>,
{
    bars.into_iter()
        .fold(time::Ticks(0), |acc, ts| acc + ts.ticks_per_bar(ppqn))
}