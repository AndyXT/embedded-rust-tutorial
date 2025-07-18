# Type System Overview

Rust's type system provides compile-time guarantees that prevent entire classes of bugs common in C embedded development.

## Zero-Cost Abstractions

Rust's type system enables powerful abstractions without runtime overhead:

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};


use core::fmt;
use core::mem;

// Generic function - no runtime cost
fn encrypt_block<const N: usize>(data: &mut [u8; N], key: &[u8; 32]) {
    // Implementation is monomorphized at compile time
    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key[i % 32];
    }
}

// Usage - compiler generates specific versions
encrypt_block(&mut [0u8; 16], &key);  // Generates version for N=16
encrypt_block(&mut [0u8; 32], &key);  // Generates version for N=32

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

## Const Generics for Crypto

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};

#[derive(Debug)]
pub struct CryptoError(&'static str);


use core::mem;
use core::fmt;

use core::result::Result;

// Type-safe buffer sizes
struct CryptoBuffer<const SIZE: usize> {
    data: [u8; SIZE],
    used: usize,
}

impl<const SIZE: usize> CryptoBuffer<SIZE> {
    fn new() -> Self {
        Self {
            data: [0; SIZE],
            used: 0,
        }
    }
    
    fn push(&mut self, byte: u8) -> Result<(), CryptoError> {
        if self.used >= SIZE {
            return Err(CryptoError::BufferFull);
        }
        self.data[self.used] = byte;
        self.used += 1;
        Ok(())
    }
}

// Different buffer sizes are different types
type SmallBuffer = CryptoBuffer<64>;
type LargeBuffer = CryptoBuffer<1024>;

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

## Trait System for Algorithm Abstraction

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use aes::{Aes256, cipher::{KeyInit, BlockEncrypt, BlockDecrypt}};

#[derive(Debug)]
pub struct CryptoError(&'static str);


use core::mem;
use core::fmt;

use core::result::Result;

trait Cipher {
    const KEY_SIZE: usize;
    const BLOCK_SIZE: usize;
    
    fn encrypt_block(&self, key: &[u8], block: &mut [u8]) -> Result<(), CryptoError>;
    fn decrypt_block(&self, key: &[u8], block: &mut [u8]) -> Result<(), CryptoError>;
}

struct Aes128;
impl Cipher for Aes128 {
    const KEY_SIZE: usize = 16;
    const BLOCK_SIZE: usize = 16;
    
    fn encrypt_block(&self, key: &[u8], block: &mut [u8]) -> Result<(), CryptoError> {
        if key.len() != Self::KEY_SIZE || block.len() != Self::BLOCK_SIZE {
            return Err(CryptoError::InvalidSize);
        }
        // AES-128 implementation
        Ok(())
    }
    
    fn decrypt_block(&self, key: &[u8], block: &mut [u8]) -> Result<(), CryptoError> {
        if key.len() != Self::KEY_SIZE || block.len() != Self::BLOCK_SIZE {
            return Err(CryptoError::InvalidSize);
        }
        // AES-128 implementation
        Ok(())
    }
}

// Generic function works with any cipher
fn process_data<C: Cipher>(cipher: &C, key: &[u8], data: &mut [u8]) -> Result<(), CryptoError> {
    if key.len() != C::KEY_SIZE {
        return Err(CryptoError::InvalidKeySize);
    }
    
    for chunk in data.chunks_exact_mut(C::BLOCK_SIZE) {
        cipher.encrypt_block(key, chunk)?;
    }
    
    Ok(())
}
```

**→ Related:**
- [Type System Advantages for Security](./type-system-advantages-for-security.md) - Security-focused type patterns
- [Advanced Type System Features](./advanced-types.md) - Advanced patterns and techniques
```