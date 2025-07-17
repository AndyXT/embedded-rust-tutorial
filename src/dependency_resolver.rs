//! Dependency resolution and Cargo.toml generation for code examples

#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};
#[cfg(feature = "std")]
use std::path::Path;
#[cfg(feature = "std")]
use std::string::String;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeSet as HashSet;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use core::option::Option::{self, Some, None};
use core::convert::Into;
use core::default::Default;
use core::matches;

use crate::{CodeExample, ExampleContext};

/// Represents a dependency with version and feature information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    /// Crate name
    pub name: String,
    /// Version requirement (e.g., "1.0", "^0.5", "=0.3.2")
    pub version: String,
    /// Optional features to enable
    pub features: Vec<String>,
    /// Whether this dependency is optional
    pub optional: bool,
    /// Target-specific dependency (e.g., only for embedded targets)
    pub target: Option<String>,
    /// Default features flag
    pub default_features: bool,
}

impl Dependency {
    /// Create a new dependency with default settings
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            features: Vec::new(),
            optional: false,
            target: None,
            default_features: true,
        }
    }

    /// Add features to this dependency
    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }

    /// Mark this dependency as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Set target-specific dependency
    pub fn for_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Disable default features
    pub fn no_default_features(mut self) -> Self {
        self.default_features = false;
        self
    }

    /// Generate the TOML representation of this dependency
    pub fn to_toml_value(&self) -> String {
        if self.features.is_empty() && self.default_features && !self.optional && self.target.is_none() {
            // Simple version-only dependency
            format!("\"{}\"", self.version)
        } else {
            // Complex dependency with features/options
            let mut parts = Vec::new();
            parts.push(format!("version = \"{}\"", self.version));
            
            if !self.default_features {
                parts.push("default-features = false".to_string());
            }
            
            if !self.features.is_empty() {
                let features_str = self.features.iter()
                    .map(|f| format!("\"{}\"", f))
                    .collect::<Vec<_>>()
                    .join(", ");
                parts.push(format!("features = [{}]", features_str));
            }
            
            if self.optional {
                parts.push("optional = true".to_string());
            }
            
            format!("{{ {} }}", parts.join(", "))
        }
    }
}

/// Resolves dependencies for code examples and generates Cargo.toml content
pub struct DependencyResolver {
    /// Known dependency mappings with versions and features
    dependency_catalog: HashMap<String, DependencyInfo>,
}

/// Information about a known dependency
#[derive(Debug, Clone)]
struct DependencyInfo {
    /// Default version to use
    version: String,
    /// Default features for different contexts
    default_features: HashMap<String, Vec<String>>,
    /// Whether to disable default features by default
    no_default_features: bool,
    /// Target-specific information
    targets: Vec<String>,
}

impl DependencyResolver {
    /// Create a new dependency resolver with embedded crypto focus
    pub fn new() -> Self {
        let mut catalog = HashMap::new();
        
        // Embedded dependencies
        catalog.insert("heapless".to_string(), DependencyInfo {
            version: "0.8".to_string(),
            default_features: HashMap::new(),
            no_default_features: false,
            targets: vec!["thumbv*".to_string()],
        });
        
        catalog.insert("cortex-m".to_string(), DependencyInfo {
            version: "0.7".to_string(),
            default_features: HashMap::new(),
            no_default_features: false,
            targets: vec!["thumbv*".to_string()],
        });
        
        catalog.insert("cortex-m-rt".to_string(), DependencyInfo {
            version: "0.7".to_string(),
            default_features: HashMap::new(),
            no_default_features: false,
            targets: vec!["thumbv*".to_string()],
        });
        
        catalog.insert("embedded-hal".to_string(), DependencyInfo {
            version: "1.0".to_string(),
            default_features: HashMap::new(),
            no_default_features: false,
            targets: vec!["thumbv*".to_string()],
        });
        
        // Panic handlers
        catalog.insert("panic-halt".to_string(), DependencyInfo {
            version: "0.2".to_string(),
            default_features: HashMap::new(),
            no_default_features: false,
            targets: vec!["thumbv*".to_string()],
        });
        
        catalog.insert("panic-abort".to_string(), DependencyInfo {
            version: "0.3".to_string(),
            default_features: HashMap::new(),
            no_default_features: false,
            targets: vec!["thumbv*".to_string()],
        });
        
        // Cryptography dependencies
        catalog.insert("zeroize".to_string(), DependencyInfo {
            version: "1.7".to_string(),
            default_features: {
                let mut features = HashMap::new();
                features.insert("std".to_string(), vec!["derive".to_string()]);
                features.insert("crypto".to_string(), vec!["derive".to_string()]);
                features
            },
            no_default_features: false,
            targets: Vec::new(),
        });
        
        catalog.insert("aes".to_string(), DependencyInfo {
            version: "0.8".to_string(),
            default_features: HashMap::new(),
            no_default_features: true,
            targets: Vec::new(),
        });
        
        catalog.insert("sha2".to_string(), DependencyInfo {
            version: "0.10".to_string(),
            default_features: HashMap::new(),
            no_default_features: true,
            targets: Vec::new(),
        });
        
        catalog.insert("chacha20".to_string(), DependencyInfo {
            version: "0.9".to_string(),
            default_features: HashMap::new(),
            no_default_features: true,
            targets: Vec::new(),
        });
        
        catalog.insert("ed25519-dalek".to_string(), DependencyInfo {
            version: "2.0".to_string(),
            default_features: HashMap::new(),
            no_default_features: true,
            targets: Vec::new(),
        });
        
        catalog.insert("rand".to_string(), DependencyInfo {
            version: "0.8".to_string(),
            default_features: {
                let mut features = HashMap::new();
                features.insert("std".to_string(), vec!["std".to_string()]);
                features.insert("no_std".to_string(), vec!["small_rng".to_string()]);
                features
            },
            no_default_features: false,
            targets: Vec::new(),
        });
        
        catalog.insert("rand_core".to_string(), DependencyInfo {
            version: "0.6".to_string(),
            default_features: HashMap::new(),
            no_default_features: true,
            targets: Vec::new(),
        });
        
        // Hardware-specific dependencies
        catalog.insert("stm32f4xx-hal".to_string(), DependencyInfo {
            version: "0.19".to_string(),
            default_features: {
                let mut features = HashMap::new();
                features.insert("hardware".to_string(), vec!["stm32f407".to_string()]);
                features
            },
            no_default_features: false,
            targets: vec!["thumbv7em-none-eabihf".to_string()],
        });
        
        catalog.insert("stm32f3xx-hal".to_string(), DependencyInfo {
            version: "0.10".to_string(),
            default_features: {
                let mut features = HashMap::new();
                features.insert("hardware".to_string(), vec!["stm32f303xc".to_string()]);
                features
            },
            no_default_features: false,
            targets: vec!["thumbv7em-none-eabihf".to_string()],
        });
        
        catalog.insert("nrf52840-hal".to_string(), DependencyInfo {
            version: "0.16".to_string(),
            default_features: HashMap::new(),
            no_default_features: false,
            targets: vec!["thumbv7em-none-eabihf".to_string()],
        });

        Self {
            dependency_catalog: catalog,
        }
    }

    /// Resolve dependencies for a single code example
    pub fn resolve_dependencies(&self, example: &CodeExample) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        let mut seen = HashSet::new();

        // Extract dependencies from code analysis
        let detected_deps = self.detect_dependencies_from_code(&example.code);
        
        for dep_name in detected_deps {
            if seen.contains(&dep_name) {
                continue;
            }
            seen.insert(dep_name.clone());

            if let Some(dep_info) = self.dependency_catalog.get(&dep_name) {
                let mut dependency = Dependency::new(&dep_name, &dep_info.version);
                
                // Configure based on context
                match &example.context {
                    ExampleContext::Std { features } => {
                        if let Some(context_features) = dep_info.default_features.get("std") {
                            dependency = dependency.with_features(context_features.clone());
                        }
                        // Add any explicit features from the example
                        let mut all_features = dependency.features;
                        all_features.extend(features.clone());
                        all_features.sort();
                        all_features.dedup();
                        dependency.features = all_features;
                    }
                    ExampleContext::NoStd { target, features } => {
                        if let Some(context_features) = dep_info.default_features.get("no_std") {
                            dependency = dependency.with_features(context_features.clone());
                        }
                        if dep_info.no_default_features {
                            dependency = dependency.no_default_features();
                        }
                        if !dep_info.targets.is_empty() {
                            dependency = dependency.for_target(target);
                        }
                        // Add any explicit features from the example
                        let mut all_features = dependency.features;
                        all_features.extend(features.clone());
                        all_features.sort();
                        all_features.dedup();
                        dependency.features = all_features;
                    }
                    ExampleContext::Hardware { platform: _, features } => {
                        if let Some(context_features) = dep_info.default_features.get("hardware") {
                            dependency = dependency.with_features(context_features.clone());
                        }
                        if dep_info.no_default_features {
                            dependency = dependency.no_default_features();
                        }
                        // Add any explicit features from the example
                        let mut all_features = dependency.features;
                        all_features.extend(features.clone());
                        all_features.sort();
                        all_features.dedup();
                        dependency.features = all_features;
                    }
                    ExampleContext::Crypto { features, .. } => {
                        if let Some(context_features) = dep_info.default_features.get("crypto") {
                            dependency = dependency.with_features(context_features.clone());
                        }
                        if dep_info.no_default_features {
                            dependency = dependency.no_default_features();
                        }
                        // Add any explicit features from the example
                        let mut all_features = dependency.features;
                        all_features.extend(features.clone());
                        all_features.sort();
                        all_features.dedup();
                        dependency.features = all_features;
                    }
                    ExampleContext::Snippet { .. } => {
                        // Snippets don't need dependencies resolved
                        continue;
                    }
                }

                dependencies.push(dependency);
            } else {
                // Unknown dependency - add with basic configuration
                let mut dependency = Dependency::new(&dep_name, "1.0");
                
                // For no_std contexts, disable default features by default
                if matches!(example.context, ExampleContext::NoStd { .. } | ExampleContext::Hardware { .. }) {
                    dependency = dependency.no_default_features();
                }
                
                dependencies.push(dependency);
            }
        }

        dependencies
    }

    /// Detect dependencies from code content using pattern matching
    fn detect_dependencies_from_code(&self, code: &str) -> Vec<String> {
        let mut dependencies = Vec::new();

        // Pattern matching for use statements
        for line in code.lines() {
            let line = line.trim();
            
            // Match "use crate_name::" or "use crate_name;"
            if line.starts_with("use ") && !line.starts_with("use std::") && !line.starts_with("use core::") && !line.starts_with("use alloc::") {
                if let Some(crate_name) = self.extract_crate_name_from_use(line) {
                    dependencies.push(crate_name);
                }
            }
            
            // Match extern crate declarations
            if line.starts_with("extern crate ") {
                if let Some(crate_name) = line.strip_prefix("extern crate ") {
                    let crate_name = crate_name.split_whitespace().next().unwrap_or("").trim_end_matches(';');
                    if !crate_name.is_empty() {
                        dependencies.push(crate_name.to_string());
                    }
                }
            }
        }

        // Additional pattern-based detection for common embedded/crypto patterns
        self.detect_implicit_dependencies(code, &mut dependencies);

        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    /// Extract crate name from a use statement
    fn extract_crate_name_from_use(&self, use_line: &str) -> Option<String> {
        // Remove "use " prefix
        let use_content = use_line.strip_prefix("use ")?;
        
        // Handle different use patterns:
        // use crate_name::...
        // use crate_name;
        // use crate_name as alias;
        
        let first_part = use_content.split("::").next()?;
        let crate_name = first_part.split_whitespace().next()?;
        let crate_name = crate_name.trim_end_matches(';');
        
        // Convert underscores to hyphens for crate names
        let normalized_name = crate_name.replace('_', "-");
        
        // Filter out standard library and local modules
        if matches!(normalized_name.as_str(), "std" | "core" | "alloc" | "self" | "super" | "crate") {
            return None;
        }
        
        Some(normalized_name)
    }

    /// Detect implicit dependencies based on code patterns
    fn detect_implicit_dependencies(&self, code: &str, dependencies: &mut Vec<String>) {
        // Panic handlers - detect from attributes or function names
        if code.contains("#[panic_handler]") || code.contains("panic_halt") {
            dependencies.push("panic-halt".to_string());
        }
        if code.contains("panic_abort") {
            dependencies.push("panic-abort".to_string());
        }

        // Entry point detection
        if code.contains("#[entry]") || code.contains("cortex_m_rt::entry") {
            dependencies.push("cortex-m-rt".to_string());
        }

        // Hardware abstraction layer detection
        if code.contains("embedded_hal") {
            dependencies.push("embedded-hal".to_string());
        }

        // Specific hardware platform detection
        if code.contains("stm32f4") || code.contains("STM32F4") {
            dependencies.push("stm32f4xx-hal".to_string());
        }
        if code.contains("stm32f3") || code.contains("STM32F3") {
            dependencies.push("stm32f3xx-hal".to_string());
        }
        if code.contains("nrf52") || code.contains("NRF52") {
            dependencies.push("nrf52840-hal".to_string());
        }

        // Crypto algorithm detection
        if code.contains("Aes") || code.contains("aes::") {
            dependencies.push("aes".to_string());
        }
        if code.contains("Sha") || code.contains("sha2::") {
            dependencies.push("sha2".to_string());
        }
        if code.contains("ChaCha") || code.contains("chacha20::") {
            dependencies.push("chacha20".to_string());
        }
        if code.contains("Ed25519") || code.contains("ed25519") {
            dependencies.push("ed25519-dalek".to_string());
        }

        // Random number generation
        if code.contains("rand::") || code.contains("Rng") {
            dependencies.push("rand".to_string());
        }
        if code.contains("rand_core::") || code.contains("RngCore") {
            dependencies.push("rand_core".to_string());
        }
    }

    /// Generate a complete Cargo.toml content for a set of examples
    pub fn generate_cargo_toml(&self, examples: &[CodeExample], package_name: &str) -> String {
        let mut all_dependencies = HashMap::new();
        let mut target_dependencies: HashMap<String, HashMap<String, Dependency>> = HashMap::new();

        // Collect all dependencies from examples
        for example in examples {
            if !example.should_compile() {
                continue;
            }

            let deps = self.resolve_dependencies(example);
            for dep in deps {
                if let Some(target) = &dep.target {
                    // Target-specific dependency
                    target_dependencies
                        .entry(target.clone())
                        .or_insert_with(HashMap::new)
                        .insert(dep.name.clone(), dep);
                } else {
                    // Regular dependency
                    all_dependencies.insert(dep.name.clone(), dep);
                }
            }
        }

        // Determine if this is a no_std project
        let is_no_std = examples.iter().any(|e| e.context.is_no_std());

        // Generate TOML content
        let mut toml_content = String::new();
        
        // Package section
        toml_content.push_str(&format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

"#, package_name));

        // Lib section for no_std projects
        if is_no_std {
            toml_content.push_str(r#"[lib]
name = "lib"
crate-type = ["staticlib"]

"#);
        }

        // Dependencies section
        if !all_dependencies.is_empty() {
            toml_content.push_str("[dependencies]\n");
            let mut sorted_deps: Vec<_> = all_dependencies.values().collect();
            sorted_deps.sort_by(|a, b| a.name.cmp(&b.name));
            
            for dep in sorted_deps {
                toml_content.push_str(&format!("{} = {}\n", dep.name, dep.to_toml_value()));
            }
            toml_content.push('\n');
        }

        // Target-specific dependencies
        for (target, deps) in target_dependencies {
            toml_content.push_str(&format!("[target.'cfg(target = \"{}\")'.dependencies]\n", target));
            let mut sorted_deps: Vec<_> = deps.values().collect();
            sorted_deps.sort_by(|a, b| a.name.cmp(&b.name));
            
            for dep in sorted_deps {
                toml_content.push_str(&format!("{} = {}\n", dep.name, dep.to_toml_value()));
            }
            toml_content.push('\n');
        }

        // Profile sections for embedded projects
        if is_no_std {
            toml_content.push_str(r#"[profile.dev]
debug = true
opt-level = 0

[profile.release]
debug = true
opt-level = "s"
lto = true
codegen-units = 1

"#);
        }

        toml_content
    }

    /// Generate a minimal Cargo.toml for a single example
    pub fn generate_example_cargo_toml(&self, example: &CodeExample) -> String {
        self.generate_cargo_toml(&[example.clone()], &format!("example_{}", example.id))
    }

    /// Add a custom dependency to the catalog
    pub fn add_dependency_info(&mut self, name: String, info: DependencyInfo) {
        self.dependency_catalog.insert(name, info);
    }

    /// Get all known dependency names
    pub fn known_dependencies(&self) -> Vec<String> {
        self.dependency_catalog.keys().cloned().collect()
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::{CodeExample, ExampleContext};
    use std::path::PathBuf;

    #[test]
    fn test_dependency_creation() {
        let dep = Dependency::new("heapless", "0.8")
            .with_features(vec!["serde".to_string()])
            .optional()
            .no_default_features();

        assert_eq!(dep.name, "heapless");
        assert_eq!(dep.version, "0.8");
        assert_eq!(dep.features, vec!["serde"]);
        assert!(dep.optional);
        assert!(!dep.default_features);
    }

    #[test]
    fn test_dependency_toml_generation() {
        // Simple dependency
        let simple_dep = Dependency::new("heapless", "0.8");
        assert_eq!(simple_dep.to_toml_value(), "\"0.8\"");

        // Complex dependency
        let complex_dep = Dependency::new("aes", "0.8")
            .with_features(vec!["std".to_string()])
            .no_default_features();
        assert_eq!(
            complex_dep.to_toml_value(),
            "{ version = \"0.8\", default-features = false, features = [\"std\"] }"
        );
    }

    #[test]
    fn test_dependency_resolver_creation() {
        let resolver = DependencyResolver::new();
        let known_deps = resolver.known_dependencies();
        
        assert!(known_deps.contains(&"heapless".to_string()));
        assert!(known_deps.contains(&"zeroize".to_string()));
        assert!(known_deps.contains(&"aes".to_string()));
        assert!(known_deps.contains(&"cortex-m".to_string()));
    }

    #[test]
    fn test_crate_name_extraction() {
        let resolver = DependencyResolver::new();

        assert_eq!(
            resolver.extract_crate_name_from_use("use heapless::Vec;"),
            Some("heapless".to_string())
        );
        assert_eq!(
            resolver.extract_crate_name_from_use("use cortex_m::interrupt;"),
            Some("cortex-m".to_string())
        );
        assert_eq!(
            resolver.extract_crate_name_from_use("use std::collections::HashMap;"),
            None
        );
        assert_eq!(
            resolver.extract_crate_name_from_use("use self::module;"),
            None
        );
    }

    #[test]
    fn test_dependency_detection_from_code() {
        let resolver = DependencyResolver::new();
        
        let code = r#"
use heapless::Vec;
use cortex_m::interrupt;
use zeroize::Zeroize;
use aes::Aes256;
"#;

        let deps = resolver.detect_dependencies_from_code(code);
        assert!(deps.contains(&"heapless".to_string()));
        assert!(deps.contains(&"cortex-m".to_string()));
        assert!(deps.contains(&"zeroize".to_string()));
        assert!(deps.contains(&"aes".to_string()));
    }

    #[test]
    fn test_implicit_dependency_detection() {
        let resolver = DependencyResolver::new();
        
        let code_with_panic = r#"
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
"#;
        let mut deps = Vec::new();
        resolver.detect_implicit_dependencies(code_with_panic, &mut deps);
        assert!(deps.contains(&"panic-halt".to_string()));

        let code_with_entry = r#"
#[entry]
fn main() -> ! {
    loop {}
}
"#;
        let mut deps = Vec::new();
        resolver.detect_implicit_dependencies(code_with_entry, &mut deps);
        assert!(deps.contains(&"cortex-m-rt".to_string()));
    }

    #[test]
    fn test_resolve_dependencies_for_std_example() {
        let resolver = DependencyResolver::new();
        
        let example = CodeExample::new(
            "test_std".to_string(),
            PathBuf::from("test.md"),
            1,
            "use zeroize::Zeroize;\nlet mut key = [0u8; 32];".to_string(),
            ExampleContext::Std { features: vec!["crypto".to_string()] },
        );

        let deps = resolver.resolve_dependencies(&example);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "zeroize");
        assert_eq!(deps[0].version, "1.7");
        assert!(deps[0].features.contains(&"derive".to_string()));
        assert!(deps[0].features.contains(&"crypto".to_string()));
    }

    #[test]
    fn test_resolve_dependencies_for_no_std_example() {
        let resolver = DependencyResolver::new();
        
        let example = CodeExample::new(
            "test_no_std".to_string(),
            PathBuf::from("test.md"),
            1,
            "use heapless::Vec;\nuse cortex_m::interrupt;".to_string(),
            ExampleContext::NoStd { 
                target: "thumbv7em-none-eabihf".to_string(),
                features: vec![]
            },
        );

        let deps = resolver.resolve_dependencies(&example);
        assert_eq!(deps.len(), 2);
        
        let heapless_dep = deps.iter().find(|d| d.name == "heapless").unwrap();
        assert_eq!(heapless_dep.version, "0.8");
        assert_eq!(heapless_dep.target, Some("thumbv7em-none-eabihf".to_string()));
        
        let cortex_m_dep = deps.iter().find(|d| d.name == "cortex-m").unwrap();
        assert_eq!(cortex_m_dep.version, "0.7");
    }

    #[test]
    fn test_resolve_dependencies_for_crypto_example() {
        let resolver = DependencyResolver::new();
        
        let example = CodeExample::new(
            "test_crypto".to_string(),
            PathBuf::from("test.md"),
            1,
            "use aes::Aes256;\nuse zeroize::Zeroize;".to_string(),
            ExampleContext::Crypto { 
                algorithm: Some("AES".to_string()),
                features: vec!["hardware".to_string()]
            },
        );

        let deps = resolver.resolve_dependencies(&example);
        assert_eq!(deps.len(), 2);
        
        let aes_dep = deps.iter().find(|d| d.name == "aes").unwrap();
        assert_eq!(aes_dep.version, "0.8");
        assert!(!aes_dep.default_features);
        assert!(aes_dep.features.contains(&"hardware".to_string()));
        
        let zeroize_dep = deps.iter().find(|d| d.name == "zeroize").unwrap();
        assert_eq!(zeroize_dep.version, "1.7");
        assert!(zeroize_dep.features.contains(&"derive".to_string()));
        assert!(zeroize_dep.features.contains(&"hardware".to_string()));
    }

    #[test]
    fn test_cargo_toml_generation() {
        let resolver = DependencyResolver::new();
        
        let examples = vec![
            CodeExample::new(
                "std_example".to_string(),
                PathBuf::from("test.md"),
                1,
                "use zeroize::Zeroize;".to_string(),
                ExampleContext::Std { features: vec![] },
            ),
            CodeExample::new(
                "no_std_example".to_string(),
                PathBuf::from("test.md"),
                10,
                "use heapless::Vec;\nuse cortex_m::interrupt;".to_string(),
                ExampleContext::NoStd { 
                    target: "thumbv7em-none-eabihf".to_string(),
                    features: vec![]
                },
            ),
        ];

        let cargo_toml = resolver.generate_cargo_toml(&examples, "test_project");
        
        assert!(cargo_toml.contains("[package]"));
        assert!(cargo_toml.contains("name = \"test_project\""));
        assert!(cargo_toml.contains("[dependencies]"));
        assert!(cargo_toml.contains("zeroize = { version = \"1.7\", features = [\"derive\"] }"));
        assert!(cargo_toml.contains("[target.'cfg(target = \"thumbv7em-none-eabihf\")'.dependencies]"));
        assert!(cargo_toml.contains("heapless = { version = \"0.8\" }"));
        assert!(cargo_toml.contains("cortex-m = { version = \"0.7\" }"));
        
        // Should include no_std profile settings
        assert!(cargo_toml.contains("[profile.release]"));
        assert!(cargo_toml.contains("opt-level = \"s\""));
    }

    #[test]
    fn test_snippet_dependencies_ignored() {
        let resolver = DependencyResolver::new();
        
        let example = CodeExample::new(
            "snippet_example".to_string(),
            PathBuf::from("test.md"),
            1,
            "let key = generate_key();".to_string(),
            ExampleContext::Snippet { reason: "incomplete".to_string() },
        );

        let deps = resolver.resolve_dependencies(&example);
        assert_eq!(deps.len(), 0);
    }

    #[test]
    fn test_hardware_specific_dependencies() {
        let resolver = DependencyResolver::new();
        
        let example = CodeExample::new(
            "hardware_example".to_string(),
            PathBuf::from("test.md"),
            1,
            "use stm32f4xx_hal::gpio::*;".to_string(),
            ExampleContext::Hardware { 
                platform: "STM32F4".to_string(),
                features: vec!["gpio".to_string()]
            },
        );

        let deps = resolver.resolve_dependencies(&example);
        assert_eq!(deps.len(), 1);
        
        let stm32_dep = &deps[0];
        assert_eq!(stm32_dep.name, "stm32f4xx-hal");
        assert_eq!(stm32_dep.version, "0.19");
        assert!(stm32_dep.features.contains(&"stm32f407".to_string()));
        assert!(stm32_dep.features.contains(&"gpio".to_string()));
    }
}