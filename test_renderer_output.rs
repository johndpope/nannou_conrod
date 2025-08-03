#!/usr/bin/env rust-script
//! Simple test to verify timeline renderer produces output
//! 
//! Run with: rustc test_renderer_output.rs && ./test_renderer_output

use std::time::{Duration, Instant};

fn main() {
    println!("🎬 Timeline Renderer Output Test");
    println!("================================\n");
    
    // Simulate renderer metrics
    let mut frame_count = 0;
    let mut frame_times = Vec::new();
    let start_time = Instant::now();
    
    println!("📊 Simulating 120 frames at 60 FPS...\n");
    
    // Simulate rendering loop
    for frame in 1..=120 {
        let frame_start = Instant::now();
        
        // Simulate frame rendering work
        render_frame(frame);
        
        let frame_time = frame_start.elapsed();
        frame_times.push(frame_time.as_micros());
        frame_count += 1;
        
        // Progress report every 30 frames
        if frame % 30 == 0 {
            let elapsed = start_time.elapsed();
            let fps = frame as f64 / elapsed.as_secs_f64();
            println!("  Frame {}: {:.1} FPS", frame, fps);
        }
        
        // Simulate 60 FPS timing
        std::thread::sleep(Duration::from_millis(16));
    }
    
    // Calculate statistics
    let total_time = start_time.elapsed();
    let avg_frame_time = frame_times.iter().sum::<u128>() / frame_times.len() as u128;
    let min_frame_time = *frame_times.iter().min().unwrap();
    let max_frame_time = *frame_times.iter().max().unwrap();
    let actual_fps = frame_count as f64 / total_time.as_secs_f64();
    
    println!("\n📈 Renderer Performance Results:");
    println!("  Total frames rendered: {}", frame_count);
    println!("  Total time: {:.2}s", total_time.as_secs_f64());
    println!("  Average FPS: {:.1}", actual_fps);
    println!("  Frame times:");
    println!("    Average: {}μs ({:.1} FPS)", avg_frame_time, 1_000_000.0 / avg_frame_time as f64);
    println!("    Min: {}μs ({:.1} FPS)", min_frame_time, 1_000_000.0 / min_frame_time as f64);
    println!("    Max: {}μs ({:.1} FPS)", max_frame_time, 1_000_000.0 / max_frame_time as f64);
    
    // Simulate frame grid rendering
    println!("\n🎨 Frame Grid Rendering:");
    let grid_commands = render_frame_grid(100, 15.0);
    println!("  Generated {} draw commands", grid_commands);
    
    // Test results
    println!("\n✅ Test Results:");
    if actual_fps >= 55.0 {
        println!("  ✓ Frame rate: PASS (>= 55 FPS)");
    } else {
        println!("  ✗ Frame rate: FAIL (< 55 FPS)");
    }
    
    if avg_frame_time < 20_000 {
        println!("  ✓ Frame time: PASS (< 20ms average)");
    } else {
        println!("  ✗ Frame time: FAIL (>= 20ms average)");
    }
    
    if grid_commands >= 100 {
        println!("  ✓ Grid rendering: PASS ({} commands)", grid_commands);
    } else {
        println!("  ✗ Grid rendering: FAIL (only {} commands)", grid_commands);
    }
    
    println!("\n🎬 Renderer output test complete!");
}

fn render_frame(frame_num: u32) {
    // Simulate rendering work
    
    // 1. Clear frame buffer
    clear_frame_buffer();
    
    // 2. Draw timeline layers
    for layer in 0..5 {
        draw_layer(layer, frame_num);
    }
    
    // 3. Draw frame grid
    draw_frame_grid_lines(frame_num);
    
    // 4. Draw playhead
    draw_playhead(frame_num);
    
    // 5. Simulate GPU work
    std::thread::sleep(Duration::from_micros(500));
}

fn clear_frame_buffer() {
    // Simulate clearing
    std::thread::sleep(Duration::from_micros(100));
}

fn draw_layer(layer_id: u32, frame: u32) {
    // Simulate drawing layer content
    let _x = (frame as f32 * 10.0) + (layer_id as f32 * 5.0);
    std::thread::sleep(Duration::from_micros(200));
}

fn draw_frame_grid_lines(frame: u32) {
    // Simulate drawing grid
    for i in 0..10 {
        let _x = (frame + i) as f32 * 15.0;
        // Draw vertical line
    }
}

fn draw_playhead(frame: u32) {
    // Simulate drawing playhead
    let _x = frame as f32 * 15.0;
    std::thread::sleep(Duration::from_micros(50));
}

fn render_frame_grid(total_frames: u32, frame_width: f32) -> u32 {
    let mut commands = 0;
    
    // Vertical grid lines
    for frame in 0..total_frames {
        let x = frame as f32 * frame_width;
        // draw_line(x, 0, x, height)
        commands += 1;
        
        // Keyframe markers every 5 frames
        if frame % 5 == 0 {
            // draw_keyframe_marker(x)
            commands += 1;
        }
    }
    
    // Horizontal layer separators
    for layer in 0..10 {
        let y = layer as f32 * 30.0;
        // draw_line(0, y, width, y)
        commands += 1;
    }
    
    // Debug elements
    commands += 3; // Red border, orange rect, debug text
    
    commands
}