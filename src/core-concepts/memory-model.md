# Memory Model Differences {memory-model-differences}

Rust's memory model provides stronger guarantees than C while maintaining zero-cost abstractions. Understanding these differences is crucial for embedded cryptography development.

## Key Differences from C

### Memory Safety Guarantees
- **No use-after-free** - Ownership system prevents accessing freed memory
- **No double-free** - Drop trait ensures resources are freed exactly once
- **No buffer overflows** - Bounds checking on array access
- **No null pointer dereferences** - Option<T> replaces null pointers

### Stack vs Heap in Embedded



```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};


use core::fmt;
use core::mem;

// Stack allocation - preferred in embedded
let key = [0u8; 32];  // Fixed size, known at compile time

// Heap allocation - avoid in no_std embedded
// let key = vec![0u8; 32];  // Dynamic allocation, not available in no_std

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

### Memory Layout Control

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};


use core::fmt;
use core::mem;

#[repr(C)]
struct CryptoHeader {
    magic: u32,
    version: u16,
    flags: u16,
    key_id: [u8; 16],
}

#[repr(packed)]
struct PackedCryptoData {
    header: u32,
    payload: [u8; 64],
}
```

**→ Related:** [Ownership and Memory Management](./ownership.md) - Core ownership concepts
```