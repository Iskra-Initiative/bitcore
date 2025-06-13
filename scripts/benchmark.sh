#!/bin/bash
# -- script to run performance benchmarks

set -e

echo "bitcore performance benchmarks"
echo "=============================="

# check if criterion is available
if ! grep -q "criterion" Cargo.toml; then
    echo "error: criterion not found in Cargo.toml"
    echo "add criterion to [dev-dependencies] to run benchmarks"
    exit 1
fi

echo "✓ criterion found in dependencies"

# check if socat is available for serial benchmarks
if command -v socat &> /dev/null; then
    echo "✓ socat found - will include serial benchmarks"
    SOCAT_AVAILABLE=true
else
    echo "⚠ socat not found - skipping serial benchmarks"
    echo "  install socat to benchmark actual serial operations"
    SOCAT_AVAILABLE=false
fi

echo ""

# check for baseline argument
BASELINE=""
if [ "$1" = "--save-baseline" ]; then
    BASELINE="--save-baseline main"
    echo "saving baseline as 'main'"
elif [ "$1" = "--compare" ]; then
    BASELINE="--baseline main"
    echo "comparing against 'main' baseline"
fi

echo "running performance benchmarks..."
echo ""

# run benchmarks
cargo bench $BASELINE

echo ""
echo "benchmark results saved to: target/criterion/"
echo ""

# show summary if available
if [ -f "target/criterion/report/index.html" ]; then
    echo "detailed HTML report: target/criterion/report/index.html"
    
    # open report if on macOS or Linux with display
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "opening benchmark report..."
        open target/criterion/report/index.html
    elif [[ "$OSTYPE" == "linux-gnu"* ]] && [ -n "$DISPLAY" ]; then
        echo "opening benchmark report..."
        xdg-open target/criterion/report/index.html
    fi
fi

echo ""
echo "benchmark usage:"
echo "  ./scripts/benchmark.sh                 # run benchmarks"
echo "  ./scripts/benchmark.sh --save-baseline # save current as baseline"
echo "  ./scripts/benchmark.sh --compare       # compare against baseline"
echo ""
echo "benchmarks completed!"
