//! Core code example testing infrastructure for embedded Rust tutorial
//! 
//! This module provides functionality to extract, validate, and test Rust code examples
//! from markdown files to ensure they compile correctly.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]

#[cfg(not(feature = "std"))]
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
#[cfg(all(test, feature = "std"))]
mod integration_tests;

#[cfg(feature = "std")]
pub use example_tester::RustExampleTester;
pub use code_extractor::{CodeExtractor, CodeExample, ExampleContext};
pub use test_config::{TestConfig, TestResult, CompilationError, ValidationReport};
#[cfg(feature = "std")]
pub use toolchain_manager::{ToolchainManager, TargetInfo, TargetType, SystemInfo};
pub use dependency_resolver::{DependencyResolver, Dependency};
pub use error_recovery::{ErrorRecoveryManager, GracefulDegradationManager, RecoveryAction, DegradationStrategy};

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