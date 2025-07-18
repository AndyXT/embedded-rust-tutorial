# 2.4 Build Configuration




```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};

use core::mem;
use core::fmt;
use core::result::Result;
// build.rs - Hardware-specific optimizations
use core::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    
    // Enable hardware features based on target
    match target.as_str() {
        t if t.contains("thumbv8m") => {
            defmt::info!("cargo:rustc-cfg=feature=\"hw_crypto\"");
            defmt::info!("cargo:rustc-cfg=feature=\"trustzone\"");
        }
        t if t.contains("cortex-r5") => {
            defmt::info!("cargo:rustc-cfg=feature=\"xilinx_r5\"");
        }
        _ => {}
    }
}
```