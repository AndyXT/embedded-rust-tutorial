//! Configuration and result types for code example testing

#[cfg(feature = "std")]
use std::time::Duration;
#[cfg(feature = "std")]
use std::path::PathBuf;

#[cfg(not(feature = "std"))]
use core::time::Duration;
#[cfg(not(feature = "std"))]
use heapless::String as PathBuf; // Placeholder for no_std mode

// Import standard types conditionally
#[cfg(feature = "std")]
use std::string::String;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use core::option::Option::{self, Some, None};
use core::result::Result;
use core::default::Default;

/// Configuration for testing code examples
#[derive(Debug, Clone, PartialEq)]
pub struct TestConfig {
    /// Rust targets to test against (e.g., "x86_64-unknown-linux-gnu", "thumbv7em-none-eabihf")
    pub targets: Vec<String>,
    /// Features to enable during compilation
    pub features: Vec<String>,
    /// Whether to test in no_std mode
    pub no_std: bool,
    /// Whether to enable embedded-specific testing
    pub embedded_mode: bool,
    /// Timeout for compilation attempts
    pub compilation_timeout: Duration,
    /// Working directory for temporary test projects
    pub work_dir: PathBuf,
}

#[cfg(feature = "std")]
impl Default for TestConfig {
    fn default() -> Self {
        Self {
            targets: vec!["x86_64-unknown-linux-gnu".to_string()],
            features: vec![],
            no_std: false,
            embedded_mode: false,
            compilation_timeout: Duration::from_secs(30),
            work_dir: std::env::temp_dir().join("rust_example_tests"),
        }
    }
}

impl TestConfig {
    /// Create a new configuration for embedded testing
    pub fn embedded() -> Self {
        Self {
            targets: vec![
                "thumbv7em-none-eabihf".to_string(),
                "thumbv6m-none-eabi".to_string(),
            ],
            features: vec!["embedded".to_string()],
            no_std: true,
            embedded_mode: true,
            ..Default::default()
        }
    }
    
    /// Create a new configuration for cryptography testing
    pub fn crypto() -> Self {
        Self {
            features: vec!["crypto".to_string(), "zeroize".to_string()],
            ..Default::default()
        }
    }
}

/// Result of testing a single code example
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Unique identifier for the example
    pub example_id: String,
    /// Whether compilation succeeded
    pub success: bool,
    /// Time taken to compile
    pub compilation_time: Duration,
    /// Target that was tested
    pub target: String,
    /// Error details if compilation failed
    pub error: Option<CompilationError>,
    /// Compilation warnings
    pub warnings: Vec<String>,
    /// Standard output from compilation
    pub stdout: String,
    /// Standard error from compilation
    pub stderr: String,
}

impl TestResult {
    /// Create a successful test result
    pub fn success(example_id: String, target: String, compilation_time: Duration) -> Self {
        Self {
            example_id,
            success: true,
            compilation_time,
            target,
            error: None,
            warnings: vec![],
            stdout: String::new(),
            stderr: String::new(),
        }
    }
    
    /// Create a failed test result
    pub fn failure(
        example_id: String, 
        target: String, 
        compilation_time: Duration,
        error: CompilationError,
        stderr: String,
    ) -> Self {
        Self {
            example_id,
            success: false,
            compilation_time,
            target,
            error: Some(error),
            warnings: vec![],
            stdout: String::new(),
            stderr,
        }
    }
}

/// Detailed compilation error information
#[derive(Debug, Clone)]
pub enum CompilationError {
    /// Syntax error in the code
    SyntaxError { 
        message: String, 
        line: Option<usize>,
        column: Option<usize>,
    },
    /// Missing dependencies
    DependencyError { 
        missing: Vec<String> 
    },
    /// Unsupported features
    FeatureError { 
        unsupported: Vec<String> 
    },
    /// Incompatible target
    TargetError { 
        incompatible_target: String,
        reason: String,
    },
    /// Compilation timeout
    TimeoutError { 
        duration: Duration 
    },
    /// Generic compilation failure
    CompilationFailed {
        exit_code: i32,
        message: String,
    },
    /// IO error during testing
    IoError {
        message: String,
    },
}

impl std::fmt::Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationError::SyntaxError { message, line, column } => {
                write!(f, "Syntax error")?;
                if let Some(line) = line {
                    write!(f, " at line {}", line)?;
                    if let Some(column) = column {
                        write!(f, ", column {}", column)?;
                    }
                }
                write!(f, ": {}", message)
            }
            CompilationError::DependencyError { missing } => {
                write!(f, "Missing dependencies: {}", missing.join(", "))
            }
            CompilationError::FeatureError { unsupported } => {
                write!(f, "Unsupported features: {}", unsupported.join(", "))
            }
            CompilationError::TargetError { incompatible_target, reason } => {
                write!(f, "Target '{}' incompatible: {}", incompatible_target, reason)
            }
            CompilationError::TimeoutError { duration } => {
                write!(f, "Compilation timeout after {:?}", duration)
            }
            CompilationError::CompilationFailed { exit_code, message } => {
                write!(f, "Compilation failed (exit code {}): {}", exit_code, message)
            }
            CompilationError::IoError { message } => {
                write!(f, "IO error: {}", message)
            }
        }
    }
}

impl std::error::Error for CompilationError {}

/// Statistics about error recovery attempts
#[derive(Debug, Clone, Default)]
pub struct RecoveryStatistics {
    /// Number of successful recovery attempts
    pub successful_recoveries: usize,
    /// Number of failed recovery attempts
    pub failed_recoveries: usize,
    /// Number of examples marked as snippets during recovery
    pub marked_as_snippets: usize,
    /// Number of fallback target attempts
    pub fallback_attempts: usize,
    /// Number of feature disabling attempts
    pub feature_disabling_attempts: usize,
    /// Number of timeout extensions
    pub timeout_extensions: usize,
}

/// Summary report for multiple test results
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Total number of examples tested
    pub total_examples: usize,
    /// Number of successful compilations
    pub successful: usize,
    /// Number of failed compilations
    pub failed: usize,
    /// Number of skipped examples
    pub skipped: usize,
    /// Number of examples marked as snippets
    pub snippets: usize,
    /// Number of examples that required recovery attempts
    pub recovered: usize,
    /// Individual test results
    pub results: Vec<TestResult>,
    /// Overall testing duration
    pub total_duration: Duration,
    /// Recovery statistics
    pub recovery_stats: RecoveryStatistics,
}

impl ValidationReport {
    /// Create a new empty validation report
    pub fn new() -> Self {
        Self {
            total_examples: 0,
            successful: 0,
            failed: 0,
            skipped: 0,
            snippets: 0,
            recovered: 0,
            results: vec![],
            total_duration: Duration::from_secs(0),
            recovery_stats: RecoveryStatistics::default(),
        }
    }
    
    /// Add a test result to the report
    pub fn add_result(&mut self, result: TestResult) {
        self.total_examples += 1;
        self.total_duration += result.compilation_time;
        
        if result.success {
            self.successful += 1;
        } else {
            self.failed += 1;
        }
        
        self.results.push(result);
    }
    
    /// Mark an example as skipped
    pub fn skip_example(&mut self, example_id: String, reason: String) {
        self.total_examples += 1;
        self.skipped += 1;
        
        let result = TestResult {
            example_id,
            success: false,
            compilation_time: Duration::from_secs(0),
            target: "skipped".to_string(),
            error: Some(CompilationError::CompilationFailed {
                exit_code: 0,
                message: format!("Skipped: {}", reason),
            }),
            warnings: vec![],
            stdout: String::new(),
            stderr: String::new(),
        };
        
        self.results.push(result);
    }
    
    /// Mark an example as a snippet (not compiled)
    pub fn mark_as_snippet(&mut self, example_id: String, reason: String) {
        self.total_examples += 1;
        self.snippets += 1;
        
        let result = TestResult {
            example_id,
            success: true, // Snippets are considered "successful" as they're intentionally not compiled
            compilation_time: Duration::from_secs(0),
            target: "snippet".to_string(),
            error: None,
            warnings: vec![],
            stdout: format!("Marked as snippet: {}", reason),
            stderr: String::new(),
        };
        
        self.results.push(result);
    }
    
    /// Record a successful recovery attempt
    pub fn record_recovery(&mut self, _example_id: String, recovery_type: &str) {
        self.recovered += 1;
        match recovery_type {
            "snippet" => self.recovery_stats.marked_as_snippets += 1,
            "fallback" => self.recovery_stats.fallback_attempts += 1,
            "feature_disable" => self.recovery_stats.feature_disabling_attempts += 1,
            "timeout_extend" => self.recovery_stats.timeout_extensions += 1,
            _ => {}
        }
    }
    
    /// Record a failed recovery attempt
    pub fn record_failed_recovery(&mut self) {
        self.recovery_stats.failed_recoveries += 1;
    }
    
    /// Record a successful recovery attempt
    pub fn record_successful_recovery(&mut self) {
        self.recovery_stats.successful_recoveries += 1;
    }
    
    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_examples == 0 {
            0.0
        } else {
            (self.successful as f64 / self.total_examples as f64) * 100.0
        }
    }
    
    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0 && self.successful > 0
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_default() {
        let config = TestConfig::default();
        assert!(!config.no_std);
        assert!(!config.embedded_mode);
        assert_eq!(config.targets.len(), 1);
    }
    
    #[test]
    fn test_config_embedded() {
        let config = TestConfig::embedded();
        assert!(config.no_std);
        assert!(config.embedded_mode);
        assert!(config.targets.len() > 1);
    }
    
    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        assert_eq!(report.success_rate(), 0.0);
        
        let success_result = TestResult::success(
            "test1".to_string(),
            "x86_64-unknown-linux-gnu".to_string(),
            Duration::from_millis(100),
        );
        
        report.add_result(success_result);
        assert_eq!(report.success_rate(), 100.0);
        assert!(report.all_passed());
        
        let failure_result = TestResult::failure(
            "test2".to_string(),
            "x86_64-unknown-linux-gnu".to_string(),
            Duration::from_millis(50),
            CompilationError::SyntaxError {
                message: "test error".to_string(),
                line: Some(1),
                column: Some(1),
            },
            "error output".to_string(),
        );
        
        report.add_result(failure_result);
        assert_eq!(report.success_rate(), 50.0);
        assert!(!report.all_passed());
    }

    #[test]
    fn test_validation_report_snippets() {
        let mut report = ValidationReport::new();
        
        // Test marking as snippet
        report.mark_as_snippet("snippet1".to_string(), "incomplete code".to_string());
        assert_eq!(report.snippets, 1);
        assert_eq!(report.total_examples, 1);
        
        // Snippets should be considered successful
        let snippet_result = &report.results[0];
        assert!(snippet_result.success);
        assert_eq!(snippet_result.target, "snippet");
        assert!(snippet_result.stdout.contains("incomplete code"));
    }

    #[test]
    fn test_validation_report_recovery_tracking() {
        let mut report = ValidationReport::new();
        
        // Test recovery tracking
        report.record_recovery("test1".to_string(), "fallback");
        report.record_recovery("test2".to_string(), "snippet");
        report.record_recovery("test3".to_string(), "feature_disable");
        
        assert_eq!(report.recovered, 3);
        assert_eq!(report.recovery_stats.fallback_attempts, 1);
        assert_eq!(report.recovery_stats.marked_as_snippets, 1);
        assert_eq!(report.recovery_stats.feature_disabling_attempts, 1);
        
        // Test successful/failed recovery tracking
        report.record_successful_recovery();
        report.record_failed_recovery();
        
        assert_eq!(report.recovery_stats.successful_recoveries, 1);
        assert_eq!(report.recovery_stats.failed_recoveries, 1);
    }

    #[test]
    fn test_compilation_error_display() {
        let syntax_error = CompilationError::SyntaxError {
            message: "expected semicolon".to_string(),
            line: Some(5),
            column: Some(10),
        };
        let display = format!("{}", syntax_error);
        assert!(display.contains("Syntax error at line 5, column 10"));
        assert!(display.contains("expected semicolon"));

        let dependency_error = CompilationError::DependencyError {
            missing: vec!["embedded-hal".to_string(), "cortex-m".to_string()],
        };
        let display = format!("{}", dependency_error);
        assert!(display.contains("Missing dependencies: embedded-hal, cortex-m"));

        let timeout_error = CompilationError::TimeoutError {
            duration: Duration::from_secs(30),
        };
        let display = format!("{}", timeout_error);
        assert!(display.contains("Compilation timeout after 30s"));
    }

    #[test]
    fn test_recovery_statistics_default() {
        let stats = RecoveryStatistics::default();
        assert_eq!(stats.successful_recoveries, 0);
        assert_eq!(stats.failed_recoveries, 0);
        assert_eq!(stats.marked_as_snippets, 0);
        assert_eq!(stats.fallback_attempts, 0);
        assert_eq!(stats.feature_disabling_attempts, 0);
        assert_eq!(stats.timeout_extensions, 0);
    }

    #[test]
    fn test_test_result_creation() {
        let success = TestResult::success(
            "test_success".to_string(),
            "x86_64-unknown-linux-gnu".to_string(),
            Duration::from_millis(500),
        );
        assert!(success.success);
        assert!(success.error.is_none());
        assert_eq!(success.example_id, "test_success");

        let failure = TestResult::failure(
            "test_failure".to_string(),
            "thumbv7em-none-eabihf".to_string(),
            Duration::from_millis(200),
            CompilationError::TargetError {
                incompatible_target: "thumbv7em-none-eabihf".to_string(),
                reason: "missing target".to_string(),
            },
            "target not found".to_string(),
        );
        assert!(!failure.success);
        assert!(failure.error.is_some());
        assert_eq!(failure.stderr, "target not found");
    }
}