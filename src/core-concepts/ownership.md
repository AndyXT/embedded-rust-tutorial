# 3.1 Ownership and Memory Management

<details>
<summary><strong>▶️ Ownership Rules Summary</strong> - The three fundamental rules</summary>

### The Three Rules of Ownership
1. **Each value has exactly one owner** - No shared ownership without explicit mechanisms
2. **When the owner goes out of scope, the value is dropped** - Automatic cleanup with Drop trait
3. **Only one mutable reference OR multiple immutable references** - Prevents data races and use-after-free

**Why this matters for crypto:**
- Automatic key zeroization when variables go out of scope
- Compile-time prevention of use-after-free vulnerabilities
- No data races on shared cryptographic state
- Clear ownership of sensitive data throughout the program

</details>

Rust's ownership system replaces C's manual memory management with compile-time rules that prevent entire classes of crypto vulnerabilities. This is the single most important concept for C programmers to master.

## Ownership in Embedded Crypto Context




```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use core::mem;
use heapless::{Vec, String, consts::*};
use zeroize::{Zeroize, ZeroizeOnDrop};

type Vec32<T> = Vec<T, U32>;

#[derive(Debug)]
struct CryptoError(&'static str);

// Define missing types
struct AesState {
    initialized: bool,
}

impl AesState {
    fn new() -> Self {
        Self { initialized: true }
    }
}

// Stub functions for the example
fn aes_gcm_encrypt(_key: &[u8; 32], _nonce: &[u8], _plaintext: &[u8]) -> Result<Vec<u8, U32>, CryptoError> {
    let mut result = Vec::new();
    result.extend_from_slice(b"encrypted").map_err(|_| CryptoError("Buffer full"))?;
    Ok(result)
}

fn generate_session_key() -> Result<[u8; 32], CryptoError> {
    Ok([0u8; 32])
}

fn transmit_messages(_messages: &[Vec<u8, U32>]) -> Result<(), CryptoError> {
    Ok(())
}

// Define the error cases
impl CryptoError {
    const BufferTooSmall: Self = CryptoError("Buffer too small");
    const InvalidNonce: Self = CryptoError("Invalid nonce");
}

#[derive(ZeroizeOnDrop)]
struct CryptoContext {
    session_key: [u8; 32],    // Owned key material
    nonce_counter: u64,       // Owned state
    cipher_state: AesState,   // Owned cipher context
}

impl CryptoContext {
    fn new(key: [u8; 32]) -> Self {
        Self {
            session_key: key,  // key ownership transferred here
            nonce_counter: 0,
            cipher_state: AesState::new(),
        }
    }
    
    fn encrypt(&mut self, plaintext: &[u8]) -> Result<heapless::Vec<u8, 32>, CryptoError> {
        // Check plaintext size (16 bytes for auth tag overhead)
        const MAX_PLAINTEXT_SIZE: usize = 32 - 16;
        if plaintext.len() > MAX_PLAINTEXT_SIZE {
            return Err(CryptoError::BufferTooSmall);
        }
        
        // Prevent nonce overflow (important for security)
        self.nonce_counter = self.nonce_counter
            .checked_add(1)
            .ok_or(CryptoError::InvalidNonce)?;
        
        // Use owned key material safely
        let nonce = self.nonce_counter.to_le_bytes();
        aes_gcm_encrypt(&self.session_key, &nonce, plaintext)
    }
}

fn secure_communication() -> Result<(), CryptoError> {
    let master_key = generate_session_key()?;
    let mut crypto_ctx = CryptoContext::new(master_key);
    // master_key is moved, can't be accidentally reused
    
    let message1 = crypto_ctx.encrypt(b"first message")?;
    let message2 = crypto_ctx.encrypt(b"second message")?;
    // Nonce automatically incremented, no reuse possible
    
    transmit_messages(&[message1, message2])?;
    
    // crypto_ctx dropped here, session_key automatically zeroized
    Ok(())
}

#[entry]
fn main() -> ! {
    // Example usage
    let _ = secure_communication();
    loop {}
}
```

## Borrowing Rules for Crypto Operations

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use heapless::{Vec, consts::*};

type Vec32<T> = Vec<T, U32>;

#[derive(Debug)]
struct CryptoError(&'static str);

// Define missing types and functions
struct CryptoContext {
    nonce_counter: u64,
    cipher_state: CipherState,
}

struct CipherState;
impl CipherState {
    fn update_key_schedule(&mut self, _key: &[u8; 32]) {}
}

struct CryptoParams {
    starting_nonce: u64,
    key: [u8; 32],
}

fn verify_signature(_public_key: &[u8; 32], _msg: &[u8], _sig: &[u8; 64]) -> bool {
    true // Stub implementation
}

fn hkdf_expand(_master_key: &[u8; 32], _salt: &[u8; 16], _info: &[u8]) -> [u8; 32] {
    [0u8; 32] // Stub implementation
}

fn perform_crypto_operations(_enc_key: &[u8; 32], _mac_key: &[u8; 32]) -> Result<(), CryptoError> {
    Ok(())
}

// Immutable borrowing - safe concurrent reads
fn verify_multiple_signatures(
    public_key: &[u8; 32],     // Immutable borrow
    messages: &[&[u8]],        // Multiple immutable borrows allowed
    signatures: &[[u8; 64]]
) -> heapless::Vec<bool, 32> {
    messages.iter()
        .zip(signatures.iter())
        .map(|(msg, sig)| verify_signature(public_key, msg, sig))
        .collect()
}

// Mutable borrowing - exclusive access for state updates
fn update_crypto_state(ctx: &mut CryptoContext, new_params: CryptoParams) {
    // Exclusive access ensures no race conditions
    ctx.nonce_counter = new_params.starting_nonce;
    ctx.cipher_state.update_key_schedule(&new_params.key);
    // No other code can access ctx during this function
}

// Lifetime management for key derivation
struct KeyDerivationContext<'a> {
    master_key: &'a [u8; 32],  // Borrowed master key
    salt: &'a [u8; 16],        // Borrowed salt
}

impl<'a> KeyDerivationContext<'a> {
    fn derive_key(&self, info: &[u8]) -> [u8; 32] {
        // Compiler ensures master_key and salt remain valid
        hkdf_expand(self.master_key, self.salt, info)
    }
}

// Usage ensures master key outlives derived keys
fn key_hierarchy_example() -> Result<(), CryptoError> {
    let master_key = [0u8; 32];  // Stack allocated
    let salt = [1u8; 16];        // Stack allocated
    
    let kdf_ctx = KeyDerivationContext {
        master_key: &master_key,
        salt: &salt,
    };
    
    let encryption_key = kdf_ctx.derive_key(b"encryption");
    let mac_key = kdf_ctx.derive_key(b"authentication");
    
    // master_key and salt guaranteed valid throughout this scope
    perform_crypto_operations(&encryption_key, &mac_key)?;
    
    Ok(())
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

## Memory Management Patterns for Embedded

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use critical_section::Mutex;
use core::cell::RefCell;
use heapless::{Vec, consts::*};

type Vec256<T> = Vec<T, U256>;

#[derive(Debug)]
struct CryptoError(&'static str);

// Define missing types
struct AesState {
    initialized: bool,
}

impl AesState {
    fn new() -> Self {
        Self { initialized: true }
    }
    
    fn set_key(&mut self, _key: &[u8; 32]) {}
    
    fn encrypt_block(&mut self, _block: &mut [u8; 16]) -> Result<(), CryptoError> {
        Ok(())
    }
}

struct CryptoContext {
    key: [u8; 32],
}

impl CryptoContext {
    fn new(key: [u8; 32]) -> Self {
        Self { key }
    }
}

// Stack-based crypto operations (preferred in embedded)
fn stack_crypto_operations() -> Result<[u8; 16], CryptoError> {
    let mut plaintext = [0u8; 16];      // Stack allocated
    let key = [0u8; 32];                // Stack allocated
    let mut cipher_state = AesState::new(); // Stack allocated
    
    // All operations use stack memory
    cipher_state.set_key(&key);
    cipher_state.encrypt_block(&mut plaintext)?;
    
    Ok(plaintext)
    // All memory automatically cleaned up
}

// Safe global crypto state using critical sections

static GLOBAL_CRYPTO_CTX: Mutex<RefCell<Option<CryptoContext>>> = 
    Mutex::new(RefCell::new(None));

fn init_global_crypto(key: [u8; 32]) {
    critical_section::with(|cs| {
        let mut ctx = GLOBAL_CRYPTO_CTX.borrow(cs).borrow_mut();
        *ctx = Some(CryptoContext::new(key));
    });
}

fn use_global_crypto() -> Result<(), CryptoError> {
    critical_section::with(|cs| {
        let mut ctx = GLOBAL_CRYPTO_CTX.borrow(cs).borrow_mut();
        match ctx.as_mut() {
            Some(crypto) => {
                // Use crypto context safely
                crypto.process_data()
            }
            None => Err(CryptoError::NotInitialized),
        }
    })
}

// Define missing functions
fn encrypt_message(plaintext: &[u8; 16]) -> Result<[u8; 32], CryptoError> {
    let mut result = [0u8; 32];
    result[..16].copy_from_slice(plaintext);
    Ok(result)
}

fn transmit_encrypted_message(_message: &[u8; 32]) -> Result<(), CryptoError> {
    Ok(())
}

// Define error variant
impl CryptoError {
    const QueueFull: Self = CryptoError("Queue full");
}

// Heapless collections for message queues
fn message_queue_example() -> Result<(), CryptoError> {
    let mut encrypted_messages: Vec<[u8; 32], U256> = Vec::new();
    
    for i in 0..10 {
        let plaintext = [i; 16];
        let ciphertext = encrypt_message(&plaintext)?;
        encrypted_messages.push(ciphertext)
            .map_err(|_| CryptoError::QueueFull)?;
    }
    
    // Process messages without heap allocation
    for message in encrypted_messages.iter() {
        transmit_encrypted_message(message)?;
    }
    
    Ok(())
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```