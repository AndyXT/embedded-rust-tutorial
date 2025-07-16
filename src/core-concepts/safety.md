# Safety Guarantees for Crypto

Rust's safety guarantees are particularly important for cryptographic code, where vulnerabilities can have severe security implications.

## Memory Safety in Crypto Context

### Automatic Key Zeroization
```rust
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
use subtle::ConstantTimeEq;

fn secure_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    // Constant-time comparison prevents timing attacks
    a.ct_eq(b).into()
}
```

### Thread Safety for Crypto Operations
```rust
use std::sync::Arc;
use std::sync::Mutex;

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
// Prevent using crypto context in wrong state
struct Uninitialized;
struct Initialized;

struct CryptoContext<State> {
    state: std::marker::PhantomData<State>,
    key: Option<[u8; 32]>,
}

impl CryptoContext<Uninitialized> {
    fn new() -> Self {
        Self {
            state: std::marker::PhantomData,
            key: None,
        }
    }
    
    fn initialize(self, key: [u8; 32]) -> CryptoContext<Initialized> {
        CryptoContext {
            state: std::marker::PhantomData,
            key: Some(key),
        }
    }
}

impl CryptoContext<Initialized> {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Can only encrypt with initialized context
        let key = self.key.as_ref().unwrap();
        encrypt_with_key(key, data)
    }
}
```

**→ Related:** 
- [Type System Advantages for Security](./type-system-advantages-for-security.md) - Type-safe crypto patterns
- [Error Handling](./error-handling.md) - Safe error propagation