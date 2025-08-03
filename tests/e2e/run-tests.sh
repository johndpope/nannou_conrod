#!/bin/bash

# Timeline Editor E2E Test Runner

set -e

echo "🚀 Timeline Editor E2E Test Suite"
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo -e "${RED}❌ npm is not installed. Please install Node.js first.${NC}"
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ cargo is not installed. Please install Rust first.${NC}"
    exit 1
fi

# Parse command line arguments
HEADLESS=false
DEBUG=false
SPECIFIC_TEST=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --headless)
            HEADLESS=true
            shift
            ;;
        --debug)
            DEBUG=true
            shift
            ;;
        --test)
            SPECIFIC_TEST="$2"
            shift 2
            ;;
        --help)
            echo "Usage: ./run-tests.sh [options]"
            echo ""
            echo "Options:"
            echo "  --headless     Run tests in headless mode"
            echo "  --debug        Run tests in debug mode (slow, with devtools)"
            echo "  --test <name>  Run specific test file"
            echo "  --help         Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./run-tests.sh                    # Run all tests with browser visible"
            echo "  ./run-tests.sh --headless         # Run all tests in headless mode"
            echo "  ./run-tests.sh --debug            # Run in debug mode"
            echo "  ./run-tests.sh --test playback    # Run playback tests only"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}📦 Installing test dependencies...${NC}"
    npm install
fi

# Create directories
mkdir -p screenshots
mkdir -p test-results

# Build the timeline editor if needed
echo -e "${YELLOW}🔨 Building timeline editor...${NC}"
cd ../../nannou_timeline/standalone_demo

# Check if web target exists, if not use regular demo
if cargo build --bin timeline-demo-web --release 2>/dev/null; then
    echo -e "${GREEN}✅ Web target built successfully${NC}"
else
    echo -e "${YELLOW}⚠️  Web target not found, using standard demo${NC}"
    cargo build --bin timeline-demo --release
fi

cd ../../tests/e2e

# Set environment variables
if [ "$HEADLESS" = true ]; then
    export HEADLESS=true
    echo -e "${YELLOW}🤖 Running in headless mode${NC}"
fi

if [ "$DEBUG" = true ]; then
    export DEBUG=true
    echo -e "${YELLOW}🔍 Running in debug mode${NC}"
fi

# Run tests
echo -e "${YELLOW}🧪 Running tests...${NC}"
echo ""

if [ -n "$SPECIFIC_TEST" ]; then
    npm test -- "$SPECIFIC_TEST"
else
    npm test
fi

# Check exit code
if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ All tests passed!${NC}"
else
    echo ""
    echo -e "${RED}❌ Some tests failed. Check the output above.${NC}"
    echo -e "${YELLOW}📸 Screenshots saved in: ./screenshots/${NC}"
    exit 1
fi