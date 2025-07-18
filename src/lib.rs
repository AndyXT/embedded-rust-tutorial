//! Core code example testing infrastructure for embedded Rust tutorial
//! 
//! This module provides functionality to extract, validate, and test Rust code examples
//! from markdown files to ensure they compile correctly.

// Only enable no_std when explicitly building for embedded targets without std
#[cfg(all(not(feature = "std"), feature = "embedded"))]
#![no_std]
#[cfg(all(not(feature = "std"), feature = "embedded"))]
#![no_main]

#[cfg(all(not(feature = "std"), feature = "embedded"))]
extern crate alloc;

#[cfg(feature = "std")]
pub mod example_tester;
pub mod code_extractor;
pub mod test_config;
#[cfg(feature = "std")]
pub mod toolchain_manager;
pub mod dependency_resolver;
pub mod error_recovery;
#[cfg(feature = "std")]
pub mod preprocessor;
#[cfg(feature = "std")]
pub mod cache;
#[cfg(feature = "std")]
pub mod security_validator;
#[cfg(feature = "std")]
pub mod metrics_collector;
#[cfg(feature = "std")]
pub mod final_integration_validator;
#[cfg(all(test, feature = "std"))]
mod integration_tests;

// Cortex-R5 specific module for embedded development
pub mod cortex_r5;
pub mod cortex_r5_transformer;

#[cfg(feature = "std")]
pub use example_tester::RustExampleTester;
pub use code_extractor::{CodeExtractor, CodeExample, ExampleContext};
pub use test_config::{TestConfig, TestResult, CompilationError, ValidationReport};
#[cfg(feature = "std")]
pub use toolchain_manager::{ToolchainManager, TargetInfo, TargetType, SystemInfo};
pub use dependency_resolver::{DependencyResolver, Dependency};
pub use error_recovery::{ErrorRecoveryManager, GracefulDegradationManager, RecoveryAction, DegradationStrategy};
#[cfg(feature = "std")]
pub use cache::{CacheManager, CacheConfig, CacheEntry, CachedTestResult, CacheStats};
#[cfg(feature = "std")]
pub use security_validator::{SecurityValidator, SecurityValidationResult, SecurityCheck, SecurityWarning, SecurityViolation, SecurityCategory, SecurityReport, CategorySummary};
#[cfg(feature = "std")]
pub use metrics_collector::{MetricsCollector, CompilationMetrics, TargetMetrics, PerformanceMetrics, CompilationResult};
#[cfg(feature = "std")]
pub use final_integration_validator::{FinalIntegrationValidator, IntegrationValidationReport, ComponentTestResults, RequirementValidationResults, SystemIntegrationResults, IntegrationPerformanceMetrics};

// Cortex-R5 specific exports
pub use cortex_r5::{utils, config, error};
#[cfg(all(feature = "crypto", feature = "embedded"))]
pub use cortex_r5::crypto;
#[cfg(feature = "embedded")]
pub use cortex_r5::examples;
pub use cortex_r5_transformer::{CortexR5Transformer, TransformConfig, CortexR5Variant, TransformError, PatternMatcher, CollectionReplacement, StringReplacement};

#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    
    #[test]
    fn test_library_integration() {
        let config = TestConfig::default();
        let tester = RustExampleTester::new(config).expect("Failed to create tester");
        assert!(tester.is_ready());
    }
}