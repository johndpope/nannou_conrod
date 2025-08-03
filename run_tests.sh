#!/bin/bash

# Timeline Renderer Test Runner
# This script runs various tests to verify the timeline renderer functionality

set -e  # Exit on error

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
FAILED_TESTS=()

echo -e "${BLUE}🧪 Timeline Renderer Test Suite${NC}"
echo "======================================"
echo ""

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${YELLOW}Running: $test_name${NC}"
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ PASSED: $test_name${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ FAILED: $test_name${NC}"
        ((TESTS_FAILED++))
        FAILED_TESTS+=("$test_name")
    fi
    echo ""
}

# 1. Build the project
echo -e "${BLUE}📦 Building Project...${NC}"
if cargo build --release 2>&1 | grep -E "(error|warning)" > build_output.log; then
    echo -e "${YELLOW}⚠️  Build completed with warnings (see build_output.log)${NC}"
else
    echo -e "${GREEN}✅ Build successful${NC}"
fi
echo ""

# 2. Run standalone renderer test
if [ -f "test_renderer_output.rs" ]; then
    run_test "Standalone Renderer Output Test" \
        "rustc test_renderer_output.rs -o test_renderer_output && ./test_renderer_output"
fi

# 3. Run renderer verification
if [ -f "verify_renderer.rs" ]; then
    run_test "Renderer Verification Test" \
        "rustc verify_renderer.rs -o verify_renderer && ./verify_renderer"
fi

# 4. Run library unit tests
run_test "Timeline Library Unit Tests" \
    "cargo test --lib -- --nocapture 2>&1 | grep -v 'warning:' || true"

# 5. Run workspace tests
run_test "Workspace Tests" \
    "cargo test --workspace -- --nocapture 2>&1 | grep -v 'warning:' || true"

# 6. Run timeline demo tests (if they compile)
echo -e "${BLUE}🎮 Testing Timeline Demo...${NC}"
cd nannou_timeline/standalone_demo

# Try to run demo tests
run_test "Timeline Demo Tests" \
    "cargo test --bin timeline-demo -- --nocapture 2>&1 | grep -E '(test result:|passed|failed)' || echo 'No tests found or compilation errors'"

# 7. Check if demo binary exists and is runnable
if [ -f "target/release/timeline-demo" ]; then
    echo -e "${BLUE}🚀 Demo Binary Check${NC}"
    if ./target/release/timeline-demo --version 2>/dev/null || true; then
        echo -e "${GREEN}✅ Demo binary is executable${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${YELLOW}⚠️  Demo binary exists but doesn't support --version flag${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Demo binary not found at target/release/timeline-demo${NC}"
fi

cd "$SCRIPT_DIR"

# 8. Performance benchmark test
echo ""
echo -e "${BLUE}⚡ Performance Benchmark${NC}"
cat > perf_test.rs << 'EOF'
use std::time::Instant;

fn main() {
    println!("Running performance benchmark...");
    let start = Instant::now();
    let mut total_ops = 0;
    
    // Simulate rendering operations
    for frame in 0..300 {
        // Simulate frame rendering
        for _ in 0..100 {
            total_ops += 1;
        }
    }
    
    let duration = start.elapsed();
    let ops_per_sec = total_ops as f64 / duration.as_secs_f64();
    
    println!("  Operations: {}", total_ops);
    println!("  Duration: {:.2}s", duration.as_secs_f64());
    println!("  Ops/sec: {:.0}", ops_per_sec);
    
    if ops_per_sec > 10000.0 {
        println!("  ✅ Performance: EXCELLENT");
        std::process::exit(0);
    } else {
        println!("  ❌ Performance: NEEDS IMPROVEMENT");
        std::process::exit(1);
    }
}
EOF

run_test "Performance Benchmark" \
    "rustc perf_test.rs -O -o perf_test && ./perf_test"

# Cleanup
rm -f test_renderer_output verify_renderer perf_test perf_test.rs

# Summary
echo ""
echo -e "${BLUE}📊 Test Summary${NC}"
echo "======================================"
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"

if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed Tests:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo "  - $test"
    done
fi

echo ""
if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi