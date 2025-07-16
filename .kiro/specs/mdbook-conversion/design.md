# Design Document

## Overview

This design outlines the conversion of the embedded Rust tutorial from a single markdown document into a structured mdBook. The conversion will transform the existing comprehensive tutorial into a multi-chapter online book while preserving all content and improving navigation and user experience.

The current tutorial is approximately 6,000+ lines and covers six major sections. The design focuses on logical content splitting, maintaining cross-references, and leveraging mdBook's features for enhanced readability.

## Architecture

### Book Structure

The mdBook will follow this chapter organization based on the existing content structure:

```
src/
├── SUMMARY.md                 # Table of contents
├── introduction.md            # Introduction and how to use
├── quick-reference/
│   ├── README.md             # Quick Reference overview
│   ├── syntax-mapping.md     # C to Rust syntax
│   ├── memory-patterns.md    # Memory and pointer patterns
│   ├── control-flow.md       # Control flow and functions
│   ├── error-handling.md     # Error handling patterns
│   ├── crypto-reference.md   # Crypto-specific patterns
│   ├── embedded-reference.md # Embedded-specific patterns
│   └── gotchas.md           # Critical differences and gotchas
├── environment-setup/
│   ├── README.md             # Environment setup overview
│   ├── installation.md       # Rust installation and toolchain
│   ├── target-config.md      # Target configuration
│   ├── project-structure.md  # Project structure and dependencies
│   ├── build-config.md       # Build configuration
│   └── verification.md       # Verification and testing
├── core-concepts/
│   ├── README.md             # Core concepts overview
│   ├── ownership.md          # Ownership and memory management
│   ├── error-handling.md     # Error handling without exceptions
│   ├── type-system.md        # Type system advantages
│   ├── advanced-types.md     # Advanced type system features
│   ├── functional.md         # Functional programming
│   ├── memory-model.md       # Memory model differences
│   └── safety.md            # Safety guarantees for crypto
├── embedded-patterns/
│   ├── README.md             # Embedded patterns overview
│   ├── no-std.md            # No-std programming essentials
│   ├── hardware-abstraction.md # Hardware abstraction patterns
│   ├── interrupts.md         # Interrupt handling
│   ├── static-memory.md      # Static memory management
│   └── dma-integration.md    # DMA and hardware integration
├── cryptography/
│   ├── README.md             # Cryptography overview
│   ├── secure-patterns.md    # Secure coding patterns
│   ├── constant-time.md      # Constant-time implementations
│   ├── key-management.md     # Key management and zeroization
│   ├── hardware-crypto.md    # Hardware crypto acceleration
│   └── side-channels.md      # Side-channel mitigations
└── migration/
    ├── README.md             # Migration overview
    ├── strategies.md         # Incremental migration strategies
    ├── ffi-integration.md    # FFI integration with C libraries
    ├── testing.md           # Testing and validation
    ├── debugging.md         # Debugging and tooling
    └── performance.md       # Performance considerations
```

### Content Splitting Strategy

**Chapter-Level Splitting:**
- Each major section (1-6) becomes a chapter directory
- Chapter README.md provides overview and navigation
- Subsections become individual markdown files

**Cross-Reference Preservation:**
- Internal links will be updated to use mdBook's relative linking
- Cross-chapter references will use full paths
- Anchor links will be preserved where possible

**Content Preservation:**
- All existing content will be preserved
- Code examples will maintain syntax highlighting
- Tables and formatting will be preserved
- Expandable sections will be converted to appropriate mdBook features

## Components and Interfaces

### mdBook Configuration

**book.toml Configuration:**
```toml
[book]
authors = ["Embedded Rust Tutorial Contributors"]
language = "en"
multilingual = false
src = "src"
title = "Embedded Rust Tutorial for Cryptography Engineers"
description = "A streamlined guide for experienced embedded cryptography C programmers transitioning to Rust"

[preprocessor.links]

[output.html]
default-theme = "navy"
preferred-dark-theme = "navy"
copy-fonts = true
mathjax-support = true
git-repository-url = "https://github.com/your-repo/embedded-rust-tutorial"
edit-url-template = "https://github.com/your-repo/embedded-rust-tutorial/edit/main/{path}"

[output.html.search]
enable = true
limit-results = 30
teaser-word-count = 30
use-boolean-and = true
boost-title = 2
boost-hierarchy = 1
boost-paragraph = 1
expand = true
heading-split-level = 3

[output.html.print]
enable = true

[output.html.fold]
enable = false
level = 0
```

### Theme and Styling

**Custom CSS Integration:**
- Syntax highlighting optimized for Rust code
- Table styling for comparison tables
- Code block enhancements for copy functionality
- Responsive design for mobile devices

**Navigation Enhancements:**
- Chapter-based navigation sidebar
- Search functionality across all content
- Previous/next navigation between pages
- Breadcrumb navigation for deep sections

### Build System Integration

**GitHub Actions Workflow:**
```yaml
name: Deploy mdBook

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup mdBook
      uses: peaceiris/actions-mdbook@v1
      with:
        mdbook-version: 'latest'
    
    - name: Build book
      run: mdbook build
    
    - name: Deploy to GitHub Pages
      uses: peaceiris/actions-gh-pages@v3
      if: github.ref == 'refs/heads/main'
      with:
        github_token: ${{ secrets.GITHUB_TOKEN }}
        publish_dir: ./book
```

## Data Models

### Content Structure Model

**Chapter Structure:**
```rust
struct Chapter {
    title: String,
    path: String,
    sections: Vec<Section>,
    overview: String,
}

struct Section {
    title: String,
    filename: String,
    content: String,
    cross_references: Vec<CrossReference>,
}

struct CrossReference {
    source_anchor: String,
    target_chapter: String,
    target_section: String,
    link_text: String,
}
```

### Link Management

**Internal Link Mapping:**
- Original anchor links → mdBook relative paths
- Cross-section references → inter-chapter links
- Table of contents → SUMMARY.md structure
- Quick reference links → dedicated reference pages

## Error Handling

### Content Conversion Errors

**Missing Content Detection:**
- Automated verification that all original content is preserved
- Cross-reference validation to ensure no broken links
- Code block validation to ensure proper syntax highlighting

**Build Process Error Handling:**
- mdBook build validation
- Link checking for internal references
- Asset validation for any embedded images or files

### Deployment Error Handling

**Static Site Generation:**
- Build failure recovery and reporting
- Asset optimization and validation
- Cross-browser compatibility testing

## Testing Strategy

### Content Validation

**Automated Testing:**
1. **Content Preservation Test:** Verify all original content exists in converted form
2. **Link Validation Test:** Ensure all internal links work correctly
3. **Code Example Test:** Validate that all Rust code examples compile
4. **Cross-Reference Test:** Verify cross-chapter references are correct

**Manual Testing:**
1. **Navigation Testing:** Verify smooth navigation between chapters
2. **Search Functionality:** Test search across all content
3. **Mobile Responsiveness:** Test on various screen sizes
4. **Print Layout:** Verify print-friendly formatting

### Build and Deployment Testing

**CI/CD Pipeline Testing:**
1. **Build Validation:** Ensure mdBook builds successfully
2. **Deployment Testing:** Verify successful deployment to hosting platform
3. **Performance Testing:** Check page load times and search performance

### User Experience Testing

**Usability Validation:**
1. **Learning Path Testing:** Verify logical flow between chapters
2. **Reference Usage:** Test quick reference accessibility
3. **Code Copy Testing:** Verify code copy functionality works
4. **Cross-Platform Testing:** Test on different browsers and devices

## Implementation Considerations

### Content Migration Strategy

**Phase 1: Structure Setup**
- Create mdBook project structure
- Configure book.toml with appropriate settings
- Set up build and deployment pipeline

**Phase 2: Content Splitting**
- Split main document into logical chapters
- Create chapter overview pages
- Preserve all existing content

**Phase 3: Link and Reference Updates**
- Update internal links to use mdBook conventions
- Create cross-chapter reference system
- Validate all links and anchors

**Phase 4: Enhancement and Optimization**
- Add mdBook-specific features
- Optimize for search and navigation
- Implement responsive design improvements

### Maintenance and Updates

**Content Synchronization:**
- Process for updating book content
- Version control integration
- Automated testing for content changes

**Feature Enhancements:**
- Plugin integration for advanced features
- Custom preprocessors for specialized content
- Analytics integration for usage tracking