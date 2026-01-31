#!/bin/bash
# -- script to run socat-based integration tests

set -e

echo "bitcore socat integration tests"
echo "==============================="

# check if socat is installed
if ! command -v socat &> /dev/null; then
    echo "error: socat is not installed or not in PATH"
    echo ""
    echo "to install socat:"
    echo "  ubuntu/debian: sudo apt-get install socat"
    echo "  fedora/rhel:   sudo dnf install socat"
    echo "  macos:         brew install socat"
    echo "  windows:       install via msys2 or wsl"
    echo ""
    exit 1
fi

echo "✓ socat found: $(socat -V | head -1)"
echo ""

# check if we're on a supported platform
case "$(uname -s)" in
    Linux*)
        echo "✓ platform: Linux (supported)"
        ;;
    Darwin*)
        echo "✓ platform: macOS (supported)"
        ;;
    CYGWIN*|MINGW*|MSYS*)
        echo "✓ platform: Windows (supported via msys2/cygwin)"
        ;;
    *)
        echo "⚠ platform: $(uname -s) (may not be supported)"
        ;;
esac

echo ""
echo "running socat integration tests..."
echo "note: these tests create virtual serial port pairs using socat"
echo ""

# run the socat tests
cargo test --test socat_tests -- --ignored --nocapture

echo ""
echo "socat integration tests completed!"
