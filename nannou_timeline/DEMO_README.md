# 🎬 Flash-style Timeline Demos

This directory contains a complete Flash CS6/Adobe Animate-inspired timeline implementation for nannou.

## 🚀 Quick Start

### Run the main demo:
```bash
./build_demo.sh
```

### Run various demos:
```bash
./run_demos.sh
```

## 📁 Available Demos

### 1. **Standalone Timeline Demo** (`standalone_demo/`)
Full Flash IDE interface with:
- Complete timeline with all Flash controls
- Stage/Canvas area
- Library panel for assets
- Properties panel
- Debug console (F12)
- Resizable panels

### 2. **Interactive Timeline Example** (`examples/interactive_timeline.rs`)
Focused timeline demo with:
- Layer panel with add/delete/duplicate buttons
- Visibility and lock toggles per layer
- Right-click context menus
- Flash-style visual feedback

### 3. **Fixed Timeline Test** (`examples/test_fixed_timeline.rs`)
Simple test showing the fixed timeline implementation.

## 🎮 Controls

### Timeline Controls:
- **Space** - Play/Pause
- **,** (comma) - Previous frame
- **.** (period) - Next frame
- **F5** - Insert frame
- **F6** - Insert keyframe
- **Shift+F6** - Clear keyframe
- **F7** - Insert blank keyframe

### Layer Controls:
- **Click** - Select layer
- **Right-click** - Context menu
- **👁** - Toggle visibility
- **🔒** - Toggle lock
- **Ctrl+Alt+L** - Add new layer

### Playback Controls:
- **Enter** - Play/Pause
- **Home** - Go to first frame
- **End** - Go to last frame

## 🛠️ Features

### Layer Panel
- Add/Delete/Duplicate layers
- Visibility toggle per layer
- Lock/Unlock layers
- Layer type indicators
- Drag to reorder (planned)

### Timeline
- Keyframe indicators
- Tween visualization
- Frame selection
- Snap-to-grid
- Audio waveforms
- Onion skinning

### Context Menus
- Layer options (duplicate, delete, rename, etc.)
- Frame options (insert keyframe, create tween, etc.)
- Copy/paste frames

## 🐛 Troubleshooting

If you encounter dependency conflicts:
1. Clean the build: `cargo clean`
2. Update dependencies: `cargo update`
3. Build from standalone_demo directory: `cd standalone_demo && cargo build`

## 📊 Testing

Run the comprehensive test suite:
```bash
cargo test -- --nocapture
```

Run UI tests specifically:
```bash
cargo test --test ui_tests -- --nocapture
```

## 🎨 Flash CS6 Compatibility

This implementation recreates the core Flash CS6/Adobe Animate timeline experience including:
- Frame-by-frame animation
- Motion and shape tweens
- Layer management
- Audio synchronization
- Easing curves
- Onion skinning
- Symbol library (planned)

Enjoy creating animations with a familiar Flash-style interface! 🎉