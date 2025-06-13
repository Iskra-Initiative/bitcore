#!/bin/bash
# -- script to generate local coverage reports

set -e

echo "bitcore coverage report generator"
echo "================================="

# check if cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

echo "✓ cargo-llvm-cov found"
echo ""

# check if socat is available for integration tests
if command -v socat &> /dev/null; then
    echo "✓ socat found - will include integration tests"
    SOCAT_AVAILABLE=true
else
    echo "⚠ socat not found - skipping integration tests"
    echo "  install socat to include integration test coverage"
    SOCAT_AVAILABLE=false
fi

echo ""
echo "generating coverage report..."

# clean previous coverage data
cargo llvm-cov clean

# run unit tests with coverage
echo "running unit tests..."
cargo llvm-cov --no-report test unit_tests

# run socat tests if available
if [ "$SOCAT_AVAILABLE" = true ]; then
    echo "running socat integration tests..."
    cargo llvm-cov --no-report test --test socat_tests -- --ignored
fi

# generate reports
echo "generating HTML report..."
cargo llvm-cov report --html

echo "generating LCOV report..."
cargo llvm-cov report --lcov --output-path coverage.lcov

echo ""
echo "coverage reports generated:"
echo "  HTML: target/llvm-cov/html/index.html"
echo "  LCOV: coverage.lcov"
echo ""

# generate coverage badge
echo "generating coverage badge..."
./scripts/generate_coverage_badge.sh

echo ""

# open HTML report if on macOS or Linux with display
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "opening HTML report..."
    open target/llvm-cov/html/index.html
elif [[ "$OSTYPE" == "linux-gnu"* ]] && [ -n "$DISPLAY" ]; then
    echo "opening HTML report..."
    xdg-open target/llvm-cov/html/index.html
else
    echo "to view the HTML report, open: target/llvm-cov/html/index.html"
fi

echo "coverage report generation completed!"
