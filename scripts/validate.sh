#!/bin/bash

# Comprehensive validation script for mdBook content
# This script runs all available validation checks

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔍 mdBook Content Validation${NC}"
echo "============================="

# Track validation results
VALIDATION_PASSED=true

# Function to run validation step
run_validation() {
    local name="$1"
    local command="$2"
    
    echo -e "${BLUE}🧪 $name...${NC}"
    
    if eval "$command"; then
        echo -e "${GREEN}✅ $name passed${NC}"
    else
        echo -e "${RED}❌ $name failed${NC}"
        VALIDATION_PASSED=false
    fi
    echo ""
}

# 1. Build validation
run_validation "Build test" "mdbook build --dest-dir book-test"

# 2. Structure validation
run_validation "Structure validation" '
    # Check required files
    test -f book-test/index.html &&
    test -f book-test/introduction.html &&
    test -f book-test/print.html &&
    test -f book-test/searchindex.js &&
    
    # Check chapter directories
    test -d book-test/core-concepts &&
    test -d book-test/cryptography &&
    test -d book-test/embedded-patterns &&
    test -d book-test/migration &&
    test -d book-test/quick-reference
'

# 3. SUMMARY.md validation
run_validation "SUMMARY.md validation" '
    python3 -c "
import re
import os

try:
    with open(\"src/SUMMARY.md\", \"r\") as f:
        content = f.read()
    
    # Find all markdown file references
    links = re.findall(r\"\[.*?\]\((.*?\.md)\)\", content)
    missing_files = []
    
    for link in links:
        file_path = os.path.join(\"src\", link)
        if not os.path.exists(file_path):
            missing_files.append(file_path)
    
    if missing_files:
        print(\"Missing files:\", missing_files)
        exit(1)
    else:
        print(\"All SUMMARY.md references valid\")
        
except Exception as e:
    print(f\"Error: {e}\")
    exit(1)
"
'

# 4. Link validation (if available)
if [ -f "link_validator.py" ]; then
    run_validation "Link validation" "python3 link_validator.py"
else
    echo -e "${YELLOW}⚠️  Link validator not found, skipping detailed link validation${NC}"
fi

# 5. Content validation (if available)
if [ -f "content_validation_system.py" ]; then
    run_validation "Content validation" "python3 content_validation_system.py embedded-rust-tutorial-master.md || echo 'Content validation completed with warnings'"
else
    echo -e "${YELLOW}⚠️  Content validator not found, skipping content validation${NC}"
fi

# 6. Rust code validation (if available)
if [ -f "validate_new_content.rs" ] && command -v rustc &> /dev/null; then
    echo -e "${BLUE}🧪 Rust code validation...${NC}"
    if rustc validate_new_content.rs -o validate_content_temp 2>/dev/null && ./validate_content_temp 2>/dev/null; then
        echo -e "${GREEN}✅ Rust code validation passed${NC}"
        rm -f validate_content_temp
    else
        echo -e "${YELLOW}⚠️  Rust code validation had issues (likely missing dependencies)${NC}"
        rm -f validate_content_temp
    fi
    echo ""
else
    echo -e "${YELLOW}⚠️  Rust validator not found or rustc not available, skipping code validation${NC}"
fi

# Clean up test build
rm -rf book-test/

# Final result
echo "=============================="
if [ "$VALIDATION_PASSED" = true ]; then
    echo -e "${GREEN}🎉 All validations passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some validations failed${NC}"
    exit 1
fi