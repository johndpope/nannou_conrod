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
  - Enhanced Flash CS6/Animate-style timeline implementation in `timeline_egui_fixed.rs`
  - Internationalization (i18n) support with JSON language files
  - Context menus, tooltips, and professional UI polish
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

#### Enhanced Flash-Style Timeline (`timeline_egui_fixed.rs`)
The enhanced timeline implementation provides:
- **Complete Flash CS6/Animate IDE interface**: Layers panel, timeline grid, playback controls
- **Internationalization (i18n)**: Full multi-language support (English, Spanish, Japanese, Chinese)
- **Context Menus**: Right-click menus for layers and frames with appropriate actions
- **Tooltips**: Hover tooltips on all interactive elements
- **Professional UI Polish**: Proper visual feedback, keyboard shortcuts, and user experience
- **Interactive Stage/Canvas**: Full stage manipulation with drag-and-drop items

To run the enhanced timeline demo:
```bash
cd nannou_timeline/standalone_demo
cargo run
```

Key features:
- **Stage Interaction**:
  - Pre-populated stage items (rectangles, circles, text, movieclips)
  - Click to select items, drag to move them around
  - Right-click on stage for context menu to add new items
  - Right-click on items for options: rename, duplicate, bring to front/back, rotate, delete
  - Visual selection highlighting
  - Hover tooltips showing item names
- **Timeline Features**:
  - Hover over any UI element to see helpful tooltips
  - Right-click on layers or frames for context-sensitive menus
  - Switch languages in real-time via the language selector
  - Full Flash-style keyboard shortcuts (F5, F6, etc.)
- **Developer Console** (F12):
  - Real-time activity logging
  - Language switching support
  - Auto-scroll capability
  - Tracks all user interactions

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