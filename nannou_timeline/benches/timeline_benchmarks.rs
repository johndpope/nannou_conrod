//! Performance benchmarks for the timeline widget
//! 
//! Measures rendering performance, interaction responsiveness, and scalability

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use egui::{Context, CentralPanel, Ui};
use nannou_timeline::{Timeline, TimelineConfig, ui::MockRiveEngine, RiveEngine, LayerId, LayerType};

/// Create a timeline with specified number of layers and frames
fn create_complex_timeline(num_layers: usize, num_frames: u32) -> (Timeline, Box<dyn RiveEngine>) {
    let config = TimelineConfig {
        frame_width: 10.0,
        default_track_height: 30.0,
        ..Default::default()
    };
    
    let timeline = Timeline::with_config(config);
    let mut engine = Box::new(MockRiveEngine::new());
    
    // Add layers
    for i in 0..num_layers {
        let layer_name = format!("Layer {}", i);
        engine.add_layer(layer_name, LayerType::Normal);
        
        // Add keyframes at intervals
        let layer_id = LayerId::new(&format!("layer_{}", i));
        for frame in (0..num_frames).step_by(10) {
            engine.insert_keyframe(layer_id.clone(), frame);
        }
    }
    
    (timeline, engine)
}

/// Benchmark timeline rendering with different layer counts
fn bench_timeline_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeline_rendering");
    
    for layer_count in [10, 50, 100, 200] {
        group.bench_with_input(
            BenchmarkId::new("layers", layer_count),
            &layer_count,
            |b, &layer_count| {
                let (mut timeline, mut engine) = create_complex_timeline(layer_count, 100);
                let ctx = Context::default();
                
                b.iter(|| {
                    ctx.begin_frame(egui::RawInput::default());
                    CentralPanel::default().show(&ctx, |ui| {
                        timeline.show(ui, &mut engine);
                    });
                    ctx.end_frame();
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark frame selection operations
fn bench_frame_selection(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_selection");
    
    for selection_size in [10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::new("frames", selection_size),
            &selection_size,
            |b, &selection_size| {
                let (mut timeline, _engine) = create_complex_timeline(10, 1000);
                
                b.iter(|| {
                    // Clear selection
                    timeline.state.selected_frames.clear();
                    
                    // Select multiple frames
                    for frame in 0..selection_size {
                        timeline.state.selected_frames.push(black_box(frame));
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark keyframe operations
fn bench_keyframe_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyframe_operations");
    
    // Benchmark keyframe insertion
    group.bench_function("insert_keyframe", |b| {
        let (_timeline, mut engine) = create_complex_timeline(10, 100);
        let layer_id = LayerId::new("test_layer");
        let mut frame = 0u32;
        
        b.iter(|| {
            engine.insert_keyframe(layer_id.clone(), black_box(frame));
            frame = (frame + 1) % 100;
        });
    });
    
    // Benchmark keyframe deletion
    group.bench_function("delete_keyframe", |b| {
        let (_timeline, mut engine) = create_complex_timeline(10, 100);
        let layer_id = LayerId::new("test_layer");
        
        // Pre-populate keyframes
        for frame in 0..100 {
            engine.insert_keyframe(layer_id.clone(), frame);
        }
        
        let mut frame = 0u32;
        b.iter(|| {
            engine.delete_keyframe(layer_id.clone(), black_box(frame));
            frame = (frame + 1) % 100;
        });
    });
    
    // Benchmark keyframe copy/paste
    group.bench_function("copy_paste_keyframe", |b| {
        let (_timeline, mut engine) = create_complex_timeline(10, 100);
        let layer_id = LayerId::new("test_layer");
        
        // Add source keyframe
        engine.insert_keyframe(layer_id.clone(), 0);
        
        let mut target_frame = 10u32;
        b.iter(|| {
            if let Some(data) = engine.copy_keyframe(layer_id.clone(), 0) {
                engine.paste_keyframe(layer_id.clone(), black_box(target_frame), data);
            }
            target_frame = (target_frame + 1) % 100;
        });
    });
    
    group.finish();
}

/// Benchmark zoom operations
fn bench_zoom_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("zoom_operations");
    
    for zoom_level in [0.1, 0.5, 1.0, 2.0, 5.0] {
        group.bench_with_input(
            BenchmarkId::new("zoom", zoom_level),
            &zoom_level,
            |b, &zoom_level| {
                let (mut timeline, mut engine) = create_complex_timeline(50, 500);
                let ctx = Context::default();
                
                b.iter(|| {
                    timeline.state.zoom_level = black_box(zoom_level);
                    
                    ctx.begin_frame(egui::RawInput::default());
                    CentralPanel::default().show(&ctx, |ui| {
                        timeline.show(ui, &mut engine);
                    });
                    ctx.end_frame();
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark scrolling performance
fn bench_scrolling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scrolling");
    
    group.bench_function("horizontal_scroll", |b| {
        let (mut timeline, mut engine) = create_complex_timeline(50, 1000);
        let ctx = Context::default();
        let mut scroll_x = 0.0f32;
        
        b.iter(|| {
            timeline.state.scroll_x = black_box(scroll_x);
            scroll_x = (scroll_x + 10.0) % 1000.0;
            
            ctx.begin_frame(egui::RawInput::default());
            CentralPanel::default().show(&ctx, |ui| {
                timeline.show(ui, &mut engine);
            });
            ctx.end_frame();
        });
    });
    
    group.bench_function("vertical_scroll", |b| {
        let (mut timeline, mut engine) = create_complex_timeline(100, 100);
        let ctx = Context::default();
        let mut scroll_y = 0.0f32;
        
        b.iter(|| {
            timeline.state.scroll_y = black_box(scroll_y);
            scroll_y = (scroll_y + 5.0) % 500.0;
            
            ctx.begin_frame(egui::RawInput::default());
            CentralPanel::default().show(&ctx, |ui| {
                timeline.show(ui, &mut engine);
            });
            ctx.end_frame();
        });
    });
    
    group.finish();
}

/// Benchmark snap calculations
fn bench_snap_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("snap_calculations");
    
    group.bench_function("snap_to_frame", |b| {
        let (timeline, _engine) = create_complex_timeline(10, 100);
        let modifiers = egui::Modifiers::default();
        let mut position = 0.0f32;
        
        b.iter(|| {
            let snapped = timeline.snap_position(black_box(position), &modifiers);
            position = (position + 1.3) % 1000.0;
            black_box(snapped);
        });
    });
    
    group.bench_function("snap_with_shift", |b| {
        let (timeline, _engine) = create_complex_timeline(10, 100);
        let mut modifiers = egui::Modifiers::default();
        modifiers.shift = true;
        let mut position = 0.0f32;
        
        b.iter(|| {
            let snapped = timeline.snap_position(black_box(position), &modifiers);
            position = (position + 1.3) % 1000.0;
            black_box(snapped);
        });
    });
    
    group.finish();
}

/// Benchmark playback performance
fn bench_playback(c: &mut Criterion) {
    let mut group = c.benchmark_group("playback");
    
    group.bench_function("frame_advance", |b| {
        let (mut timeline, mut engine) = create_complex_timeline(20, 500);
        timeline.state.is_playing = true;
        
        b.iter(|| {
            let current_frame = timeline.state.playhead_frame;
            let next_frame = (current_frame + 1) % 500;
            timeline.state.playhead_frame = black_box(next_frame);
            engine.seek(next_frame);
        });
    });
    
    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    
    group.bench_function("layer_addition", |b| {
        b.iter(|| {
            let (_timeline, mut engine) = create_complex_timeline(0, 100);
            
            for i in 0..10 {
                let layer_name = format!("Layer {}", i);
                engine.add_layer(black_box(layer_name), LayerType::Normal);
            }
        });
    });
    
    group.bench_function("selection_allocation", |b| {
        let (mut timeline, _engine) = create_complex_timeline(10, 100);
        
        b.iter(|| {
            timeline.state.selected_frames.clear();
            timeline.state.selected_frames.reserve(100);
            
            for frame in 0..100 {
                timeline.state.selected_frames.push(black_box(frame));
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_timeline_rendering,
    bench_frame_selection,
    bench_keyframe_operations,
    bench_zoom_operations,
    bench_scrolling,
    bench_snap_calculations,
    bench_playback,
    bench_memory_patterns
);

criterion_main!(benches);