//! Script editor templates

pub const LOOP_ANIMATION: &str = r#"// Loop Animation Script
// This script will make the timeline loop continuously

fn on_frame_enter(frame) {
    if frame == timeline.total_frames() {
        timeline.goto_and_play(1);
    }
}

// Register the frame handler
timeline.on_frame(on_frame_enter);
"#;

pub const STOP_AT_FRAME: &str = r#"// Stop at Frame Script
// This script will stop the timeline at a specific frame

let target_frame = 30; // Change this to your desired frame

fn on_frame_enter(frame) {
    if frame == target_frame {
        timeline.stop();
    }
}

// Register the frame handler
timeline.on_frame(on_frame_enter);
"#;

pub const ANIMATE_OBJECT: &str = r#"// Animate Object Script
// This script animates an object's properties over time

let target = stage.get_item("my_object"); // Change to your object's name
let start_x = target.x;
let start_y = target.y;
let duration = 30; // frames

fn on_frame_enter(frame) {
    if frame <= duration {
        let progress = frame / duration;
        target.x = start_x + (100 * progress); // Move 100 pixels right
        target.y = start_y + (50 * Math.sin(progress * Math.PI)); // Sine wave motion
        target.rotation = 360 * progress; // Full rotation
        target.alpha = 1.0 - (progress * 0.5); // Fade to 50%
    }
}

timeline.on_frame(on_frame_enter);
"#;

pub const CREATE_OBJECT: &str = r#"// Create Object Script
// This script creates a new object on the stage

// Create a new rectangle
let rect = stage.create_rectangle({
    x: 100,
    y: 100,
    width: 50,
    height: 50,
    color: 0xFF0000, // Red
    name: "dynamic_rect"
});

// Animate the created object
fn on_frame_enter(frame) {
    rect.rotation = frame * 2;
    rect.scale_x = 1.0 + Math.sin(frame * 0.1) * 0.2;
    rect.scale_y = rect.scale_x;
}

timeline.on_frame(on_frame_enter);
"#;