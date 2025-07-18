#!/bin/bash
# Local CI validation script - test CI steps locally before pushing

set -e

echo "🧪 Running local CI validation..."
echo ""

# Check mdBook
echo "📚 Checking mdBook..."
if command -v mdbook &> /dev/null; then
    echo "✅ mdBook is installed: $(mdbook --version)"
else
    echo "❌ mdBook is not installed. Install with: cargo install mdbook"
    exit 1
fi

# Build mdBook
echo ""
echo "📚 Building mdBook..."
mdbook build
if [ -f "book/index.html" ]; then
    echo "✅ mdBook built successfully"
else
    echo "❌ mdBook build failed"
    exit 1
fi

# Check Rust
echo ""
echo "🦀 Checking Rust..."
if command -v rustc &> /dev/null; then
    echo "✅ Rust is installed: $(rustc --version)"
else
    echo "❌ Rust is not installed"
    exit 1
fi

# Check formatting
echo ""
echo "🎨 Checking code formatting..."
if cargo fmt -- --check; then
    echo "✅ Code formatting is correct"
else
    echo "⚠️  Code formatting issues found (run: cargo fmt)"
fi

# Build project
echo ""
echo "🔨 Building project..."
if cargo build --release; then
    echo "✅ Project builds successfully"
else
    echo "❌ Build failed"
    exit 1
fi

# Run tests
echo ""
echo "🧪 Running tests..."
if cargo test; then
    echo "✅ Tests pass"
else
    echo "⚠️  Some tests failed"
fi

# Check for embedded target
echo ""
echo "🎯 Checking embedded targets..."
if rustup target list --installed | grep -q "thumbv7em-none-eabihf"; then
    echo "✅ Embedded target installed"
    
    echo "🔨 Building for embedded target..."
    if cargo build --target thumbv7em-none-eabihf --features embedded --release; then
        echo "✅ Embedded build successful"
    else
        echo "⚠️  Embedded build failed"
    fi
else
    echo "⚠️  Embedded target not installed (run: rustup target add thumbv7em-none-eabihf)"
fi

echo ""
echo "✨ Local CI validation complete!"
echo ""
echo "Summary:"
echo "- mdBook: ✅"
echo "- Rust build: ✅"
echo "- Tests: Check output above"
echo "- Formatting: Check output above"
echo ""
echo "If all checks pass, your code should pass CI!"