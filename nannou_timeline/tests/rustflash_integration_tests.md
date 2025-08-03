# RustFlash Editor Engine & nannou_conrod Timeline IDE Integration Test Suite

## Test Environment Setup

### Prerequisites
- Rust 1.75+ with cargo
- RustFlash Editor engine running as separate process or integrated library
- nannou_conrod timeline IDE compiled and ready
- Mock data: Sample Rive animations with multiple layers and scripted frames
- Performance monitoring tools (cargo-flamegraph, tracy, etc.)

### Test Configuration
```toml
[test-config]
target_fps = 60
max_frame_latency_ms = 16.67  # 1 frame at 60 FPS
max_script_execution_ms = 5.0
timeline_sync_tolerance_ms = 1.0
```

## 1. Frame Rate and Timing Synchronization Tests

### Test Case 1.1: Basic 60 FPS Playback
**Setup:**
- Create timeline with 300 frames (5 seconds at 60 FPS)
- Add 3 animated layers with keyframes every 15 frames
- No scripts attached

**Steps:**
1. Start playback from frame 0
2. Monitor frame updates for 5 seconds
3. Record actual frame times and compare to expected

**Expected Results:**
- Each frame renders within 16.67ms ± 2ms
- No frame drops or stutters
- Playhead position matches engine's current frame exactly

**Acceptance Criteria:**
- 99% of frames render on time
- Maximum frame latency < 20ms
- Zero desync between timeline and engine

### Test Case 1.2: Variable Frame Rate Handling
**Setup:**
- Configure timeline for 24 FPS
- Engine running at 60 FPS internally

**Steps:**
1. Play animation for 120 frames (5 seconds at 24 FPS)
2. Verify frame interpolation/skipping logic
3. Check visual smoothness

**Expected Results:**
- Timeline advances 24 frames per second
- Engine properly interpolates between timeline frames
- No visual stuttering despite FPS mismatch

### Test Case 1.3: Heavy Load Performance
**Setup:**
- 50 layers with complex animations
- 10 Rhai scripts running per frame
- Multiple shape tweens and effects

**Steps:**
1. Start playback and monitor performance
2. Profile CPU/GPU usage
3. Check for frame drops

**Expected Results:**
- Maintains 60 FPS with < 10% frame drops
- Scripts complete within 5ms budget
- Graceful degradation if overloaded

**Performance Benchmarks:**
```
Frame Budget: 16.67ms
- Engine Update: < 8ms
- Script Execution: < 5ms  
- UI Rendering: < 3ms
- IPC/Sync Overhead: < 1ms
```

## 2. Animation Playback Control Tests

### Test Case 2.1: Play/Pause/Stop Commands
**Setup:**
- Timeline with 100 frames
- Multiple tweened animations

**Steps:**
1. Send play() command → verify animation starts
2. Wait 1 second → send pause() → verify freeze
3. Send play() → verify resume from paused frame
4. Send stop() → verify return to frame 0

**Expected Results:**
- Commands execute within 1 frame (16.67ms)
- State transitions are atomic
- No intermediate/glitched frames

### Test Case 2.2: Frame Scrubbing
**Setup:**
- Timeline with complex animations
- User dragging playhead rapidly

**Steps:**
1. Rapidly seek to frames: 0, 50, 25, 75, 10, 90
2. Verify each seek completes correctly
3. Check for visual artifacts or lag

**Expected Results:**
- Each seek completes in < 50ms
- Display objects update to correct positions
- No visual ghosting or trails

### Test Case 2.3: Loop Playback
**Setup:**
- 60-frame looping animation
- Loop script: `if (currentFrame >= 59) gotoAndPlay(0);`

**Steps:**
1. Start playback
2. Monitor 5 complete loops
3. Check frame timing at loop points

**Expected Results:**
- Seamless loops with no pause
- Consistent frame timing across loops
- Scripts execute at correct frames

## 3. Timeline Scrolling and Navigation Tests

### Test Case 3.1: Horizontal Timeline Scrolling
**Setup:**
- 1000-frame timeline
- Zoom level showing 100 frames

**Steps:**
1. Scroll timeline left/right using scrollbar
2. Use keyboard shortcuts (arrow keys)
3. Use mouse wheel with Shift modifier

**Expected Results:**
- Smooth scrolling at 60 FPS
- Playhead remains visible if playing
- Frame numbers update correctly

### Test Case 3.2: Vertical Layer Scrolling
**Setup:**
- 100 layers in timeline
- Viewport showing 20 layers

**Steps:**
1. Scroll through all layers
2. Verify layer rendering and updates
3. Test with playing animation

**Expected Results:**
- No lag when scrolling layers
- Animations continue playing smoothly
- Selected layers remain highlighted

### Test Case 3.3: Zoom In/Out
**Setup:**
- Standard timeline view

**Steps:**
1. Zoom in to show individual frames clearly
2. Zoom out to show entire timeline
3. Test zoom during playback

**Expected Results:**
- Zoom animations smooth
- Frame rendering adapts to zoom level
- Playhead scales appropriately

## 4. Rhai Script Execution Tests

### Test Case 4.1: Frame Script Execution
**Setup:**
- Scripts on frames 1, 15, 30, 45, 60
- Each script modifies display objects

**Scripts:**
```rhai
// Frame 1
let obj = stage.getChildByName("circle");
obj.x = 100;
obj.y = 100;

// Frame 15  
let obj = stage.getChildByName("circle");
obj.rotation = 45;
obj.alpha = 0.5;

// Frame 30
timeline.gotoAndPlay(45);
```

**Steps:**
1. Play animation from frame 0
2. Verify script execution at each frame
3. Check object property changes

**Expected Results:**
- Scripts execute exactly once per frame
- Properties update immediately
- No script errors or crashes

### Test Case 4.2: Script Performance Limits
**Setup:**
- Heavy computational script
- 5ms execution budget

**Script:**
```rhai
// Intentionally heavy operation
let sum = 0;
for i in 0..10000 {
    sum += i * i;
}
print("Sum: " + sum);
```

**Steps:**
1. Execute script on frame
2. Monitor execution time
3. Check for frame drops

**Expected Results:**
- Script completes or timeouts at 5ms
- Frame still renders on time
- Warning logged for slow scripts

### Test Case 4.3: Script Error Handling
**Setup:**
- Scripts with various errors

**Scripts:**
```rhai
// Syntax error
let x = 

// Runtime error
let obj = stage.getChildByName("nonexistent");
obj.x = 100;  // Should handle gracefully

// Type error
timeline.gotoAndPlay("not a number");
```

**Steps:**
1. Execute each error script
2. Verify error handling
3. Check timeline continues

**Expected Results:**
- Errors logged with frame/line info
- Timeline continues playing
- No crashes or hangs

## 5. Display Object Update Tests

### Test Case 5.1: Property Animation Sync
**Setup:**
- Motion tween animating x, y, rotation, scale, alpha
- 30-frame animation

**Steps:**
1. Play animation
2. Sample object properties every frame
3. Verify smooth interpolation

**Expected Results:**
- Properties interpolate smoothly
- Values match tween curves exactly
- Visual rendering matches properties

### Test Case 5.2: Layer Visibility Toggle
**Setup:**
- 10 layers with animations
- Visibility toggling during playback

**Steps:**
1. Start playback
2. Toggle layer visibility rapidly
3. Verify rendering updates

**Expected Results:**
- Instant visibility changes
- No rendering artifacts
- Hidden layers don't impact performance

### Test Case 5.3: Dynamic Object Creation
**Setup:**
- Script creating objects at runtime

**Script:**
```rhai
for i in 0..10 {
    let obj = ScriptDisplayObject::new("dynamic_" + i);
    obj.x = i * 50;
    obj.y = 100;
    stage.addChild(obj);
}
```

**Steps:**
1. Execute script
2. Verify objects appear
3. Continue animation

**Expected Results:**
- Objects created immediately
- Proper z-ordering maintained
- No memory leaks

## 6. Edge Cases and Error Scenarios

### Test Case 6.1: Rapid Play/Pause Toggle
**Setup:**
- Standard animation

**Steps:**
1. Toggle play/pause 100 times in 1 second
2. Check final state
3. Verify no crashes

**Expected Results:**
- System remains stable
- Final state is deterministic
- No queued commands

### Test Case 6.2: Timeline Overflow
**Setup:**
- Seek beyond total frames

**Steps:**
1. Seek to frame 99999
2. Seek to negative frame
3. Verify handling

**Expected Results:**
- Clamps to valid range
- No crashes or errors
- Playhead at frame 0 or max

### Test Case 6.3: Memory Pressure
**Setup:**
- Large timeline (10000 frames)
- Many layers and objects

**Steps:**
1. Load timeline
2. Play entire animation
3. Monitor memory usage

**Expected Results:**
- Memory usage stable
- No leaks detected
- Graceful handling if OOM

## 7. Integration Test Scenarios

### Test Case 7.1: Full Workflow Test
**Setup:**
- Complete animation project

**Steps:**
1. Load project
2. Add new layer
3. Insert keyframes
4. Add motion tween
5. Attach script
6. Play animation
7. Export/Save

**Expected Results:**
- All operations succeed
- Data persists correctly
- Smooth user experience

### Test Case 7.2: Multi-Window Sync
**Setup:**
- Timeline in main window
- Stage preview in separate window

**Steps:**
1. Play animation
2. Verify windows stay in sync
3. Test with window switching

**Expected Results:**
- Perfect synchronization
- No lag between windows
- Consistent frame display

### Test Case 7.3: Undo/Redo During Playback
**Setup:**
- Animation playing
- User performs edits

**Steps:**
1. During playback, insert keyframe
2. Undo the insertion
3. Redo the insertion
4. Verify playback continues

**Expected Results:**
- Edits apply immediately
- No playback interruption
- Correct undo/redo behavior

## 8. Automated Test Implementation

### Unit Tests
```rust
#[test]
fn test_frame_sync() {
    let engine = create_mock_engine();
    let timeline = Timeline::new();
    
    engine.seek(30);
    timeline.sync_with_engine(&engine);
    
    assert_eq!(timeline.get_playhead(), 30);
}

#[test]
fn test_script_timeout() {
    let mut ctx = ScriptContext::new();
    let result = ctx.execute_with_timeout(
        "while true { }", 
        Duration::from_millis(5)
    );
    
    assert!(result.is_err());
    assert!(result.unwrap_err().is_timeout());
}
```

### Integration Tests
```rust
#[test]
fn test_play_pause_performance() {
    let mut system = IntegrationTestSystem::new();
    
    let start = Instant::now();
    system.play();
    thread::sleep(Duration::from_millis(100));
    system.pause();
    let elapsed = start.elapsed();
    
    assert!(elapsed.as_millis() < 120); // Max 20% overhead
}
```

### Benchmark Tests
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_frame_render(c: &mut Criterion) {
    let system = create_test_system();
    
    c.bench_function("render_frame", |b| {
        b.iter(|| {
            system.render_frame(black_box(30))
        })
    });
}

criterion_group!(benches, bench_frame_render);
criterion_main!(benches);
```

## 9. Performance Acceptance Criteria

### Minimum Requirements
- 60 FPS with up to 20 layers
- < 50ms seek time for any frame
- < 5ms script execution per frame
- < 100MB memory for 1000-frame timeline

### Target Performance
- 60 FPS with 50+ layers
- < 20ms seek time
- < 2ms script execution
- < 50MB memory overhead

### Stress Test Limits
- 30 FPS minimum with 100 layers
- < 100ms worst-case seek
- Graceful degradation beyond limits

## 10. Continuous Integration

### CI Pipeline
```yaml
test-integration:
  script:
    - cargo test --test integration_tests
    - cargo bench --bench performance
    - ./scripts/memory_leak_test.sh
    
  artifacts:
    - performance_report.html
    - memory_profile.svg
```

### Monitoring Metrics
- Frame time percentiles (p50, p95, p99)
- Script execution time distribution
- Memory allocation patterns
- IPC message latency

This comprehensive test suite ensures robust integration between the RustFlash Editor engine and nannou_conrod timeline IDE, covering performance, functionality, error handling, and edge cases.