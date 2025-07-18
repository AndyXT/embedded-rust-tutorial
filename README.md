# Embedded Rust Tutorial for Cryptography Engineers

A comprehensive mdBook tutorial for experienced embedded cryptography C programmers transitioning to Rust. This book focuses on ARM Cortex-R5 development with no_std and no heap allocation.

## 📚 Quick Start

1. Install mdBook:
   ```bash
   cargo install mdbook
   # or download from https://github.com/rust-lang/mdBook/releases
   ```

2. Clone and serve:
   ```bash
   git clone <this-repo>
   cd rusty
   mdbook serve
   ```

3. Open http://localhost:3000 in your browser

## 🔨 Building

```bash
# Build the book
mdbook build

# Serve locally with auto-reload  
mdbook serve --open

# Clean build
mdbook clean

# Run local validation
./scripts/ci_local_test.sh
```

## 📁 Structure

```
rusty/
├── book.toml          # mdBook configuration
├── src/               # Markdown source files
│   ├── SUMMARY.md     # Table of contents
│   ├── introduction.md
│   ├── core-concepts/
│   ├── cryptography/
│   ├── embedded-patterns/
│   ├── environment-setup/
│   ├── migration/
│   └── quick-reference/
├── theme/             # Custom theme files
└── book/              # Generated HTML (git ignored)
```

## 🚀 Deployment

The book automatically deploys to GitHub Pages when changes are pushed to the main branch via GitHub Actions.

## 📖 Topics Covered

- **Core Concepts**: Ownership, borrowing, error handling, type system
- **Embedded Patterns**: no_std development, static memory, interrupts
- **Cryptography**: Constant-time operations, secure patterns, side-channel mitigations
- **Migration Strategies**: Moving from C to Rust incrementally
- **Quick Reference**: C-to-Rust syntax mapping and common patterns

## 🎯 Target Audience

- Experienced embedded C programmers
- Cryptography engineers
- Security-focused developers
- ARM Cortex-R5/M developers

## 📋 Key Features

- **No basic programming explanations** - Assumes C expertise
- **Crypto-focused examples** - Real-world cryptographic patterns
- **Quick reference tables** - Immediate C-to-Rust translations
- **Embedded-specific** - no_std, no heap, hardware integration
- **Production-ready examples** - Copy-paste ready code

## 🤝 Contributing

Contributions are welcome! Please ensure:
- Examples are complete and compile for Cortex-R5 targets
- Code follows no_std and no heap constraints
- Documentation is clear and concise

## 📄 License

MIT