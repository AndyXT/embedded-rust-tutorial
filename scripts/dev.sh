#!/bin/bash

# Quick development script for mdBook
# This script provides common development tasks

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🚀 mdBook Development Helper${NC}"
echo "=============================="

# Check if mdbook is installed
if ! command -v mdbook &> /dev/null; then
    echo -e "${YELLOW}⚠️  mdbook not found. Installing...${NC}"
    if command -v cargo &> /dev/null; then
        cargo install mdbook
    else
        echo "Please install Rust and Cargo first, then run:"
        echo "  cargo install mdbook"
        exit 1
    fi
fi

# Quick serve with auto-reload
echo -e "${GREEN}📚 Starting mdBook development server with auto-reload...${NC}"
echo -e "${BLUE}🌐 Server will be available at: http://localhost:3000${NC}"
echo -e "${YELLOW}📝 Files will auto-reload when you save changes${NC}"
echo -e "${YELLOW}🛑 Press Ctrl+C to stop${NC}"
echo ""

mdbook serve --open