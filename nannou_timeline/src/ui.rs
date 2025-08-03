//! UI helper utilities for the timeline

use egui::{*, self};

/// Helper to create consistent button styles
pub fn timeline_button(ui: &mut Ui, text: &str) -> Response {
    ui.add(Button::new(text).min_size(vec2(30.0, 20.0)))
}

/// Helper to create icon buttons
pub fn icon_button(ui: &mut Ui, icon: &str, size: f32) -> Response {
    ui.add(Button::new(icon).min_size(vec2(size, size)))
}

/// Helper to draw a separator line
pub fn separator_line(ui: &mut Ui, vertical: bool) {
    let rect = ui.available_rect_before_wrap();
    let stroke = ui.style().visuals.widgets.noninteractive.bg_stroke;
    
    if vertical {
        ui.painter().line_segment(
            [rect.center_top(), rect.center_bottom()],
            stroke,
        );
    } else {
        ui.painter().line_segment(
            [rect.left_center(), rect.right_center()],
            stroke,
        );
    }
}

/// Mock Rive engine for testing
pub struct MockRiveEngine {
    layers: Vec<crate::layer::LayerInfo>,
    current_frame: u32,
    total_frames: u32,
    fps: f32,
    is_playing: bool,
}

impl MockRiveEngine {
    pub fn new() -> Self {
        Self {
            layers: crate::layer::create_mock_layers(),
            current_frame: 0,
            total_frames: 100,
            fps: 24.0,
            is_playing: false,
        }
    }
}

impl crate::RiveEngine for MockRiveEngine {
    fn get_layers(&self) -> Vec<crate::layer::LayerInfo> {
        self.layers.clone()
    }

    fn get_frame_data(&self, layer_id: crate::LayerId, frame: u32) -> crate::frame::FrameData {
        crate::frame::create_mock_frame_data(&layer_id, frame)
    }

    fn play(&mut self) {
        self.is_playing = true;
        println!("MockRiveEngine: Playing");
    }

    fn pause(&mut self) {
        self.is_playing = false;
        println!("MockRiveEngine: Paused");
    }

    fn seek(&mut self, frame: u32) {
        self.current_frame = frame.min(self.total_frames);
        println!("MockRiveEngine: Seeking to frame {}", self.current_frame);
    }

    fn get_current_frame(&self) -> u32 {
        self.current_frame
    }

    fn get_total_frames(&self) -> u32 {
        self.total_frames
    }

    fn get_fps(&self) -> f32 {
        self.fps
    }
    
    fn insert_frame(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Inserting frame at {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would modify the timeline data
    }
    
    fn remove_frame(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Removing frame at {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would modify the timeline data
    }
    
    fn insert_keyframe(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Inserting keyframe at {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would create a new keyframe
    }
    
    fn clear_keyframe(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Clearing keyframe at {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would remove the keyframe
    }
    
    fn create_motion_tween(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Creating motion tween at {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would create a motion tween between keyframes
    }
    
    fn create_shape_tween(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Creating shape tween at {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would create a shape tween between keyframes
    }
    
    // New keyframe manipulation methods
    fn move_keyframe(&mut self, layer_id: crate::LayerId, from_frame: u32, to_frame: u32) {
        println!("MockRiveEngine: Moving keyframe from frame {} to frame {} on layer {:?}", from_frame, to_frame, layer_id);
        // In a real implementation, this would move keyframe data in the timeline
    }
    
    fn copy_keyframe(&mut self, layer_id: crate::LayerId, frame: u32) -> Option<crate::frame::FrameData> {
        println!("MockRiveEngine: Copying keyframe at frame {} on layer {:?}", frame, layer_id);
        // Return mock frame data for copy operation
        Some(crate::frame::create_mock_frame_data(&layer_id, frame))
    }
    
    fn paste_keyframe(&mut self, layer_id: crate::LayerId, frame: u32, data: crate::frame::FrameData) {
        println!("MockRiveEngine: Pasting keyframe at frame {} on layer {:?} with data {:?}", frame, layer_id, data);
        // In a real implementation, this would paste the frame data
    }
    
    fn delete_keyframe(&mut self, layer_id: crate::LayerId, frame: u32) {
        println!("MockRiveEngine: Deleting keyframe at frame {} on layer {:?}", frame, layer_id);
        // In a real implementation, this would remove the keyframe
    }
    
    // Property manipulation methods
    fn set_property(&mut self, layer_id: crate::LayerId, frame: u32, property: &str, value: bool) {
        println!("MockRiveEngine: Setting property '{}' to {} at frame {} on layer {:?}", property, value, frame, layer_id);
        // In a real implementation, this would set layer properties at specific frames
    }
    
    fn get_property(&self, layer_id: crate::LayerId, frame: u32, property: &str) -> bool {
        println!("MockRiveEngine: Getting property '{}' at frame {} on layer {:?}", property, frame, layer_id);
        // Return mock property value
        match property {
            "visible" => true,
            "locked" => false,
            _ => false,
        }
    }
    
    fn rename_layer(&mut self, layer_id: crate::LayerId, new_name: String) {
        println!("MockRiveEngine: Renaming layer {:?} to '{}'", layer_id, new_name);
        // Find and rename the layer
        if let Some(layer) = self.layers.iter_mut().find(|l| l.id == layer_id) {
            layer.name = new_name;
        }
    }
}

/// Mock audio engine for testing and demo
pub struct MockAudioEngine {
    /// Loaded audio sources
    loaded_audio: std::collections::HashMap<crate::audio::AudioId, crate::audio::AudioSource>,
    /// Generated waveforms
    waveforms: std::collections::HashMap<crate::audio::AudioId, crate::audio::WaveformData>,
    /// Currently playing audio
    playing: std::collections::HashSet<crate::audio::AudioId>,
    /// Global volume
    global_volume: f32,
}

impl MockAudioEngine {
    pub fn new() -> Self {
        Self {
            loaded_audio: std::collections::HashMap::new(),
            waveforms: std::collections::HashMap::new(),
            playing: std::collections::HashSet::new(),
            global_volume: 1.0,
        }
    }
    
    /// Create mock audio source for testing
    pub fn create_mock_audio(&mut self, filename: &str, duration: f32) -> crate::audio::AudioSource {
        let mut source = crate::audio::AudioSource::new(std::path::PathBuf::from(filename));
        source.duration = duration;
        source.loaded = true;
        
        // Store in loaded audio
        self.loaded_audio.insert(source.id.clone(), source.clone());
        
        source
    }
    
    /// Generate mock waveform data
    fn generate_mock_waveform(&self, audio_id: &crate::audio::AudioId, fps: f32) -> crate::audio::WaveformData {
        let mut waveform = crate::audio::WaveformData::new(audio_id.clone(), fps);
        
        if let Some(source) = self.loaded_audio.get(audio_id) {
            let total_frames = (source.duration * fps) as usize;
            
            // Generate synthetic waveform data (sine wave with noise)
            for frame in 0..total_frames {
                let time = frame as f32 / fps;
                let frequency = 440.0; // A4 note
                let amplitude = 0.5;
                
                // Create synthetic audio signal
                let base_wave = amplitude * (2.0 * std::f32::consts::PI * frequency * time).sin();
                
                // Add some noise and variation
                let noise = (time * 13.7).sin() * 0.1;
                let envelope = 1.0 - (time / source.duration); // Fade out
                
                let sample = (base_wave + noise) * envelope;
                let peak = sample.abs();
                
                waveform.peaks.push((-peak, peak));
            }
            
            waveform.complete = true;
        }
        
        waveform
    }
}

impl crate::audio::AudioEngine for MockAudioEngine {
    fn load_audio(&mut self, file_path: &std::path::Path) -> Result<crate::audio::AudioSource, crate::audio::AudioError> {
        println!("MockAudioEngine: Loading audio from {:?}", file_path);
        
        // Simulate loading different audio files
        let filename = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown.wav");
            
        let duration = match filename {
            f if f.contains("short") => 2.5,
            f if f.contains("loop") => 4.0,
            f if f.contains("music") => 30.0,
            _ => 10.0,
        };
        
        let mut source = crate::audio::AudioSource::new(file_path.to_path_buf());
        source.duration = duration;
        source.loaded = true;
        
        // Detect mock file format
        match file_path.extension().and_then(|e| e.to_str()) {
            Some("mp3") | Some("wav") | Some("ogg") | Some("m4a") => {
                self.loaded_audio.insert(source.id.clone(), source.clone());
                Ok(source)
            }
            _ => Err(crate::audio::AudioError::UnsupportedFormat),
        }
    }
    
    fn unload_audio(&mut self, audio_id: &crate::audio::AudioId) -> Result<(), crate::audio::AudioError> {
        println!("MockAudioEngine: Unloading audio {:?}", audio_id);
        self.loaded_audio.remove(audio_id);
        self.waveforms.remove(audio_id);
        self.playing.remove(audio_id);
        Ok(())
    }
    
    fn play_segment(&mut self, audio_id: &crate::audio::AudioId, start_time: f32, duration: f32, volume: f32) -> Result<(), crate::audio::AudioError> {
        if !self.loaded_audio.contains_key(audio_id) {
            return Err(crate::audio::AudioError::AudioNotLoaded);
        }
        
        println!("MockAudioEngine: Playing segment of {:?} from {:.2}s for {:.2}s at volume {:.2}", 
                 audio_id, start_time, duration, volume);
        
        self.playing.insert(audio_id.clone());
        Ok(())
    }
    
    fn stop_audio(&mut self, audio_id: &crate::audio::AudioId) -> Result<(), crate::audio::AudioError> {
        println!("MockAudioEngine: Stopping audio {:?}", audio_id);
        self.playing.remove(audio_id);
        Ok(())
    }
    
    fn set_global_volume(&mut self, volume: f32) {
        self.global_volume = volume.clamp(0.0, 1.0);
        println!("MockAudioEngine: Setting global volume to {:.2}", self.global_volume);
    }
    
    fn generate_waveform(&mut self, audio_id: &crate::audio::AudioId, fps: f32) -> Result<crate::audio::WaveformData, crate::audio::AudioError> {
        if !self.loaded_audio.contains_key(audio_id) {
            return Err(crate::audio::AudioError::AudioNotLoaded);
        }
        
        println!("MockAudioEngine: Generating waveform for {:?} at {} fps", audio_id, fps);
        
        let waveform = self.generate_mock_waveform(audio_id, fps);
        self.waveforms.insert(audio_id.clone(), waveform.clone());
        
        Ok(waveform)
    }
    
    fn is_playing(&self, audio_id: &crate::audio::AudioId) -> bool {
        self.playing.contains(audio_id)
    }
}