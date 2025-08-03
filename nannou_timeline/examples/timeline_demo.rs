//! Demo of the Flash-inspired timeline widget

use eframe::egui;
use nannou_timeline::{Timeline, ui::MockRiveEngine};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1200.0, 600.0)),
        ..Default::default()
    };
    
    eframe::run_native(
        "Timeline Demo - Flash-style Animation Timeline",
        options,
        Box::new(|_cc| Box::new(TimelineApp::default())),
    )
}

struct TimelineApp {
    timeline: Timeline,
    engine: Box<dyn nannou_timeline::RiveEngine>,
}

impl Default for TimelineApp {
    fn default() -> Self {
        Self {
            timeline: Timeline::new(),
            engine: Box::new(MockRiveEngine::new()),
        }
    }
}

impl eframe::App for TimelineApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Flash-style Timeline Demo");
            
            ui.separator();
            
            // Instructions
            ui.horizontal(|ui| {
                ui.label("Instructions:");
                ui.label("• Right-click in frame grid for context menu");
                ui.label("• Ctrl/Cmd + Wheel to zoom");
                ui.label("• Shift + Wheel for horizontal scroll");
                ui.label("• F5/F6 for frame operations");
            });
            
            ui.separator();
            
            // Show the timeline
            self.timeline.show(ui, &mut self.engine);
            
            // Request repaint for smooth animations
            ctx.request_repaint();
        });
    }
}