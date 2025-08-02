# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**IMPORTANT**: This is the `nannou_conrod` repository - GUI crates for the nannou creative coding framework. This project is **NO LONGER MAINTAINED** and recommends using `nannou_egui` instead.

## Build and Development Commands

```bash
# Build the entire workspace
cargo build

# Run tests (requires libxcb-composite0-dev on Linux)
cargo test

# Check formatting
cargo fmt --all -- --check

# Run a specific example
cargo run --example simple_ui
cargo run --example timeline_demo
cargo run --example named_color_reference

# Build documentation
cargo doc --no-deps
```

## Architecture Overview

### Workspace Structure
- **nannou_conrod**: Core GUI integration between conrod and nannou
- **nannou_timeline**: Timeline widget for temporal data visualization/control
- **examples**: Demo applications showing usage patterns

### Key Components

#### nannou_conrod::Ui
The main integration point - wraps `conrod_core::Ui` with WGPU rendering:
- Builder pattern for construction
- Manages conrod renderer and font resources
- Converts winit events to conrod events
- Provides `draw()` method for rendering to nannou frames

#### Timeline System
The timeline crate provides:
- Timeline widget with playhead control
- Multiple track types (piano roll, toggle, bang, numeric automation)
- Musical time structures (bars, beats)
- Envelope-based automation curves

### Important Design Patterns

1. **Builder Pattern**: Used for UI construction
   ```rust
   let ui = app.new_ui()
       .font_path(font_path)
       .theme(theme)
       .build()
       .unwrap();
   ```

2. **Widget IDs**: Use `widget::Id` generation for conrod widgets
3. **Event Handling**: Convert winit events through `ui.handle_input(event)`
4. **Rendering**: Call `ui.draw(&frame)` in nannou's draw function

### Dependencies to Note
- Pinned to specific nannou version via git patch
- Uses winit 0.26 (must match nannou's version)
- conrod_* crates at version 0.76

### Common Development Tasks

When modifying this codebase:
1. Check CI requirements - tests need `libxcb-composite0-dev` on Linux
2. Maintain compatibility with conrod 0.76 API
3. Follow existing widget patterns when adding new UI components
4. Update examples when adding significant features

### Migration Notice
Since this project is deprecated, any new GUI work should consider using `nannou_egui` instead. See https://github.com/PistonDevelopers/conrod/issues/1454 for context on why conrod development has ceased.