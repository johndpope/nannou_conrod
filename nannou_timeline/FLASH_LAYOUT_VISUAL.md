# Flash-Style Layout Visual Description

## Full Window Layout (1200x600)

```
┌────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                    │
│                                                                                    │
│                                                                                    │
│                        Stage / Canvas                         │  📚 Library       │
│                          850x350                              │  ──────────────   │
│                                                               │  Assets | Props   │
│     ┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┬       │  ──────────────   │
│     │     │     │     │     │     │     │     │     │       │                   │
│     ├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤       │  🎭 Symbols:      │
│     │     │     │     │     │     │     │     │     │       │    Symbol_1       │
│     ├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤       │    Symbol_2       │
│     │     │     │     │     │     │     │     │     │       │    Symbol_3       │
│     ├─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤       │    Symbol_4       │
│     │     │     │     │     │     │     │     │     │       │    Symbol_5       │
│     └─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴       │                   │
│                                                               │  🖼️ Bitmaps:      │
│                                                               │    Image_1        │
│                                                               │    Image_2        │
│                                                               │    Image_3        │
│                                                               │                   │
│                                                               │  🔊 Sounds:       │
│                                                               │    Sound_1        │
│                                                               │    Sound_2        │
│                                                               │                   │
├───────────────────────────────────────────────────────────────┴───────────────────┤
│ TIMELINE (200px height)                                                            │
├─────────────────┬──────────────────────────────────────────────────────────────────┤
│                 │ 0    5    10   15   20   25   30   35   40   45   50            │
│ 👁️ Background    │ ●════●════●════●════●════●════●════●════●════●                │
│ 👁️ Character     │ ●    ●────────●    ●────────●    ●────────●                   │
│ 👁️ Effects       │      ●════════●         ●════●                                 │
│    Glow         │                                                                  │
│ 👁️🔒 Particles    │           ●════════════●                                       │
├─────────────────┴──────────────────────────────────────────────────────────────────┤
│ ▶ ⏸ ⏹  Frame: 0/100  FPS: 24 fps (Film) ▼  Zoom: [-] 100% [+]                   │
└────────────────────────────────────────────────────────────────────────────────────┘
```

## Key Features Visible:

1. **Three-Panel Layout**:
   - **Stage/Canvas** (Center-Left): Large area for content preview with grid
   - **Library** (Right): Asset browser with symbols, bitmaps, sounds
   - **Timeline** (Bottom): Full timeline widget with layers and frames

2. **Resizable Panels**:
   - Vertical splitter between Stage and Library (drag to resize)
   - Horizontal splitter between Stage and Timeline (drag to resize)
   - Corner handle for diagonal resize

3. **Interactive Elements**:
   - All panels respond to mouse interaction
   - Splitters change color on hover
   - Cursor changes to resize arrows on splitters
   - Timeline has full functionality (zoom, scroll, right-click menus)

4. **Visual Styling**:
   - Dark theme matching Flash CS6/Animate CC
   - Grid pattern on stage for alignment reference
   - Clear panel borders and separators
   - Proper contrast for readability

## Responsive Behavior:
- Window can be resized - all panels adjust proportionally
- Minimum sizes enforced (150px for library, 100px for timeline)
- Timeline scrolls independently within its allocated space
- Library has scrollable content area
- Stage shows current dimensions dynamically

This creates a professional animation workspace similar to Adobe Flash/Animate!