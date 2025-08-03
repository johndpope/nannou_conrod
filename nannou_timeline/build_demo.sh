#!/bin/bash

# Build and run the Flash-style Timeline Demo
# This script builds and runs the standalone timeline demo with all Flash CS6 features

set -e  # Exit on error

echo "🎬 Building Flash-style Timeline Demo..."
echo "=========================================="

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Must run from nannou_timeline directory${NC}"
    echo "Please cd to nannou_timeline first"
    exit 1
fi

# Clean previous builds
echo -e "${YELLOW}Cleaning previous builds...${NC}"
cd standalone_demo
cargo clean

# Build the demo
echo -e "${YELLOW}Building timeline demo...${NC}"
cargo build --release

# Check if build succeeded
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Build successful!${NC}"
    echo ""
    echo "🚀 Starting Flash-style Timeline Demo..."
    echo "=========================================="
    echo ""
    echo "📋 Controls:"
    echo "  • Click layers to select"
    echo "  • Right-click for context menus"
    echo "  • Use buttons to add/remove layers"
    echo "  • Toggle visibility with 👁 icon"
    echo "  • Press Space to play/pause"
    echo "  • Press F12 to toggle debug console"
    echo ""
    echo "🎯 Flash CS6 Features:"
    echo "  • Layer panel with add/delete/duplicate controls"
    echo "  • Frame grid with keyframe indicators"
    echo "  • Timeline toolbar with playback controls"
    echo "  • Context menus for layers and frames"
    echo "  • Audio waveform visualization"
    echo "  • Snap-to-grid with visual guides"
    echo ""
    
    # Run the demo
    cargo run --release
else
    echo -e "${RED}❌ Build failed!${NC}"
    echo "Please check the error messages above"
    exit 1
fi