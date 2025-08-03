#!/bin/bash

# Run various Timeline demos
# This script provides options to run different timeline demonstrations

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎬 Flash-style Timeline Demo Runner${NC}"
echo "======================================"
echo ""

# Function to display menu
show_menu() {
    echo "Select a demo to run:"
    echo ""
    echo "1) Standalone Timeline Demo (Full Flash IDE)"
    echo "2) Interactive Timeline Example"
    echo "3) Fixed Timeline Test"
    echo "4) Run All Tests"
    echo "5) Run UI Tests Only"
    echo "6) Build All Examples"
    echo "7) Exit"
    echo ""
    echo -n "Enter your choice [1-7]: "
}

# Function to run standalone demo
run_standalone() {
    echo -e "${YELLOW}Running Standalone Timeline Demo...${NC}"
    cd standalone_demo
    cargo run --release
    cd ..
}

# Function to run interactive example
run_interactive() {
    echo -e "${YELLOW}Running Interactive Timeline Example...${NC}"
    cargo run --example interactive_timeline
}

# Function to run fixed timeline test
run_fixed_test() {
    echo -e "${YELLOW}Running Fixed Timeline Test...${NC}"
    cargo run --example test_fixed_timeline
}

# Function to run all tests
run_tests() {
    echo -e "${YELLOW}Running all tests...${NC}"
    cargo test -- --nocapture
}

# Function to run UI tests
run_ui_tests() {
    echo -e "${YELLOW}Running UI tests...${NC}"
    cargo test --test ui_tests -- --nocapture
}

# Function to build all examples
build_all() {
    echo -e "${YELLOW}Building all examples...${NC}"
    cargo build --examples
    echo -e "${GREEN}✅ All examples built successfully!${NC}"
}

# Main loop
while true; do
    show_menu
    read choice
    
    case $choice in
        1)
            run_standalone
            ;;
        2)
            run_interactive
            ;;
        3)
            run_fixed_test
            ;;
        4)
            run_tests
            ;;
        5)
            run_ui_tests
            ;;
        6)
            build_all
            ;;
        7)
            echo -e "${GREEN}Goodbye!${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}Invalid option. Please try again.${NC}"
            ;;
    esac
    
    echo ""
    echo "Press Enter to continue..."
    read
    clear
done