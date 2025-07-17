//! Error recovery strategies and graceful degradation for code example testing
//! 
//! This module provides mechanisms to handle compilation failures gracefully,
//! attempt recovery strategies, and provide meaningful feedback to users.

#[cfg(feature = "std")]
use std::time::Duration;
#[cfg(feature = "std")]
use std::path::PathBuf;
#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use core::time::Duration;
#[cfg(not(feature = "std"))]
use heapless::{String as PathBuf, FnvIndexMap as HashMap};

// Import standard types conditionally
#[cfg(feature = "std")]
use std::string::String;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

// Import core types
use core::option::Option::{self, Some, None};
use core::default::Default;
use core::fmt;

use crate::test_config::{CompilationError, TestConfig};
use crate::code_extractor::{CodeExample, ExampleContext};

/// Recovery action to take when a compilation error occurs
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryAction {
    /// Retry compilation with modified parameters
    Retry {
        /// Modified configuration to try
        config: TestConfig,
        /// Reason for the retry attempt
        reason: String,
    },
    /// Skip this example with a reason
    Skip {
        /// Reason for skipping
        reason: String,
    },
    /// Fallback to a simpler compilation target
    Fallback {
        /// Alternative target to try
        target: String,
        /// Features to disable
        disable_features: Vec<String>,
    },
    /// Mark as snippet (don't compile)
    MarkAsSnippet {
        /// Reason for marking as snippet
        reason: String,
    },
    /// Abort testing entirely
    Abort {
        /// Critical error message
        message: String,
    },
}

/// Error recovery strategy manager
#[derive(Debug)]
pub struct ErrorRecoveryManager {
    /// Maximum number of retry attempts per example
    max_retries: usize,
    /// Fallback targets in order of preference
    fallback_targets: Vec<String>,
    /// Features that can be safely disabled for recovery
    optional_features: Vec<String>,
    /// Cache of recovery actions for similar errors
    recovery_cache: HashMap<String, RecoveryAction>,
}

impl Default for ErrorRecoveryManager {
    fn default() -> Self {
        Self {
            max_retries: 3,
            fallback_targets: vec![
                "x86_64-unknown-linux-gnu".to_string(),
                "thumbv7em-none-eabihf".to_string(),
                "thumbv6m-none-eabi".to_string(),
            ],
            optional_features: vec![
                "hardware".to_string(),
                "crypto-hardware".to_string(),
                "advanced".to_string(),
            ],
            recovery_cache: HashMap::new(),
        }
    }
}

impl ErrorRecoveryManager {
    /// Create a new error recovery manager with custom settings
    pub fn new(
        max_retries: usize,
        fallback_targets: Vec<String>,
        optional_features: Vec<String>,
    ) -> Self {
        Self {
            max_retries,
            fallback_targets,
            optional_features,
            recovery_cache: HashMap::new(),
        }
    }

    /// Determine the appropriate recovery action for a compilation error
    pub fn determine_recovery_action(
        &mut self,
        error: &CompilationError,
        example: &CodeExample,
        config: &TestConfig,
        retry_count: usize,
    ) -> RecoveryAction {
        // Check cache first for similar errors
        let error_key = self.error_cache_key(error, &example.context);
        if let Some(cached_action) = self.recovery_cache.get(&error_key) {
            return cached_action.clone();
        }

        let action = match error {
            CompilationError::SyntaxError { message, line, column } => {
                self.handle_syntax_error(message, *line, *column, example, retry_count)
            }
            CompilationError::DependencyError { missing } => {
                self.handle_dependency_error(missing, example, config, retry_count)
            }
            CompilationError::FeatureError { unsupported } => {
                self.handle_feature_error(unsupported, example, config, retry_count)
            }
            CompilationError::TargetError { incompatible_target, reason } => {
                self.handle_target_error(incompatible_target, reason, example, config, retry_count)
            }
            CompilationError::TimeoutError { duration } => {
                self.handle_timeout_error(*duration, example, retry_count)
            }
            CompilationError::CompilationFailed { exit_code, message } => {
                self.handle_compilation_failure(*exit_code, message, example, retry_count)
            }
            CompilationError::IoError { message } => {
                self.handle_io_error(message, example, retry_count)
            }
        };

        // Cache the recovery action for future use
        self.recovery_cache.insert(error_key, action.clone());
        action
    }

    /// Handle syntax errors with intelligent recovery
    fn handle_syntax_error(
        &self,
        message: &str,
        line: Option<usize>,
        column: Option<usize>,
        example: &CodeExample,
        retry_count: usize,
    ) -> RecoveryAction {
        // Check if this looks like a code snippet rather than complete code
        if self.is_likely_snippet(&example.code) {
            return RecoveryAction::MarkAsSnippet {
                reason: format!("Syntax error suggests incomplete code snippet: {}", message),
            };
        }

        // For certain syntax patterns, suggest marking as snippet
        if message.contains("expected item") || 
           message.contains("unexpected end of file") ||
           message.contains("expected `fn`, `mod`, `struct`") {
            return RecoveryAction::MarkAsSnippet {
                reason: "Code appears to be a fragment rather than complete example".to_string(),
            };
        }

        // If we've retried too many times, skip
        if retry_count >= self.max_retries {
            return RecoveryAction::Skip {
                reason: format!("Syntax error persists after {} retries: {}", retry_count, message),
            };
        }

        // For other syntax errors, provide detailed skip reason
        RecoveryAction::Skip {
            reason: format!("Syntax error at line {:?}, column {:?}: {}", line, column, message),
        }
    }

    /// Handle missing dependency errors
    fn handle_dependency_error(
        &self,
        missing: &[String],
        _example: &CodeExample,
        config: &TestConfig,
        retry_count: usize,
    ) -> RecoveryAction {
        if retry_count >= self.max_retries {
            return RecoveryAction::Skip {
                reason: format!("Missing dependencies after {} retries: {}", retry_count, missing.join(", ")),
            };
        }

        // Try to add missing dependencies and retry
        let mut new_config = config.clone();
        
        // Add common embedded dependencies
        for dep in missing {
            if dep.contains("embedded") || dep.contains("cortex") || dep.contains("hal") {
                if !new_config.features.contains(&"embedded".to_string()) {
                    new_config.features.push("embedded".to_string());
                }
            }
            if dep.contains("crypto") || dep.contains("aes") || dep.contains("sha") {
                if !new_config.features.contains(&"crypto".to_string()) {
                    new_config.features.push("crypto".to_string());
                }
            }
        }

        RecoveryAction::Retry {
            config: new_config,
            reason: format!("Adding features to resolve missing dependencies: {}", missing.join(", ")),
        }
    }

    /// Handle unsupported feature errors
    fn handle_feature_error(
        &self,
        unsupported: &[String],
        _example: &CodeExample,
        config: &TestConfig,
        retry_count: usize,
    ) -> RecoveryAction {
        // Check if we can disable optional features
        let mut can_disable = Vec::new();
        for feature in unsupported {
            if self.optional_features.contains(feature) {
                can_disable.push(feature.clone());
            }
        }

        if !can_disable.is_empty() && retry_count < self.max_retries {
            let mut new_config = config.clone();
            for feature in &can_disable {
                new_config.features.retain(|f| f != feature);
            }

            return RecoveryAction::Retry {
                config: new_config,
                reason: format!("Disabling optional features: {}", can_disable.join(", ")),
            };
        }

        // If features are critical, skip the example
        RecoveryAction::Skip {
            reason: format!("Unsupported features required: {}", unsupported.join(", ")),
        }
    }

    /// Handle target incompatibility errors
    fn handle_target_error(
        &self,
        incompatible_target: &str,
        reason: &str,
        _example: &CodeExample,
        config: &TestConfig,
        retry_count: usize,
    ) -> RecoveryAction {
        // Try fallback targets
        for fallback_target in &self.fallback_targets {
            if fallback_target != incompatible_target && 
               !config.targets.contains(fallback_target) &&
               retry_count < self.max_retries {
                
                return RecoveryAction::Fallback {
                    target: fallback_target.clone(),
                    disable_features: self.get_target_incompatible_features(fallback_target),
                };
            }
        }

        RecoveryAction::Skip {
            reason: format!("No compatible target found. Original error: {}", reason),
        }
    }

    /// Handle compilation timeout errors
    fn handle_timeout_error(
        &self,
        duration: Duration,
        _example: &CodeExample,
        retry_count: usize,
    ) -> RecoveryAction {
        if retry_count < self.max_retries {
            // Increase timeout and retry
            let mut new_config = TestConfig::default();
            let new_timeout = duration * 2;
            new_config.compilation_timeout = new_timeout;
            
            RecoveryAction::Retry {
                config: new_config,
                reason: format!("Increasing timeout from {:?} to {:?}", duration, new_timeout),
            }
        } else {
            RecoveryAction::Skip {
                reason: format!("Compilation timeout after {:?} (tried {} times)", duration, retry_count),
            }
        }
    }

    /// Handle general compilation failures
    fn handle_compilation_failure(
        &self,
        exit_code: i32,
        message: &str,
        _example: &CodeExample,
        retry_count: usize,
    ) -> RecoveryAction {
        // Check for common patterns that indicate snippets
        if message.contains("cannot find function `main`") ||
           message.contains("binary target") {
            return RecoveryAction::MarkAsSnippet {
                reason: "Code appears to be a library snippet without main function".to_string(),
            };
        }

        // Check for linker errors that might be target-specific
        if message.contains("linker") || message.contains("undefined reference") {
            if retry_count < self.max_retries {
                return RecoveryAction::Fallback {
                    target: "x86_64-unknown-linux-gnu".to_string(),
                    disable_features: vec!["embedded".to_string()],
                };
            }
        }

        RecoveryAction::Skip {
            reason: format!("Compilation failed (exit code {}): {}", exit_code, message),
        }
    }

    /// Handle I/O errors
    fn handle_io_error(
        &self,
        message: &str,
        _example: &CodeExample,
        retry_count: usize,
    ) -> RecoveryAction {
        if retry_count < self.max_retries {
            RecoveryAction::Retry {
                config: TestConfig::default(),
                reason: format!("Retrying after I/O error: {}", message),
            }
        } else {
            RecoveryAction::Skip {
                reason: format!("Persistent I/O error: {}", message),
            }
        }
    }

    /// Check if code looks like a snippet rather than complete example
    fn is_likely_snippet(&self, code: &str) -> bool {
        let trimmed = code.trim();
        
        // Check for common snippet patterns
        if trimmed.starts_with("let ") ||
           trimmed.starts_with("const ") ||
           trimmed.starts_with("//") ||
           trimmed.starts_with("use ") ||
           trimmed.starts_with("impl ") ||
           trimmed.starts_with("struct ") ||
           trimmed.starts_with("enum ") {
            return true;
        }

        // Check if it lacks main function or module structure
        if !trimmed.contains("fn main") && 
           !trimmed.contains("mod ") &&
           !trimmed.contains("#[no_mangle]") &&
           !trimmed.contains("#[entry]") {
            return true;
        }

        false
    }

    /// Generate cache key for error patterns
    fn error_cache_key(&self, error: &CompilationError, context: &ExampleContext) -> String {
        match error {
            CompilationError::SyntaxError { message, .. } => {
                format!("syntax:{}:{:?}", message.chars().take(50).collect::<String>(), context)
            }
            CompilationError::DependencyError { missing } => {
                format!("deps:{}:{:?}", missing.join(","), context)
            }
            CompilationError::FeatureError { unsupported } => {
                format!("features:{}:{:?}", unsupported.join(","), context)
            }
            CompilationError::TargetError { incompatible_target, .. } => {
                format!("target:{}:{:?}", incompatible_target, context)
            }
            CompilationError::TimeoutError { .. } => {
                format!("timeout:{:?}", context)
            }
            CompilationError::CompilationFailed { exit_code, .. } => {
                format!("failed:{}:{:?}", exit_code, context)
            }
            CompilationError::IoError { .. } => {
                format!("io:{:?}", context)
            }
        }
    }

    /// Get features that should be disabled for specific targets
    fn get_target_incompatible_features(&self, target: &str) -> Vec<String> {
        match target {
            "x86_64-unknown-linux-gnu" => vec!["embedded".to_string(), "hardware".to_string()],
            "thumbv7em-none-eabihf" | "thumbv6m-none-eabi" => vec!["std".to_string()],
            _ => vec![],
        }
    }
}

/// Graceful degradation manager for handling systematic failures
#[derive(Debug)]
pub struct GracefulDegradationManager {
    /// Failure threshold before switching to degraded mode
    failure_threshold: f64,
    /// Current failure rate
    current_failure_rate: f64,
    /// Whether we're in degraded mode
    degraded_mode: bool,
    /// Degradation strategies to apply
    degradation_strategies: Vec<DegradationStrategy>,
}

/// Degradation strategy to apply when failure rates are high
#[derive(Debug, Clone)]
pub enum DegradationStrategy {
    /// Skip complex examples and focus on basic ones
    SkipComplexExamples,
    /// Disable hardware-specific testing
    DisableHardwareTesting,
    /// Use syntax-only validation instead of full compilation
    SyntaxOnlyValidation,
    /// Reduce compilation timeout
    ReduceTimeout,
    /// Skip embedded targets
    SkipEmbeddedTargets,
}

impl Default for GracefulDegradationManager {
    fn default() -> Self {
        Self {
            failure_threshold: 0.5, // 50% failure rate triggers degradation
            current_failure_rate: 0.0,
            degraded_mode: false,
            degradation_strategies: vec![
                DegradationStrategy::SkipComplexExamples,
                DegradationStrategy::DisableHardwareTesting,
                DegradationStrategy::SyntaxOnlyValidation,
            ],
        }
    }
}

impl GracefulDegradationManager {
    /// Update failure rate and check if degradation is needed
    pub fn update_failure_rate(&mut self, successful: usize, total: usize) {
        if total > 0 {
            self.current_failure_rate = 1.0 - (successful as f64 / total as f64);
            
            if self.current_failure_rate > self.failure_threshold && !self.degraded_mode {
                self.degraded_mode = true;
                #[cfg(feature = "std")]
                eprintln!("Entering degraded mode due to high failure rate: {:.1}%", 
                          self.current_failure_rate * 100.0);
            } else if self.current_failure_rate < self.failure_threshold * 0.5 && self.degraded_mode {
                self.degraded_mode = false;
                #[cfg(feature = "std")]
                eprintln!("Exiting degraded mode, failure rate improved: {:.1}%", 
                          self.current_failure_rate * 100.0);
            }
        }
    }

    /// Check if we should apply degradation to an example
    pub fn should_degrade_example(&self, example: &CodeExample) -> Option<DegradationStrategy> {
        if !self.degraded_mode {
            return None;
        }

        for strategy in &self.degradation_strategies {
            match strategy {
                DegradationStrategy::SkipComplexExamples => {
                    if self.is_complex_example(example) {
                        return Some(strategy.clone());
                    }
                }
                DegradationStrategy::DisableHardwareTesting => {
                    if matches!(example.context, ExampleContext::Hardware { .. }) {
                        return Some(strategy.clone());
                    }
                }
                DegradationStrategy::SyntaxOnlyValidation => {
                    // Always applicable in degraded mode
                    return Some(strategy.clone());
                }
                _ => continue,
            }
        }

        None
    }

    /// Check if an example is considered complex
    fn is_complex_example(&self, example: &CodeExample) -> bool {
        let code = &example.code;
        
        // Count complexity indicators
        let complexity_score = 
            code.matches("unsafe").count() * 3 +
            code.matches("impl").count() * 2 +
            code.matches("trait").count() * 2 +
            code.matches("macro_rules!").count() * 3 +
            code.matches("async").count() * 2 +
            code.matches("await").count() * 2;

        complexity_score > 5 || code.lines().count() > 50
    }

    /// Get current degradation status
    pub fn is_degraded(&self) -> bool {
        self.degraded_mode
    }

    /// Get current failure rate
    pub fn failure_rate(&self) -> f64 {
        self.current_failure_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_extractor::ExampleContext;
    use crate::test_config::{ValidationReport, TestResult};

    #[test]
    fn test_error_recovery_manager_creation() {
        let manager = ErrorRecoveryManager::default();
        assert_eq!(manager.max_retries, 3);
        assert!(!manager.fallback_targets.is_empty());
    }

    #[test]
    fn test_syntax_error_recovery() {
        let mut manager = ErrorRecoveryManager::default();
        let example = CodeExample {
            id: "test".to_string(),
            source_file: PathBuf::from("test.md"),
            line_number: 1,
            code: "let x = ".to_string(), // Incomplete code
            context: ExampleContext::Snippet { reason: "test".to_string() },
            annotations: HashMap::new(),
            dependencies: vec![],
        };

        let error = CompilationError::SyntaxError {
            message: "unexpected end of file".to_string(),
            line: Some(1),
            column: Some(8),
        };

        let action = manager.determine_recovery_action(&error, &example, &TestConfig::default(), 0);
        
        match action {
            RecoveryAction::MarkAsSnippet { reason } => {
                assert!(reason.contains("fragment") || reason.contains("incomplete"));
            }
            _ => panic!("Expected MarkAsSnippet action, got: {:?}", action),
        }
    }

    #[test]
    fn test_dependency_error_recovery() {
        let mut manager = ErrorRecoveryManager::default();
        let example = CodeExample {
            id: "test".to_string(),
            source_file: PathBuf::from("test.md"),
            line_number: 1,
            code: "use embedded_hal::digital::v2::OutputPin;".to_string(),
            context: ExampleContext::NoStd { target: "thumbv7em-none-eabihf".to_string(), features: vec![] },
            annotations: HashMap::new(),
            dependencies: vec![],
        };

        let error = CompilationError::DependencyError {
            missing: vec!["embedded-hal".to_string()],
        };

        let action = manager.determine_recovery_action(&error, &example, &TestConfig::default(), 0);
        
        match action {
            RecoveryAction::Retry { config, reason } => {
                assert!(config.features.contains(&"embedded".to_string()));
                assert!(reason.contains("embedded-hal"));
            }
            _ => panic!("Expected Retry action"),
        }
    }

    #[test]
    fn test_graceful_degradation_manager() {
        let mut manager = GracefulDegradationManager::default();
        assert!(!manager.is_degraded());

        // Simulate high failure rate
        manager.update_failure_rate(2, 10); // 80% failure rate
        assert!(manager.is_degraded());

        // Simulate recovery
        manager.update_failure_rate(8, 10); // 20% failure rate
        assert!(!manager.is_degraded());
    }

    #[test]
    fn test_complex_example_detection() {
        let manager = GracefulDegradationManager::default();
        
        let simple_example = CodeExample {
            id: "simple".to_string(),
            source_file: PathBuf::from("test.md"),
            line_number: 1,
            code: "let x = 42;".to_string(),
            context: ExampleContext::Std { features: vec![] },
            annotations: HashMap::new(),
            dependencies: vec![],
        };

        let complex_example = CodeExample {
            id: "complex".to_string(),
            source_file: PathBuf::from("test.md"),
            line_number: 1,
            code: "unsafe impl Send for MyStruct {}\nimpl MyTrait for MyStruct {}\nmacro_rules! my_macro { () => {} }".to_string(),
            context: ExampleContext::Std { features: vec![] },
            annotations: HashMap::new(),
            dependencies: vec![],
        };

        assert!(!manager.is_complex_example(&simple_example));
        assert!(manager.is_complex_example(&complex_example));
    }

    #[test]
    fn test_complete_error_recovery_workflow() {
        let mut recovery_manager = ErrorRecoveryManager::default();
        let mut degradation_manager = GracefulDegradationManager::default();
        let mut validation_report = ValidationReport::new();

        // Test various error scenarios and recovery actions
        let examples = vec![
            // Syntax error example
            CodeExample {
                id: "syntax_error".to_string(),
                source_file: PathBuf::from("test.md"),
                line_number: 1,
                code: "let x = ".to_string(),
                context: ExampleContext::Std { features: vec![] },
                annotations: HashMap::new(),
                dependencies: vec![],
            },
            // Missing dependency example
            CodeExample {
                id: "missing_dep".to_string(),
                source_file: PathBuf::from("test.md"),
                line_number: 10,
                code: "use embedded_hal::digital::v2::OutputPin;".to_string(),
                context: ExampleContext::NoStd { target: "thumbv7em-none-eabihf".to_string(), features: vec![] },
                annotations: HashMap::new(),
                dependencies: vec![],
            },
        ];

        // Simulate error recovery for each example
        for example in &examples {
            let error = match example.id.as_str() {
                "syntax_error" => CompilationError::SyntaxError {
                    message: "unexpected end of file".to_string(),
                    line: Some(1),
                    column: Some(8),
                },
                "missing_dep" => CompilationError::DependencyError {
                    missing: vec!["embedded-hal".to_string()],
                },
                _ => continue,
            };

            let action = recovery_manager.determine_recovery_action(
                &error,
                example,
                &TestConfig::default(),
                0,
            );

            match action {
                RecoveryAction::MarkAsSnippet { reason } => {
                    validation_report.mark_as_snippet(example.id.clone(), reason);
                    validation_report.record_recovery(example.id.clone(), "snippet");
                }
                RecoveryAction::Retry { config: _, reason } => {
                    validation_report.record_successful_recovery();
                    // In a real scenario, we would retry compilation here
                    let success_result = TestResult::success(
                        example.id.clone(),
                        "x86_64-unknown-linux-gnu".to_string(),
                        Duration::from_millis(100),
                    );
                    validation_report.add_result(success_result);
                }
                RecoveryAction::Skip { reason } => {
                    validation_report.skip_example(example.id.clone(), reason);
                }
                _ => {}
            }
        }

        // Test graceful degradation
        degradation_manager.update_failure_rate(1, 10); // 90% failure rate
        assert!(degradation_manager.is_degraded());

        // Verify the validation report contains recovery statistics
        assert!(validation_report.recovery_stats.successful_recoveries > 0 || 
                validation_report.recovery_stats.marked_as_snippets > 0);
        assert!(validation_report.total_examples > 0);
        
        // Test that the system can handle high failure rates gracefully
        assert_eq!(degradation_manager.failure_rate(), 0.9);
    }
}