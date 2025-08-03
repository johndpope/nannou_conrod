//! Test example for the fixed timeline implementation
//! Run with: cargo run --example test_fixed_timeline

use eframe::egui::{self, UiBuilder};
use nannou_timeline::{timeline_egui_fixed::Timeline, TimelineConfig, ui::MockRiveEngine, RiveEngine};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 600.0])
            .with_title("Test Fixed Timeline"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Test Fixed Timeline",
        options,
        Box::new(|_cc| Ok(Box::new(TestApp::default()))),
    )
}

struct TestApp {
    timeline: Timeline,
    engine: Box<dyn RiveEngine>,
}

impl Default for TestApp {
    fn default() -> Self {
        Self {
            timeline: Timeline::new(),
            engine: Box::new(MockRiveEngine::new()),
        }
    }
}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Fixed Timeline Test");
            ui.separator();
            
            let available_rect = ui.available_rect_before_wrap();
            
            // Ensure minimum size to prevent crash
            let timeline_rect = egui::Rect::from_min_size(
                available_rect.min,
                egui::vec2(
                    available_rect.width().max(400.0),
                    available_rect.height().max(300.0)
                ),
            );
            
            ui.scope_builder(UiBuilder::new().max_rect(timeline_rect), |ui| {
                // Set clip rect to ensure no overflow
                ui.set_clip_rect(timeline_rect);
                self.timeline.show(ui, &mut self.engine);
            });
        });
    }
}