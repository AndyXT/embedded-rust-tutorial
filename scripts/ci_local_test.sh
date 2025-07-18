#!/bin/bash
# Local CI validation script for mdBook

set -e

echo "🧪 Running local mdBook validation..."
echo ""

# Check mdBook
echo "📚 Checking mdBook..."
if command -v mdbook &> /dev/null; then
    echo "✅ mdBook is installed: $(mdbook --version)"
else
    echo "❌ mdBook is not installed. Install with:"
    echo "   cargo install mdbook"
    echo "   or download from: https://github.com/rust-lang/mdBook/releases"
    exit 1
fi

# Clean previous build
echo ""
echo "🧹 Cleaning previous build..."
rm -rf book/

# Build mdBook
echo ""
echo "📚 Building mdBook..."
mdbook build

# Validate build
echo ""
echo "✅ Validating build output..."

# Check critical files
required_files=(
    "book/index.html"
    "book/introduction.html" 
    "book/print.html"
    "book/searchindex.js"
)

all_good=true
for file in "${required_files[@]}"; do
    if [ ! -f "$file" ]; then
        echo "❌ Missing required file: $file"
        all_good=false
    fi
done

# Check directories
required_dirs=(
    "book/core-concepts"
    "book/cryptography"
    "book/embedded-patterns"
    "book/migration"
    "book/quick-reference"
)

for dir in "${required_dirs[@]}"; do
    if [ ! -d "$dir" ]; then
        echo "❌ Missing required directory: $dir"
        all_good=false
    fi
done

if [ "$all_good" = true ]; then
    echo "✅ All required files and directories present"
fi

# Quick link check
echo ""
echo "🔗 Checking for obvious broken links..."
broken_count=$(find book -name "*.html" -type f -exec grep -l "404" {} \; 2>/dev/null | wc -l)
if [ "$broken_count" -gt 0 ]; then
    echo "⚠️  Some files contain '404' - might have broken links"
else
    echo "✅ No obvious broken links found"
fi

echo ""
echo "✨ Local validation complete!"
echo ""
echo "To serve the book locally, run:"
echo "   mdbook serve"
echo ""
echo "Then open http://localhost:3000 in your browser"