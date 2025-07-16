#!/bin/bash

# Local build script for mdBook development
# Usage: ./scripts/build.sh [options]
#   --serve    Start local development server
#   --test     Run validation tests
#   --clean    Clean build directory first
#   --help     Show this help

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default options
SERVE=false
TEST=false
CLEAN=false
HELP=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --serve)
            SERVE=true
            shift
            ;;
        --test)
            TEST=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --help)
            HELP=true
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Show help
if [ "$HELP" = true ]; then
    echo "Local mdBook build script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --serve    Start local development server (http://localhost:3000)"
    echo "  --test     Run validation tests after building"
    echo "  --clean    Clean build directory before building"
    echo "  --help     Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    # Just build the book"
    echo "  $0 --serve           # Build and serve locally"
    echo "  $0 --clean --test    # Clean build and run tests"
    exit 0
fi

echo -e "${BLUE}🔧 mdBook Local Build Script${NC}"
echo "================================"

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    echo -e "${RED}❌ mdbook is not installed${NC}"
    echo "Please install mdbook first:"
    echo "  cargo install mdbook"
    echo "  # or"
    echo "  brew install mdbook"
    exit 1
fi

# Clean if requested
if [ "$CLEAN" = true ]; then
    echo -e "${YELLOW}🧹 Cleaning build directory...${NC}"
    rm -rf book/
    rm -rf book-test/
fi

# Build the book
echo -e "${BLUE}📚 Building mdBook...${NC}"
if mdbook build; then
    echo -e "${GREEN}✅ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

# Run tests if requested
if [ "$TEST" = true ]; then
    echo -e "${BLUE}🧪 Running validation tests...${NC}"
    
    # Test 1: Check required files exist
    echo "Checking required files..."
    required_files=(
        "book/index.html"
        "book/introduction.html"
        "book/print.html"
        "book/searchindex.js"
    )
    
    for file in "${required_files[@]}"; do
        if [ -f "$file" ]; then
            echo -e "  ${GREEN}✓${NC} $file"
        else
            echo -e "  ${RED}✗${NC} $file (missing)"
            exit 1
        fi
    done
    
    # Test 2: Check chapter directories
    echo "Checking chapter directories..."
    required_dirs=(
        "book/core-concepts"
        "book/cryptography"
        "book/embedded-patterns"
        "book/migration"
        "book/quick-reference"
    )
    
    for dir in "${required_dirs[@]}"; do
        if [ -d "$dir" ]; then
            echo -e "  ${GREEN}✓${NC} $dir"
        else
            echo -e "  ${RED}✗${NC} $dir (missing)"
            exit 1
        fi
    done
    
    # Test 3: Run link validation if available
    if [ -f "link_validator.py" ]; then
        echo "Running link validation..."
        if python3 link_validator.py; then
            echo -e "  ${GREEN}✓${NC} Link validation passed"
        else
            echo -e "  ${YELLOW}⚠${NC} Link validation had warnings"
        fi
    fi
    
    echo -e "${GREEN}✅ All tests passed${NC}"
fi

# Serve if requested
if [ "$SERVE" = true ]; then
    echo -e "${BLUE}🌐 Starting development server...${NC}"
    echo -e "${GREEN}📖 Book will be available at: http://localhost:3000${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
    echo ""
    mdbook serve --open
fi

echo -e "${GREEN}🎉 Done!${NC}"