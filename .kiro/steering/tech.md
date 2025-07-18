# Technology Stack

## Build System

**Primary Tool:** mdBook - Static site generator for technical documentation
- Configuration: `book.toml`
- Source directory: `src/`
- Output directory: `book/` (git ignored)

## Tech Stack

- **Documentation:** Markdown with mdBook extensions
- **Target Platform:** ARM Cortex-R5 (`thumbv7em-none-eabihf`)
- **Rust Environment:** `no_std` (no standard library), no heap allocation
- **Theme:** Custom navy theme with additional CSS/JS
- **Search:** Built-in mdBook search with custom configuration
- **Validation:** Python scripts for link checking and content validation

## Dependencies

### Required Tools
- **mdBook** - Install via `cargo install mdbook` or package manager
- **Python 3.8+** - For validation scripts
- **Rust** (optional) - For code validation

### Optional Tools
- **Cargo** - For installing mdBook
- **Git** - For version control and GitHub Pages deployment

## Common Commands

### Development
```bash
# Quick development server
make dev
# or
mdbook serve --open

# Build the book
make build
# or
mdbook build

# Clean build artifacts
make clean
# or
mdbook clean
```

### Testing and Validation
```bash
# Run all validation tests
make test
# or
./scripts/ci_local_test.sh

# Manual validation steps
mdbook build                    # Build test
python3 link_validator.py      # Link validation (if exists)
python3 content_validation_system.py  # Content validation (if exists)
```

### Installation
```bash
# Install mdBook if needed
make install
# or
cargo install mdbook

# Install Python dependencies (if validation scripts exist)
pip install requests beautifulsoup4 markdown
```

## Build Configuration

### mdBook Settings (`book.toml`)
- **Theme:** Navy (dark theme optimized for technical content)
- **Search:** Enabled with custom limits and boosting
- **Math:** MathJax support enabled
- **Playground:** Disabled (code examples are Cortex-R5 specific)
- **Print:** Enabled for PDF generation
- **Git Integration:** Edit links and repository references

### Custom Features
- **Custom CSS:** `theme/custom.css` for styling
- **Custom JS:** `theme/custom.js` and `theme/cortex-r5-playground.js`
- **Preprocessors:** Links preprocessor, rust-examples preprocessor (if configured)

## Deployment

### GitHub Pages (Automated)
- **Trigger:** Push to main/master branch
- **Workflow:** `.github/workflows/test.yml` and pages workflow
- **Custom Domain:** Configured via CNAME in book output

### Local Development
- **Server:** `mdbook serve` on http://localhost:3000
- **Auto-reload:** Enabled for development workflow
- **Hot-reload:** Changes reflected immediately in browser

## Code Examples

All code examples are:
- **Target:** ARM Cortex-R5 (`armv7r-none-eabi`)
- **Environment:** `no_std`, no heap allocation
- **Compilation:** `cargo build --target armv7r-none-eabi --release`
- **Note:** Cannot run in Rust Playground due to embedded target requirements