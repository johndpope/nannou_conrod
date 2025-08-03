//! Video layer support for timeline

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Video source types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VideoSource {
    /// Reference to an external file
    File(PathBuf),
    /// URL-based streaming
    Url(String),
    /// Embedded video data
    Embedded(Vec<u8>),
}

/// Video embedding mode
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoEmbedMode {
    /// Include video data in project file
    Embedded,
    /// Reference external file
    Linked,
    /// URL-based streaming
    Streaming,
}

/// Video layer properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VideoLayer {
    /// Video source
    pub source: VideoSource,
    /// How the video is embedded in the project
    pub embed_mode: VideoEmbedMode,
    /// Starting frame in timeline
    pub start_frame: u32,
    /// Trim start time in seconds
    pub trim_start: f32,
    /// Trim end time in seconds
    pub trim_end: f32,
    /// Playback rate (1.0 = normal speed)
    pub playback_rate: f32,
    /// Video duration in seconds
    pub duration: f32,
    /// Video frame rate
    pub fps: f32,
    /// Video resolution
    pub resolution: (u32, u32),
}

impl VideoLayer {
    /// Create a new video layer from a file
    pub fn from_file(path: PathBuf) -> Self {
        Self {
            source: VideoSource::File(path),
            embed_mode: VideoEmbedMode::Linked,
            start_frame: 0,
            trim_start: 0.0,
            trim_end: 0.0,
            playback_rate: 1.0,
            duration: 0.0, // Will be set when video is loaded
            fps: 30.0,     // Default, will be updated when video is loaded
            resolution: (1920, 1080), // Default HD resolution
        }
    }
    
    /// Create a new video layer from a URL
    pub fn from_url(url: String) -> Self {
        Self {
            source: VideoSource::Url(url),
            embed_mode: VideoEmbedMode::Streaming,
            start_frame: 0,
            trim_start: 0.0,
            trim_end: 0.0,
            playback_rate: 1.0,
            duration: 0.0,
            fps: 30.0,
            resolution: (1920, 1080),
        }
    }
    
    /// Get the effective duration after trimming
    pub fn effective_duration(&self) -> f32 {
        if self.trim_end > 0.0 {
            self.trim_end - self.trim_start
        } else {
            self.duration - self.trim_start
        }
    }
    
    /// Get the number of frames this video spans in the timeline
    pub fn frame_count(&self) -> u32 {
        (self.effective_duration() * self.fps / self.playback_rate).ceil() as u32
    }
    
    /// Get video time for a given timeline frame
    pub fn video_time_for_frame(&self, frame: u32) -> f32 {
        let timeline_time = (frame - self.start_frame) as f32 / self.fps;
        self.trim_start + (timeline_time * self.playback_rate)
    }
}

/// Thumbnail quality level
#[derive(Clone, Debug)]
pub struct ThumbnailQuality {
    /// How many frames per thumbnail
    pub frames_per_thumbnail: u32,
    /// Thumbnail resolution
    pub resolution: (u32, u32),
}

/// Thumbnail cache for video frames
#[derive(Debug)]
pub struct ThumbnailCache {
    /// Cached thumbnails (frame number -> texture data)
    thumbnails: HashMap<u32, Vec<u8>>,
    /// Quality levels for different zoom levels
    quality_levels: Vec<ThumbnailQuality>,
    /// Current quality level index
    current_quality: usize,
}

impl ThumbnailCache {
    /// Create a new thumbnail cache
    pub fn new() -> Self {
        let quality_levels = vec![
            // Low quality for zoomed out view
            ThumbnailQuality {
                frames_per_thumbnail: 30,
                resolution: (64, 36),
            },
            // Medium quality for normal view
            ThumbnailQuality {
                frames_per_thumbnail: 10,
                resolution: (128, 72),
            },
            // High quality for zoomed in view
            ThumbnailQuality {
                frames_per_thumbnail: 1,
                resolution: (256, 144),
            },
        ];
        
        Self {
            thumbnails: HashMap::new(),
            quality_levels,
            current_quality: 1, // Start with medium quality
        }
    }
    
    /// Get thumbnail data for a frame
    pub fn get_thumbnail(&self, frame: u32) -> Option<&Vec<u8>> {
        self.thumbnails.get(&frame)
    }
    
    /// Store thumbnail data for a frame
    pub fn store_thumbnail(&mut self, frame: u32, data: Vec<u8>) {
        self.thumbnails.insert(frame, data);
    }
    
    /// Set quality level (0 = low, 1 = medium, 2 = high)
    pub fn set_quality_level(&mut self, level: usize) {
        if level < self.quality_levels.len() {
            self.current_quality = level;
        }
    }
    
    /// Get current quality level
    pub fn current_quality(&self) -> &ThumbnailQuality {
        &self.quality_levels[self.current_quality]
    }
    
    /// Clear all cached thumbnails
    pub fn clear(&mut self) {
        self.thumbnails.clear();
    }
}

impl Default for ThumbnailCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Video decoder trait for abstracting different video backends
pub trait VideoDecoder {
    type Error;
    
    /// Open a video file
    fn open(path: &std::path::Path) -> Result<Self, Self::Error>
    where
        Self: Sized;
    
    /// Get a frame at the specified time (in seconds)
    fn get_frame(&mut self, time: f32) -> Result<VideoFrame, Self::Error>;
    
    /// Get video duration in seconds
    fn duration(&self) -> f32;
    
    /// Get video frame rate
    fn fps(&self) -> f32;
    
    /// Get video resolution
    fn resolution(&self) -> (u32, u32);
}

/// Video frame data
#[derive(Clone, Debug)]
pub struct VideoFrame {
    /// Frame image data (RGBA format)
    pub data: Vec<u8>,
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
    /// Timestamp in seconds
    pub timestamp: f32,
}

impl VideoFrame {
    /// Create a new video frame
    pub fn new(data: Vec<u8>, width: u32, height: u32, timestamp: f32) -> Self {
        Self {
            data,
            width,
            height,
            timestamp,
        }
    }
    
    /// Get the size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Mock video decoder for testing
#[derive(Debug)]
pub struct MockVideoDecoder {
    duration: f32,
    fps: f32,
    resolution: (u32, u32),
}

impl MockVideoDecoder {
    pub fn new(duration: f32, fps: f32, resolution: (u32, u32)) -> Self {
        Self {
            duration,
            fps,
            resolution,
        }
    }
}

impl VideoDecoder for MockVideoDecoder {
    type Error = String;
    
    fn open(_path: &std::path::Path) -> Result<Self, Self::Error> {
        Ok(Self::new(10.0, 30.0, (1920, 1080)))
    }
    
    fn get_frame(&mut self, time: f32) -> Result<VideoFrame, Self::Error> {
        // Generate a simple test pattern
        let (width, height) = self.resolution;
        let mut data = vec![0u8; (width * height * 4) as usize];
        
        // Create a simple gradient based on time
        let time_factor = (time * 60.0) as u8;
        for y in 0..height {
            for x in 0..width {
                let idx = ((y * width + x) * 4) as usize;
                data[idx] = ((x + time_factor as u32) % 256) as u8;     // R
                data[idx + 1] = ((y + time_factor as u32) % 256) as u8; // G
                data[idx + 2] = time_factor;                             // B
                data[idx + 3] = 255;                                     // A
            }
        }
        
        Ok(VideoFrame::new(data, width, height, time))
    }
    
    fn duration(&self) -> f32 {
        self.duration
    }
    
    fn fps(&self) -> f32 {
        self.fps
    }
    
    fn resolution(&self) -> (u32, u32) {
        self.resolution
    }
}