//! Example demonstrating the Flash-inspired timeline widget with egui

use nannou::prelude::*;
use nannou_egui::{self, egui};
use nannou_timeline::{Timeline, TimelineConfig, RiveEngine, ui::MockRiveEngine};

struct Model {
    timeline: Timeline,
    rive_engine: Box<dyn RiveEngine>,
    egui: nannou_egui::Egui,
}

fn main() {
    nannou::app(model)
        .update(update)
        .run();
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .title("Flash-inspired Timeline Demo")
        .size(1200, 600)
        .raw_event(raw_window_event)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .mouse_pressed(mouse_pressed)
        .mouse_moved(mouse_moved)
        .mouse_released(mouse_released)
        .mouse_wheel(mouse_wheel)
        .view(view)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let egui = nannou_egui::Egui::from_window(&window);

    Model {
        timeline: Timeline::new(),
        rive_engine: Box::new(MockRiveEngine::new()),
        egui,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let Model {
        ref mut egui,
        ref mut timeline,
        ref mut rive_engine,
        ..
    } = *model;

    egui.set_elapsed_time(update.since_start);
    
    let ctx = egui.begin_frame();

    // Show the timeline in a central panel
    egui::CentralPanel::default()
        .show(&ctx, |ui| {
            ui.heading("Flash-inspired Timeline");
            ui.separator();
            
            // Add the timeline widget
            timeline.show(ui, rive_engine);
            
            ui.separator();
            ui.label("Instructions:");
            ui.label("• Click on layers to select them");
            ui.label("• Drag the playhead to scrub through frames");
            ui.label("• Use play/pause buttons to control playback");
            ui.label("• Zoom in/out with the + and - buttons");
            ui.label("• All interactions are logged to console (stubbed functionality)");
        });

    // You can also add additional panels
    egui::Window::new("Timeline Info")
        .default_pos(egui::pos2(10.0, 10.0))
        .show(&ctx, |ui| {
            ui.label(format!("Current Frame: {}", rive_engine.get_current_frame()));
            ui.label(format!("Total Frames: {}", rive_engine.get_total_frames()));
            ui.label(format!("FPS: {:.1}", rive_engine.get_fps()));
            ui.separator();
            ui.label("Layers:");
            for layer in rive_engine.get_layers() {
                ui.label(format!(
                    "  {} - Visible: {}, Locked: {}",
                    layer.name, layer.visible, layer.locked
                ));
            }
        });
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(DARK_GRAY);
    
    // Draw egui
    model.egui.draw_to_frame(&frame).unwrap();
}

// Handle raw window events for egui
fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn key_pressed(_app: &App, _model: &mut Model, _key: Key) {}

fn key_released(_app: &App, _model: &mut Model, _key: Key) {}

fn mouse_pressed(_app: &App, _model: &mut Model, _button: MouseButton) {}

fn mouse_moved(_app: &App, _model: &mut Model, _pos: Point2) {}

fn mouse_released(_app: &App, _model: &mut Model, _button: MouseButton) {}

fn mouse_wheel(_app: &App, _model: &mut Model, _delta: MouseScrollDelta, _phase: TouchPhase) {}