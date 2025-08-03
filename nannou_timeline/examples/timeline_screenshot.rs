//! Generate a visual representation of the timeline state

use nannou_timeline::{Timeline, TimelineConfig, TimelineStyle, ui::MockRiveEngine, FpsPreset, RiveEngine};
use egui::Color32;

fn main() {
    println!("Timeline Feature Demonstration");
    println!("=============================\n");
    
    // Create timeline with custom config
    let config = TimelineConfig {
        layer_panel_width: 200.0,
        ruler_height: 30.0,
        controls_height: 50.0,
        default_track_height: 30.0,
        frame_width: 12.0,
        fps: FpsPreset::Film, // 24 fps like Flash
        frame_labels: vec![
            nannou_timeline::FrameLabel::new(0, "Start"),
            nannou_timeline::FrameLabel::new(24, "1 sec"),
            nannou_timeline::FrameLabel::new(48, "2 sec"),
        ],
        style: TimelineStyle {
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
        },
        snap: nannou_timeline::SnapConfig::default(),
    };
    
    let mut timeline = Timeline::with_config(config);
    let engine = MockRiveEngine::new();
    
    // Display timeline features
    println!("📽️  TIMELINE FEATURES:");
    println!("====================\n");
    
    println!("⏱️  Frame-based Time System:");
    println!("   - Current FPS: {}", timeline.config.fps.label());
    println!("   - Frame width: {}px", timeline.config.frame_width);
    println!("   - Total frames: {}", engine.get_total_frames());
    println!("   - Current frame: {}", engine.get_current_frame());
    
    println!("\n📊 Layers:");
    for layer in engine.get_layers() {
        println!("   - {} ({}{})", 
            layer.name,
            if layer.visible { "👁️ " } else { "   " },
            if layer.locked { "🔒" } else { "  " }
        );
    }
    
    println!("\n🎬 Frame Operations (Right-click menu):");
    println!("   - Insert Frame (F5)");
    println!("   - Remove Frame (Shift+F5)");
    println!("   - Insert Keyframe (F6)");
    println!("   - Clear Keyframe (Shift+F6)");
    println!("   - Create Motion Tween");
    println!("   - Create Shape Tween");
    println!("   - Insert Frame Label");
    
    println!("\n🔍 Navigation:");
    println!("   - Zoom: {}% (Ctrl/Cmd + Wheel)", (timeline.state.zoom_level * 100.0) as i32);
    println!("   - Horizontal scroll: Shift + Wheel");
    println!("   - Vertical scroll: Mouse wheel");
    println!("   - Playhead position: Frame {}", timeline.state.playhead_frame);
    
    println!("\n⌨️  Keyboard Shortcuts:");
    println!("   - Space: Play/Pause");
    println!("   - Home/End: First/Last frame");
    println!("   - Left/Right: Previous/Next frame");
    println!("   - F5/F6: Frame operations");
    
    println!("\n✨ New Features Implemented:");
    println!("   ✅ Frame-based time (replaced musical bars/beats)");
    println!("   ✅ FPS presets (Film, PAL, NTSC, Web, High)");
    println!("   ✅ Right-click context menus");
    println!("   ✅ Smooth scrolling with ScrollArea");
    println!("   ✅ Zoom centered on cursor (0.1x - 5x)");
    println!("   ✅ Viewport culling for performance");
    println!("   ✅ Synced layer/frame grid scrolling");
    
    // Simulate some timeline state
    timeline.state.zoom_level = 1.5;
    timeline.state.playhead_frame = 48;
    timeline.state.scroll_x = 100.0;
    
    println!("\n📍 Current State:");
    println!("   - Playhead at frame: {}", timeline.state.playhead_frame);
    println!("   - Zoom level: {}x", timeline.state.zoom_level);
    println!("   - Horizontal scroll: {}px", timeline.state.scroll_x);
    println!("   - Playing: {}", if timeline.state.is_playing { "Yes" } else { "No" });
}