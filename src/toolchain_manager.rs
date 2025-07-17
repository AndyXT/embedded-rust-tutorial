//! Toolchain management for multi-target Rust compilation testing

use crate::{TestConfig, CompilationError};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::string::String;
use std::vec::Vec;
use core::option::Option::{self, Some, None};
use core::result::Result::{self, Ok, Err};
use core::default::Default;

/// Manages Rust toolchains and targets for compilation testing
#[derive(Debug, Clone)]
pub struct ToolchainManager {
    /// Available targets and their status
    installed_targets: HashMap<String, TargetInfo>,
    /// Default toolchain to use
    default_toolchain: String,
    /// Cache of target availability checks
    target_cache: HashMap<String, bool>,
}

/// Information about a compilation target
#[derive(Debug, Clone)]
pub struct TargetInfo {
    /// Target triple (e.g., "thumbv7em-none-eabihf")
    pub target: String,
    /// Whether the target is installed
    pub installed: bool,
    /// Target type (std, no_std, embedded)
    pub target_type: TargetType,
    /// Required features for this target
    pub required_features: Vec<String>,
    /// Default dependencies for this target
    pub default_dependencies: Vec<String>,
}

/// Type of compilation target
#[derive(Debug, Clone, PartialEq)]
pub enum TargetType {
    /// Standard library target
    Std,
    /// No-std target
    NoStd,
    /// Embedded target (implies no_std)
    Embedded,
}

impl ToolchainManager {
    /// Create a new ToolchainManager
    pub fn new() -> Result<Self, CompilationError> {
        let mut manager = Self {
            installed_targets: HashMap::new(),
            default_toolchain: "stable".to_string(),
            target_cache: HashMap::new(),
        };
        
        manager.initialize_targets()?;
        Ok(manager)
    }
    
    /// Initialize the list of known targets and check their availability
    fn initialize_targets(&mut self) -> Result<(), CompilationError> {
        // Define known targets with their characteristics
        let known_targets = vec![
            // Standard targets
            TargetInfo {
                target: "x86_64-unknown-linux-gnu".to_string(),
                installed: false, // Will be checked
                target_type: TargetType::Std,
                required_features: vec![],
                default_dependencies: vec!["std".to_string()],
            },
            TargetInfo {
                target: "x86_64-pc-windows-msvc".to_string(),
                installed: false,
                target_type: TargetType::Std,
                required_features: vec![],
                default_dependencies: vec!["std".to_string()],
            },
            TargetInfo {
                target: "x86_64-apple-darwin".to_string(),
                installed: false,
                target_type: TargetType::Std,
                required_features: vec![],
                default_dependencies: vec!["std".to_string()],
            },
            
            // Embedded ARM targets
            TargetInfo {
                target: "thumbv7em-none-eabihf".to_string(),
                installed: false,
                target_type: TargetType::Embedded,
                required_features: vec!["cortex-m".to_string()],
                default_dependencies: vec![
                    "cortex-m".to_string(),
                    "cortex-m-rt".to_string(),
                    "panic-halt".to_string(),
                ],
            },
            TargetInfo {
                target: "thumbv6m-none-eabi".to_string(),
                installed: false,
                target_type: TargetType::Embedded,
                required_features: vec!["cortex-m".to_string()],
                default_dependencies: vec![
                    "cortex-m".to_string(),
                    "cortex-m-rt".to_string(),
                    "panic-halt".to_string(),
                ],
            },
            TargetInfo {
                target: "thumbv7m-none-eabi".to_string(),
                installed: false,
                target_type: TargetType::Embedded,
                required_features: vec!["cortex-m".to_string()],
                default_dependencies: vec![
                    "cortex-m".to_string(),
                    "cortex-m-rt".to_string(),
                    "panic-halt".to_string(),
                ],
            },
            TargetInfo {
                target: "thumbv8m.main-none-eabihf".to_string(),
                installed: false,
                target_type: TargetType::Embedded,
                required_features: vec!["cortex-m".to_string()],
                default_dependencies: vec![
                    "cortex-m".to_string(),
                    "cortex-m-rt".to_string(),
                    "panic-halt".to_string(),
                ],
            },
            
            // RISC-V targets
            TargetInfo {
                target: "riscv32imac-unknown-none-elf".to_string(),
                installed: false,
                target_type: TargetType::Embedded,
                required_features: vec!["riscv".to_string()],
                default_dependencies: vec![
                    "riscv".to_string(),
                    "riscv-rt".to_string(),
                    "panic-halt".to_string(),
                ],
            },
            TargetInfo {
                target: "riscv64gc-unknown-none-elf".to_string(),
                installed: false,
                target_type: TargetType::Embedded,
                required_features: vec!["riscv".to_string()],
                default_dependencies: vec![
                    "riscv".to_string(),
                    "riscv-rt".to_string(),
                    "panic-halt".to_string(),
                ],
            },
        ];
        
        // Check which targets are actually installed
        for mut target_info in known_targets {
            target_info.installed = self.check_target_installed(&target_info.target)?;
            self.installed_targets.insert(target_info.target.clone(), target_info);
        }
        
        Ok(())
    }
    
    /// Check if a specific target is installed
    pub fn check_target_installed(&mut self, target: &str) -> Result<bool, CompilationError> {
        // Check cache first
        if let Some(&cached) = self.target_cache.get(target) {
            return Ok(cached);
        }
        
        // Run rustup to check if target is installed
        let output = Command::new("rustup")
            .args(&["target", "list", "--installed"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| CompilationError::IoError {
                message: format!("Failed to run rustup: {}", e),
            })?;
        
        if !output.status.success() {
            return Err(CompilationError::IoError {
                message: "Failed to query installed targets".to_string(),
            });
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let installed = stdout.lines().any(|line| line.trim() == target);
        
        // Cache the result
        self.target_cache.insert(target.to_string(), installed);
        
        Ok(installed)
    }
    
    /// Install a target if it's not already installed
    pub fn install_target(&mut self, target: &str) -> Result<(), CompilationError> {
        if self.check_target_installed(target)? {
            return Ok(()); // Already installed
        }
        
        println!("Installing target: {}", target);
        
        let output = Command::new("rustup")
            .args(&["target", "add", target])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| CompilationError::IoError {
                message: format!("Failed to run rustup target add: {}", e),
            })?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CompilationError::TargetError {
                incompatible_target: target.to_string(),
                reason: format!("Failed to install target: {}", stderr),
            });
        }
        
        // Update cache and target info
        self.target_cache.insert(target.to_string(), true);
        if let Some(target_info) = self.installed_targets.get_mut(target) {
            target_info.installed = true;
        }
        
        Ok(())
    }
    
    /// Get available targets for a given configuration
    pub fn get_available_targets(&self, config: &TestConfig) -> Vec<String> {
        let mut available = Vec::new();
        
        for target in &config.targets {
            if let Some(target_info) = self.installed_targets.get(target) {
                if target_info.installed {
                    // Check if target type matches configuration
                    let compatible = match (&target_info.target_type, config.no_std, config.embedded_mode) {
                        (TargetType::Std, false, false) => true,
                        (TargetType::NoStd, true, _) => true,
                        (TargetType::Embedded, true, true) => true,
                        _ => false,
                    };
                    
                    if compatible {
                        available.push(target.clone());
                    }
                }
            }
        }
        
        // If no targets are available, try to use a default
        if available.is_empty() {
            if config.embedded_mode {
                // Try to find any installed embedded target
                for (target, info) in &self.installed_targets {
                    if info.installed && info.target_type == TargetType::Embedded {
                        available.push(target.clone());
                        break;
                    }
                }
            } else {
                // Use host target as fallback
                available.push("x86_64-unknown-linux-gnu".to_string());
            }
        }
        
        available
    }
    
    /// Get target information
    pub fn get_target_info(&self, target: &str) -> Option<&TargetInfo> {
        self.installed_targets.get(target)
    }
    
    /// Get all installed targets
    pub fn get_installed_targets(&self) -> Vec<&TargetInfo> {
        self.installed_targets
            .values()
            .filter(|info| info.installed)
            .collect()
    }
    
    /// Get targets by type
    pub fn get_targets_by_type(&self, target_type: TargetType) -> Vec<&TargetInfo> {
        self.installed_targets
            .values()
            .filter(|info| info.installed && info.target_type == target_type)
            .collect()
    }
    
    /// Ensure required targets are installed for a configuration
    pub fn ensure_targets_for_config(&mut self, config: &TestConfig) -> Result<Vec<String>, CompilationError> {
        let mut ensured_targets = Vec::new();
        
        for target in &config.targets {
            // Try to install the target if it's not available
            if !self.check_target_installed(target)? {
                match self.install_target(target) {
                    Ok(()) => {
                        ensured_targets.push(target.clone());
                        println!("Successfully installed target: {}", target);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to install target {}: {}", target, e);
                        // Continue with other targets rather than failing completely
                    }
                }
            } else {
                ensured_targets.push(target.clone());
            }
        }
        
        if ensured_targets.is_empty() {
            return Err(CompilationError::TargetError {
                incompatible_target: config.targets.join(", "),
                reason: "No targets could be installed or are available".to_string(),
            });
        }
        
        Ok(ensured_targets)
    }
    
    /// Get recommended dependencies for a target
    pub fn get_recommended_dependencies(&self, target: &str) -> Vec<String> {
        self.installed_targets
            .get(target)
            .map(|info| info.default_dependencies.clone())
            .unwrap_or_default()
    }
    
    /// Check if cargo is available
    pub fn check_cargo_available(&self) -> bool {
        Command::new("cargo")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
    
    /// Check if rustup is available
    pub fn check_rustup_available(&self) -> bool {
        Command::new("rustup")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
    
    /// Get system information for debugging
    pub fn get_system_info(&self) -> Result<SystemInfo, CompilationError> {
        let cargo_version = self.get_tool_version("cargo")?;
        let rustc_version = self.get_tool_version("rustc")?;
        let rustup_version = self.get_tool_version("rustup").ok();
        
        Ok(SystemInfo {
            cargo_version,
            rustc_version,
            rustup_version,
            installed_targets: self.get_installed_targets().len(),
            available_targets: self.installed_targets.len(),
        })
    }
    
    /// Get version of a tool
    fn get_tool_version(&self, tool: &str) -> Result<String, CompilationError> {
        let output = Command::new(tool)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| CompilationError::IoError {
                message: format!("Failed to run {}: {}", tool, e),
            })?;
        
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(version.lines().next().unwrap_or("unknown").to_string())
        } else {
            Err(CompilationError::IoError {
                message: format!("{} not available", tool),
            })
        }
    }
}

impl Default for ToolchainManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            installed_targets: HashMap::new(),
            default_toolchain: "stable".to_string(),
            target_cache: HashMap::new(),
        })
    }
}

/// System information for debugging
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub cargo_version: String,
    pub rustc_version: String,
    pub rustup_version: Option<String>,
    pub installed_targets: usize,
    pub available_targets: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_toolchain_manager_creation() {
        let manager = ToolchainManager::new();
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_target_info_creation() {
        let target_info = TargetInfo {
            target: "x86_64-unknown-linux-gnu".to_string(),
            installed: true,
            target_type: TargetType::Std,
            required_features: vec![],
            default_dependencies: vec!["std".to_string()],
        };
        
        assert_eq!(target_info.target_type, TargetType::Std);
        assert!(target_info.installed);
    }
    
    #[test]
    fn test_get_available_targets() {
        let manager = ToolchainManager::default();
        let config = TestConfig::default();
        
        let targets = manager.get_available_targets(&config);
        // Should have at least one target (fallback)
        assert!(!targets.is_empty());
    }
    
    #[test]
    fn test_embedded_config_targets() {
        let manager = ToolchainManager::default();
        let config = TestConfig::embedded();
        
        let targets = manager.get_available_targets(&config);
        // May be empty if no embedded targets are installed, which is fine
        println!("Available embedded targets: {:?}", targets);
    }
    
    #[test]
    fn test_target_type_matching() {
        let manager = ToolchainManager::default();
        
        // Test std target
        if let Some(info) = manager.get_target_info("x86_64-unknown-linux-gnu") {
            assert_eq!(info.target_type, TargetType::Std);
        }
        
        // Test embedded target
        if let Some(info) = manager.get_target_info("thumbv7em-none-eabihf") {
            assert_eq!(info.target_type, TargetType::Embedded);
        }
    }
    
    #[test]
    fn test_cargo_availability() {
        let manager = ToolchainManager::default();
        // This should pass in most development environments
        assert!(manager.check_cargo_available());
    }
    
    #[test]
    fn test_system_info() {
        let manager = ToolchainManager::default();
        let system_info = manager.get_system_info();
        
        if let Ok(info) = system_info {
            assert!(!info.cargo_version.is_empty());
            assert!(!info.rustc_version.is_empty());
            println!("System info: {:?}", info);
        }
    }
    
    #[test]
    fn test_recommended_dependencies() {
        let manager = ToolchainManager::default();
        
        let deps = manager.get_recommended_dependencies("thumbv7em-none-eabihf");
        assert!(deps.contains(&"cortex-m".to_string()));
        assert!(deps.contains(&"cortex-m-rt".to_string()));
    }
}