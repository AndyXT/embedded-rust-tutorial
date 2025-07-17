//! Tests for mdBook preprocessor functionality

use super::*;
use std::collections::HashMap;
use serde_json;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> PreprocessorContext {
        let mut config = HashMap::new();
        let mut preprocessor_config = HashMap::new();
        let mut rust_examples_config = HashMap::new();
        let mut config_values = HashMap::new();
        
        config_values.insert("embedded-target".to_string(), serde_json::Value::String("thumbv7em-none-eabihf".to_string()));
        config_values.insert("no-std-mode".to_string(), serde_json::Value::Bool(true));
        config_values.insert("validation-enabled".to_string(), serde_json::Value::Bool(true));
        
        rust_examples_config.insert("config".to_string(), serde_json::Value::Object(config_values.into_iter().collect()));
        preprocessor_config.insert("rust-examples".to_string(), serde_json::Value::Object(rust_examples_config.into_iter().collect()));
        config.insert("preprocessor".to_string(), serde_json::Value::Object(preprocessor_config.into_iter().collect()));
        
        PreprocessorContext {
            root: "/test".to_string(),
            config,
            renderer: "html".to_string(),
        }
    }

    fn create_test_book_item(content: &str) -> BookItem {
        BookItem {
            name: "Test Chapter".to_string(),
            content: content.to_string(),
            number: Some(vec![1]),
            sub_items: vec![],
            path: Some("test.md".to_string()),
            source_path: Some("src/test.md".to_string()),
            parent_names: vec![],
        }
    }

    #[test]
    fn test_preprocessor_creation() {
        let preprocessor = RustExamplesPreprocessor::new();
        assert!(preprocessor.tester.is_none());
        assert_eq!(preprocessor.compilation_cache.len(), 0);
    }

    #[test]
    fn test_config_loading() {
        let mut preprocessor = RustExamplesPreprocessor::new();
        let context = create_test_context();
        
        preprocessor.load_config(&context).unwrap();
        
        assert_eq!(preprocessor.config.embedded_target, Some("thumbv7em-none-eabihf".to_string()));
        assert_eq!(preprocessor.config.no_std_mode, Some(true));
        assert_eq!(preprocessor.config.validation_enabled, Some(true));
    }

    #[test]
    fn test_annotation_parsing_no_std() {
        let preprocessor = RustExamplesPreprocessor::new();
        let context = preprocessor.parse_annotations("no_std,features=crypto");
        
        match context {
            ExampleContext::NoStd { target, features } => {
                assert_eq!(target, "thumbv7em-none-eabihf");
                assert_eq!(features, vec!["crypto"]);
            }
            _ => panic!("Expected NoStd context"),
        }
    }

    #[test]
    fn test_annotation_parsing_hardware() {
        let preprocessor = RustExamplesPreprocessor::new();
        let context = preprocessor.parse_annotations("hardware=stm32f4,features=hal");
        
        match context {
            ExampleContext::Hardware { platform, features } => {
                assert_eq!(platform, "stm32f4");
                assert_eq!(features, vec!["hal"]);
            }
            _ => panic!("Expected Hardware context"),
        }
    }

    #[test]
    fn test_annotation_parsing_crypto() {
        let preprocessor = RustExamplesPreprocessor::new();
        let context = preprocessor.parse_annotations("crypto=aes,features=zeroize");
        
        match context {
            ExampleContext::Crypto { algorithm, features } => {
                assert_eq!(algorithm, "aes");
                assert_eq!(features, vec!["zeroize"]);
            }
            _ => panic!("Expected Crypto context"),
        }
    }

    #[test]
    fn test_annotation_parsing_snippet() {
        let preprocessor = RustExamplesPreprocessor::new();
        let context = preprocessor.parse_annotations("snippet=incomplete");
        
        match context {
            ExampleContext::Snippet { reason } => {
                assert_eq!(reason, "incomplete");
            }
            _ => panic!("Expected Snippet context"),
        }
    }

    #[test]
    fn test_annotation_parsing_std() {
        let preprocessor = RustExamplesPreprocessor::new();
        let context = preprocessor.parse_annotations("features=std,tokio");
        
        match context {
            ExampleContext::Std { features } => {
                assert_eq!(features, vec!["std", "tokio"]);
            }
            _ => panic!("Expected Std context"),
        }
    }

    #[test]
    fn test_feature_extraction() {
        let preprocessor = RustExamplesPreprocessor::new();
        
        let features = preprocessor.extract_features("features=crypto,hardware,embedded");
        assert_eq!(features, vec!["crypto", "hardware", "embedded"]);
        
        let features = preprocessor.extract_features("no_std");
        assert!(features.is_empty());
        
        let features = preprocessor.extract_features("features=single");
        assert_eq!(features, vec!["single"]);
    }

    #[test]
    fn test_platform_extraction() {
        let preprocessor = RustExamplesPreprocessor::new();
        
        assert_eq!(preprocessor.extract_platform("hardware=stm32f4"), Some("stm32f4".to_string()));
        assert_eq!(preprocessor.extract_platform("hardware=esp32"), Some("esp32".to_string()));
        assert_eq!(preprocessor.extract_platform("no_std"), None);
    }

    #[test]
    fn test_algorithm_extraction() {
        let preprocessor = RustExamplesPreprocessor::new();
        
        assert_eq!(preprocessor.extract_algorithm("crypto=aes"), Some("aes".to_string()));
        assert_eq!(preprocessor.extract_algorithm("crypto=sha256"), Some("sha256".to_string()));
        assert_eq!(preprocessor.extract_algorithm("no_std"), None);
    }

    #[test]
    fn test_snippet_reason_extraction() {
        let preprocessor = RustExamplesPreprocessor::new();
        
        assert_eq!(preprocessor.extract_snippet_reason("snippet=incomplete"), Some("incomplete".to_string()));
        assert_eq!(preprocessor.extract_snippet_reason("snippet=pseudocode"), Some("pseudocode".to_string()));
        assert_eq!(preprocessor.extract_snippet_reason("no_std"), None);
    }

    #[test]
    fn test_validation_decision() {
        let preprocessor = RustExamplesPreprocessor::new();
        
        // Should validate standard examples
        let context = ExampleContext::Std { features: vec![] };
        assert!(preprocessor.should_validate_example(&context, ""));
        
        // Should not validate snippets
        let context = ExampleContext::Snippet { reason: "incomplete".to_string() };
        assert!(!preprocessor.should_validate_example(&context, ""));
        
        // Should not validate no_compile examples
        let context = ExampleContext::Std { features: vec![] };
        assert!(!preprocessor.should_validate_example(&context, "no_compile"));
        assert!(!preprocessor.should_validate_example(&context, "no_run"));
    }

    #[test]
    fn test_runnable_decision() {
        let preprocessor = RustExamplesPreprocessor::new();
        
        // Standard examples should be runnable
        let context = ExampleContext::Std { features: vec![] };
        assert!(preprocessor.is_runnable(&context, ""));
        
        // Snippets should not be runnable
        let context = ExampleContext::Snippet { reason: "incomplete".to_string() };
        assert!(!preprocessor.is_runnable(&context, ""));
        
        // no_run examples should not be runnable
        let context = ExampleContext::Std { features: vec![] };
        assert!(!preprocessor.is_runnable(&context, "no_run"));
        
        // Embedded examples should not be runnable (for now)
        let context = ExampleContext::NoStd { target: "thumbv7em-none-eabihf".to_string(), features: vec![] };
        assert!(!preprocessor.is_runnable(&context, ""));
    }

    #[test]
    fn test_status_indicator_creation() {
        let preprocessor = RustExamplesPreprocessor::new();
        
        // Test successful compilation
        let success_result = TestResult {
            example_id: "test".to_string(),
            success: true,
            compilation_time: std::time::Duration::from_millis(1500),
            target: "x86_64".to_string(),
            error: None,
            warnings: vec![],
        };
        
        let indicator = preprocessor.create_status_indicator(&success_result);
        assert!(indicator.contains("compilation-success"));
        assert!(indicator.contains("✓ Compiles"));
        assert!(indicator.contains("1.50s"));
        
        // Test failed compilation
        let error_result = TestResult {
            example_id: "test".to_string(),
            success: false,
            compilation_time: std::time::Duration::from_millis(500),
            target: "x86_64".to_string(),
            error: Some(CompilationError::SyntaxError { 
                message: "missing semicolon".to_string(), 
                line: 5 
            }),
            warnings: vec![],
        };
        
        let indicator = preprocessor.create_status_indicator(&error_result);
        assert!(indicator.contains("compilation-failed"));
        assert!(indicator.contains("✗ Compilation Error"));
        assert!(indicator.contains("0.50s"));
        assert!(indicator.contains("title="));
    }

    #[test]
    fn test_enhanced_annotations_creation() {
        let preprocessor = RustExamplesPreprocessor::new();
        
        // Test NoStd context
        let context = ExampleContext::NoStd { 
            target: "thumbv7em-none-eabihf".to_string(), 
            features: vec!["crypto".to_string()] 
        };
        let annotations = preprocessor.create_enhanced_annotations("", &context);
        assert!(annotations.contains("no_std"));
        assert!(annotations.contains("target=thumbv7em-none-eabihf"));
        
        // Test Hardware context
        let context = ExampleContext::Hardware { 
            platform: "stm32f4".to_string(), 
            features: vec![] 
        };
        let annotations = preprocessor.create_enhanced_annotations("existing", &context);
        assert!(annotations.contains("existing"));
        assert!(annotations.contains("hardware=stm32f4"));
        
        // Test Crypto context
        let context = ExampleContext::Crypto { 
            algorithm: "aes".to_string(), 
            features: vec![] 
        };
        let annotations = preprocessor.create_enhanced_annotations("", &context);
        assert!(annotations.contains("crypto=aes"));
        
        // Test Snippet context
        let context = ExampleContext::Snippet { 
            reason: "incomplete".to_string() 
        };
        let annotations = preprocessor.create_enhanced_annotations("", &context);
        assert!(annotations.contains("snippet=incomplete"));
    }

    #[test]
    fn test_playground_config_creation() {
        let preprocessor = RustExamplesPreprocessor::new();
        
        // Test Std context with features
        let context = ExampleContext::Std { 
            features: vec!["tokio".to_string(), "serde".to_string()] 
        };
        let config = preprocessor.create_playground_config(&context);
        assert!(config.contains("playground-config"));
        assert!(config.contains("\"edition\": \"2021\""));
        assert!(config.contains("\"tokio\""));
        assert!(config.contains("\"serde\""));
        
        // Test Std context without features
        let context = ExampleContext::Std { features: vec![] };
        let config = preprocessor.create_playground_config(&context);
        assert!(config.contains("playground-config"));
        assert!(config.contains("\"edition\": \"2021\""));
        
        // Test non-Std context (should return empty)
        let context = ExampleContext::NoStd { 
            target: "thumbv7em-none-eabihf".to_string(), 
            features: vec![] 
        };
        let config = preprocessor.create_playground_config(&context);
        assert!(config.is_empty());
    }

    #[test]
    fn test_content_processing() {
        let mut preprocessor = RustExamplesPreprocessor::new();
        
        let input_content = r#"
# Test Chapter

Here's a simple Rust example:

```rust
fn main() {
    println!("Hello, world!");
}
```

And here's a no_std example:

```rust,no_std
#![no_std]
#![no_main]

use panic_halt as _;
```

And a snippet:

```rust,snippet
// This is incomplete
let x = 
```
"#;

        let processed = preprocessor.process_content(input_content, Some("test.md")).unwrap();
        
        // Should contain enhanced code blocks
        assert!(processed.contains("```rust"));
        assert!(processed.contains("no_std"));
        assert!(processed.contains("snippet"));
        
        // Should preserve non-code content
        assert!(processed.contains("# Test Chapter"));
        assert!(processed.contains("Here's a simple Rust example:"));
    }

    #[test]
    fn test_book_processing() {
        let mut preprocessor = RustExamplesPreprocessor::new();
        
        let content = r#"
```rust
fn main() {
    println!("Hello!");
}
```
"#;
        
        let mut book = Book {
            sections: vec![create_test_book_item(content)],
            non_exhaustive: None,
        };
        
        preprocessor.process_book(&mut book).unwrap();
        
        // Should have processed the content
        assert!(book.sections[0].content.contains("```rust"));
    }

    #[test]
    fn test_test_config_creation() {
        let mut preprocessor = RustExamplesPreprocessor::new();
        preprocessor.config.embedded_target = Some("thumbv7em-none-eabihf".to_string());
        preprocessor.config.feature_gates = Some(vec!["crypto".to_string(), "hardware".to_string()]);
        preprocessor.config.no_std_mode = Some(true);
        preprocessor.config.compilation_timeout = Some(45);
        
        let config = preprocessor.create_test_config();
        
        assert!(config.targets.contains(&"thumbv7em-none-eabihf".to_string()));
        assert!(config.features.contains(&"crypto".to_string()));
        assert!(config.features.contains(&"hardware".to_string()));
        assert_eq!(config.no_std, true);
        assert_eq!(config.embedded_mode, true);
        assert_eq!(config.timeout_seconds, 45);
    }

    #[test]
    fn test_html_escape() {
        use crate::preprocessor::html_escape;
        
        let input = r#"<script>alert("xss")</script> & "quotes" & 'apostrophes'"#;
        let escaped = html_escape::encode_text(input);
        
        assert!(escaped.contains("&lt;script&gt;"));
        assert!(escaped.contains("&amp;"));
        assert!(escaped.contains("&quot;"));
        assert!(escaped.contains("&#x27;"));
        assert!(!escaped.contains("<script>"));
        assert!(!escaped.contains("alert(\"xss\")"));
    }

    #[test]
    fn test_simple_hash() {
        use crate::preprocessor::simple_hash;
        
        let hash1 = simple_hash("test content");
        let hash2 = simple_hash("test content");
        let hash3 = simple_hash("different content");
        
        // Same content should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different content should produce different hash
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_complex_annotation_parsing() {
        let preprocessor = RustExamplesPreprocessor::new();
        
        // Test complex annotation with multiple features
        let context = preprocessor.parse_annotations("no_std,features=crypto,hardware,embedded,target=custom");
        match context {
            ExampleContext::NoStd { target, features } => {
                assert_eq!(target, "thumbv7em-none-eabihf"); // Should use default
                assert_eq!(features, vec!["crypto", "hardware", "embedded", "target=custom"]);
            }
            _ => panic!("Expected NoStd context"),
        }
        
        // Test hardware with multiple features
        let context = preprocessor.parse_annotations("hardware=esp32,features=wifi,bluetooth");
        match context {
            ExampleContext::Hardware { platform, features } => {
                assert_eq!(platform, "esp32");
                assert_eq!(features, vec!["wifi", "bluetooth"]);
            }
            _ => panic!("Expected Hardware context"),
        }
    }

    #[test]
    fn test_edge_cases() {
        let preprocessor = RustExamplesPreprocessor::new();
        
        // Empty annotations
        let context = preprocessor.parse_annotations("");
        assert!(matches!(context, ExampleContext::Std { .. }));
        
        // Only commas
        let context = preprocessor.parse_annotations(",,,");
        assert!(matches!(context, ExampleContext::Std { .. }));
        
        // Malformed features
        let features = preprocessor.extract_features("features=");
        assert_eq!(features, vec![""]);
        
        // Multiple equals signs
        let platform = preprocessor.extract_platform("hardware=stm32=f4");
        assert_eq!(platform, Some("stm32=f4".to_string()));
    }

    #[test]
    fn test_validation_with_disabled_config() {
        let mut preprocessor = RustExamplesPreprocessor::new();
        preprocessor.config.validation_enabled = Some(false);
        
        let context = ExampleContext::Std { features: vec![] };
        assert!(!preprocessor.should_validate_example(&context, ""));
        
        let context = ExampleContext::NoStd { 
            target: "thumbv7em-none-eabihf".to_string(), 
            features: vec![] 
        };
        assert!(!preprocessor.should_validate_example(&context, ""));
    }
}