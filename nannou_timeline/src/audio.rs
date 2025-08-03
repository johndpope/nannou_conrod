//! Audio layer support for Flash-style timeline integration

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Unique identifier for audio sources
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioId(pub String);

impl AudioId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Generate a new unique audio ID
    pub fn generate() -> Self {
        Self(format!("audio_{}", uuid::Uuid::new_v4()))
    }
}

/// Audio sync mode determines how audio plays relative to timeline
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioSyncMode {
    /// Play once when frame is reached
    Event,
    /// Start playing if not already playing
    Start,
    /// Stop audio playback
    Stop,
    /// Sync audio position to timeline position
    Stream,
}

impl Default for AudioSyncMode {
    fn default() -> Self {
        AudioSyncMode::Event
    }
}

/// Audio source information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioSource {
    /// Unique identifier for this audio
    pub id: AudioId,
    /// File path to audio file
    pub file_path: PathBuf,
    /// Original filename for display
    pub filename: String,
    /// Duration in seconds
    pub duration: f32,
    /// Sample rate (Hz)
    pub sample_rate: u32,
    /// Number of channels (1=mono, 2=stereo)
    pub channels: u32,
    /// Whether the audio file is loaded
    pub loaded: bool,
}

impl AudioSource {
    pub fn new(file_path: PathBuf) -> Self {
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
            
        Self {
            id: AudioId::generate(),
            file_path,
            filename,
            duration: 0.0,
            sample_rate: 44100,
            channels: 2,
            loaded: false,
        }
    }
    
    /// Get display name for UI
    pub fn display_name(&self) -> &str {
        &self.filename
    }
}

/// Audio layer configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioLayer {
    /// Audio source
    pub source: AudioSource,
    /// How audio syncs with timeline
    pub sync_mode: AudioSyncMode,
    /// Volume level (0.0 to 1.0)
    pub volume: f32,
    /// Frame where audio begins
    pub start_frame: u32,
    /// Trim from beginning (seconds)
    pub trim_start: f32,
    /// Trim from end (seconds) 
    pub trim_end: f32,
    /// Volume envelope points
    pub volume_envelope: VolumeEnvelope,
    /// Whether to loop the audio
    pub loop_audio: bool,
}

impl AudioLayer {
    pub fn new(source: AudioSource, start_frame: u32) -> Self {
        Self {
            source,
            sync_mode: AudioSyncMode::default(),
            volume: 1.0,
            start_frame,
            trim_start: 0.0,
            trim_end: 0.0,
            volume_envelope: VolumeEnvelope::default(),
            loop_audio: false,
        }
    }
    
    /// Get the effective duration after trimming
    pub fn effective_duration(&self) -> f32 {
        (self.source.duration - self.trim_start - self.trim_end).max(0.0)
    }
    
    /// Get frame range this audio spans
    pub fn frame_range(&self, fps: f32) -> std::ops::Range<u32> {
        let duration_frames = (self.effective_duration() * fps) as u32;
        self.start_frame..(self.start_frame + duration_frames)
    }
    
    /// Get audio time at a given frame
    pub fn audio_time_at_frame(&self, frame: u32, fps: f32) -> Option<f32> {
        if frame < self.start_frame {
            return None;
        }
        
        let frame_offset = frame - self.start_frame;
        let audio_time = self.trim_start + (frame_offset as f32 / fps);
        
        if audio_time <= self.source.duration - self.trim_end {
            Some(audio_time)
        } else {
            None
        }
    }
}

/// Volume envelope for dynamic volume control
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VolumeEnvelope {
    /// Control points: (frame, volume)
    pub points: Vec<(u32, f32)>,
}

impl VolumeEnvelope {
    pub fn new() -> Self {
        Self {
            points: vec![(0, 1.0)], // Start with 100% volume
        }
    }
    
    /// Add or update a volume point
    pub fn set_point(&mut self, frame: u32, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        
        // Find existing point or insert position
        match self.points.binary_search_by_key(&frame, |(f, _)| *f) {
            Ok(index) => {
                // Update existing point
                self.points[index].1 = volume;
            }
            Err(index) => {
                // Insert new point
                self.points.insert(index, (frame, volume));
            }
        }
    }
    
    /// Remove a volume point
    pub fn remove_point(&mut self, frame: u32) {
        self.points.retain(|(f, _)| *f != frame);
        
        // Ensure we always have at least one point
        if self.points.is_empty() {
            self.points.push((0, 1.0));
        }
    }
    
    /// Get interpolated volume at a specific frame
    pub fn volume_at_frame(&self, frame: u32) -> f32 {
        if self.points.is_empty() {
            return 1.0;
        }
        
        // Find surrounding points
        match self.points.binary_search_by_key(&frame, |(f, _)| *f) {
            Ok(index) => {
                // Exact match
                self.points[index].1
            }
            Err(index) => {
                if index == 0 {
                    // Before first point
                    self.points[0].1
                } else if index >= self.points.len() {
                    // After last point
                    self.points[self.points.len() - 1].1
                } else {
                    // Interpolate between points
                    let (f1, v1) = self.points[index - 1];
                    let (f2, v2) = self.points[index];
                    
                    let t = (frame - f1) as f32 / (f2 - f1) as f32;
                    v1 + (v2 - v1) * t
                }
            }
        }
    }
}

impl Default for VolumeEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

/// Waveform data for visualization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaveformData {
    /// Audio source this belongs to
    pub audio_id: AudioId,
    /// Peaks per frame: (min, max) values
    pub peaks: Vec<(f32, f32)>,
    /// Frames per second this was generated for
    pub fps: f32,
    /// Whether generation is complete
    pub complete: bool,
}

impl WaveformData {
    pub fn new(audio_id: AudioId, fps: f32) -> Self {
        Self {
            audio_id,
            peaks: Vec::new(),
            fps,
            complete: false,
        }
    }
    
    /// Get peaks for a specific frame range
    pub fn peaks_for_range(&self, start_frame: u32, end_frame: u32) -> &[(f32, f32)] {
        let start = start_frame as usize;
        let end = (end_frame as usize).min(self.peaks.len());
        
        if start < self.peaks.len() && start < end {
            &self.peaks[start..end]
        } else {
            &[]
        }
    }
}

/// Audio engine interface for playback and processing
pub trait AudioEngine: Send + Sync {
    /// Load an audio file and return its ID
    fn load_audio(&mut self, file_path: &std::path::Path) -> Result<AudioSource, AudioError>;
    
    /// Unload audio from memory
    fn unload_audio(&mut self, audio_id: &AudioId) -> Result<(), AudioError>;
    
    /// Play audio segment
    fn play_segment(&mut self, audio_id: &AudioId, start_time: f32, duration: f32, volume: f32) -> Result<(), AudioError>;
    
    /// Stop audio playback
    fn stop_audio(&mut self, audio_id: &AudioId) -> Result<(), AudioError>;
    
    /// Set global volume
    fn set_global_volume(&mut self, volume: f32);
    
    /// Generate waveform data for visualization
    fn generate_waveform(&mut self, audio_id: &AudioId, fps: f32) -> Result<WaveformData, AudioError>;
    
    /// Check if audio is currently playing
    fn is_playing(&self, audio_id: &AudioId) -> bool;
}

/// Audio-related errors
#[derive(Debug, Clone)]
pub enum AudioError {
    FileNotFound,
    UnsupportedFormat,
    DecodingError(String),
    PlaybackError(String),
    AudioNotLoaded,
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::FileNotFound => write!(f, "Audio file not found"),
            AudioError::UnsupportedFormat => write!(f, "Unsupported audio format"),
            AudioError::DecodingError(msg) => write!(f, "Audio decoding error: {}", msg),
            AudioError::PlaybackError(msg) => write!(f, "Audio playback error: {}", msg),
            AudioError::AudioNotLoaded => write!(f, "Audio not loaded"),
        }
    }
}

impl std::error::Error for AudioError {}