# Build and Deployment Guide

This document describes how to build, test, and deploy the Embedded Rust Tutorial mdBook.

## Prerequisites

### Required Tools

- **mdBook**: The static site generator for the book
  ```bash
  # Install via Cargo (recommended)
  cargo install mdbook
  
  # Or via package manager
  # macOS
  brew install mdbook
  
  # Ubuntu/Debian
  sudo apt install mdbook
  ```

- **Python 3.8+**: For validation scripts
  ```bash
  # Install required Python packages
  pip install requests beautifulsoup4 markdown
  ```

- **Rust** (optional): For code validation
  ```bash
  # Install via rustup
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

## Local Development

### Quick Start

The fastest way to start developing:

```bash
# Start development server with auto-reload
./scripts/dev.sh
```

This will:
- Install mdBook if not present
- Start a local server at http://localhost:3000
- Auto-reload when you save changes

### Manual Build

```bash
# Simple build
mdbook build

# Build and serve locally
mdbook serve --open

# Build to custom directory
mdbook build --dest-dir custom-output
```

### Development Scripts

We provide several scripts in the `scripts/` directory:

#### `./scripts/build.sh`

Comprehensive build script with options:

```bash
# Just build
./scripts/build.sh

# Build and serve locally
./scripts/build.sh --serve

# Clean build with validation
./scripts/build.sh --clean --test

# Show all options
./scripts/build.sh --help
```

#### `./scripts/dev.sh`

Quick development server startup:

```bash
./scripts/dev.sh
```

#### `./scripts/validate.sh`

Run all validation checks:

```bash
./scripts/validate.sh
```

## Validation and Testing

### Local Validation

Before committing changes, run validation:

```bash
# Run all validations
./scripts/validate.sh

# Or run specific validations
mdbook build --dest-dir book-test  # Build test
python3 link_validator.py         # Link validation
python3 content_validation_system.py  # Content validation
```

### What Gets Validated

1. **Build Validation**: Ensures mdBook builds successfully
2. **Structure Validation**: Checks all required files and directories exist
3. **SUMMARY.md Validation**: Verifies all referenced files exist
4. **Link Validation**: Checks internal links work correctly
5. **Content Validation**: Ensures no content loss during conversion
6. **Code Validation**: Validates Rust code examples compile

## Continuous Integration

### GitHub Actions Workflows

We have three CI workflows:

#### 1. `test.yml` - Validation and Testing

Runs on every push and PR:
- Validates book configuration
- Checks SUMMARY.md structure
- Tests build output
- Validates links and content
- Tests Rust code examples

#### 2. `deploy.yml` - Build and Deploy

Runs on pushes to main/master:
- Tests the build
- Deploys to GitHub Pages (legacy method)

#### 3. `pages.yml` - GitHub Pages Deployment

Modern GitHub Pages deployment:
- Uses GitHub's official Pages actions
- Supports custom domains
- Better security with OIDC

### Setting Up GitHub Pages

1. **Enable GitHub Pages** in your repository settings
2. **Set source** to "GitHub Actions"
3. **Push to main branch** to trigger deployment

The book will be available at: `https://yourusername.github.io/repository-name`

### Custom Domain Setup

To use a custom domain:

1. Add your domain to the `cname` field in `.github/workflows/pages.yml`
2. Configure DNS to point to GitHub Pages
3. Enable HTTPS in repository settings

## Configuration

### book.toml

The main configuration file controls:

- Book metadata (title, authors, description)
- Build settings (source directory, output format)
- Theme and styling options
- Search functionality
- Print support
- Git integration

Key settings:

```toml
[book]
title = "Embedded Rust Tutorial for Cryptography Engineers"
src = "src"

[output.html]
default-theme = "navy"
git-repository-url = "https://github.com/your-repo/embedded-rust-tutorial"

[output.html.search]
enable = true
limit-results = 30
```

### Theme Customization

Custom styling is in:
- `theme/custom.css` - Custom CSS styles
- `theme/custom.js` - Custom JavaScript

## Troubleshooting

### Common Issues

#### mdBook Not Found
```bash
# Install mdBook
cargo install mdbook
# or
brew install mdbook
```

#### Build Fails
```bash
# Check book.toml syntax
mdbook build --dest-dir test-build

# Validate SUMMARY.md
./scripts/validate.sh
```

#### Links Broken
```bash
# Run link validation
python3 link_validator.py

# Check SUMMARY.md references
./scripts/validate.sh
```

#### GitHub Pages Not Updating
1. Check Actions tab for failed workflows
2. Verify GitHub Pages is enabled
3. Check branch protection rules
4. Ensure GITHUB_TOKEN has proper permissions

### Debug Mode

For detailed build information:

```bash
# Verbose build output
RUST_LOG=debug mdbook build

# Test build with different destination
mdbook build --dest-dir debug-build
```

## Performance Optimization

### Build Performance

- Use `mdbook build` for production builds
- Use `mdbook serve` for development (faster rebuilds)
- Consider using `--dest-dir` for parallel builds

### Search Performance

The search index is automatically optimized, but you can tune:

```toml
[output.html.search]
limit-results = 30        # Limit search results
teaser-word-count = 30   # Preview text length
boost-title = 2          # Title relevance boost
```

## Deployment Options

### GitHub Pages (Recommended)

Automatic deployment via GitHub Actions (see workflows above).

### Netlify

1. Connect your GitHub repository
2. Set build command: `mdbook build`
3. Set publish directory: `book`

### Custom Server

```bash
# Build static files
mdbook build

# Copy book/ directory to your web server
rsync -av book/ user@server:/var/www/html/
```

## Contributing

When contributing to the book:

1. **Test locally** with `./scripts/dev.sh`
2. **Validate changes** with `./scripts/validate.sh`
3. **Check CI passes** before merging
4. **Update this guide** if adding new build features

## Support

For build issues:
1. Check this guide first
2. Run `./scripts/validate.sh` for diagnostics
3. Check GitHub Actions logs
4. Open an issue with build output