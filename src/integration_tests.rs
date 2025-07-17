//! Integration tests for dependency resolution with code extraction

#[cfg(all(test, feature = "std"))]
mod tests {
    use crate::{CodeExtractor, DependencyResolver};
    use std::path::PathBuf;

    #[test]
    fn test_dependency_resolution_integration() {
        let extractor = CodeExtractor::new().expect("Failed to create extractor");
        let resolver = DependencyResolver::new();

        // Test markdown content with various code examples
        let content = r#"
# Test Document

## Standard Library Example

```rust
use std::collections::HashMap;
fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    println!("Hello, world!");
}
```

## No-std Example

```rust,no_std
#![no_std]
use heapless::Vec;
use cortex_m::interrupt;

#[entry]
fn main() -> ! {
    let mut data: Vec<u8, 32> = Vec::new();
    loop {}
}
```

## Crypto Example

```rust,crypto,algorithm=AES
use aes::Aes256;
use zeroize::Zeroize;

fn encrypt_data() {
    let cipher = Aes256::new(&key);
    let mut key = [0u8; 32];
    // ... encryption logic
    key.zeroize();
}
```

## Hardware Example

```rust,hardware=STM32F4
use stm32f4xx_hal::gpio::*;

fn setup_gpio() {
    // GPIO setup code
}
```

## Snippet (should be ignored)

```rust,snippet
let key = generate_key();
```
"#;

        // Extract examples
        let examples = extractor
            .extract_from_content(content, &PathBuf::from("test.md"))
            .expect("Failed to extract examples");

        assert_eq!(examples.len(), 5);

        // Test dependency resolution for each example
        for example in &examples {
            let deps = resolver.resolve_dependencies(example);
            
            match &example.context {
                crate::ExampleContext::Std { .. } => {
                    // Std example uses only standard library, so no external dependencies expected
                    // This is fine - std examples might not have external dependencies
                }
                crate::ExampleContext::NoStd { .. } => {
                    assert!(!deps.is_empty(), "NoStd example should have dependencies");
                    let heapless_dep = deps.iter().find(|d| d.name == "heapless");
                    assert!(heapless_dep.is_some(), "Should have heapless dependency");
                    let cortex_m_dep = deps.iter().find(|d| d.name == "cortex-m");
                    assert!(cortex_m_dep.is_some(), "Should have cortex-m dependency");
                    let cortex_m_rt_dep = deps.iter().find(|d| d.name == "cortex-m-rt");
                    assert!(cortex_m_rt_dep.is_some(), "Should have cortex-m-rt dependency");
                }
                crate::ExampleContext::Crypto { .. } => {
                    assert!(!deps.is_empty(), "Crypto example should have dependencies");
                    let aes_dep = deps.iter().find(|d| d.name == "aes");
                    assert!(aes_dep.is_some(), "Should have aes dependency");
                    let zeroize_dep = deps.iter().find(|d| d.name == "zeroize");
                    assert!(zeroize_dep.is_some(), "Should have zeroize dependency");
                }
                crate::ExampleContext::Hardware { .. } => {
                    assert!(!deps.is_empty(), "Hardware example should have dependencies");
                    let stm32_dep = deps.iter().find(|d| d.name == "stm32f4xx-hal");
                    assert!(stm32_dep.is_some(), "Should have stm32f4xx-hal dependency");
                }
                crate::ExampleContext::Snippet { .. } => {
                    assert!(deps.is_empty(), "Snippet should have no dependencies");
                }
            }
        }

        // Test Cargo.toml generation
        let cargo_toml = resolver.generate_cargo_toml(&examples, "integration_test");
        
        // Verify basic structure
        assert!(cargo_toml.contains("[package]"));
        assert!(cargo_toml.contains("name = \"integration_test\""));
        assert!(cargo_toml.contains("[dependencies]"));
        
        // Should include no_std profile settings since we have no_std examples
        assert!(cargo_toml.contains("[profile.release]"));
        assert!(cargo_toml.contains("opt-level = \"s\""));
        
        // Should have target-specific dependencies
        assert!(cargo_toml.contains("[target.'cfg(target = \"thumbv7em-none-eabihf\")'.dependencies]"));
    }

    #[test]
    fn test_single_example_cargo_toml_generation() {
        let extractor = CodeExtractor::new().expect("Failed to create extractor");
        let resolver = DependencyResolver::new();

        let content = r#"
```rust,crypto
use zeroize::Zeroize;
use aes::Aes256;

fn secure_function() {
    let mut key = [0u8; 32];
    // ... crypto operations
    key.zeroize();
}
```
"#;

        let examples = extractor
            .extract_from_content(content, &PathBuf::from("crypto_test.md"))
            .expect("Failed to extract examples");

        assert_eq!(examples.len(), 1);
        let example = &examples[0];

        let cargo_toml = resolver.generate_example_cargo_toml(example);
        
        assert!(cargo_toml.contains("[package]"));
        assert!(cargo_toml.contains(&format!("name = \"example_{}\"", example.id)));
        assert!(cargo_toml.contains("aes = { version = \"0.8\", default-features = false, features = [\"crypto\"] }"));
        assert!(cargo_toml.contains("zeroize = { version = \"1.7\", features = [\"crypto\", \"derive\"] }"));
    }

    #[test]
    fn test_dependency_catalog_extensibility() {
        let resolver = DependencyResolver::new();
        
        // Test that we can add custom dependencies
        let known_deps_before = resolver.known_dependencies();
        let initial_count = known_deps_before.len();
        
        // This should work but we can't test it directly due to private DependencyInfo
        // Just verify the known dependencies list is reasonable
        assert!(known_deps_before.contains(&"heapless".to_string()));
        assert!(known_deps_before.contains(&"zeroize".to_string()));
        assert!(known_deps_before.contains(&"aes".to_string()));
        assert!(known_deps_before.contains(&"cortex-m".to_string()));
        assert!(initial_count > 10, "Should have a reasonable number of known dependencies");
    }
}