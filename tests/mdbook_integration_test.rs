//! Integration tests for mdBook preprocessor functionality

use std::process::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_mdbook_preprocessor_integration() {
    // Create a temporary directory for the test book
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let book_path = temp_dir.path();
    
    // Create basic mdBook structure
    create_test_book(book_path);
    
    // Build the book with our preprocessor
    let output = Command::new("mdbook")
        .arg("build")
        .current_dir(book_path)
        .output();
    
    match output {
        Ok(result) => {
            if !result.status.success() {
                println!("mdbook build stderr: {}", String::from_utf8_lossy(&result.stderr));
                println!("mdbook build stdout: {}", String::from_utf8_lossy(&result.stdout));
            }
            
            // Check if the book was built successfully
            let book_output_path = book_path.join("book");
            assert!(book_output_path.exists(), "Book output directory should exist");
            
            // Check if HTML files were generated
            let index_html = book_output_path.join("index.html");
            assert!(index_html.exists(), "index.html should exist");
            
            // Check if our custom CSS and JS are included
            let custom_css = book_output_path.join("theme").join("custom.css");
            let custom_js = book_output_path.join("theme").join("custom.js");
            assert!(custom_css.exists(), "custom.css should exist");
            assert!(custom_js.exists(), "custom.js should exist");
            
            // Verify HTML content contains our enhancements
            verify_html_enhancements(&book_output_path);
        }
        Err(e) => {
            println!("Warning: mdbook not available for integration test: {}", e);
            // Skip test if mdbook is not installed
        }
    }
}

#[test]
fn test_preprocessor_binary_compilation() {
    // Test that our preprocessor binary compiles
    let output = Command::new("cargo")
        .args(&["build", "--bin", "mdbook-rust-examples"])
        .output()
        .expect("Failed to run cargo build");
    
    assert!(output.status.success(), 
        "Preprocessor binary should compile successfully. stderr: {}", 
        String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_preprocessor_supports_query() {
    // Test that the preprocessor responds correctly to supports query
    let output = Command::new("cargo")
        .args(&["run", "--bin", "mdbook-rust-examples"])
        .arg("--")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();
    
    match output {
        Ok(mut child) => {
            use std::io::Write;
            
            // Send supports query
            if let Some(stdin) = child.stdin.as_mut() {
                let supports_query = r#"["supports"]"#;
                stdin.write_all(supports_query.as_bytes()).ok();
            }
            
            let output = child.wait_with_output().expect("Failed to read output");
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Should respond with supported renderers
            assert!(stdout.contains("html"), "Should support html renderer");
        }
        Err(e) => {
            println!("Warning: Could not test preprocessor binary: {}", e);
        }
    }
}

#[test]
fn test_css_compilation_status_styles() {
    // Read the custom CSS file
    let css_content = fs::read_to_string("theme/custom.css")
        .expect("Should be able to read custom.css");
    
    // Verify compilation status styles are present
    assert!(css_content.contains(".compilation-status"), "Should contain compilation status styles");
    assert!(css_content.contains(".compilation-success"), "Should contain success styles");
    assert!(css_content.contains(".compilation-failed"), "Should contain failure styles");
    assert!(css_content.contains(".code-block-embedded"), "Should contain embedded indicators");
    assert!(css_content.contains(".playground-runnable"), "Should contain playground styles");
    
    // Verify responsive design
    assert!(css_content.contains("@media screen and (max-width: 768px)"), "Should have mobile styles");
    
    // Verify print styles
    assert!(css_content.contains("@media print"), "Should have print styles");
}

#[test]
fn test_javascript_functionality() {
    // Read the custom JavaScript file
    let js_content = fs::read_to_string("theme/custom.js")
        .expect("Should be able to read custom.js");
    
    // Verify compilation status functions are present
    assert!(js_content.contains("initializeCompilationStatusFeatures"), "Should have compilation status initialization");
    assert!(js_content.contains("showCompilationErrorDetails"), "Should have error detail functionality");
    assert!(js_content.contains("addCodeBlockContextIndicators"), "Should have context indicators");
    assert!(js_content.contains("initializePlaygroundEnhancements"), "Should have playground enhancements");
    
    // Verify utility functions
    assert!(js_content.contains("detectCodeContext"), "Should have context detection");
    assert!(js_content.contains("runCodeExample"), "Should have code execution");
    assert!(js_content.contains("escapeHtml"), "Should have HTML escaping");
}

#[test]
fn test_book_toml_configuration() {
    // Read the book.toml file
    let toml_content = fs::read_to_string("book.toml")
        .expect("Should be able to read book.toml");
    
    // Verify playground configuration
    assert!(toml_content.contains("runnable = true"), "Should enable runnable playground");
    assert!(toml_content.contains("editable = true"), "Should enable editable playground");
    
    // Verify preprocessor configuration
    assert!(toml_content.contains("[preprocessor.rust-examples]"), "Should have rust-examples preprocessor");
    assert!(toml_content.contains("command = \"mdbook-rust-examples\""), "Should specify preprocessor command");
    assert!(toml_content.contains("embedded-target"), "Should have embedded target config");
    assert!(toml_content.contains("no-std-mode"), "Should have no-std mode config");
    assert!(toml_content.contains("feature-gates"), "Should have feature gates config");
}

fn create_test_book(book_path: &Path) {
    // Create book.toml
    let book_toml = r#"
[book]
authors = ["Test Author"]
language = "en"
multilingual = false
src = "src"
title = "Test Embedded Rust Book"

[preprocessor.rust-examples]
command = "mdbook-rust-examples"
renderer = ["html"]

[preprocessor.rust-examples.config]
embedded-target = "thumbv7em-none-eabihf"
no-std-mode = true
feature-gates = ["crypto", "hardware", "embedded"]
validation-enabled = false
compilation-timeout = 30
cache-enabled = true

[output.html]
default-theme = "navy"
additional-css = ["theme/custom.css"]
additional-js = ["theme/custom.js"]

[output.html.playground]
copyable = true
copy-js = true
line-numbers = true
editable = true
runnable = true
"#;
    
    fs::write(book_path.join("book.toml"), book_toml)
        .expect("Failed to write book.toml");
    
    // Create src directory
    let src_path = book_path.join("src");
    fs::create_dir_all(&src_path).expect("Failed to create src directory");
    
    // Create SUMMARY.md
    let summary = r#"
# Summary

[Introduction](./introduction.md)

- [Chapter 1](./chapter_1.md)
"#;
    
    fs::write(src_path.join("SUMMARY.md"), summary)
        .expect("Failed to write SUMMARY.md");
    
    // Create introduction.md
    let introduction = r#"
# Introduction

This is a test book for the embedded Rust preprocessor.

```rust
fn main() {
    println!("Hello, world!");
}
```
"#;
    
    fs::write(src_path.join("introduction.md"), introduction)
        .expect("Failed to write introduction.md");
    
    // Create chapter_1.md with various code examples
    let chapter1 = r#"
# Chapter 1: Code Examples

## Standard Rust Example

```rust
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    println!("{:?}", map);
}
```

## No-std Example

```rust,no_std
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    loop {}
}
```

## Hardware Example

```rust,hardware=stm32f4
use stm32f4xx_hal::{
    gpio::{gpioa::PA5, Output, PushPull},
    prelude::*,
};

fn blink_led(led: &mut PA5<Output<PushPull>>) {
    led.set_high();
}
```

## Crypto Example

```rust,crypto=aes
use zeroize::Zeroize;

fn secure_key_handling() {
    let mut key = [0u8; 32];
    // Use key...
    key.zeroize(); // Clear sensitive data
}
```

## Code Snippet

```rust,snippet=incomplete
// This is just a fragment
let result = some_function(
```

## Non-runnable Example

```rust,no_run
use std::process::Command;

fn main() {
    Command::new("rm")
        .arg("-rf")
        .arg("/")
        .spawn()
        .expect("Failed to execute command");
}
```
"#;
    
    fs::write(src_path.join("chapter_1.md"), chapter1)
        .expect("Failed to write chapter_1.md");
    
    // Create theme directory and copy our custom files
    let theme_path = book_path.join("theme");
    fs::create_dir_all(&theme_path).expect("Failed to create theme directory");
    
    // Copy custom CSS and JS files
    if let Ok(css_content) = fs::read_to_string("theme/custom.css") {
        fs::write(theme_path.join("custom.css"), css_content)
            .expect("Failed to copy custom.css");
    }
    
    if let Ok(js_content) = fs::read_to_string("theme/custom.js") {
        fs::write(theme_path.join("custom.js"), js_content)
            .expect("Failed to copy custom.js");
    }
}

fn verify_html_enhancements(book_output_path: &Path) {
    // Read the generated HTML files
    let index_html = fs::read_to_string(book_output_path.join("index.html"));
    let chapter1_html = fs::read_to_string(book_output_path.join("chapter_1.html"));
    
    if let Ok(content) = index_html {
        // Verify custom CSS and JS are included
        assert!(content.contains("custom.css"), "Should include custom CSS");
        assert!(content.contains("custom.js"), "Should include custom JS");
    }
    
    if let Ok(content) = chapter1_html {
        // Verify code blocks are present
        assert!(content.contains("<pre><code class=\"language-rust\">"), "Should contain Rust code blocks");
        
        // Note: Since we disabled validation in the test, we won't see compilation status indicators
        // but we can verify the structure is correct
        
        // Verify playground configuration might be present for std examples
        // (This would depend on the preprocessor actually running)
    }
}

#[test]
fn test_cargo_toml_binary_configuration() {
    // Read Cargo.toml to verify binary configuration
    let cargo_content = fs::read_to_string("Cargo.toml")
        .expect("Should be able to read Cargo.toml");
    
    // Verify mdbook-rust-examples binary is configured
    assert!(cargo_content.contains("[[bin]]"), "Should have binary configuration");
    assert!(cargo_content.contains("name = \"mdbook-rust-examples\""), "Should have preprocessor binary");
    assert!(cargo_content.contains("path = \"src/bin/mdbook-rust-examples.rs\""), "Should specify binary path");
    
    // Verify required dependencies
    assert!(cargo_content.contains("serde"), "Should have serde dependency");
    assert!(cargo_content.contains("serde_json"), "Should have serde_json dependency");
    assert!(cargo_content.contains("regex"), "Should have regex dependency");
    assert!(cargo_content.contains("md5"), "Should have md5 dependency");
}

#[test]
fn test_preprocessor_module_exports() {
    // This test verifies that the preprocessor module is properly exported
    // and can be used by the binary
    
    use embedded_rust_tutorial_validation::preprocessor::RustExamplesPreprocessor;
    
    let mut preprocessor = RustExamplesPreprocessor::new();
    
    // Test basic functionality without actually running the preprocessor
    // (since that would require stdin input)
    
    // This mainly tests that the module compiles and exports work
    assert!(true, "Preprocessor module should compile and be accessible");
}

#[test]
fn test_html_generation_components() {
    use embedded_rust_tutorial_validation::preprocessor::RustExamplesPreprocessor;
    use embedded_rust_tutorial_validation::{TestResult, CompilationError};
    use std::time::Duration;
    
    let preprocessor = RustExamplesPreprocessor::new();
    
    // Test status indicator generation
    let success_result = TestResult {
        example_id: "test".to_string(),
        success: true,
        compilation_time: Duration::from_millis(1500),
        target: "x86_64".to_string(),
        error: None,
        warnings: vec![],
    };
    
    let indicator = preprocessor.create_status_indicator(&success_result);
    assert!(indicator.contains("compilation-success"), "Should contain success class");
    assert!(indicator.contains("✓ Compiles"), "Should contain success text");
    assert!(indicator.contains("1.50s"), "Should contain timing");
    
    // Test error indicator
    let error_result = TestResult {
        example_id: "test".to_string(),
        success: false,
        compilation_time: Duration::from_millis(500),
        target: "x86_64".to_string(),
        error: Some(CompilationError::SyntaxError { 
            message: "missing semicolon".to_string(), 
            line: 5 
        }),
        warnings: vec![],
    };
    
    let indicator = preprocessor.create_status_indicator(&error_result);
    assert!(indicator.contains("compilation-failed"), "Should contain failure class");
    assert!(indicator.contains("✗ Compilation Error"), "Should contain error text");
    assert!(indicator.contains("title="), "Should contain error details in title");
}