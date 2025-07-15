# Content Validation and Cross-Reference System

This system provides comprehensive validation for the embedded Rust tutorial, checking code examples, cross-references, and content redundancy.

## Quick Start

### Run Complete Validation
```bash
python validate_tutorial.py
```

### Run Individual Components
```bash
# Code example validation only
python code_example_tester.py embedded-rust-tutorial.md

# Cross-reference validation only
python cross_reference_validator.py embedded-rust-tutorial.md

# Redundancy analysis only
python redundancy_detector.py embedded-rust-tutorial.md

# Comprehensive validation
python content_validation_system.py embedded-rust-tutorial.md
```

## System Components

### 1. Code Example Tester (`code_example_tester.py`)
- **Purpose**: Validates Rust code examples for syntax and compilation
- **Features**:
  - Extracts code blocks from markdown
  - Tests Rust syntax using `rustc --parse-only`
  - Handles embedded-specific code patterns
  - Skips non-Rust content (diagrams, C code, comments)
  - Generates detailed error reports

### 2. Cross-Reference Validator (`cross_reference_validator.py`)
- **Purpose**: Validates internal links and cross-references
- **Features**:
  - Extracts all anchors and links from document
  - Validates that all internal links resolve correctly
  - Identifies orphaned anchors (unused sections)
  - Analyzes section connectivity
  - Provides suggestions for fixing broken links

### 3. Redundancy Detector (`redundancy_detector.py`)
- **Purpose**: Identifies duplicate and redundant content
- **Features**:
  - Segments content by type (paragraphs, code, tables, lists)
  - Finds exact duplicates and near-duplicates
  - Identifies conceptually similar content
  - Calculates redundancy statistics
  - Provides diff summaries for similar content

### 4. Comprehensive Validation System (`content_validation_system.py`)
- **Purpose**: Integrates all validation components
- **Features**:
  - Runs all validation checks
  - Calculates overall quality score
  - Generates actionable recommendations
  - Creates comprehensive reports
  - Provides executive summary

## Output Reports

All reports are saved to the `validation_output/` directory:

### Main Reports
- `comprehensive_validation_report.md` - Executive summary with scores and recommendations
- `validation_summary.json` - Machine-readable summary data

### Detailed Reports
- `code_validation_report.md` - Detailed code example analysis
- `cross_reference_report.json` - Link validation details
- `redundancy_report.json` - Content redundancy analysis

## Quality Scoring

The system calculates an overall quality score (0-100) based on:

- **Code Quality (40%)**: Percentage of code examples that compile successfully
- **Cross-References (30%)**: Percentage of internal links that resolve correctly  
- **Content Organization (30%)**: Inverse of content redundancy rate

### Grade Scale
- **A (90-100)**: Excellent - Ready for publication
- **B (80-89)**: Good - Minor issues to address
- **C (70-79)**: Fair - Moderate improvements needed
- **D (60-69)**: Poor - Significant issues present
- **F (0-59)**: Critical Issues - Major problems require immediate attention

## Prerequisites

### Required Tools
- Python 3.7+
- Rust toolchain (for code validation)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Python Dependencies
The system uses only standard library modules:
- `re`, `json`, `pathlib`, `subprocess`, `tempfile`
- `dataclasses`, `typing`, `collections`, `difflib`

## Usage Examples

### Basic Validation
```bash
# Validate main tutorial
python validate_tutorial.py

# Validate specific document
python validate_tutorial.py my-document.md
```

### Continuous Integration
```bash
# Exit with error code if critical issues found
python content_validation_system.py embedded-rust-tutorial.md
if [ $? -ne 0 ]; then
    echo "Validation failed - check reports"
    exit 1
fi
```

### Development Workflow
1. Make changes to tutorial
2. Run validation: `python validate_tutorial.py`
3. Check comprehensive report for issues
4. Fix critical issues first (broken links, syntax errors)
5. Address warnings (redundancy, orphaned anchors)
6. Re-run validation to verify improvements

## Interpreting Results

### Code Validation Issues
- **Syntax Errors**: Code doesn't parse correctly
- **Compilation Failures**: Code parses but doesn't compile
- **Skipped Examples**: Non-Rust content (diagrams, C code)

### Cross-Reference Issues
- **Broken Links**: Links to non-existent anchors
- **Orphaned Anchors**: Sections never referenced
- **Missing Anchors**: Links without corresponding targets

### Redundancy Issues
- **Exact Duplicates**: Identical content in multiple places
- **Near Duplicates**: Very similar content (90%+ similarity)
- **Conceptual Overlaps**: Different sections covering same topics

## Customization

### Adding New Code Patterns
Edit `code_example_tester.py` to handle new code patterns:
```python
def is_rust_code(self, code: str) -> bool:
    # Add new patterns to skip
    skip_patterns = ['NEW_PATTERN']
    # Add new Rust patterns to detect
    rust_patterns = ['new_rust_pattern']
```

### Adjusting Redundancy Thresholds
Edit `redundancy_detector.py` to change similarity thresholds:
```python
# Near duplicates threshold (default: 0.9)
if similarity >= 0.9:
    
# Similar content threshold (default: 0.7-0.9)
if 0.7 <= similarity < 0.9:
```

### Custom Scoring Weights
Edit `content_validation_system.py` to adjust component weights:
```python
def calculate_overall_score(self, ...):
    code_score = code_validation["success_rate"] * 40      # 40% weight
    cross_ref_score = cross_ref_validation["success_rate"] * 30  # 30% weight
    organization_score = (1 - redundancy_penalty) * 30    # 30% weight
```

## Troubleshooting

### Common Issues

**"rustc not found"**
- Install Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Restart terminal or run: `source ~/.cargo/env`

**"Permission denied"**
- Make scripts executable: `chmod +x *.py`

**"Module not found"**
- Ensure all validation scripts are in the same directory
- Check Python path: `python -c "import sys; print(sys.path)"`

**High false positive rate**
- Review `is_rust_code()` function in `code_example_tester.py`
- Add patterns for content that should be skipped

### Performance Optimization

For large documents:
- Run individual components separately
- Use `--parse-only` flag for faster code checking
- Adjust redundancy detection thresholds

## Contributing

To improve the validation system:

1. **Add new validation rules**: Extend existing validators
2. **Improve pattern recognition**: Enhance code/content detection
3. **Add new report formats**: Create additional output formats
4. **Optimize performance**: Improve processing speed for large documents

## License

This validation system is part of the embedded Rust tutorial project and follows the same license terms.