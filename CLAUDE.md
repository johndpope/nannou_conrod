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
  - Enhanced Flash CS6/Animate-style timeline implementation in `timeline_egui.rs`
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

#### Enhanced Flash-Style Timeline (`timeline_egui.rs`)
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
- **Scene Management** (Issue #31):
  - Multi-scene navigation with Flash CS6-style tabs
  - Scene creation, deletion, and renaming
  - Scene switching with Ctrl+PageUp/PageDown shortcuts
  - Scene modification tracking and visual indicators
  - Scene properties including frame rate, stage size, background color

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

### Recent Completed Features

#### Scene Management System (Issue #31) - ✅ COMPLETED
- **Core Architecture**: Complete Scene and SceneManager data structures with CRUD operations
- **UI Components**: Scene tabs with active highlighting, modification indicators, inline renaming
- **Navigation**: Keyboard shortcuts (Ctrl+PageUp/PageDown, Ctrl+T) and mouse interactions
- **Integration**: Scene tabs displayed at top of timeline interface with real-time logging
- **Technical**: Event-driven architecture, proper egui borrowing management, serde serialization

#### Timeline Layer Enhancements - ✅ COMPLETED  
- **Visual Improvements**: Horizontal separator lines beneath each layer in timeline
- **User Experience**: Double-click layer names to edit, right-click context menus
- **Auto-focus**: New layers automatically scroll into view with text field focus
- **Interaction**: Fixed layer name interaction handling and context menu positioning

#### Material Design Icons Integration (Issue #43) - ✅ COMPLETED
- **Icon System**: Integrated egui_material_icons crate for professional toolbar icons
- **Toolbar Enhancement**: Updated all tool icons (Arrow, Brush, Rectangle, etc.) with material design
- **Compatibility**: Resolved version compatibility issues with egui 0.32

### Bug and Defect Tracking

**IMPORTANT**: All bugs, defects, and issues discovered during development MUST be tracked in GitHub Issues.

#### Bug Reporting Requirements:
1. **Create GitHub Issue immediately** when any bug/defect is discovered
2. **Use descriptive titles** that clearly identify the problem
3. **Include reproduction steps** and affected files
4. **Add appropriate labels**: `bug`, `task-master` (if part of Task Master workflow)
5. **Link to related work** - reference the original issue/PR that introduced the bug
6. **Set priority** based on impact (affects demo functionality = high priority)

#### Bug Issue Template:
```
Title: [Component] Brief description of bug

Description:
- What happens vs expected behavior
- Symptoms observed
- Performance/functionality impact

Reproduction Steps:
1. Step one
2. Step two
3. Observe issue

Files to Investigate:
- path/to/file.rs - specific area
- path/to/other.rs - related logic

Related Work:
- Part of Issue #XX implementation
- Introduced in PR #YY
```

#### Issue State Management:
- Use `in-progress` label for active work
- Reference issues in commit messages
- Close issues only when fully resolved and tested
- Create follow-up issues for incomplete work rather than leaving things "by the wayside"

### GitHub Issue Workflow

#### Issue Labels:
- `bug` - Something isn't working correctly
- `enhancement` - New feature or improvement request  
- `in-progress` - Issue is currently being worked on
- `task-master` - Issues for Task Master AI workflow
- `documentation` - Documentation improvements

#### Working with Issues:
```bash
# List all open issues
gh issue list

# List issues by label
gh issue list --label "in-progress"
gh issue list --label "task-master"

# Create new issue
gh issue create --title "Title" --body "Description" --label "bug,task-master"

# Add labels to existing issue
gh issue edit 25 --add-label "in-progress"

# Close completed issue
gh issue close 25 --comment "Implementation complete"
```

### Migration Notice
Since this project is deprecated, any new GUI work should consider using `nannou_egui` instead. See https://github.com/PistonDevelopers/conrod/issues/1454 for context on why conrod development has ceased.