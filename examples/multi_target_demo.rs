//! Demonstration of the multi-target compilation testing framework

use embedded_rust_tutorial_validation::{
    RustExampleTester, TestConfig, CodeExample, ExampleContext
};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Multi-Target Compilation Testing Framework Demo");
    println!("===============================================");
    
    // Create a test configuration
    let mut config = TestConfig::default();
    config.targets = vec![
        "x86_64-unknown-linux-gnu".to_string(),
    ];
    
    // Create the tester
    let mut tester = RustExampleTester::new(config)?;
    
    println!("✓ Created RustExampleTester");
    
    // Check if the system is ready
    if !tester.is_ready() {
        println!("✗ System not ready - cargo not available");
        return Ok(());
    }
    
    println!("✓ System ready - cargo available");
    
    // Create some example code to test
    let examples = vec![
        CodeExample::new(
            "hello_world".to_string(),
            PathBuf::from("demo.md"),
            1,
            r#"fn main() {
    println!("Hello, world!");
}"#.to_string(),
            ExampleContext::Std { features: vec![] },
        ),
        
        CodeExample::new(
            "math_function".to_string(),
            PathBuf::from("demo.md"),
            10,
            r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}"#.to_string(),
            ExampleContext::Std { features: vec![] },
        ),
        
        CodeExample::new(
            "snippet_example".to_string(),
            PathBuf::from("demo.md"),
            20,
            "let incomplete = ".to_string(),
            ExampleContext::Snippet { reason: "incomplete example".to_string() },
        ),
    ];
    
    println!("✓ Created {} test examples", examples.len());
    
    // Test each example individually
    println!("\nTesting individual examples:");
    for example in &examples {
        let result = tester.test_example(example);
        
        if example.should_compile() {
            if result.success {
                println!("  ✓ {} compiled successfully in {:?}", 
                         example.id, result.compilation_time);
            } else {
                println!("  ✗ {} failed to compile: {:?}", 
                         example.id, result.error);
            }
        } else {
            println!("  ⊝ {} skipped (marked as snippet)", example.id);
        }
    }
    
    // Generate a validation report
    println!("\nGenerating validation report:");
    let report = tester.validate_examples(&examples);
    
    println!("  Total examples: {}", report.total_examples);
    println!("  Successful: {}", report.successful);
    println!("  Failed: {}", report.failed);
    println!("  Skipped: {}", report.skipped);
    println!("  Success rate: {:.1}%", report.success_rate());
    println!("  Total time: {:?}", report.total_duration);
    
    // Test multi-target compilation for the first example
    println!("\nTesting multi-target compilation:");
    let multi_results = tester.test_example_all_targets(&examples[0]);
    
    for result in &multi_results {
        if result.success {
            println!("  ✓ Target {} succeeded in {:?}", 
                     result.target, result.compilation_time);
        } else {
            println!("  ✗ Target {} failed: {:?}", 
                     result.target, result.error);
        }
    }
    
    // Show toolchain information
    println!("\nToolchain information:");
    let system_info = tester.toolchain_manager().get_system_info()?;
    println!("  Cargo: {}", system_info.cargo_version);
    println!("  Rustc: {}", system_info.rustc_version);
    if let Some(rustup) = &system_info.rustup_version {
        println!("  Rustup: {}", rustup);
    }
    println!("  Installed targets: {}", system_info.installed_targets);
    
    println!("\n✓ Demo completed successfully!");
    
    Ok(())
}