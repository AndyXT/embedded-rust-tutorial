# Safety Guarantees for Crypto

Rust's safety guarantees are particularly important for cryptographic code, where vulnerabilities can have severe security implications.

## Memory Safety in Crypto Context

### Automatic Key Zeroization



```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};


use core::fmt;
use core::mem;
use zeroize::{Zeroize, ZeroizeOnDrop};


#[derive(ZeroizeOnDrop)]
struct CryptoKey {
    material: [u8; 32],
}

impl Drop for CryptoKey {
    fn drop(&mut self) {
        // Automatically zeroize sensitive data
        self.material.zeroize();
    }
}
```

### Preventing Timing Attacks

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};
use subtle::{Choice, ConstantTimeEq};


use core::fmt;
use core::mem;


fn secure_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    // Constant-time comparison prevents timing attacks
    a.ct_eq(b).into()
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

### Thread Safety for Crypto Operations

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

use core::sync::Arc;
use core::sync::Mutex;

// Safe sharing of crypto state across threads
struct SharedCryptoState {
    counter: Arc<Mutex<u64>>,
    key: Arc<[u8; 32]>,
}

impl SharedCryptoState {
    fn increment_counter(&self) -> Result<u64, CryptoError> {
        let mut counter = self.counter.lock()
            .map_err(|_| CryptoError::LockPoisoned)?;
        
        *counter = counter.checked_add(1)
            .ok_or(CryptoError::CounterOverflow)?;
        
        Ok(*counter)
    }
}
```

## Compile-Time Safety Checks

### Type-Safe Protocol States

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;

#[derive(Debug)]
pub struct CryptoError(&'static str);


use core::mem;
use heapless::Vec;

use core::fmt;
use core::result::Result;

// Prevent using crypto context in wrong state
struct Uninitialized;
struct Initialized;

struct CryptoContext<State> {
    state: core::marker::PhantomData<State>,
    key: Option<[u8; 32]>,
}

impl CryptoContext<Uninitialized> {
    fn new() -> Self {
        Self {
            state: core::marker::PhantomData,
            key: None,
        }
    }
    
    fn initialize(self, key: [u8; 32]) -> CryptoContext<Initialized> {
        CryptoContext {
            state: core::marker::PhantomData,
            key: Some(key),
        }
    }
}

impl CryptoContext<Initialized> {
    fn encrypt(&self, data: &[u8]) -> Result<heapless::Vec<u8, 32>, CryptoError> {
        // Can only encrypt with initialized context
        let key = self.key.as_ref().unwrap();
        encrypt_with_key(key, data)
    }
}
```

**→ Related:** 
- [Type System Advantages for Security](./type-system-advantages-for-security.md) - Type-safe crypto patterns
- [Error Handling](./error-handling.md) - Safe error propagation
```