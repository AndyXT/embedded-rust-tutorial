//! CLI interface for the Rust example testing framework
//! 
//! This binary provides a command-line interface to test Rust code examples
//! from markdown files, integrating with the existing validation infrastructure.

use std::env;
use std::path::PathBuf;
use std::process;
use serde_json;

use embedded_rust_tutorial_validation::{
    CodeExtractor, RustExampleTester, TestConfig, ValidationReport,
    ExampleContext, CodeExample
};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <markdown_file>", args[0]);
        process::exit(1);
    }
    
    let markdown_path = PathBuf::from(&args[1]);
    
    if !markdown_path.exists() {
        eprintln!("Error: File {} does not exist", markdown_path.display());
        process::exit(1);
    }
    
    // Extract code examples from markdown
    let extractor = CodeExtractor::new();
    let examples = match extractor.extract_from_file(&markdown_path) {
        Ok(examples) => examples,
        Err(e) => {
            eprintln!("Error extracting examples: {:?}", e);
            process::exit(1);
        }
    };
    
    // Create test configuration
    let config = TestConfig::default();
    
    // Create tester
    let mut tester = match RustExampleTester::new(config) {
        Ok(tester) => tester,
        Err(e) => {
            eprintln!("Error creating tester: {:?}", e);
            process::exit(1);
        }
    };
    
    // Ensure required targets are available
    if let Err(e) = tester.ensure_targets() {
        eprintln!("Warning: Some targets may not be available: {:?}", e);
    }
    
    // Run validation
    let report = tester.validate_examples(&examples);
    
    // Output results as JSON for Python integration
    let json_output = serde_json::json!({
        "total_examples": report.total_examples,
        "successful": report.successful,
        "failed": report.failed,
        "skipped": report.skipped,
        "success_rate": report.success_rate(),
        "results": report.results.iter().map(|r| {
            serde_json::json!({
                "example_id": r.example_id,
                "success": r.success,
                "target": r.target,
                "compilation_time_ms": r.compilation_time.as_millis(),
                "error": r.error.as_ref().map(|e| format!("{:?}", e)),
                "warnings_count": r.warnings.len()
            })
        }).collect::<Vec<_>>()
    });
    
    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
}