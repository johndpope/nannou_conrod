#!/bin/bash

# Timeline UI Test Runner
# Runs all timeline tests with various configurations

set -e

echo "=== Timeline UI Test Suite ==="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to run tests and check result
run_test_suite() {
    local test_name=$1
    local test_command=$2
    
    echo -e "${YELLOW}Running $test_name...${NC}"
    if eval "$test_command"; then
        echo -e "${GREEN}✓ $test_name passed${NC}\n"
    else
        echo -e "${RED}✗ $test_name failed${NC}\n"
        exit 1
    fi
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "tests" ]; then
    echo -e "${RED}Error: Must run from nannou_timeline directory${NC}"
    exit 1
fi

# Run different test suites
run_test_suite "Basic UI Tests" "cargo test --test ui_tests"
run_test_suite "Custom Harness Tests" "cargo test --test custom_test_harness"
run_test_suite "Integration Tests" "cargo test --test eframe_integration_tests"
run_test_suite "Timeline Core Tests" "cargo test --test timeline_tests"

# Run standalone demo tests if available
if [ -d "standalone_demo" ]; then
    echo -e "${YELLOW}Running Standalone Demo Tests...${NC}"
    cd standalone_demo
    if cargo test; then
        echo -e "${GREEN}✓ Demo tests passed${NC}\n"
    else
        echo -e "${RED}✗ Demo tests failed${NC}\n"
    fi
    cd ..
fi

# Optional: Run benchmarks
if [ "$1" == "--bench" ]; then
    echo -e "${YELLOW}Running Performance Benchmarks...${NC}"
    cargo bench
    echo -e "${GREEN}✓ Benchmarks complete${NC}"
    echo "Benchmark results available in target/criterion/"
fi

# Optional: Run with coverage
if [ "$1" == "--coverage" ]; then
    echo -e "${YELLOW}Running Tests with Coverage...${NC}"
    # Requires cargo-tarpaulin
    if command -v cargo-tarpaulin &> /dev/null; then
        cargo tarpaulin --out Html --output-dir coverage
        echo -e "${GREEN}✓ Coverage report generated in coverage/tarpaulin-report.html${NC}"
    else
        echo -e "${YELLOW}Install cargo-tarpaulin for coverage:${NC}"
        echo "cargo install cargo-tarpaulin"
    fi
fi

echo -e "${GREEN}=== All Tests Passed! ===${NC}"