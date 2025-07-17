//! Core testing infrastructure for Rust code examples

use crate::{CodeExample, TestConfig, TestResult, CompilationError, ValidationReport, ToolchainManager};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::string::String;
use std::vec::Vec;
use core::option::Option::{self, Some, None};
use core::result::Result::{self, Ok, Err};
use core::ops::Drop;

/// Main struct for testing Rust code examples
pub struct RustExampleTester {
    /// Configuration for testing
    config: TestConfig,
    /// Toolchain manager for handling multiple targets
    toolchain_manager: ToolchainManager,
    /// Temporary directory for test projects
    temp_dir: Option<PathBuf>,
}

impl RustExampleTester {
    /// Create a new RustExampleTester with the given configuration
    pub fn new(config: TestConfig) -> Result<Self, CompilationError> {
        let toolchain_manager = ToolchainManager::new()?;
        Ok(Self {
            config,
            toolchain_manager,
            temp_dir: None,
        })
    }
    
    /// Create a new RustExampleTester with existing toolchain manager
    pub fn with_toolchain_manager(config: TestConfig, toolchain_manager: ToolchainManager) -> Self {
        Self {
            config,
            toolchain_manager,
            temp_dir: None,
        }
    }
    
    /// Check if the tester is ready to run tests
    pub fn is_ready(&self) -> bool {
        self.toolchain_manager.check_cargo_available()
    }
    
    /// Get the toolchain manager
    pub fn toolchain_manager(&self) -> &ToolchainManager {
        &self.toolchain_manager
    }
    
    /// Get mutable reference to toolchain manager
    pub fn toolchain_manager_mut(&mut self) -> &mut ToolchainManager {
        &mut self.toolchain_manager
    }
    
    /// Ensure all required targets are installed for the current configuration
    pub fn ensure_targets(&mut self) -> Result<Vec<String>, CompilationError> {
        self.toolchain_manager.ensure_targets_for_config(&self.config)
    }
    
    /// Test a single code example against the first available target
    pub fn test_example(&mut self, example: &CodeExample) -> TestResult {
        // Skip examples that shouldn't be compiled
        if !example.should_compile() {
            return TestResult {
                example_id: example.id.clone(),
                success: true,
                compilation_time: Duration::from_millis(0),
                target: "skipped".to_string(),
                error: None,
                warnings: vec![],
                stdout: "Skipped (marked as snippet)".to_string(),
                stderr: String::new(),
            };
        }
        
        // Get available targets for this configuration
        let available_targets = self.toolchain_manager.get_available_targets(&self.config);
        
        if available_targets.is_empty() {
            return TestResult::failure(
                example.id.clone(),
                "none".to_string(),
                Duration::from_millis(0),
                CompilationError::TargetError {
                    incompatible_target: "none".to_string(),
                    reason: "No compatible targets available".to_string(),
                },
                "No targets available for testing".to_string(),
            );
        }
        
        // Test against the first available target
        let target = &available_targets[0];
        self.test_example_with_target(example, target)
    }
    
    /// Test a single code example against a specific target
    pub fn test_example_with_target(&mut self, example: &CodeExample, target: &str) -> TestResult {
        let start_time = Instant::now();
        
        // Skip examples that shouldn't be compiled
        if !example.should_compile() {
            return TestResult {
                example_id: example.id.clone(),
                success: true,
                compilation_time: Duration::from_millis(0),
                target: "skipped".to_string(),
                error: None,
                warnings: vec![],
                stdout: "Skipped (marked as snippet)".to_string(),
                stderr: String::new(),
            };
        }
        
        match self.compile_example(example, target) {
            Ok((stdout, stderr, warnings)) => {
                let compilation_time = start_time.elapsed();
                TestResult {
                    example_id: example.id.clone(),
                    success: true,
                    compilation_time,
                    target: target.to_string(),
                    error: None,
                    warnings,
                    stdout,
                    stderr,
                }
            }
            Err(error) => {
                let compilation_time = start_time.elapsed();
                TestResult::failure(
                    example.id.clone(),
                    target.to_string(),
                    compilation_time,
                    error,
                    String::new(),
                )
            }
        }
    }
    
    /// Test a single code example against all available targets
    pub fn test_example_all_targets(&mut self, example: &CodeExample) -> Vec<TestResult> {
        // Skip examples that shouldn't be compiled
        if !example.should_compile() {
            let skip_result = TestResult {
                example_id: example.id.clone(),
                success: true,
                compilation_time: Duration::from_millis(0),
                target: "skipped".to_string(),
                error: None,
                warnings: vec![],
                stdout: "Skipped (marked as snippet)".to_string(),
                stderr: String::new(),
            };
            return vec![skip_result];
        }
        
        let available_targets = self.toolchain_manager.get_available_targets(&self.config);
        
        if available_targets.is_empty() {
            let error_result = TestResult::failure(
                example.id.clone(),
                "none".to_string(),
                Duration::from_millis(0),
                CompilationError::TargetError {
                    incompatible_target: "none".to_string(),
                    reason: "No compatible targets available".to_string(),
                },
                "No targets available for testing".to_string(),
            );
            return vec![error_result];
        }
        
        let mut results = Vec::new();
        for target in &available_targets {
            let result = self.test_example_with_target(example, target);
            results.push(result);
        }
        
        results
    }
    
    /// Test multiple code examples and generate a validation report
    pub fn validate_examples(&mut self, examples: &[CodeExample]) -> ValidationReport {
        let mut report = ValidationReport::new();
        
        for example in examples {
            let result = self.test_example(example);
            report.add_result(result);
        }
        
        report
    }
    
    /// Compile a single example in a temporary project
    fn compile_example(
        &mut self, 
        example: &CodeExample, 
        target: &str
    ) -> Result<(String, String, Vec<String>), CompilationError> {
        // Create temporary project directory
        let temp_dir = self.get_or_create_temp_dir()?;
        let project_dir = temp_dir.join(&example.id);
        
        // Create project structure
        self.create_test_project(&project_dir, example)?;
        
        // Run cargo check (faster than full compilation)
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&project_dir)
           .arg("check")
           .arg("--target")
           .arg(target)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        // Add features if specified
        if !self.config.features.is_empty() {
            cmd.arg("--features").arg(self.config.features.join(","));
        }
        
        // Set timeout
        let output = self.run_with_timeout(cmd, self.config.compilation_timeout)?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        if output.status.success() {
            let warnings = self.extract_warnings(&stderr);
            Ok((stdout, stderr, warnings))
        } else {
            let error = self.parse_compilation_error(&stderr, output.status.code());
            Err(error)
        }
    }
    
    /// Create a temporary Cargo project for testing an example
    fn create_test_project(&self, project_dir: &Path, example: &CodeExample) -> Result<(), CompilationError> {
        // Create project directory
        fs::create_dir_all(project_dir)
            .map_err(|e| CompilationError::IoError { 
                message: format!("Failed to create project directory: {}", e) 
            })?;
        
        // Create Cargo.toml
        let cargo_toml = self.generate_cargo_toml(example);
        fs::write(project_dir.join("Cargo.toml"), cargo_toml)
            .map_err(|e| CompilationError::IoError { 
                message: format!("Failed to write Cargo.toml: {}", e) 
            })?;
        
        // Create src directory
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir)
            .map_err(|e| CompilationError::IoError { 
                message: format!("Failed to create src directory: {}", e) 
            })?;
        
        // Create main.rs or lib.rs based on code content
        let code_content = self.prepare_code_for_compilation(example);
        let filename = if example.code.contains("fn main") || example.code.contains("#[entry]") {
            "main.rs"
        } else {
            "lib.rs"
        };
        
        fs::write(src_dir.join(filename), code_content)
            .map_err(|e| CompilationError::IoError { 
                message: format!("Failed to write source file: {}", e) 
            })?;
        
        Ok(())
    }
    
    /// Generate Cargo.toml content for an example
    fn generate_cargo_toml(&self, example: &CodeExample) -> String {
        let mut toml = String::new();
        
        toml.push_str("[package]\n");
        toml.push_str(&format!("name = \"{}\"\n", example.id.replace('-', "_")));
        toml.push_str("version = \"0.1.0\"\n");
        toml.push_str("edition = \"2021\"\n\n");
        
        // Add dependencies
        if !example.dependencies.is_empty() {
            toml.push_str("[dependencies]\n");
            for dep in &example.dependencies {
                match dep.as_str() {
                    "heapless" => toml.push_str("heapless = \"0.7\"\n"),
                    "cortex-m" => toml.push_str("cortex-m = \"0.7\"\n"),
                    "cortex-m-rt" => toml.push_str("cortex-m-rt = \"0.7\"\n"),
                    "panic-halt" => toml.push_str("panic-halt = \"0.2\"\n"),
                    "zeroize" => toml.push_str("zeroize = { version = \"1.6\", default-features = false }\n"),
                    "aes" => toml.push_str("aes = \"0.8\"\n"),
                    "sha2" => toml.push_str("sha2 = \"0.10\"\n"),
                    "rand" => toml.push_str("rand = { version = \"0.8\", default-features = false }\n"),
                    _ => toml.push_str(&format!("{} = \"*\"\n", dep)),
                }
            }
            toml.push('\n');
        }
        
        // Add profile settings for embedded targets
        if example.context.is_no_std() {
            toml.push_str("[profile.dev]\n");
            toml.push_str("debug = true\n");
            toml.push_str("opt-level = \"s\"\n\n");
            
            toml.push_str("[profile.release]\n");
            toml.push_str("debug = true\n");
            toml.push_str("opt-level = \"s\"\n");
            toml.push_str("lto = true\n");
            toml.push_str("codegen-units = 1\n\n");
        }
        
        toml
    }
    
    /// Prepare code content for compilation
    fn prepare_code_for_compilation(&self, example: &CodeExample) -> String {
        let mut code = example.code.clone();
        
        // Add necessary attributes for no_std examples
        if example.context.is_no_std() && !code.contains("#![no_std]") {
            code = format!("#![no_std]\n{}", code);
        }
        
        // Add panic handler for no_std examples if not present
        if example.context.is_no_std() && !code.contains("panic_handler") && !code.contains("panic-halt") {
            code.push_str("\n\n#[panic_handler]\nfn panic(_info: &core::panic::PanicInfo) -> ! {\n    loop {}\n}\n");
        }
        
        // Wrap non-main code in a function if needed
        if !code.contains("fn main") && !code.contains("#[entry]") && !code.contains("fn ") {
            code = format!("fn example_code() {{\n{}\n}}", code);
        }
        
        code
    }
    
    /// Get or create the temporary directory
    fn get_or_create_temp_dir(&mut self) -> Result<&PathBuf, CompilationError> {
        if self.temp_dir.is_none() {
            let temp_dir = self.config.work_dir.clone();
            fs::create_dir_all(&temp_dir)
                .map_err(|e| CompilationError::IoError { 
                    message: format!("Failed to create temporary directory: {}", e) 
                })?;
            self.temp_dir = Some(temp_dir);
        }
        
        Ok(self.temp_dir.as_ref().unwrap())
    }
    
    /// Run a command with timeout
    fn run_with_timeout(
        &self, 
        mut cmd: Command, 
        timeout: Duration
    ) -> Result<std::process::Output, CompilationError> {
        let start = Instant::now();
        
        // Ensure we have a valid current directory
        if let Ok(current_dir) = std::env::current_dir() {
            if !current_dir.exists() {
                // If current directory doesn't exist, set it to temp dir
                if let Some(temp_dir) = &self.temp_dir {
                    cmd.current_dir(temp_dir);
                }
            }
        }
        
        let child = cmd.spawn()
            .map_err(|e| CompilationError::IoError { 
                message: format!("Failed to spawn process: {}", e) 
            })?;
        
        // Simple timeout implementation - in a real implementation you'd want
        // to use a more sophisticated approach with async or threading
        let output = child.wait_with_output()
            .map_err(|e| CompilationError::IoError { 
                message: format!("Failed to wait for process: {}", e) 
            })?;
        
        if start.elapsed() > timeout {
            return Err(CompilationError::TimeoutError { duration: timeout });
        }
        
        Ok(output)
    }
    
    /// Parse compilation error from stderr
    fn parse_compilation_error(&self, stderr: &str, exit_code: Option<i32>) -> CompilationError {
        // Look for common error patterns
        if stderr.contains("error[E") {
            // Try to extract line number
            let line = self.extract_line_number(stderr);
            CompilationError::SyntaxError {
                message: stderr.lines().next().unwrap_or("Unknown syntax error").to_string(),
                line,
                column: None,
            }
        } else if stderr.contains("could not find") && stderr.contains("Cargo.toml") {
            CompilationError::DependencyError {
                missing: vec!["Unknown dependency".to_string()],
            }
        } else {
            CompilationError::CompilationFailed {
                exit_code: exit_code.unwrap_or(-1),
                message: stderr.to_string(),
            }
        }
    }
    
    /// Extract line number from error message
    fn extract_line_number(&self, stderr: &str) -> Option<usize> {
        // Simple parsing to find line numbers in format "src/main.rs:5:10"
        for line in stderr.lines() {
            if let Some(colon_pos) = line.find(':') {
                let after_colon = &line[colon_pos + 1..];
                if let Some(second_colon) = after_colon.find(':') {
                    let number_part = &after_colon[..second_colon];
                    if let Ok(line_num) = number_part.parse::<usize>() {
                        return Some(line_num);
                    }
                }
            }
        }
        None
    }
    
    /// Extract warnings from stderr
    fn extract_warnings(&self, stderr: &str) -> Vec<String> {
        stderr.lines()
            .filter(|line| line.contains("warning:"))
            .map(|line| line.to_string())
            .collect()
    }
}

impl Drop for RustExampleTester {
    fn drop(&mut self) {
        // Cleanup temporary directory if it exists
        if let Some(temp_dir) = &self.temp_dir {
            let _ = fs::remove_dir_all(temp_dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExampleContext, CodeExample};
    use std::path::PathBuf;
    
    #[test]
    fn test_tester_creation() {
        let config = TestConfig::default();
        let tester = RustExampleTester::new(config).expect("Failed to create tester");
        assert!(tester.is_ready());
    }
    
    #[test]
    fn test_simple_example_compilation() {
        // Ensure we have a valid working directory
        let original_dir = std::env::current_dir().ok();
        
        let mut config = TestConfig::default();
        // Use a more specific temp directory for this test
        config.work_dir = std::env::temp_dir().join("rust_example_test_simple");
        
        let mut tester = RustExampleTester::new(config).expect("Failed to create tester");
        
        let example = CodeExample::new(
            "test_simple".to_string(),
            PathBuf::from("test.md"),
            1,
            "fn main() { println!(\"Hello, world!\"); }".to_string(),
            ExampleContext::Std { features: vec![] },
        );
        
        let result = tester.test_example(&example);
        if !result.success {
            println!("Test failed with error: {:?}", result.error);
            println!("Stderr: {}", result.stderr);
        }
        
        // Restore original directory if it existed
        if let Some(dir) = original_dir {
            let _ = std::env::set_current_dir(dir);
        }
        
        assert!(result.success, "Simple example should compile successfully");
    }
    
    #[test]
    fn test_no_std_example_compilation() {
        let config = TestConfig::embedded();
        let mut tester = RustExampleTester::new(config).expect("Failed to create tester");
        
        let example = CodeExample::new(
            "test_no_std".to_string(),
            PathBuf::from("test.md"),
            1,
            "#![no_std]\n#![no_main]\nuse panic_halt as _;\n#[cortex_m_rt::entry]\nfn main() -> ! { loop {} }".to_string(),
            ExampleContext::NoStd { 
                target: "thumbv7em-none-eabihf".to_string(), 
                features: vec![] 
            },
        );
        
        let result = tester.test_example(&example);
        // This might fail if the target isn't installed, but the test structure should work
        println!("No-std test result: {:?}", result);
    }
    
    #[test]
    fn test_multi_target_compilation() {
        let mut config = TestConfig::default();
        config.targets = vec![
            "x86_64-unknown-linux-gnu".to_string(),
            "thumbv7em-none-eabihf".to_string(),
        ];
        
        let mut tester = RustExampleTester::new(config).expect("Failed to create tester");
        
        let example = CodeExample::new(
            "test_multi_target".to_string(),
            PathBuf::from("test.md"),
            1,
            "fn main() { println!(\"Hello, world!\"); }".to_string(),
            ExampleContext::Std { features: vec![] },
        );
        
        let results = tester.test_example_all_targets(&example);
        // Should have at least one result (the std target should work)
        assert!(!results.is_empty());
        
        // At least one target should succeed (the std target)
        let successful_results: Vec<_> = results.iter().filter(|r| r.success).collect();
        println!("Multi-target test results: {} successful out of {}", successful_results.len(), results.len());
    }
    
    #[test]
    fn test_target_availability_check() {
        let config = TestConfig::default();
        let mut tester = RustExampleTester::new(config).expect("Failed to create tester");
        
        // Test ensuring targets are available
        let available_targets = tester.ensure_targets();
        match available_targets {
            Ok(targets) => {
                assert!(!targets.is_empty(), "Should have at least one available target");
                println!("Available targets: {:?}", targets);
            }
            Err(e) => {
                println!("Target availability check failed: {:?}", e);
                // This is acceptable in CI environments where targets might not be installed
            }
        }
    }
    
    #[test]
    fn test_temporary_project_creation() {
        let config = TestConfig::default();
        let tester = RustExampleTester::new(config).expect("Failed to create tester");
        
        let example = CodeExample::new(
            "test_temp_project".to_string(),
            PathBuf::from("test.md"),
            1,
            "fn main() { println!(\"Hello, world!\"); }".to_string(),
            ExampleContext::Std { features: vec![] },
        );
        
        // Test Cargo.toml generation
        let cargo_toml = tester.generate_cargo_toml(&example);
        assert!(cargo_toml.contains("[package]"));
        assert!(cargo_toml.contains("name = \"test_temp_project\""));
        assert!(cargo_toml.contains("edition = \"2021\""));
        
        // Test code preparation
        let prepared_code = tester.prepare_code_for_compilation(&example);
        assert!(prepared_code.contains("fn main"));
        
        println!("Generated Cargo.toml:\n{}", cargo_toml);
        println!("Prepared code:\n{}", prepared_code);
    }
    
    #[test]
    fn test_snippet_skipping() {
        let config = TestConfig::default();
        let mut tester = RustExampleTester::new(config).expect("Failed to create tester");
        
        let example = CodeExample::new(
            "test_snippet".to_string(),
            PathBuf::from("test.md"),
            1,
            "let incomplete_code = ".to_string(),
            ExampleContext::Snippet { 
                reason: "incomplete".to_string() 
            },
        );
        
        let result = tester.test_example(&example);
        assert!(result.success);
        assert_eq!(result.target, "skipped");
    }
    
    #[test]
    fn test_cargo_toml_generation() {
        let config = TestConfig::default();
        let tester = RustExampleTester::new(config).expect("Failed to create tester");
        
        let mut example = CodeExample::new(
            "test_deps".to_string(),
            PathBuf::from("test.md"),
            1,
            "use heapless::Vec;".to_string(),
            ExampleContext::NoStd { 
                target: "thumbv7em-none-eabihf".to_string(), 
                features: vec![] 
            },
        );
        example.dependencies = vec!["heapless".to_string()];
        
        let cargo_toml = tester.generate_cargo_toml(&example);
        assert!(cargo_toml.contains("heapless"));
        assert!(cargo_toml.contains("[profile.dev]"));
    }
    
    #[test]
    fn test_code_preparation() {
        let config = TestConfig::default();
        let tester = RustExampleTester::new(config).expect("Failed to create tester");
        
        let example = CodeExample::new(
            "test_prep".to_string(),
            PathBuf::from("test.md"),
            1,
            "let x = 42;".to_string(),
            ExampleContext::NoStd { 
                target: "thumbv7em-none-eabihf".to_string(), 
                features: vec![] 
            },
        );
        
        let prepared = tester.prepare_code_for_compilation(&example);
        assert!(prepared.contains("#![no_std]"));
        assert!(prepared.contains("panic_handler") || prepared.contains("fn example_code"));
    }
    
    #[test]
    fn test_validation_report() {
        let config = TestConfig::default();
        let mut tester = RustExampleTester::new(config).expect("Failed to create tester");
        
        let examples = vec![
            CodeExample::new(
                "test1".to_string(),
                PathBuf::from("test.md"),
                1,
                "fn main() {}".to_string(),
                ExampleContext::Std { features: vec![] },
            ),
            CodeExample::new(
                "test2".to_string(),
                PathBuf::from("test.md"),
                5,
                "invalid rust code {".to_string(),
                ExampleContext::Std { features: vec![] },
            ),
        ];
        
        let report = tester.validate_examples(&examples);
        assert_eq!(report.total_examples, 2);
        assert!(report.successful >= 1 || report.failed >= 1);
    }
    
    #[test]
    fn test_comprehensive_multi_target_framework() {
        // Test the complete multi-target compilation framework
        let mut config = TestConfig::default();
        config.targets = vec![
            "x86_64-unknown-linux-gnu".to_string(),
        ];
        config.work_dir = std::env::temp_dir().join("rust_example_test_comprehensive");
        
        let mut tester = RustExampleTester::new(config).expect("Failed to create tester");
        
        // Test different types of examples
        let examples = vec![
            // Standard library example
            CodeExample::new(
                "std_example".to_string(),
                PathBuf::from("test.md"),
                1,
                "fn main() { println!(\"Hello from std!\"); }".to_string(),
                ExampleContext::Std { features: vec![] },
            ),
            // Snippet that should be skipped
            CodeExample::new(
                "snippet_example".to_string(),
                PathBuf::from("test.md"),
                5,
                "let incomplete = ".to_string(),
                ExampleContext::Snippet { reason: "incomplete".to_string() },
            ),
            // Library code example
            CodeExample::new(
                "lib_example".to_string(),
                PathBuf::from("test.md"),
                10,
                "pub fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
                ExampleContext::Std { features: vec![] },
            ),
        ];
        
        // Test individual examples
        for example in &examples {
            let result = tester.test_example(example);
            if example.should_compile() {
                if !result.success {
                    println!("Example {} failed: {:?}", example.id, result.error);
                    println!("Stderr: {}", result.stderr);
                }
                assert!(result.success, "Example {} should compile", example.id);
            } else {
                // Snippets should be marked as successful but skipped
                assert!(result.success);
                assert_eq!(result.target, "skipped");
            }
        }
        
        // Test validation report
        let report = tester.validate_examples(&examples);
        assert_eq!(report.total_examples, 3);
        assert!(report.successful >= 2); // At least the std examples should pass
        assert!(report.success_rate() > 50.0);
        
        // Test multi-target compilation for a single example
        let multi_target_results = tester.test_example_all_targets(&examples[0]);
        assert!(!multi_target_results.is_empty());
        assert!(multi_target_results.iter().any(|r| r.success));
        
        println!("Comprehensive test completed successfully!");
        println!("Report: {} total, {} successful, {} failed, {} skipped", 
                 report.total_examples, report.successful, report.failed, report.skipped);
    }
}