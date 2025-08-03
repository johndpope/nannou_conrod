# Flash-Inspired Timeline UI Mockup

## Visual Layout

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Timeline - Flash-style Animation                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│ Instructions: • Right-click for menu • Ctrl+Wheel zoom • F5/F6 frames      │
├─────────────────┬───────────────────────────────────────────────────────────┤
│                 │ 0    5    10   15   20   25   30   35   40   45   50     │
│ LAYERS          │ |    |    |    |    |    |    |    |    |    |    |      │
│                 │ ▼ (playhead)                                              │
├─────────────────┼───────────────────────────────────────────────────────────┤
│ 👁️ Background    │ ●════●════●════●════●════●════●════●════●════●         │
├─────────────────┼───────────────────────────────────────────────────────────┤
│ 👁️ Character     │ ●    ●────────●    ●────────●    ●────────●            │
├─────────────────┼───────────────────────────────────────────────────────────┤
│ 👁️ Effects       │      ●════════●         ●════●                          │
├─────────────────┼───────────────────────────────────────────────────────────┤
│    Glow         │                                                           │
├─────────────────┼───────────────────────────────────────────────────────────┤
│ 👁️🔒 Particles    │           ●════════════●                                │
├─────────────────┴───────────────────────────────────────────────────────────┤
│ ▶ ⏸ ⏹  Frame: 0/100  FPS: 24 fps (Film) ▼  Zoom: [-] 100% [+]           │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Legend
- `●` = Keyframe (black circle)
- `════` = Tween frames (blue-ish fill)
- `────` = Regular frames (gray fill)
- `👁️` = Layer visible
- `🔒` = Layer locked
- `▼` = Playhead (red)

## Right-Click Context Menu
```
┌─────────────────────┐
│ Insert Frame (F5)   │
│ Remove Frame (⇧F5)  │
│ ─────────────────── │
│ Insert Keyframe (F6)│
│ Clear Keyframe (⇧F6)│
│ ─────────────────── │
│ Create Motion Tween │
│ Create Shape Tween  │
│ ─────────────────── │
│ Insert Frame Label..│
└─────────────────────┘
```

## Key Features Visible:
1. **Frame-based timeline** - Numbers represent frames, not musical time
2. **Layer panel** - Shows layer hierarchy with visibility/lock toggles
3. **Frame grid** - Visual representation of keyframes and tweens
4. **Playback controls** - Play/pause/stop with FPS selector
5. **Zoom controls** - Percentage-based zoom (10% to 500%)
6. **Scrollable area** - Both horizontal (frames) and vertical (layers)
7. **Playhead** - Red indicator showing current frame position

## Interaction Examples:
- **Zoom**: Hold Ctrl/Cmd + scroll wheel centers zoom on cursor
- **Horizontal scroll**: Shift + scroll wheel or drag scrollbar
- **Vertical scroll**: Regular scroll wheel when over layers
- **Select frame**: Click on any frame cell
- **Context menu**: Right-click on frame for operations
- **Drag playhead**: Click and drag red playhead in ruler
- **Keyboard nav**: Arrow keys move frame by frame

This implementation provides the core Flash timeline experience with modern egui rendering!