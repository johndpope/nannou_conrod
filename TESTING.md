# Timeline Renderer Testing Guide

## Overview

This document describes the testing infrastructure for the Timeline Renderer, including test scripts, verification methods, and CI integration.

## Test Scripts

### 1. `run_tests.sh` - Comprehensive Test Suite
Full test suite that runs all available tests including unit tests, integration tests, and performance benchmarks.

```bash
./run_tests.sh
```

**Features:**
- Builds the entire project
- Runs standalone renderer tests
- Executes library unit tests
- Runs workspace tests
- Performance benchmarking
- Colored output with pass/fail summary

### 2. `test_renderer_only.sh` - Quick Renderer Verification
Focused test specifically for verifying renderer output generation.

```bash
./test_renderer_only.sh
```

**Features:**
- Quick execution (<1 second)
- Tests frame grid rendering
- Verifies layer rendering
- Checks animation capability
- Minimal dependencies

### 3. `ci_tests.sh` - CI/CD Integration Tests
Designed for continuous integration environments like GitHub Actions.

```bash
./ci_tests.sh
```

**Features:**
- GitHub Actions compatible output
- Exit codes for CI systems
- Grouped output sections
- Error annotations
- Summary statistics

## Manual Test Commands

### Build Tests
```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release

# Check without building
cargo check --all-targets
```

### Unit Tests
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_renderer_produces_output

# Run tests in specific module
cargo test renderer_tests::
```

### Performance Tests
```bash
# Run the standalone performance test
rustc test_renderer_output.rs && ./test_renderer_output

# Run with optimization
rustc -O verify_renderer.rs && ./verify_renderer
```

## Test Coverage

### Renderer Output Tests
- ✅ Frame grid rendering (vertical lines, keyframe markers)
- ✅ Layer rendering (backgrounds, separators)
- ✅ Playhead rendering
- ✅ Animation frame updates
- ✅ Performance metrics (FPS, frame time)

### Integration Tests
- ✅ Timeline state management
- ✅ Mock engine integration
- ✅ Draw command generation
- ✅ Render loop simulation

### Performance Benchmarks
- ✅ Frame rendering time (<20ms requirement)
- ✅ 60 FPS capability verification
- ✅ Draw call efficiency
- ✅ Memory usage (indirectly via performance)

## Expected Results

### Successful Test Output
```
✅ RENDERER OUTPUT: VERIFIED
The renderer is successfully producing output!

📊 Results:
  Total draw calls: 464
  Effective FPS: >60
```

### Key Metrics
- **Draw Calls**: >400 per frame (indicates active rendering)
- **Frame Time**: <16.67ms (60 FPS capability)
- **Average Render Time**: ~2.1ms (leaves headroom for other operations)

## Troubleshooting

### Common Issues

1. **Compilation Errors**
   - Run `cargo clean` and rebuild
   - Check Rust version: `rustc --version` (1.70+ recommended)

2. **Test Failures**
   - Check build logs: `cargo build 2>&1 | tee build.log`
   - Verify dependencies: `cargo tree`

3. **Performance Issues**
   - Run in release mode: `cargo build --release`
   - Check system resources during tests

## Adding New Tests

To add new renderer tests:

1. Create test in `src/tests/` directory
2. Add test module to `src/tests/mod.rs`
3. Update test scripts if needed
4. Document expected outcomes

Example test structure:
```rust
#[cfg(test)]
mod new_renderer_test {
    #[test]
    fn test_new_feature() {
        // Test implementation
        assert!(renderer_output > expected);
    }
}
```

## CI Integration

For GitHub Actions, add to `.github/workflows/test.yml`:

```yaml
- name: Run Timeline Tests
  run: |
    chmod +x ci_tests.sh
    ./ci_tests.sh
```

## Issue #41 Test Coverage

The current test infrastructure covers these test cases from Issue #41:

- ✅ TC2.1: Animation playback controls
- ✅ TC5.1: Display object rendering
- ✅ TC5.2: Rendering updates
- ✅ TC8.1: Performance benchmarking
- ⚠️  TC1.1-1.3: Frame rate timing (partial - desktop app)
- ⚠️  TC3.1-3.2: Timeline scrolling (requires UI interaction)
- ⚠️  TC4.1-4.2: Rhai scripting (not yet implemented)

## Next Steps

1. Implement Rhai script execution tests once integrated
2. Add visual regression tests with screenshot comparison
3. Create automated UI interaction tests for desktop app
4. Add memory profiling to performance tests