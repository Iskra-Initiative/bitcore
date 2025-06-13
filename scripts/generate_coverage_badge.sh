#!/bin/bash
# -- script to generate coverage badge

set -e

echo "generating coverage badge..."

# check if lcov file exists
if [ ! -f "coverage.lcov" ]; then
    echo "error: coverage.lcov not found"
    echo "run ./scripts/coverage.sh first to generate coverage data"
    exit 1
fi

# extract coverage percentage
COVERAGE=$(lcov --summary coverage.lcov 2>/dev/null | grep -o 'lines......: [0-9.]*%' | grep -o '[0-9.]*' || echo "0")

echo "coverage: $COVERAGE%"

# determine badge color based on coverage
if (( $(echo "$COVERAGE >= 90" | bc -l) )); then
    COLOR="brightgreen"
elif (( $(echo "$COVERAGE >= 80" | bc -l) )); then
    COLOR="green"
elif (( $(echo "$COVERAGE >= 70" | bc -l) )); then
    COLOR="yellow"
elif (( $(echo "$COVERAGE >= 60" | bc -l) )); then
    COLOR="orange"
else
    COLOR="red"
fi

# generate badge URL
BADGE_URL="https://img.shields.io/badge/coverage-${COVERAGE}%25-${COLOR}"

echo "badge URL: $BADGE_URL"
echo ""
echo "add this to your README.md:"
echo "[![Coverage](${BADGE_URL})](https://github.com/yourusername/bitcore/actions)"
echo ""

# optionally download the badge
if command -v curl &> /dev/null; then
    echo "downloading badge..."
    curl -s "$BADGE_URL" -o coverage-badge.svg
    echo "badge saved as: coverage-badge.svg"
fi

echo "coverage badge generated!"
