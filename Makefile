# Makefile for mdBook development
# Provides convenient shortcuts for common tasks

.PHONY: help build serve test validate clean install dev

# Default target
help:
	@echo "Available targets:"
	@echo "  build     - Build the mdBook"
	@echo "  serve     - Build and serve locally with auto-reload"
	@echo "  test      - Build and run validation tests"
	@echo "  validate  - Run comprehensive validation"
	@echo "  clean     - Clean build artifacts"
	@echo "  install   - Install mdBook if not present"
	@echo "  dev       - Quick development server startup"
	@echo ""
	@echo "Examples:"
	@echo "  make build"
	@echo "  make serve"
	@echo "  make test"

# Build the book
build:
	@echo "🔧 Building mdBook..."
	mdbook build

# Serve locally with auto-reload
serve:
	@echo "🌐 Starting development server..."
	mdbook serve --open

# Build and test
test: build
	@echo "🧪 Running tests..."
	@./scripts/build.sh --test

# Run comprehensive validation
validate:
	@echo "🔍 Running validation..."
	@./scripts/validate.sh

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	rm -rf book/
	rm -rf book-test/
	rm -f validate_content_temp

# Install mdBook if not present
install:
	@if ! command -v mdbook >/dev/null 2>&1; then \
		echo "📦 Installing mdBook..."; \
		cargo install mdbook; \
	else \
		echo "✅ mdBook already installed"; \
	fi

# Quick development startup
dev:
	@./scripts/dev.sh