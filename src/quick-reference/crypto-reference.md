# 1.5 Crypto-Specific Quick Reference

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `uint8_t key[32]; /* clear on exit */` | `let key = SecureKey::<32>::new(data);` | Automatic zeroization via `Drop` |
| `if (memcmp(a, b, len) == 0)` | `constant_time_eq(a, b)` | Timing-attack resistant comparison |
| `volatile uint8_t* reg = 0x40000000;` | `unsafe { ptr::read_volatile(reg) }` | Hardware register access |
| `#ifdef CRYPTO_HW_ACCEL` | `#[cfg(feature = "hw-crypto")]` | Conditional compilation |
| `RAND_bytes(buffer, len)` | `rng.fill_bytes(&mut buffer)` | Cryptographically secure RNG |
| `explicit_bzero(key, sizeof(key))` | `key.zeroize()` | Secure memory clearing |

#### Secure Key Management Example




```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};


use core::fmt;
use core::mem;
use zeroize::{Zeroize, ZeroizeOnDrop};


#[derive(ZeroizeOnDrop)]
struct SecureKey<const N: usize> {
    key: [u8; N],
}

impl<const N: usize> SecureKey<N> {
    fn new(key_material: [u8; N]) -> Self {
        Self { key: key_material }
    }
    
    fn as_bytes(&self) -> &[u8; N] {
        &self.key
    }
}

// Automatically zeroized when dropped
let key = SecureKey::<32>::new(key_data);
// Use key...
// key is automatically zeroized here when it goes out of scope

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```