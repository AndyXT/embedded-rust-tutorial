# 2.4 Build Configuration

## Build Script (build.rs)

Build scripts allow compile-time configuration based on target architecture:

```rust
// build.rs - Configure build based on target
fn main() {
    let target = std::env::var("TARGET").unwrap();
    
    // Configure based on target architecture
    if target.starts_with("armv7r") {
        // Cortex-R5 specific configuration
        println!("cargo:rustc-cfg=cortex_r5");
        println!("cargo:rustc-link-arg=-Tlink.x");
    } else if target.starts_with("thumbv7em") {
        // Cortex-M4/M7 configuration
        println!("cargo:rustc-cfg=cortex_m");
        println!("cargo:rustc-cfg=has_fpu");
    }
    
    // Add memory.x to linker search path
    println!("cargo:rustc-link-search=memory");
    println!("cargo:rerun-if-changed=memory.x");
}
```

## Conditional Compilation

Use target configuration in your code:

```rust
#![no_std]
#![no_main]

// Architecture-specific features
#[cfg(cortex_r5)]
mod r5_specific {
    pub fn init_tcm() {
        // Initialize Tightly Coupled Memory
    }
}

#[cfg(cortex_m)]
mod m4_specific {
    pub fn init_fpu() {
        // Initialize Floating Point Unit
    }
}

// Common entry point
#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    // Platform-specific initialization
    #[cfg(cortex_r5)]
    r5_specific::init_tcm();
    
    #[cfg(cortex_m)]
    m4_specific::init_fpu();
    
    // Your application code
    loop {}
}
```

## Profile Configuration

Optimize for embedded targets in Cargo.toml:

```toml
[profile.dev]
opt-level = "s"     # Optimize for size even in debug
debug = true        # Keep debug info
lto = false         # Faster builds
codegen-units = 16  # Parallel compilation

[profile.release]
opt-level = "s"     # Size optimization
debug = false       # No debug info
lto = "fat"         # Link Time Optimization
codegen-units = 1   # Better optimization
strip = true        # Remove symbols
panic = "abort"     # Smaller binary

[profile.release-debug]
inherits = "release"
debug = true        # Keep debug symbols for debugging

# Special profile for minimal size
[profile.minimal]
inherits = "release"
opt-level = "z"     # Optimize for size
strip = true
lto = "fat"
codegen-units = 1
panic = "abort"
```

## Feature Flags

Configure features for different use cases:

```toml
[features]
default = []

# Target-specific features
cortex-r5 = ["cortex-r-rt"]
cortex-m4 = ["cortex-m-rt", "cortex-m/critical-section-single-core"]

# Functionality features
crypto-sw = ["sha2", "aes"]
crypto-hw = []  # Hardware crypto acceleration
defmt-logging = ["defmt", "defmt-rtt"]

# Debug features
debug-print = []
semihosting = ["cortex-m-semihosting"]
```

## Linker Configuration

Additional linker arguments in .cargo/config.toml:

```toml
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
rustflags = [
    # Linker arguments
    "-C", "link-arg=-Tlink.x",
    "-C", "link-arg=--nmagic",
    
    # Target features
    "-C", "target-cpu=cortex-r5",
    
    # Optimization flags
    "-C", "inline-threshold=5",
    "-C", "no-vectorize-loops",
]

[target.thumbv7em-none-eabihf]
rustflags = [
    "-C", "link-arg=-Tlink.x",
    "-C", "target-cpu=cortex-m4",
]
```

## Build Optimization Tips

1. **Binary Size Reduction:**
   ```toml
   # In Cargo.toml
   [dependencies]
   panic-halt = "0.2"      # Minimal panic handler
   # Avoid panic-semihosting or panic-itm for production
   ```

2. **Conditional Dependencies:**
   ```toml
   [target.'cfg(cortex_r5)'.dependencies]
   cortex-r-rt = "0.1"
   
   [target.'cfg(cortex_m)'.dependencies]
   cortex-m-rt = "0.7"
   ```

3. **Build Speed:**
   ```bash
   # Use sccache for faster rebuilds
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   
   # Use mold linker (Linux)
   cargo install mold
   export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
   ```

4. **Cross-Compilation:**
   ```bash
   # Install cross for easy cross-compilation
   cargo install cross
   cross build --target armv7r-none-eabi --release
   ```