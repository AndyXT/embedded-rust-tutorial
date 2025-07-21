# 2.5 Verification and Testing

## Step-by-Step Setup Verification

```bash
# 1. Verify installation
rustc --version && cargo --version

# 2. Test target compilation
cargo new --bin setup-test && cd setup-test
echo 'cortex-m = "0.7"' >> Cargo.toml
echo 'cortex-m-rt = "0.7"' >> Cargo.toml
echo 'panic-halt = "0.2"' >> Cargo.toml

# 3. Build for your target
cargo build --target thumbv7em-none-eabihf --release

# 4. Verify binary size (should be small)
cargo size --target thumbv7em-none-eabihf --release

# 5. Test crypto compilation
echo 'sha2 = { version = "0.10", default-features = false }' >> Cargo.toml
cargo build --target thumbv7em-none-eabihf --release
```

## Minimal Verification Application




```rust
#![no_std]
#![no_main]

use core::{fmt};

use core::fmt;
use core::mem;
// src/main.rs - Setup verification test

use panic_halt as _;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // Basic functionality test
    let test_data = [0x42u8; 32];
    let mut result = 0u8;
    
    // Simple crypto-like operation
    for &byte in &test_data {
        result ^= byte;
    }
    
    // If this compiles and the result is 0x42, setup is working
    assert_eq!(result, 0x42);
    
    loop {
        cortex_m::asm::wfi();
    }
}
```

**✅ Setup Complete** - Your environment is ready for embedded Rust crypto development.

**→ Next:** [Core Language Concepts](../core-concepts/README.md) - Essential Rust concepts for crypto development
```