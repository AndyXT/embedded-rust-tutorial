# 3.2 Error Handling Without Exceptions

Rust's explicit error handling with `Result<T, E>` and `Option<T>` prevents silent failures that are catastrophic in cryptographic systems. Every error condition must be explicitly handled.

**Advanced Error Handling:**
- → [Advanced Type System Features](./advanced-types.md) - See how enums enable sophisticated error types with data

#### Comprehensive Crypto Error Types




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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CryptoError {
    // Key-related errors
    InvalidKeySize,
    WeakKey,
    KeyExpired,
    KeyNotFound,
    
    // Algorithm errors
    InvalidNonce,
    InvalidTag,
    AuthenticationFailed,
    DecryptionFailed,
    
    // Hardware errors
    HardwareNotAvailable,
    HardwareTimeout,
    HardwareError(u32),
    
    // Resource errors
    InsufficientEntropy,
    BufferTooSmall,
    QueueFull,
    OutOfMemory,
    
    // Protocol errors
    InvalidState,
    ProtocolViolation,
    HandshakeFailed,
}

impl CryptoError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            CryptoError::HardwareTimeout | 
            CryptoError::InsufficientEntropy |
            CryptoError::QueueFull => true,
            _ => false,
        }
    }
}

type CryptoResult<T> = Result<T, CryptoError>;
```

#### Error Propagation in Crypto Pipelines

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;
use aes::{Aes256, cipher::{KeyInit, BlockEncrypt, BlockDecrypt}};

#[derive(Debug)]
pub struct CryptoError(&'static str);


use core::mem;
use heapless::Vec;

use core::fmt;
use core::result::Result;

// Crypto operations with explicit error handling
fn encrypt_aes_gcm(
    key: &[u8; 32], 
    nonce: &[u8; 12], 
    plaintext: &[u8],
    aad: &[u8]
) -> CryptoResult<heapless::Vec<u8, 32>> {
    // Validate inputs explicitly
    if key.iter().all(|&b| b == 0) {
        return Err(CryptoError::WeakKey);
    }
    
    if nonce.iter().all(|&b| b == 0) {
        return Err(CryptoError::InvalidNonce);
    }
    
    // Perform encryption with error checking
    let mut cipher = Aes256Gcm::new(key)?;
    let ciphertext = cipher.encrypt(nonce, plaintext, aad)?;
    
    Ok(ciphertext)
}

// Error propagation with ? operator
fn secure_message_processing(message: &[u8]) -> CryptoResult<heapless::Vec<u8, 32>> {
    let session_key = derive_session_key()?;     // Propagates key derivation errors
    let nonce = generate_nonce()?;               // Propagates RNG errors
    let aad = compute_aad(&message)?;            // Propagates AAD computation errors
    
    let encrypted = encrypt_aes_gcm(&session_key, &nonce, message, &aad)?;
    
    Ok(encrypted)
}

// Comprehensive error handling
fn handle_crypto_pipeline(messages: &[&[u8]]) -> CryptoResult<heapless::Vec<Vec<u8, 32>>> {
    let mut results = Vec::new();
    
    for message in messages {
        match secure_message_processing(message) {
            Ok(encrypted) => {
                results.push(encrypted);
            }
            Err(CryptoError::InsufficientEntropy) => {
                // Recoverable error - wait and retry
                wait_for_entropy()?;
                let encrypted = secure_message_processing(message)?;
                results.push(encrypted);
            }
            Err(CryptoError::HardwareTimeout) => {
                // Fallback to software crypto
                let encrypted = software_encrypt_message(message)?;
                results.push(encrypted);
            }
            Err(e) if e.is_recoverable() => {
                // Generic recoverable error handling
                retry_with_backoff(|| secure_message_processing(message))?;
            }
            Err(e) => {
                // Non-recoverable error
                log_crypto_error(&e);
                return Err(e);
            }
        }
    }
    
    Ok(results)
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

#### Option Types for Safe Nullable Crypto State

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

// Safe handling of optional crypto contexts
struct CryptoManager {
    active_sessions: heapless::FnvIndexMap<u32, CryptoContext, 16>,
}

impl CryptoManager {
    fn get_session(&mut self, session_id: u32) -> Option<&mut CryptoContext> {
        self.active_sessions.get_mut(&session_id)
    }
    
    fn process_message(&mut self, session_id: u32, message: &[u8]) -> CryptoResult<heapless::Vec<u8, 32>> {
        match self.get_session(session_id) {
            Some(ctx) => {
                // Session exists, safe to use
                ctx.encrypt(message)
            }
            None => {
                // No session found, handle gracefully
                Err(CryptoError::InvalidState)
            }
        }
    }
    
    fn cleanup_expired_sessions(&mut self) {
        self.active_sessions.retain(|_, ctx| !ctx.is_expired());
    }
}

// Chaining Option operations
fn find_and_use_key(key_id: u32) -> Option<heapless::Vec<u8, 32>> {
    get_key_from_storage(key_id)?     // Returns None if key not found
        .validate()?                  // Returns None if key invalid
        .decrypt_with_master_key()    // Returns None if decryption fails
}
```

#### Error Recovery Patterns for Embedded Crypto

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

// Retry with exponential backoff for transient errors
fn retry_with_backoff<F, T>(mut operation: F) -> CryptoResult<T>
where
    F: FnMut() -> CryptoResult<T>,
{
    const MAX_RETRIES: u32 = 3;
    let mut delay_ms = 10;
    
    for attempt in 0..MAX_RETRIES {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if e.is_recoverable() && attempt < MAX_RETRIES - 1 => {
                // Wait before retry
                cortex_m::asm::delay(delay_ms * 1000);
                delay_ms *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
    
    Err(CryptoError::HardwareTimeout)
}

// Graceful degradation for hardware failures
fn encrypt_with_fallback(data: &[u8], key: &[u8; 32]) -> CryptoResult<heapless::Vec<u8, 32>> {
    // Try hardware acceleration first
    match hardware_encrypt(data, key) {
        Ok(result) => Ok(result),
        Err(CryptoError::HardwareNotAvailable) => {
            // Fall back to software implementation
            software_encrypt(data, key)
        }
        Err(e) => Err(e), // Propagate other errors
    }
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```