# Design Document

## Overview

This design implements a comprehensive system for making all Rust code examples in the embedded Rust tutorial compilable and testable, similar to the official Rust book. The solution integrates with the existing mdBook infrastructure and CI/CD pipeline to provide automated validation of code examples while maintaining the tutorial's embedded cryptography focus.

## Architecture

### Core Components

1. **mdBook Integration Layer**
   - Extends existing `book.toml` configuration
   - Integrates with mdBook's built-in testing capabilities
   - Provides custom preprocessors for embedded-specific handling

2. **Code Example Testing Framework**
   - Builds upon existing validation infrastructure
   - Supports multiple compilation targets (std, no_std, embedded)
   - Handles feature-gated and platform-specific code

3. **CI/CD Integration**
   - Extends existing GitHub Actions workflow
   - Provides fast feedback on code example validity
   - Supports multiple Rust toolchain versions

4. **Development Tools**
   - Local testing utilities for contributors
   - Code example generation and validation helpers
   - Integration with existing validation suite

## Components and Interfaces

### 1. mdBook Configuration Enhancement

**File: `book.toml`**
```toml
[output.html.playground]
copyable = true
copy-js = true  
line-numbers = true
editable = true
runnable = true

[preprocessor.rust-examples]
command = "mdbook-rust-examples"
renderer = ["html"]

[preprocessor.rust-examples.config]
embedded-target = "thumbv7em-none-eabihf"
no-std-mode = true
feature-gates = ["crypto", "hardware"]
```

### 2. Code Example Testing Framework

**Core Interface: `RustExampleTester`**
```rust
pub struct RustExampleTester {
    config: TestConfig,
    toolchain: ToolchainManager,
    validator: CodeValidator,
}

pub struct TestConfig {
    pub targets: Vec<String>,
    pub features: Vec<String>,
    pub no_std: bool,
    pub embedded_mode: bool,
}

impl RustExampleTester {
    pub fn test_example(&self, code: &str, context: ExampleContext) -> TestResult;
    pub fn validate_all_examples(&self, book_path: &Path) -> ValidationReport;
    pub fn generate_test_project(&self, examples: &[CodeExample]) -> Result<TempProject>;
}
```

### 3. Example Context System

**Context Types:**
- `StdContext`: Standard library examples
- `NoStdContext`: No-std embedded examples  
- `HardwareContext`: Hardware-specific examples
- `CryptoContext`: Cryptography-focused examples
- `SnippetContext`: Code snippets that shouldn't compile standalone

**Context Detection:**
```rust
pub enum ExampleContext {
    Std { features: Vec<String> },
    NoStd { target: String, features: Vec<String> },
    Hardware { platform: String, features: Vec<String> },
    Crypto { algorithm: String, features: Vec<String> },
    Snippet { reason: String }, // Won't be compiled
}
```

### 4. Code Example Annotation System

**Markdown Annotations:**
```markdown
# Standard compilable example
```rust
use std::collections::HashMap;
fn main() { println!("Hello"); }
```

# No-std embedded example  
```rust,no_std
#![no_std]
use heapless::Vec;
```

# Hardware-specific example
```rust,hardware=stm32f4
use stm32f4xx_hal::gpio::*;
```

# Code snippet (won't compile standalone)
```rust,snippet
// This is just a fragment
let key = generate_key();
```

# Feature-gated example
```rust,features=crypto,hardware
use crypto_hardware::aes::*;
```
```

## Data Models

### 1. Code Example Model

```rust
#[derive(Debug, Clone)]
pub struct CodeExample {
    pub id: String,
    pub source_file: PathBuf,
    pub line_number: usize,
    pub code: String,
    pub context: ExampleContext,
    pub annotations: Vec<Annotation>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]  
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
    pub optional: bool,
}
```

### 2. Test Result Model

```rust
#[derive(Debug)]
pub struct TestResult {
    pub example_id: String,
    pub success: bool,
    pub compilation_time: Duration,
    pub target: String,
    pub error: Option<CompilationError>,
    pub warnings: Vec<CompilationWarning>,
}

#[derive(Debug)]
pub struct ValidationReport {
    pub total_examples: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub results: Vec<TestResult>,
    pub summary: ValidationSummary,
}
```

## Error Handling

### 1. Compilation Error Categories

```rust
#[derive(Debug)]
pub enum CompilationError {
    SyntaxError { message: String, line: usize },
    DependencyError { missing: Vec<String> },
    FeatureError { unsupported: Vec<String> },
    TargetError { incompatible_target: String },
    TimeoutError { duration: Duration },
}
```

### 2. Error Recovery Strategies

- **Syntax Errors**: Provide detailed error messages with suggestions
- **Dependency Errors**: Auto-generate Cargo.toml with required dependencies
- **Feature Errors**: Skip examples with unsupported features gracefully
- **Target Errors**: Fall back to compatible targets when possible

### 3. Graceful Degradation

- Examples that fail compilation are marked but don't break the build
- Clear indicators show which examples are validated vs. snippets
- Fallback to syntax-only validation when full compilation fails

## Testing Strategy

### 1. Unit Testing

**Test Categories:**
- Code example parsing and annotation detection
- Context inference from code content
- Dependency resolution and Cargo.toml generation
- Compilation result processing and error handling

**Key Test Cases:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_no_std_context_detection() {
        let code = "#![no_std]\nuse heapless::Vec;";
        let context = infer_context(code);
        assert!(matches!(context, ExampleContext::NoStd { .. }));
    }
    
    #[test]
    fn test_crypto_example_compilation() {
        let code = "use zeroize::Zeroize;\nlet mut key = [0u8; 32];";
        let result = test_example(code, ExampleContext::Crypto { .. });
        assert!(result.success);
    }
    
    #[test]
    fn test_hardware_feature_gating() {
        let code = "#[cfg(feature = \"stm32f4\")]\nuse stm32f4xx_hal::*;";
        let result = test_with_features(code, vec!["stm32f4"]);
        assert!(result.success);
    }
}
```

### 2. Integration Testing

**Test Scenarios:**
- Full book compilation with all examples
- CI/CD pipeline integration testing
- Cross-platform compilation testing
- Performance benchmarking for large codebases

### 3. End-to-End Testing

**Validation Workflow:**
1. Extract all code examples from markdown files
2. Categorize examples by context and annotations
3. Generate test projects for each category
4. Compile examples against multiple targets
5. Generate comprehensive validation report
6. Update book with compilation status indicators

## Implementation Phases

### Phase 1: Core Infrastructure
- Implement code example extraction and parsing
- Create basic compilation testing framework
- Integrate with existing validation system
- Add mdBook configuration enhancements

### Phase 2: Advanced Features
- Implement context inference and annotation system
- Add support for multiple compilation targets
- Create dependency resolution system
- Integrate with CI/CD pipeline

### Phase 3: User Experience
- Add visual indicators for compilation status
- Implement local development tools
- Create contributor documentation
- Add performance optimizations

### Phase 4: Advanced Validation
- Implement cross-reference validation between examples
- Add security-focused validation rules
- Create automated example generation tools
- Add integration with external validation services

## Security Considerations

### 1. Code Execution Safety
- All code compilation happens in isolated environments
- No arbitrary code execution during validation
- Sandboxed compilation with resource limits
- Secure handling of temporary files and directories

### 2. Dependency Management
- Validate all external dependencies for security
- Pin dependency versions for reproducible builds
- Audit cryptographic dependencies regularly
- Implement dependency vulnerability scanning

### 3. Embedded Security Patterns
- Validate that crypto examples follow security best practices
- Ensure memory safety patterns are demonstrated correctly
- Verify that timing-attack resistant patterns are used
- Check for proper key zeroization in examples

## Performance Considerations

### 1. Compilation Optimization
- Parallel compilation of independent examples
- Incremental compilation for unchanged examples
- Caching of compilation artifacts
- Smart dependency resolution to minimize rebuilds

### 2. CI/CD Efficiency
- Fast-fail strategies for critical errors
- Parallel job execution for different targets
- Artifact caching between builds
- Selective testing based on changed files

### 3. Development Workflow
- Local testing tools for rapid iteration
- Incremental validation during development
- Quick syntax-only checks for fast feedback
- Full validation as pre-commit hooks

## Monitoring and Metrics

### 1. Compilation Metrics
- Success/failure rates by example category
- Compilation time trends
- Dependency resolution performance
- Error pattern analysis

### 2. Quality Metrics
- Code coverage of validation tests
- Example freshness (last validation date)
- Dependency update frequency
- Security audit compliance

### 3. User Experience Metrics
- Build time impact on documentation generation
- Developer productivity improvements
- Error resolution time
- Contribution workflow efficiency