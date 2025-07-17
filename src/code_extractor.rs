//! Code example extraction from markdown files

#[cfg(feature = "std")]
use std::path::{Path, PathBuf};
#[cfg(feature = "std")]
use std::fs;
#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use heapless::FnvIndexMap as HashMap;

// Import standard types conditionally
#[cfg(feature = "std")]
use std::string::String;
#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use core::option::Option::{self, Some, None};
use core::result::Result::{self, Ok};
use core::default::Default;

/// Context information for a code example
#[derive(Debug, Clone, PartialEq)]
pub enum ExampleContext {
    /// Standard library example
    Std { 
        features: Vec<String> 
    },
    /// No-std embedded example
    NoStd { 
        target: String, 
        features: Vec<String> 
    },
    /// Hardware-specific example
    Hardware { 
        platform: String, 
        features: Vec<String> 
    },
    /// Cryptography-focused example
    Crypto { 
        algorithm: Option<String>, 
        features: Vec<String> 
    },
    /// Code snippet that shouldn't compile standalone
    Snippet { 
        reason: String 
    },
}

impl ExampleContext {
    /// Check if this context should be compiled
    pub fn should_compile(&self) -> bool {
        !matches!(self, ExampleContext::Snippet { .. })
    }
    
    /// Get the features required for this context
    pub fn features(&self) -> &[String] {
        match self {
            ExampleContext::Std { features } => features,
            ExampleContext::NoStd { features, .. } => features,
            ExampleContext::Hardware { features, .. } => features,
            ExampleContext::Crypto { features, .. } => features,
            ExampleContext::Snippet { .. } => &[],
        }
    }
    
    /// Check if this is a no_std context
    pub fn is_no_std(&self) -> bool {
        matches!(self, 
            ExampleContext::NoStd { .. } | 
            ExampleContext::Hardware { .. } |
            ExampleContext::Crypto { .. }
        )
    }
}

/// A single code example extracted from markdown
#[derive(Debug, Clone)]
pub struct CodeExample {
    /// Unique identifier for this example
    pub id: String,
    /// Source file path
    pub source_file: PathBuf,
    /// Line number in source file
    pub line_number: usize,
    /// The actual code content
    pub code: String,
    /// Context information
    pub context: ExampleContext,
    /// Raw annotations from the markdown
    pub annotations: HashMap<String, Option<String>>,
    /// Inferred dependencies
    pub dependencies: Vec<String>,
}

impl CodeExample {
    /// Create a new code example
    pub fn new(
        id: String,
        source_file: PathBuf,
        line_number: usize,
        code: String,
        context: ExampleContext,
    ) -> Self {
        let dependencies = Self::infer_dependencies(&code);
        
        Self {
            id,
            source_file,
            line_number,
            code,
            context,
            annotations: HashMap::new(),
            dependencies,
        }
    }
    
    /// Infer dependencies from code content
    fn infer_dependencies(code: &str) -> Vec<String> {
        let mut deps = Vec::new();
        
        // Common embedded dependencies
        if code.contains("heapless::") {
            deps.push("heapless".to_string());
        }
        if code.contains("cortex_m") {
            deps.push("cortex-m".to_string());
        }
        if code.contains("cortex_m_rt") {
            deps.push("cortex-m-rt".to_string());
        }
        if code.contains("panic_halt") {
            deps.push("panic-halt".to_string());
        }
        if code.contains("zeroize") || code.contains("Zeroize") {
            deps.push("zeroize".to_string());
        }
        
        // Crypto dependencies
        if code.contains("aes") || code.contains("Aes") {
            deps.push("aes".to_string());
        }
        if code.contains("sha2") || code.contains("Sha256") {
            deps.push("sha2".to_string());
        }
        if code.contains("rand") {
            deps.push("rand".to_string());
        }
        
        deps.sort();
        deps.dedup();
        deps
    }
    
    /// Check if this example should be compiled
    pub fn should_compile(&self) -> bool {
        self.context.should_compile()
    }
    
    /// Get a summary description of this example
    pub fn summary(&self) -> String {
        format!(
            "Example {} from {}:{} ({:?})",
            self.id,
            self.source_file.display(),
            self.line_number,
            self.context
        )
    }
}

/// Extracts code examples from markdown files
pub struct CodeExtractor {
}

impl CodeExtractor {
    /// Create a new code extractor
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {})
    }
    
    /// Extract all code examples from a markdown file
    pub fn extract_from_file(&self, file_path: &Path) -> Result<Vec<CodeExample>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        self.extract_from_content(&content, file_path)
    }
    
    /// Extract code examples from markdown content
    pub fn extract_from_content(
        &self, 
        content: &str, 
        source_file: &Path
    ) -> Result<Vec<CodeExample>, Box<dyn std::error::Error>> {
        let mut examples = Vec::new();
        let mut example_counter = 0;
        let lines: Vec<&str> = content.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            
            // Look for rust code block start
            if line.starts_with("```rust") {
                example_counter += 1;
                
                // Extract annotations from the first line
                let annotations_str = if line.len() > 7 && line.chars().nth(7) == Some(',') {
                    &line[8..] // Skip "```rust,"
                } else {
                    ""
                };
                
                // Find the end of the code block
                let mut code_lines = Vec::new();
                let mut j = i + 1;
                while j < lines.len() && !lines[j].starts_with("```") {
                    code_lines.push(lines[j]);
                    j += 1;
                }
                
                if j < lines.len() {
                    let code = code_lines.join("\n");
                    let line_number = i + 1; // 1-based line numbering
                    
                    // Parse annotations
                    let annotations = self.parse_annotations(annotations_str);
                    
                    // Infer context from annotations and code content
                    let context = self.infer_context(&annotations, &code);
                    
                    // Create unique ID
                    let id = format!(
                        "{}_{}", 
                        source_file.file_stem()
                            .unwrap_or_default()
                            .to_string_lossy(), 
                        example_counter
                    );
                    
                    let mut example = CodeExample::new(
                        id,
                        source_file.to_path_buf(),
                        line_number,
                        code,
                        context,
                    );
                    
                    example.annotations = annotations;
                    examples.push(example);
                    
                    i = j; // Skip to after the closing ```
                }
            }
            i += 1;
        }
        
        Ok(examples)
    }
    
    /// Parse annotations from the code block header
    /// Supports formats like: no_std, target=thumbv7em-none-eabihf, features=crypto,hardware
    fn parse_annotations(&self, annotations_str: &str) -> HashMap<String, Option<String>> {
        let mut annotations = HashMap::new();
        
        if annotations_str.is_empty() {
            return annotations;
        }
        
        // Enhanced parsing: handle quoted values and nested comma-separated lists
        let mut current_key = String::new();
        let mut current_value = String::new();
        let mut in_value = false;
        let mut in_quotes = false;
        let mut chars = annotations_str.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '=' if !in_quotes => {
                    if !current_key.is_empty() {
                        in_value = true;
                        current_value.clear();
                    }
                }
                '"' => {
                    in_quotes = !in_quotes;
                }
                ',' if !in_quotes => {
                    // End of current annotation
                    let key = current_key.trim().to_string();
                    if !key.is_empty() {
                        if in_value && !current_value.is_empty() {
                            annotations.insert(key, Some(current_value.trim().to_string()));
                        } else if !in_value {
                            annotations.insert(key, None);
                        }
                    }
                    current_key.clear();
                    current_value.clear();
                    in_value = false;
                }
                _ => {
                    if in_value {
                        current_value.push(ch);
                    } else {
                        current_key.push(ch);
                    }
                }
            }
        }
        
        // Handle the last annotation
        let key = current_key.trim().to_string();
        if !key.is_empty() {
            if in_value && !current_value.is_empty() {
                annotations.insert(key, Some(current_value.trim().to_string()));
            } else if !in_value {
                annotations.insert(key, None);
            }
        }
        
        annotations
    }
    
    /// Infer the context of a code example from annotations and content
    fn infer_context(
        &self, 
        annotations: &HashMap<String, Option<String>>, 
        code: &str
    ) -> ExampleContext {
        // Check for explicit snippet annotation first
        if annotations.contains_key("snippet") {
            let reason = annotations.get("snippet")
                .and_then(|v| v.as_ref())
                .unwrap_or(&"incomplete code".to_string())
                .clone();
            return ExampleContext::Snippet { reason };
        }
        
        // Check for explicit hardware annotation
        if let Some(platform) = annotations.get("hardware").and_then(|v| v.as_ref()) {
            let features = self.extract_features(annotations);
            return ExampleContext::Hardware { 
                platform: platform.clone(), 
                features 
            };
        }
        
        // Check for explicit no_std annotation
        if annotations.contains_key("no_std") {
            let target = annotations.get("target")
                .and_then(|v| v.as_ref())
                .unwrap_or(&"thumbv7em-none-eabihf".to_string())
                .clone();
            let features = self.extract_features(annotations);
            return ExampleContext::NoStd { target, features };
        }
        
        // Check for explicit crypto annotation
        if annotations.contains_key("crypto") {
            let algorithm = annotations.get("algorithm")
                .and_then(|v| v.as_ref())
                .cloned()
                .or_else(|| self.detect_crypto_algorithm(code));
            let features = self.extract_features(annotations);
            return ExampleContext::Crypto { algorithm, features };
        }
        
        // Infer from code content analysis
        
        // Check for no_std patterns first (more specific than hardware)
        if self.is_no_std_code(code) {
            let target = annotations.get("target")
                .and_then(|v| v.as_ref())
                .unwrap_or(&"thumbv7em-none-eabihf".to_string())
                .clone();
            let features = self.extract_features(annotations);
            return ExampleContext::NoStd { target, features };
        }
        
        // Check for hardware-specific patterns
        if self.is_hardware_code(code) {
            let platform = self.detect_hardware_platform(code)
                .unwrap_or_else(|| "generic".to_string());
            let features = self.extract_features(annotations);
            return ExampleContext::Hardware { platform, features };
        }
        
        // Check for crypto patterns
        if self.is_crypto_code(code) {
            let algorithm = self.detect_crypto_algorithm(code);
            let features = self.extract_features(annotations);
            return ExampleContext::Crypto { algorithm, features };
        }
        
        // Check if code looks like a snippet (incomplete/partial code)
        if self.is_snippet_code(code) {
            return ExampleContext::Snippet { 
                reason: "appears to be incomplete code".to_string() 
            };
        }
        
        // Default to standard library
        let features = self.extract_features(annotations);
        ExampleContext::Std { features }
    }
    
    /// Extract features from annotations
    fn extract_features(&self, annotations: &HashMap<String, Option<String>>) -> Vec<String> {
        let mut features = Vec::new();
        
        if let Some(features_str) = annotations.get("features").and_then(|v| v.as_ref()) {
            features.extend(
                features_str.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            );
        }
        
        // Add implicit features based on other annotations
        if annotations.contains_key("crypto") {
            features.push("crypto".to_string());
        }
        if annotations.contains_key("hardware") {
            features.push("hardware".to_string());
        }
        
        features.sort();
        features.dedup();
        features
    }
    
    /// Check if code appears to be cryptography-related
    fn is_crypto_code(&self, code: &str) -> bool {
        // More specific crypto patterns to avoid false positives
        let crypto_patterns = [
            // Crypto libraries and types
            "zeroize", "Zeroize", "aes::", "Aes", "sha2::", "Sha256", "Sha512",
            "chacha20", "ChaCha20", "ed25519", "Ed25519", "rsa::", "RSA",
            "hmac::", "HMAC", "pbkdf2", "PBKDF2",
            
            // Crypto operations
            "encrypt", "decrypt", "cipher", "hash_password", "verify_password",
            "sign_message", "verify_signature", "derive_key", "generate_key",
            
            // Security-specific patterns
            "SecureKey", "CryptoError", "constant_time", "timing_safe",
            "secure_random", "crypto_random",
            
            // Embedded crypto patterns
            "crypto_hardware", "hardware_rng", "secure_element",
            
            // Key management (more specific than just "key")
            "private_key", "public_key", "secret_key", "master_key",
            "encryption_key", "signing_key", "key_pair", "KeyPair",
            "key.zeroize", "key_material", "derive_key"
        ];
        
        crypto_patterns.iter().any(|pattern| code.contains(pattern))
    }
    
    /// Detect specific cryptographic algorithm from code
    fn detect_crypto_algorithm(&self, code: &str) -> Option<String> {
        if code.contains("aes") || code.contains("Aes") {
            Some("AES".to_string())
        } else if code.contains("sha") || code.contains("Sha") {
            Some("SHA".to_string())
        } else if code.contains("chacha") || code.contains("ChaCha") {
            Some("ChaCha20".to_string())
        } else if code.contains("ecdh") || code.contains("ECDH") {
            Some("ECDH".to_string())
        } else if code.contains("rsa") || code.contains("RSA") {
            Some("RSA".to_string())
        } else if code.contains("ed25519") || code.contains("Ed25519") {
            Some("Ed25519".to_string())
        } else {
            None
        }
    }
    
    /// Check if code appears to be hardware-specific
    fn is_hardware_code(&self, code: &str) -> bool {
        let hardware_keywords = [
            "stm32", "STM32", "cortex_m", "cortex-m", "embedded_hal", "hal::",
            "GPIO", "gpio::", "SPI", "spi::", "I2C", "i2c::", "UART", "uart::",
            "Timer", "timer::", "PWM", "pwm::", "ADC", "adc::", "DMA", "dma::",
            "interrupt", "Interrupt", "NVIC", "nvic", "pac::", "PAC",
            "rcc::", "RCC", "clocks", "Clocks", "pins", "Pins"
        ];
        
        hardware_keywords.iter().any(|keyword| code.contains(keyword))
    }
    
    /// Detect hardware platform from code content
    fn detect_hardware_platform(&self, code: &str) -> Option<String> {
        if code.contains("stm32f4") || code.contains("STM32F4") {
            Some("STM32F4".to_string())
        } else if code.contains("stm32f3") || code.contains("STM32F3") {
            Some("STM32F3".to_string())
        } else if code.contains("stm32l4") || code.contains("STM32L4") {
            Some("STM32L4".to_string())
        } else if code.contains("nrf52") || code.contains("NRF52") {
            Some("nRF52".to_string())
        } else if code.contains("esp32") || code.contains("ESP32") {
            Some("ESP32".to_string())
        } else if code.contains("rp2040") || code.contains("RP2040") {
            Some("RP2040".to_string())
        } else if code.contains("cortex_m") || code.contains("cortex-m") {
            Some("Cortex-M".to_string())
        } else {
            None
        }
    }
    
    /// Check if code appears to be no_std
    fn is_no_std_code(&self, code: &str) -> bool {
        // Explicit no_std indicators
        if code.contains("#![no_std]") || code.contains("#![no_main]") {
            return true;
        }
        
        // Common no_std patterns
        let no_std_indicators = [
            "heapless::", "nb::", "cortex_m", "embedded_hal",
            "panic_halt", "panic_abort", "panic_semihosting",
            "#[no_mangle]", "extern \"C\"", "core::", "alloc::"
        ];
        
        // Check for no_std indicators without std indicators
        let has_no_std_indicators = no_std_indicators.iter().any(|indicator| code.contains(indicator));
        let has_std_indicators = code.contains("std::") || code.contains("println!") || code.contains("main()");
        
        has_no_std_indicators && !has_std_indicators
    }
    
    /// Check if code appears to be a snippet (incomplete/partial code)
    fn is_snippet_code(&self, code: &str) -> bool {
        let code = code.trim();
        
        // Empty or very short code
        if code.is_empty() || code.len() < 10 {
            return true;
        }
        
        // Code that doesn't have main function or clear structure
        let has_main = code.contains("fn main") || code.contains("fn main()");
        let has_entry = code.contains("#[entry]") || code.contains("cortex_m_rt::entry");
        let has_test = code.contains("#[test]");
        let has_function = code.contains("fn ") && !code.starts_with("fn ");
        let has_struct_impl = code.contains("struct ") || code.contains("impl ");
        
        // If it has clear structure, it's not a snippet
        if has_main || has_entry || has_test || has_function || has_struct_impl {
            return false;
        }
        
        // Check for snippet-like patterns
        let snippet_patterns = [
            // Variable declarations without context
            code.starts_with("let ") && !code.contains("fn "),
            // Single expressions
            code.lines().count() <= 3 && !code.contains('{') && !code.contains('}'),
            // Comments only
            code.lines().all(|line| line.trim().starts_with("//") || line.trim().is_empty()),
            // Incomplete function calls
            code.contains("...") || code.contains("// ..."),
        ];
        
        snippet_patterns.iter().any(|&pattern| pattern)
    }
    
    /// Extract examples from multiple files
    pub fn extract_from_directory(
        &self, 
        dir_path: &Path
    ) -> Result<Vec<CodeExample>, Box<dyn std::error::Error>> {
        let mut all_examples = Vec::new();
        
        self.visit_directory(dir_path, &mut all_examples)?;
        
        Ok(all_examples)
    }
    
    /// Recursively visit directory for markdown files
    fn visit_directory(
        &self, 
        dir_path: &Path, 
        examples: &mut Vec<CodeExample>
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.visit_directory(&path, examples)?;
            } else if path.extension().map_or(false, |ext| ext == "md") {
                let file_examples = self.extract_from_file(&path)?;
                examples.extend(file_examples);
            }
        }
        
        Ok(())
    }
}

impl Default for CodeExtractor {
    fn default() -> Self {
        Self::new().expect("Failed to create default CodeExtractor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_code_extractor_creation() {
        let extractor = CodeExtractor::new();
        assert!(extractor.is_ok());
    }
    
    #[test]
    fn test_basic_code_extraction() {
        let extractor = CodeExtractor::new().unwrap();
        let content = r#"
# Test Document

Here's a simple example:

```rust
fn main() {
    println!("Hello, world!");
}
```

And another:

```rust,no_std
#![no_std]
use heapless::Vec;
```
"#;
        
        let examples = extractor.extract_from_content(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(examples.len(), 2);
        
        // First example should be std
        assert!(matches!(examples[0].context, ExampleContext::Std { .. }));
        assert!(examples[0].code.contains("println!"));
        
        // Second example should be no_std
        assert!(matches!(examples[1].context, ExampleContext::NoStd { .. }));
        assert!(examples[1].code.contains("#![no_std]"));
    }
    
    #[test]
    fn test_annotation_parsing() {
        let extractor = CodeExtractor::new().unwrap();
        let annotations = extractor.parse_annotations("no_std,target=thumbv6m-none-eabi,features=crypto");
        
        assert!(annotations.contains_key("no_std"));
        assert_eq!(annotations.get("target"), Some(&Some("thumbv6m-none-eabi".to_string())));
        assert_eq!(annotations.get("features"), Some(&Some("crypto".to_string())));
    }
    
    #[test]
    fn test_crypto_detection() {
        let extractor = CodeExtractor::new().unwrap();
        
        let crypto_code = r#"
use zeroize::Zeroize;
let mut key = [0u8; 32];
key.zeroize();
"#;
        
        assert!(extractor.is_crypto_code(crypto_code));
        
        let normal_code = "fn main() { println!('hello'); }";
        assert!(!extractor.is_crypto_code(normal_code));
    }
    
    #[test]
    fn test_dependency_inference() {
        let code_with_deps = r#"
use heapless::Vec;
use cortex_m_rt::entry;
use zeroize::Zeroize;
"#;
        
        let deps = CodeExample::infer_dependencies(code_with_deps);
        assert!(deps.contains(&"heapless".to_string()));
        assert!(deps.contains(&"cortex-m-rt".to_string()));
        assert!(deps.contains(&"zeroize".to_string()));
    }
    
    #[test]
    fn test_snippet_context() {
        let extractor = CodeExtractor::new().unwrap();
        let content = r#"
```rust,snippet
let key = generate_key();
```
"#;
        
        let examples = extractor.extract_from_content(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(examples.len(), 1);
        assert!(matches!(examples[0].context, ExampleContext::Snippet { .. }));
        assert!(!examples[0].should_compile());
    }
    
    #[test]
    fn test_enhanced_annotation_parsing() {
        let extractor = CodeExtractor::new().unwrap();
        
        // Test complex annotations with quoted values
        let annotations = extractor.parse_annotations("hardware=STM32F4,features=\"crypto,hardware\",target=thumbv7em-none-eabihf");
        assert_eq!(annotations.get("hardware"), Some(&Some("STM32F4".to_string())));
        assert_eq!(annotations.get("features"), Some(&Some("crypto,hardware".to_string())));
        assert_eq!(annotations.get("target"), Some(&Some("thumbv7em-none-eabihf".to_string())));
        
        // Test simple flags
        let annotations = extractor.parse_annotations("no_std,snippet");
        assert!(annotations.contains_key("no_std"));
        assert!(annotations.contains_key("snippet"));
        assert_eq!(annotations.get("no_std"), Some(&None));
        assert_eq!(annotations.get("snippet"), Some(&None));
    }
    
    #[test]
    fn test_hardware_context_detection() {
        let extractor = CodeExtractor::new().unwrap();
        
        // Test explicit hardware annotation
        let content = r#"
```rust,hardware=STM32F4
use stm32f4xx_hal::gpio::*;
let pins = gpioa.split();
```
"#;
        let examples = extractor.extract_from_content(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(examples.len(), 1);
        assert!(matches!(examples[0].context, ExampleContext::Hardware { .. }));
        if let ExampleContext::Hardware { platform, .. } = &examples[0].context {
            assert_eq!(platform, "STM32F4");
        }
        
        // Test hardware detection from code content (should be NoStd since no explicit hardware annotation)
        let content = r#"
```rust
use cortex_m::interrupt;
use stm32f4xx_hal::gpio::*;
```
"#;
        let examples = extractor.extract_from_content(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(examples.len(), 1);
        // This should be NoStd since it's embedded code without explicit hardware annotation
        assert!(matches!(examples[0].context, ExampleContext::NoStd { .. }));
    }
    
    #[test]
    fn test_crypto_context_detection() {
        let extractor = CodeExtractor::new().unwrap();
        
        // Test explicit crypto annotation
        let content = r#"
```rust,crypto,algorithm=AES
use aes::Aes256;
use zeroize::Zeroize;
```
"#;
        let examples = extractor.extract_from_content(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(examples.len(), 1);
        assert!(matches!(examples[0].context, ExampleContext::Crypto { .. }));
        if let ExampleContext::Crypto { algorithm, .. } = &examples[0].context {
            assert_eq!(algorithm, &Some("AES".to_string()));
        }
        
        // Test crypto detection from code content
        let content = r#"
```rust
use zeroize::Zeroize;
let mut key = [0u8; 32];
key.zeroize();
```
"#;
        let examples = extractor.extract_from_content(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(examples.len(), 1);
        assert!(matches!(examples[0].context, ExampleContext::Crypto { .. }));
    }
    
    #[test]
    fn test_no_std_context_detection() {
        let extractor = CodeExtractor::new().unwrap();
        
        // Test explicit no_std annotation
        let content = r#"
```rust,no_std,target=thumbv6m-none-eabi
#![no_std]
use heapless::Vec;
```
"#;
        let examples = extractor.extract_from_content(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(examples.len(), 1);
        assert!(matches!(examples[0].context, ExampleContext::NoStd { .. }));
        if let ExampleContext::NoStd { target, .. } = &examples[0].context {
            assert_eq!(target, "thumbv6m-none-eabi");
        }
        
        // Test no_std detection from code content
        let content = r#"
```rust
#![no_std]
#![no_main]
use cortex_m_rt::entry;
```
"#;
        let examples = extractor.extract_from_content(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(examples.len(), 1);
        assert!(matches!(examples[0].context, ExampleContext::NoStd { .. }));
    }
    
    #[test]
    fn test_snippet_detection() {
        let extractor = CodeExtractor::new().unwrap();
        
        // Test explicit snippet annotation
        let content = r#"
```rust,snippet=incomplete example
let key = generate_key();
```
"#;
        let examples = extractor.extract_from_content(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(examples.len(), 1);
        assert!(matches!(examples[0].context, ExampleContext::Snippet { .. }));
        if let ExampleContext::Snippet { reason } = &examples[0].context {
            assert_eq!(reason, "incomplete example");
        }
        
        // Test automatic snippet detection
        let content = r#"
```rust
let x = 42;
```
"#;
        let examples = extractor.extract_from_content(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(examples.len(), 1);
        assert!(matches!(examples[0].context, ExampleContext::Snippet { .. }));
    }
    
    #[test]
    fn test_feature_extraction() {
        let extractor = CodeExtractor::new().unwrap();
        
        let content = r#"
```rust,features=crypto,hardware,no_std
use zeroize::Zeroize;
```
"#;
        let examples = extractor.extract_from_content(content, &PathBuf::from("test.md")).unwrap();
        assert_eq!(examples.len(), 1);
        
        let features = examples[0].context.features();
        assert!(features.contains(&"crypto".to_string()));
        assert!(features.contains(&"hardware".to_string()));
    }
    
    #[test]
    fn test_hardware_platform_detection() {
        let extractor = CodeExtractor::new().unwrap();
        
        // Test STM32F4 detection
        assert_eq!(
            extractor.detect_hardware_platform("use stm32f4xx_hal::gpio::*;"),
            Some("STM32F4".to_string())
        );
        
        // Test nRF52 detection
        assert_eq!(
            extractor.detect_hardware_platform("use nrf52840_hal::gpio::*;"),
            Some("nRF52".to_string())
        );
        
        // Test generic Cortex-M detection
        assert_eq!(
            extractor.detect_hardware_platform("use cortex_m::interrupt;"),
            Some("Cortex-M".to_string())
        );
        
        // Test no detection
        assert_eq!(
            extractor.detect_hardware_platform("fn main() { println!(\"hello\"); }"),
            None
        );
    }
    
    #[test]
    fn test_crypto_algorithm_detection() {
        let extractor = CodeExtractor::new().unwrap();
        
        assert_eq!(
            extractor.detect_crypto_algorithm("use aes::Aes256;"),
            Some("AES".to_string())
        );
        
        assert_eq!(
            extractor.detect_crypto_algorithm("use sha2::Sha256;"),
            Some("SHA".to_string())
        );
        
        assert_eq!(
            extractor.detect_crypto_algorithm("use chacha20::ChaCha20;"),
            Some("ChaCha20".to_string())
        );
        
        assert_eq!(
            extractor.detect_crypto_algorithm("fn normal_function() {}"),
            None
        );
    }
    
    #[test]
    fn test_context_should_compile() {
        let std_context = ExampleContext::Std { features: vec![] };
        assert!(std_context.should_compile());
        
        let no_std_context = ExampleContext::NoStd { 
            target: "thumbv7em-none-eabihf".to_string(),
            features: vec![]
        };
        assert!(no_std_context.should_compile());
        
        let hardware_context = ExampleContext::Hardware {
            platform: "STM32F4".to_string(),
            features: vec![]
        };
        assert!(hardware_context.should_compile());
        
        let crypto_context = ExampleContext::Crypto {
            algorithm: Some("AES".to_string()),
            features: vec![]
        };
        assert!(crypto_context.should_compile());
        
        let snippet_context = ExampleContext::Snippet {
            reason: "incomplete".to_string()
        };
        assert!(!snippet_context.should_compile());
    }
    
    #[test]
    fn test_context_is_no_std() {
        let std_context = ExampleContext::Std { features: vec![] };
        assert!(!std_context.is_no_std());
        
        let no_std_context = ExampleContext::NoStd { 
            target: "thumbv7em-none-eabihf".to_string(),
            features: vec![]
        };
        assert!(no_std_context.is_no_std());
        
        let hardware_context = ExampleContext::Hardware {
            platform: "STM32F4".to_string(),
            features: vec![]
        };
        assert!(hardware_context.is_no_std());
        
        let crypto_context = ExampleContext::Crypto {
            algorithm: Some("AES".to_string()),
            features: vec![]
        };
        assert!(crypto_context.is_no_std());
        
        let snippet_context = ExampleContext::Snippet {
            reason: "incomplete".to_string()
        };
        assert!(!snippet_context.is_no_std());
    }
    
    #[test]
    fn test_complex_code_examples() {
        let extractor = CodeExtractor::new().unwrap();
        
        let content = r#"
# Complex Examples

## Standard Library Example
```rust
use std::collections::HashMap;
fn main() {
    let mut map = HashMap::new();
    map.insert("key", "value");
    println!("{:?}", map);
}
```

## Embedded Crypto Example
```rust,no_std,features=crypto
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use zeroize::Zeroize;
use aes::Aes256;

#[entry]
fn main() -> ! {
    let mut key = [0u8; 32];
    // ... crypto operations
    key.zeroize();
    loop {}
}
```

## Hardware Example
```rust,hardware=STM32F4,features=gpio
use stm32f4xx_hal::{gpio::*, pac};

fn setup_gpio() {
    let dp = pac::Peripherals::take().unwrap();
    let gpioa = dp.GPIOA.split();
    let led = gpioa.pa5.into_push_pull_output();
}
```

## Code Snippet
```rust,snippet=partial implementation
// This is just a fragment showing key generation
let key = generate_secure_key();
process_with_key(&key);
```
"#;
        
        let examples = extractor.extract_from_content(content, &PathBuf::from("complex.md")).unwrap();
        assert_eq!(examples.len(), 4);
        
        // Check each example type
        assert!(matches!(examples[0].context, ExampleContext::Std { .. }));
        assert!(matches!(examples[1].context, ExampleContext::NoStd { .. }));
        assert!(matches!(examples[2].context, ExampleContext::Hardware { .. }));
        assert!(matches!(examples[3].context, ExampleContext::Snippet { .. }));
        
        // Verify features are extracted correctly
        if let ExampleContext::NoStd { features, .. } = &examples[1].context {
            assert!(features.contains(&"crypto".to_string()));
        }
        
        if let ExampleContext::Hardware { features, .. } = &examples[2].context {
            assert!(features.contains(&"gpio".to_string()));
        }
    }
}