# Timeline Editor E2E Test Suite

This test suite provides comprehensive end-to-end testing for the RustFlash Timeline Editor using Puppeteer, based on the test cases defined in Issue #41.

## Prerequisites

- Node.js 18+ and npm
- Rust toolchain (for building the timeline editor)
- Chrome/Chromium browser

## Installation

```bash
cd tests/e2e
npm install
```

## Running Tests

### Run all tests in headless mode:
```bash
npm test
```

### Run tests with browser visible:
```bash
npm run test:watch
```

### Run tests in debug mode (slow motion + devtools):
```bash
npm run test:debug
```

### Run specific test suite:
```bash
npm test frame-rate-timing.test.ts
```

## Test Structure

The test suite is organized into the following categories:

### 1. Frame Rate and Timing Synchronization (`frame-rate-timing.test.ts`)
- **TC1.1**: Basic 60 FPS Playback - Verifies consistent frame rate with multiple objects
- **TC1.2**: Timeline-Engine Sync - Ensures perfect synchronization between timeline and engine
- **TC1.3**: Variable Frame Rate Handling - Tests FPS changes (24/30/60)

### 2. Animation Playback Controls (`playback-controls.test.ts`)
- **TC2.1**: Play/Pause Toggle - Response time and state management
- **TC2.2**: Stop and Reset - Full animation state reset
- **TC2.3**: Timeline Scrubbing - Real-time preview and seek performance
- **TC2.4**: Frame-by-Frame Navigation - Arrow key navigation

### 3. Timeline Scrolling and Navigation (`timeline-navigation.test.ts`)
- **TC3.1**: Horizontal Scroll During Playback - Auto-scroll and manual scroll
- **TC3.2**: Zoom In/Out During Playback - Performance during zoom operations
- **TC3.3**: Layer Panel Scrolling - Independent scrolling and layer operations

### 4. Rhai Script Execution (`script-execution.test.ts`)
- **TC4.1**: Frame Script Basic Execution - Script timing and execution order
- **TC4.2**: Script Performance Benchmarking - Heavy computation handling
- **TC4.3**: Script Error Handling - Graceful error recovery
- **TC4.4**: Cross-Frame State Persistence - Global state management

### 5. Display Object Updates (`display-object-updates.test.ts`)
- **TC5.1**: Property Animation Sync - Visual state synchronization
- **TC5.2**: Dynamic Object Creation/Destruction - Memory management
- **TC5.3**: Graphics Rendering Update - Per-frame graphics updates

### 6. Edge Cases and Error Scenarios (`edge-cases.test.ts`)
- **TC6.1**: Rapid Play/Pause Cycling - Stress testing controls
- **TC6.2**: Maximum Layer Stress Test - 100+ layers performance
- **TC6.3**: Script Memory Exhaustion - Memory limit handling
- **TC6.4**: Concurrent Operation Conflicts - Race condition prevention

## Performance Targets

The test suite validates the following performance requirements:

- **Frame Rate**: 60 FPS with 20 layers (minimum 58 FPS)
- **Seek Time**: < 20ms per seek operation
- **Script Execution**: < 5ms per frame script
- **Memory Usage**: < 500MB for 100 layers with animations
- **UI Response**: < 50ms for all user interactions

## Test Helpers

The `TimelineHelpers` class provides reusable methods for:
- Timeline navigation (play, pause, stop, seek)
- Layer operations (add, select, toggle visibility)
- Script management (set frame scripts, read console)
- Performance monitoring (FPS measurement, memory usage)
- Stage object manipulation

## Data Attributes

The tests rely on the following data-testid attributes in the application:

```html
- data-testid="timeline"
- data-testid="play-button"
- data-testid="pause-button"
- data-testid="stop-button"
- data-testid="playhead"
- data-testid="frame-counter"
- data-testid="fps-display"
- data-testid="timeline-ruler"
- data-testid="frame-grid"
- data-testid="layer-panel"
- data-testid="layer-{id}"
- data-testid="stage"
- data-testid="stage-object-{id}"
```

## Debugging Failed Tests

1. **Enable debug mode**: Run `npm run test:debug` to see the browser and slow down actions
2. **Take screenshots**: Tests automatically capture screenshots on failure in `screenshots/` directory
3. **Check console logs**: Browser console output is captured and available in test results
4. **Use breakpoints**: Add `await page.pause()` in tests to pause execution

## Adding New Tests

1. Create a new test file in `tests/` directory
2. Import TimelineHelpers: `import { TimelineHelpers } from '../helpers/timeline-helpers';`
3. Follow the existing test structure and naming conventions
4. Update this README with new test descriptions

## CI/CD Integration

The test suite can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Run E2E Tests
  run: |
    cd tests/e2e
    npm ci
    npm test
  env:
    HEADLESS: true
```

## Troubleshooting

### Tests timeout
- Increase timeout in `jest.config.js`
- Check if the application is building/starting correctly
- Verify port 8080 is available

### Browser not found
- Install Chrome/Chromium: `npx puppeteer browsers install chrome`
- Set PUPPETEER_EXECUTABLE_PATH environment variable

### Application won't start
- Check Rust compilation: `cargo build --release`
- Verify the web build target exists
- Check server logs in debug mode