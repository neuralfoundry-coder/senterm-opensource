#!/bin/bash
#
# Senterm Test Runner Script
# ==========================
# Comprehensive test runner for the Senterm project.
#
# Usage:
#   ./scripts/run-tests.sh              # Run all tests
#   ./scripts/run-tests.sh --unit       # Run unit tests only
#   ./scripts/run-tests.sh --integration # Run integration tests only
#   ./scripts/run-tests.sh --module fs  # Run tests for specific module
#   ./scripts/run-tests.sh --coverage   # Run with coverage (requires cargo-tarpaulin)
#   ./scripts/run-tests.sh --verbose    # Run with verbose output
#   ./scripts/run-tests.sh --quick      # Quick test (no doc tests)
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default values
RUN_UNIT=true
RUN_INTEGRATION=true
RUN_DOC=true
VERBOSE=false
COVERAGE=false
MODULE=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --unit)
            RUN_UNIT=true
            RUN_INTEGRATION=false
            RUN_DOC=false
            shift
            ;;
        --integration)
            RUN_UNIT=false
            RUN_INTEGRATION=true
            RUN_DOC=false
            shift
            ;;
        --module)
            MODULE="$2"
            shift 2
            ;;
        --coverage)
            COVERAGE=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --quick)
            RUN_DOC=false
            shift
            ;;
        --help|-h)
            echo "Senterm Test Runner"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --unit          Run unit tests only"
            echo "  --integration   Run integration tests only"
            echo "  --module NAME   Run tests for specific module (fs, navigation, config, viewer)"
            echo "  --coverage      Run with code coverage (requires cargo-tarpaulin)"
            echo "  --verbose       Verbose output"
            echo "  --quick         Skip doc tests"
            echo "  --help, -h      Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Change to project root
cd "$PROJECT_ROOT"

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              Senterm Test Runner                             ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Build cargo test arguments
CARGO_ARGS=""
if [ "$VERBOSE" = true ]; then
    CARGO_ARGS="$CARGO_ARGS -- --nocapture"
fi

# Track test results
UNIT_RESULT=0
INTEGRATION_RESULT=0
DOC_RESULT=0

# Run tests for specific module
if [ -n "$MODULE" ]; then
    echo -e "${YELLOW}► Running tests for module: ${MODULE}${NC}"
    echo ""
    
    case $MODULE in
        fs)
            cargo test fs:: --lib $CARGO_ARGS || UNIT_RESULT=$?
            ;;
        navigation)
            cargo test navigation:: --lib $CARGO_ARGS || UNIT_RESULT=$?
            ;;
        config)
            cargo test config:: --lib $CARGO_ARGS || UNIT_RESULT=$?
            ;;
        viewer)
            cargo test viewer:: --lib $CARGO_ARGS || UNIT_RESULT=$?
            ;;
        *)
            echo -e "${RED}Unknown module: $MODULE${NC}"
            echo "Available modules: fs, navigation, config, viewer"
            exit 1
            ;;
    esac
else
    # Run unit tests
    if [ "$RUN_UNIT" = true ]; then
        echo -e "${YELLOW}► Running Unit Tests${NC}"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        
        if [ "$COVERAGE" = true ]; then
            # Check if cargo-tarpaulin is installed
            if ! command -v cargo-tarpaulin &> /dev/null; then
                echo -e "${YELLOW}Installing cargo-tarpaulin...${NC}"
                cargo install cargo-tarpaulin
            fi
            cargo tarpaulin --bin senterm --out Html --output-dir target/coverage || UNIT_RESULT=$?
            echo -e "${GREEN}Coverage report: target/coverage/tarpaulin-report.html${NC}"
        else
            # Run unit tests from the binary crate
            cargo test --bin senterm $CARGO_ARGS || UNIT_RESULT=$?
        fi
        echo ""
    fi

    # Run integration tests
    if [ "$RUN_INTEGRATION" = true ]; then
        echo -e "${YELLOW}► Running Integration Tests${NC}"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        cargo test --test integration_tests $CARGO_ARGS || INTEGRATION_RESULT=$?
        echo ""
    fi

    # Run doc tests
    if [ "$RUN_DOC" = true ]; then
        echo -e "${YELLOW}► Running Doc Tests${NC}"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        cargo test --doc $CARGO_ARGS || DOC_RESULT=$?
        echo ""
    fi
fi

# Summary
echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    Test Summary                              ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"

TOTAL_RESULT=0

if [ "$RUN_UNIT" = true ] || [ -n "$MODULE" ]; then
    if [ $UNIT_RESULT -eq 0 ]; then
        echo -e "  Unit Tests:        ${GREEN}✓ PASSED${NC}"
    else
        echo -e "  Unit Tests:        ${RED}✗ FAILED${NC}"
        TOTAL_RESULT=1
    fi
fi

if [ "$RUN_INTEGRATION" = true ] && [ -z "$MODULE" ]; then
    if [ $INTEGRATION_RESULT -eq 0 ]; then
        echo -e "  Integration Tests: ${GREEN}✓ PASSED${NC}"
    else
        echo -e "  Integration Tests: ${RED}✗ FAILED${NC}"
        TOTAL_RESULT=1
    fi
fi

if [ "$RUN_DOC" = true ] && [ -z "$MODULE" ]; then
    if [ $DOC_RESULT -eq 0 ]; then
        echo -e "  Doc Tests:         ${GREEN}✓ PASSED${NC}"
    else
        echo -e "  Doc Tests:         ${RED}✗ FAILED${NC}"
        TOTAL_RESULT=1
    fi
fi

echo ""

if [ $TOTAL_RESULT -eq 0 ]; then
    echo -e "${GREEN}All tests passed! ✓${NC}"
else
    echo -e "${RED}Some tests failed! ✗${NC}"
fi

exit $TOTAL_RESULT

