# Timeline UI Test Suite

This directory contains comprehensive UI tests for the Flash-inspired timeline widget using multiple testing approaches.

## Test Files

### 1. `ui_tests.rs` - Basic UI Tests
Uses `egui_kittest` for testing timeline widget interactions:
- Timeline rendering and navigation
- Keyframe selection and manipulation  
- Snap-to-grid functionality
- Audio layer handling
- Drag-and-drop operations
- Flash compatibility verification

### 2. `custom_test_harness.rs` - Custom Event Simulation
Provides fine-grained control over input events:
- Direct event injection (mouse, keyboard, scroll)
- Complex interaction sequences
- Precise timing control
- State verification after interactions

### 3. `eframe_integration_tests.rs` - Full Application Tests
Tests timeline in complete application context:
- Rendering stability across frames
- Playback functionality
- Performance under stress
- Headless testing support

### 4. `../standalone_demo/tests/ui_tests.rs` - Demo Application Tests
Tests the complete Flash-style IDE demo:
- Stage object manipulation
- Tool selection and usage
- Library asset drag-and-drop
- Properties panel integration
- Context menu functionality

## Running Tests

### Run all tests:
```bash
cargo test
```

### Run specific test file:
```bash
cargo test --test ui_tests
cargo test --test custom_test_harness
cargo test --test eframe_integration_tests
```

### Run with output for debugging:
```bash
cargo test -- --nocapture
```

### Run benchmarks:
```bash
cargo bench
```

## Test Architecture

### Custom Test Harness Pattern
```rust
let mut harness = TimelineTestHarness::new();

// Queue events
harness.click_at(Pos2::new(250.0, 80.0), PointerButton::Primary);
harness.drag_from_to(start_pos, end_pos, PointerButton::Primary);
harness.press_key(Key::Space, Modifiers::default());

// Run frame to process events
harness.run_frame();

// Assert on state
assert_eq!(harness.timeline().state.playhead_frame, expected_frame);
```

### Event Simulation
The custom harness allows precise event control:
- `click_at()` - Simulate mouse clicks
- `drag_from_to()` - Simulate drag operations
- `press_key()` - Simulate keyboard input
- `scroll()` - Simulate mouse wheel
- `right_click_at()` - Context menu testing

### State Verification
Tests verify:
- Timeline state (playhead, selection, zoom)
- Engine state (frames, layers, keyframes)
- UI feedback (cursors, visual elements)
- Performance metrics (render time, memory)

## Test Categories

### 1. **Interaction Tests**
- Mouse clicks and drags
- Keyboard shortcuts
- Context menus
- Multi-selection with modifiers

### 2. **Rendering Tests**
- Timeline grid display
- Keyframe visualization
- Layer panel rendering
- Playhead positioning

### 3. **State Management Tests**
- Selection state
- Playback state
- Zoom and scroll
- Copy/paste operations

### 4. **Performance Tests**
- Large timeline rendering
- Many layers/keyframes
- Zoom performance
- Memory usage patterns

### 5. **Flash Compatibility Tests**
- Layout matching Flash CS6
- Keyframe behavior
- Snap functionality
- Color scheme

## Adding New Tests

### 1. Create test in appropriate file
### 2. Use harness for complex interactions:
```rust
#[test]
fn test_new_feature() {
    let mut harness = TimelineTestHarness::new();
    
    // Setup
    harness.timeline_mut().state.some_property = true;
    
    // Interact
    harness.click_at(Pos2::new(100.0, 50.0), PointerButton::Primary);
    harness.run_frame();
    
    // Verify
    assert!(harness.timeline().state.some_result);
}
```

### 3. Document expected behavior
### 4. Add to CI if needed

## Performance Benchmarks

Run benchmarks to track performance:
```bash
cargo bench

# Generate HTML report
cargo bench -- --save-baseline my_baseline

# Compare with baseline
cargo bench -- --baseline my_baseline
```

Benchmarks measure:
- Rendering with different layer counts
- Frame selection operations
- Keyframe manipulation
- Zoom and scroll performance
- Memory allocation patterns

## Debugging Tips

1. **Visual debugging**: Add `println!` to see event flow
2. **State inspection**: Use `dbg!()` macro for state snapshots
3. **Frame-by-frame**: Run single frames to isolate issues
4. **Event logging**: Track event sequence with custom logging

## Known Limitations

1. **Display Required**: Some tests need display access (not fully headless)
2. **Timing Sensitive**: Some interactions depend on frame timing
3. **Platform Differences**: Mouse/keyboard behavior may vary by OS

## Future Improvements

1. **Headless Backend**: Full headless testing without display
2. **Visual Regression**: Screenshot comparison tests
3. **Fuzzing**: Random interaction testing
4. **Integration**: Test with real Rive engine
5. **Accessibility**: Screen reader testing