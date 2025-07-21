# 3.2 Error Handling Without Exceptions

Rust's explicit error handling with `Result<T, E>` and `Option<T>` prevents silent failures that are catastrophic in cryptographic systems. Every error condition must be explicitly handled.

**Advanced Error Handling:**
- → [Advanced Type System Features](./advanced-types.md) - See how enums enable sophisticated error types with data

## Comprehensive Crypto Error Types




```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;

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

## Error Propagation in Crypto Pipelines

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use heapless::{Vec, consts::*};

type Vec32<T> = Vec<T, U32>;

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

type CryptoResult<T> = Result<T, CryptoError>;

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

// Stub types and functions for the example
struct Aes256Gcm;

impl Aes256Gcm {
    fn new(_key: &[u8; 32]) -> CryptoResult<Self> {
        Ok(Self)
    }
    
    fn encrypt(&mut self, _nonce: &[u8; 12], _plaintext: &[u8], _aad: &[u8]) -> CryptoResult<Vec<u8, U32>> {
        let mut result = Vec::new();
        result.extend_from_slice(b"encrypted").map_err(|_| CryptoError::BufferTooSmall)?;
        Ok(result)
    }
}

fn derive_session_key() -> CryptoResult<[u8; 32]> {
    Ok([0u8; 32])
}

fn generate_nonce() -> CryptoResult<[u8; 12]> {
    Ok([0u8; 12])
}

fn compute_aad(_message: &[u8]) -> CryptoResult<[u8; 16]> {
    Ok([0u8; 16])
}

fn wait_for_entropy() -> CryptoResult<()> {
    Ok(())
}

fn software_encrypt_message(_message: &[u8]) -> CryptoResult<Vec<u8, U32>> {
    let mut result = Vec::new();
    result.extend_from_slice(b"sw_encrypted").map_err(|_| CryptoError::BufferTooSmall)?;
    Ok(result)
}

fn retry_with_backoff<F, T>(mut f: F) -> CryptoResult<T>
where
    F: FnMut() -> CryptoResult<T>,
{
    f()
}

fn log_crypto_error(_e: &CryptoError) {}

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
fn handle_crypto_pipeline(messages: &[&[u8]]) -> CryptoResult<Vec<Vec<u8, U32>, U32>> {
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

## Option Types for Safe Nullable Crypto State

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use heapless::{Vec, FnvIndexMap, consts::*};

type Vec32<T> = Vec<T, U32>;

#[derive(Debug)]
struct CryptoError(&'static str);

type CryptoResult<T> = Result<T, CryptoError>;

// Define CryptoError variants
impl CryptoError {
    const InvalidState: Self = CryptoError("Invalid state");
}

// Define missing types
struct CryptoContext {
    created_at: u64,
}

impl CryptoContext {
    fn encrypt(&mut self, _message: &[u8]) -> CryptoResult<Vec<u8, U32>> {
        let mut result = Vec::new();
        result.extend_from_slice(b"encrypted").map_err(|_| CryptoError("Buffer full"))?;
        Ok(result)
    }
    
    fn is_expired(&self) -> bool {
        false // Stub implementation
    }
}

struct Key {
    data: [u8; 32],
}

impl Key {
    fn validate(self) -> Option<Self> {
        Some(self)
    }
    
    fn decrypt_with_master_key(self) -> Option<Vec<u8, U32>> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.data).ok()?;
        Some(result)
    }
}

fn get_key_from_storage(_key_id: u32) -> Option<Key> {
    Some(Key { data: [0u8; 32] })
}

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

#[entry]
fn main() -> ! {
    // Example usage
    let mut manager = CryptoManager {
        active_sessions: FnvIndexMap::new(),
    };
    let _ = manager.process_message(1, b"test");
    
    loop {}
}
```

## Error Recovery Patterns for Embedded Crypto

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use heapless::{Vec, consts::*};

type Vec32<T> = Vec<T, U32>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CryptoError {
    HardwareNotAvailable,
    HardwareTimeout,
    InsufficientEntropy,
}

impl CryptoError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            CryptoError::HardwareTimeout | 
            CryptoError::InsufficientEntropy => true,
            _ => false,
        }
    }
}

type CryptoResult<T> = Result<T, CryptoError>;

// Stub functions for the example
fn hardware_encrypt(_data: &[u8], _key: &[u8; 32]) -> CryptoResult<Vec<u8, U32>> {
    let mut result = Vec::new();
    result.extend_from_slice(b"hw_encrypted").map_err(|_| CryptoError::HardwareTimeout)?;
    Ok(result)
}

fn software_encrypt(_data: &[u8], _key: &[u8; 32]) -> CryptoResult<Vec<u8, U32>> {
    let mut result = Vec::new();
    result.extend_from_slice(b"sw_encrypted").map_err(|_| CryptoError::HardwareTimeout)?;
    Ok(result)
}

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
                cortex_r::asm::nop(); // Simple delay - in real code use timer
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