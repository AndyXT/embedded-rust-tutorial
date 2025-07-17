//! mdBook preprocessor for handling embedded Rust examples
//! 
//! This module provides a custom mdBook preprocessor that enhances code blocks
//! with compilation status indicators and embedded-specific handling.

use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::process;
use serde::{Deserialize, Serialize};
use serde_json;
use regex::Regex;

use crate::{RustExampleTester, TestConfig, CodeExtractor, ExampleContext, TestResult};

/// Configuration for the rust-examples preprocessor
#[derive(Debug, Clone, Deserialize)]
pub struct PreprocessorConfig {
    #[serde(rename = "embedded-target")]
    pub embedded_target: Option<String>,
    #[serde(rename = "no-std-mode")]
    pub no_std_mode: Option<bool>,
    #[serde(rename = "feature-gates")]
    pub feature_gates: Option<Vec<String>>,
    #[serde(rename = "validation-enabled")]
    pub validation_enabled: Option<bool>,
    #[serde(rename = "compilation-timeout")]
    pub compilation_timeout: Option<u64>,
    #[serde(rename = "cache-enabled")]
    pub cache_enabled: Option<bool>,
}

impl Default for PreprocessorConfig {
    fn default() -> Self {
        Self {
            embedded_target: Some("thumbv7em-none-eabihf".to_string()),
            no_std_mode: Some(true),
            feature_gates: Some(vec!["crypto".to_string(), "hardware".to_string(), "embedded".to_string()]),
            validation_enabled: Some(true),
            compilation_timeout: Some(30),
            cache_enabled: Some(true),
        }
    }
}

/// mdBook book structure
#[derive(Debug, Deserialize, Serialize)]
pub struct Book {
    pub sections: Vec<BookItem>,
    #[serde(rename = "__non_exhaustive")]
    pub non_exhaustive: Option<()>,
}

/// mdBook book item (chapter or separator)
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "Chapter")]
pub struct BookItem {
    pub name: String,
    pub content: String,
    pub number: Option<Vec<u32>>,
    pub sub_items: Vec<BookItem>,
    pub path: Option<String>,
    pub source_path: Option<String>,
    pub parent_names: Vec<String>,
}

/// mdBook preprocessor context
#[derive(Debug, Deserialize)]
pub struct PreprocessorContext {
    pub root: String,
    pub config: HashMap<String, serde_json::Value>,
    pub renderer: String,
}

/// Main preprocessor struct
pub struct RustExamplesPreprocessor {
    config: PreprocessorConfig,
    tester: Option<RustExampleTester>,
    extractor: CodeExtractor,
    compilation_cache: HashMap<String, TestResult>,
}

impl RustExamplesPreprocessor {
    /// Create a new preprocessor instance
    pub fn new() -> Self {
        Self {
            config: PreprocessorConfig::default(),
            tester: None,
            extractor: CodeExtractor::new(),
            compilation_cache: HashMap::new(),
        }
    }

    /// Run the preprocessor
    pub fn run(&mut self) -> io::Result<()> {
        let mut stdin = io::stdin();
        let mut input = String::new();
        stdin.read_to_string(&mut input)?;

        // Parse the input as JSON
        let input_json: serde_json::Value = serde_json::from_str(&input)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Check if this is a supports query
        if let Some(renderer) = input_json.get(0).and_then(|v| v.as_str()) {
            if renderer == "supports" {
                // Return supported renderers
                let supports = serde_json::json!(["html"]);
                println!("{}", supports);
                return Ok(());
            }
        }

        // Parse context and book
        let context: PreprocessorContext = serde_json::from_value(input_json[0].clone())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let mut book: Book = serde_json::from_value(input_json[1].clone())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Load configuration
        self.load_config(&context)?;

        // Initialize tester if validation is enabled
        if self.config.validation_enabled.unwrap_or(true) {
            let test_config = self.create_test_config();
            match RustExampleTester::new(test_config) {
                Ok(tester) => self.tester = Some(tester),
                Err(e) => {
                    eprintln!("Warning: Failed to initialize example tester: {}", e);
                    // Continue without validation
                }
            }
        }

        // Process the book
        self.process_book(&mut book)?;

        // Output the modified book
        let output = serde_json::to_string(&book)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        println!("{}", output);
        Ok(())
    }

    /// Load configuration from context
    fn load_config(&mut self, context: &PreprocessorContext) -> io::Result<()> {
        if let Some(preprocessor_config) = context.config.get("preprocessor") {
            if let Some(rust_examples_config) = preprocessor_config.get("rust-examples") {
                if let Some(config) = rust_examples_config.get("config") {
                    self.config = serde_json::from_value(config.clone())
                        .unwrap_or_else(|_| PreprocessorConfig::default());
                }
            }
        }
        Ok(())
    }

    /// Create test configuration from preprocessor config
    fn create_test_config(&self) -> TestConfig {
        let mut config = TestConfig::default();
        
        if let Some(target) = &self.config.embedded_target {
            config.targets.push(target.clone());
        }
        
        if let Some(features) = &self.config.feature_gates {
            config.features.extend(features.clone());
        }
        
        config.no_std = self.config.no_std_mode.unwrap_or(true);
        config.embedded_mode = true;
        config.timeout_seconds = self.config.compilation_timeout.unwrap_or(30);
        
        config
    }

    /// Process the entire book
    fn process_book(&mut self, book: &mut Book) -> io::Result<()> {
        for section in &mut book.sections {
            self.process_book_item(section)?;
        }
        Ok(())
    }

    /// Process a single book item
    fn process_book_item(&mut self, item: &mut BookItem) -> io::Result<()> {
        // Process content
        item.content = self.process_content(&item.content, item.path.as_deref())?;
        
        // Process sub-items recursively
        for sub_item in &mut item.sub_items {
            self.process_book_item(sub_item)?;
        }
        
        Ok(())
    }

    /// Process markdown content
    fn process_content(&mut self, content: &str, file_path: Option<&str>) -> io::Result<String> {
        let code_block_regex = Regex::new(r"```rust(?:,([^`\n]*))?\n((?:[^`]|`[^`]|``[^`])*)\n```")
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let mut processed_content = content.to_string();
        let mut offset = 0i32;

        for captures in code_block_regex.captures_iter(content) {
            let full_match = captures.get(0).unwrap();
            let annotations = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            let code = captures.get(2).unwrap().as_str();

            let enhanced_block = self.enhance_code_block(code, annotations, file_path)?;
            
            let start = (full_match.start() as i32 + offset) as usize;
            let end = (full_match.end() as i32 + offset) as usize;
            
            processed_content.replace_range(start..end, &enhanced_block);
            offset += enhanced_block.len() as i32 - full_match.len() as i32;
        }

        Ok(processed_content)
    }

    /// Enhance a code block with compilation status and embedded features
    fn enhance_code_block(&mut self, code: &str, annotations: &str, file_path: Option<&str>) -> io::Result<String> {
        let context = self.parse_annotations(annotations);
        let should_validate = self.should_validate_example(&context, annotations);
        
        let mut enhanced_block = String::new();
        
        // Add compilation status indicator
        if should_validate {
            if let Some(ref tester) = self.tester {
                let cache_key = simple_hash(code);
                
                let test_result = if self.config.cache_enabled.unwrap_or(true) {
                    self.compilation_cache.get(&cache_key).cloned()
                        .unwrap_or_else(|| {
                            let result = tester.test_example(code, context.clone());
                            self.compilation_cache.insert(cache_key, result.clone());
                            result
                        })
                } else {
                    tester.test_example(code, context.clone())
                };

                enhanced_block.push_str(&self.create_status_indicator(&test_result));
            }
        }

        // Create enhanced code block
        enhanced_block.push_str(&format!("```rust{}\n", self.create_enhanced_annotations(annotations, &context)));
        enhanced_block.push_str(code);
        enhanced_block.push_str("\n```");

        // Add playground configuration if runnable
        if self.is_runnable(&context, annotations) {
            enhanced_block.push_str(&self.create_playground_config(&context));
        }

        Ok(enhanced_block)
    }

    /// Parse annotations from code block
    fn parse_annotations(&self, annotations: &str) -> ExampleContext {
        if annotations.contains("no_std") || annotations.contains("no-std") {
            ExampleContext::NoStd {
                target: self.config.embedded_target.clone().unwrap_or_else(|| "thumbv7em-none-eabihf".to_string()),
                features: self.extract_features(annotations),
            }
        } else if annotations.contains("hardware") {
            ExampleContext::Hardware {
                platform: self.extract_platform(annotations).unwrap_or_else(|| "generic".to_string()),
                features: self.extract_features(annotations),
            }
        } else if annotations.contains("crypto") {
            ExampleContext::Crypto {
                algorithm: self.extract_algorithm(annotations).unwrap_or_else(|| "generic".to_string()),
                features: self.extract_features(annotations),
            }
        } else if annotations.contains("snippet") {
            ExampleContext::Snippet {
                reason: self.extract_snippet_reason(annotations).unwrap_or_else(|| "incomplete".to_string()),
            }
        } else {
            ExampleContext::Std {
                features: self.extract_features(annotations),
            }
        }
    }

    /// Extract features from annotations
    fn extract_features(&self, annotations: &str) -> Vec<String> {
        let features_regex = Regex::new(r"features=([^,\s]+)").unwrap();
        if let Some(captures) = features_regex.captures(annotations) {
            captures.get(1).unwrap().as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Extract platform from hardware annotations
    fn extract_platform(&self, annotations: &str) -> Option<String> {
        let platform_regex = Regex::new(r"hardware=([^,\s]+)").unwrap();
        platform_regex.captures(annotations)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extract algorithm from crypto annotations
    fn extract_algorithm(&self, annotations: &str) -> Option<String> {
        let algo_regex = Regex::new(r"crypto=([^,\s]+)").unwrap();
        algo_regex.captures(annotations)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extract snippet reason
    fn extract_snippet_reason(&self, annotations: &str) -> Option<String> {
        let reason_regex = Regex::new(r"snippet=([^,\s]+)").unwrap();
        reason_regex.captures(annotations)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Check if example should be validated
    fn should_validate_example(&self, context: &ExampleContext, annotations: &str) -> bool {
        if !self.config.validation_enabled.unwrap_or(true) {
            return false;
        }

        // Don't validate snippets
        if matches!(context, ExampleContext::Snippet { .. }) {
            return false;
        }

        // Don't validate if explicitly marked as no-compile
        if annotations.contains("no_run") || annotations.contains("no_compile") {
            return false;
        }

        true
    }

    /// Check if example is runnable in playground
    fn is_runnable(&self, context: &ExampleContext, annotations: &str) -> bool {
        // Snippets are not runnable
        if matches!(context, ExampleContext::Snippet { .. }) {
            return false;
        }

        // Check for explicit no_run annotation
        if annotations.contains("no_run") {
            return false;
        }

        // Standard library examples are runnable
        if matches!(context, ExampleContext::Std { .. }) {
            return true;
        }

        // Embedded examples might be runnable in simulator
        false
    }

    /// Create compilation status indicator HTML
    fn create_status_indicator(&self, result: &TestResult) -> String {
        let (status_class, status_text, status_icon) = if result.success {
            ("compilation-success", "✓ Compiles", "✓")
        } else {
            ("compilation-failed", "✗ Compilation Error", "✗")
        };

        let error_details = if let Some(ref error) = result.error {
            format!(" title=\"{}\"", html_escape::encode_text(&format!("{:?}", error)))
        } else {
            String::new()
        };

        format!(
            r#"<div class="compilation-status {}"{}>
                <span class="status-icon">{}</span>
                <span class="status-text">{}</span>
                <span class="compilation-time">{:.2}s</span>
            </div>
            "#,
            status_class,
            error_details,
            status_icon,
            status_text,
            result.compilation_time.as_secs_f64()
        )
    }

    /// Create enhanced annotations for code block
    fn create_enhanced_annotations(&self, original: &str, context: &ExampleContext) -> String {
        let mut annotations = if original.is_empty() {
            String::new()
        } else {
            format!(",{}", original)
        };

        // Add context-specific annotations
        match context {
            ExampleContext::NoStd { target, .. } => {
                if !annotations.contains("no_std") {
                    annotations.push_str(",no_std");
                }
                annotations.push_str(&format!(",target={}", target));
            }
            ExampleContext::Hardware { platform, .. } => {
                annotations.push_str(&format!(",hardware={}", platform));
            }
            ExampleContext::Crypto { algorithm, .. } => {
                annotations.push_str(&format!(",crypto={}", algorithm));
            }
            ExampleContext::Snippet { reason } => {
                annotations.push_str(&format!(",snippet={}", reason));
            }
            _ => {}
        }

        annotations
    }

    /// Create playground configuration
    fn create_playground_config(&self, context: &ExampleContext) -> String {
        match context {
            ExampleContext::Std { features } => {
                let features_str = if features.is_empty() {
                    String::new()
                } else {
                    format!(r#"features = [{}]"#, features.iter().map(|f| format!("\"{}\"", f)).collect::<Vec<_>>().join(", "))
                };
                
                format!(
                    r#"
<!-- playground-config -->
<script type="application/json" class="playground-config">
{{
    "edition": "2021",
    "mode": "debug",
    {}
}}
</script>
"#,
                    features_str
                )
            }
            _ => String::new()
        }
    }
}

// Simple hash function for cache keys
pub fn simple_hash(input: &str) -> String {
    // Simple hash for demo - in production, use a proper hash function
    format!("{:x}", input.len() * 31 + input.chars().map(|c| c as u32).sum::<u32>())
}

// Simple HTML escape for error messages
mod html_escape {
    pub fn encode_text(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
}

#[cfg(test)]
pub mod tests;