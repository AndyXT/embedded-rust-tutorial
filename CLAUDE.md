# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an **mdBook documentation project** for teaching embedded Rust to experienced C programmers, specifically targeting cryptography engineers working with ARM Cortex-R5 processors. The repository contains only documentation - no Rust source code.

## Essential Commands

```bash
# Build the book
make build
# or
mdbook build

# Serve locally with auto-reload (opens browser)
make serve
# or
mdbook serve --open

# Clean build artifacts
make clean

# Run validation tests
make test
# or
./scripts/ci_local_test.sh

# Install mdBook if not present
make install
```

## Architecture & Structure

The project uses mdBook to generate a static documentation site. Key components:

- **src/**: Markdown source files organized by topic
  - `SUMMARY.md`: Table of contents that defines book structure
  - Each subdirectory represents a major section with its own README.md
- **book/**: Generated HTML output (git-ignored)
- **theme/**: Custom styling and JavaScript functionality
  - `cortex-r5-playground.js`: Interactive code examples for Cortex-R5
- **book.toml**: mdBook configuration with search settings and custom theme

## Content Focus

All code examples in the documentation should:
- Target ARM Cortex-R5 (`armv7r-none-eabi`)
- Use `#![no_std]` (no standard library)
- Avoid heap allocation
- Include proper panic handlers (`panic_halt`)
- Use RustCrypto crates (heapless, cortex-r-rt, sha2, aes, etc.)

## CI/CD Pipeline

GitHub Actions workflow (`.github/workflows/test.yml`):
1. Builds mdBook
2. Validates required output files and directories
3. Checks for broken internal links
4. Uploads build artifacts

## Key Documentation Sections

- **Environment Setup**: Target configuration for Cortex-R5, build settings
- **Core Concepts**: Ownership, error handling, type system advantages
- **Embedded Patterns**: no_std programming, hardware abstraction, interrupts
- **Cryptography**: Constant-time operations, secure patterns, side-channel mitigations
- **Migration**: Strategies for transitioning from C to Rust
- **Quick Reference**: C-to-Rust syntax mapping

## Important Notes

- The book is designed for readers who are already expert C programmers
- Examples should be production-ready and compile without modification
- Focus on practical embedded cryptography patterns
- Maintain security-first approach in all examples