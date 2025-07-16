# 2.4 Build Configuration

```rust
// build.rs - Hardware-specific optimizations
use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    
    // Enable hardware features based on target
    match target.as_str() {
        t if t.contains("thumbv8m") => {
            println!("cargo:rustc-cfg=feature=\"hw_crypto\"");
            println!("cargo:rustc-cfg=feature=\"trustzone\"");
        }
        t if t.contains("cortex-r5") => {
            println!("cargo:rustc-cfg=feature=\"xilinx_r5\"");
        }
        _ => {}
    }
}
```