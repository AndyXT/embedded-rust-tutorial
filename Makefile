# Simple Makefile for mdBook

.PHONY: all build serve clean dev test install help

# Default target
all: build

# Build the book
build:
	@echo "📚 Building mdBook..."
	@mdbook build

# Serve the book locally with auto-reload
serve:
	@echo "🚀 Starting mdBook server..."
	@mdbook serve --open

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@mdbook clean
	@rm -rf book/

# Development mode - serve with auto-reload
dev: serve

# Run validation tests
test:
	@echo "🧪 Running validation..."
	@./scripts/ci_local_test.sh

# Install mdBook if not present
install:
	@if ! command -v mdbook >/dev/null 2>&1; then \
		echo "📦 Installing mdBook..."; \
		cargo install mdbook; \
	else \
		echo "✅ mdBook already installed"; \
	fi

# Show help
help:
	@echo "Available commands:"
	@echo "  make build   - Build the book"
	@echo "  make serve   - Serve locally with auto-reload"
	@echo "  make clean   - Clean build artifacts"
	@echo "  make dev     - Start development server"
	@echo "  make test    - Run validation tests"
	@echo "  make install - Install mdBook if needed"
	@echo "  make help    - Show this help message"