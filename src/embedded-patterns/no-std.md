# 4.1 No-std Programming Essentials

No-std programming is fundamental to embedded Rust. This section consolidates all no-std concepts and patterns from across the document into a single authoritative reference.

For memory management concepts, see [Memory Model Differences](../core-concepts/memory-model.md). For error handling patterns, refer to [Error Handling Patterns](../quick-reference/error-handling.md).

## Complete No-std Project Template




`````rust
#![no_std]
#![no_main]
#![forbid(unsafe_code)]  // Optional: forbid unsafe except in specific modules

// Essential imports for no-std embedded crypto
use panic_halt as _;
use cortex_m_rt::entry;

// Core library - always available in no-std
use core::{
    mem,                    // Memory utilities (size_of, align_of, etc.)
    ptr,                    // Pointer operations (read_volatile, write_volatile)
    slice,                  // Slice operations
    fmt::Write,             // Formatting without heap allocation
    convert::TryInto,       // Fallible conversions
    ops::{Deref, DerefMut}, // Smart pointer traits
    result::Result,         // Result type for error handling
};

// Heapless collections - essential for no-std
use heapless::{
    Vec,                    // Stack-allocated vector
    String,                 // Stack-allocated string  
    FnvIndexMap,           // Hash map without heap
    spsc::{Queue, Producer, Consumer}, // Lock-free queues
    pool::{Pool, Node},     // Memory pools
};

// Crypto dependencies (all no-std compatible)
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use sha2::{Sha256, Digest};
use zeroize::{Zeroize, ZeroizeOnDrop};
use subtle::{Choice, ConstantTimeEq};

#[entry]
fn main() -> ! {
    // Initialize hardware and crypto systems
    let mut crypto_hw = init_crypto_hardware();
    let mut crypto_ctx = CryptoContext::new();
    
    // Initialize static memory pools
    init_memory_pools();
    
    // Enable interrupts for crypto operations
    unsafe { cortex_m::interrupt::enable() };
    
    // Main application loop
    loop {
        process_crypto_requests(&mut crypto_hw, &mut crypto_ctx);
        handle_crypto_interrupts();
        cortex_m::asm::wfi(); // Wait for interrupt
    }
}

// Simple panic handler for examples - halts on panic
use panic_halt as _; // Links the panic-halt crate

// For production code with security requirements:
// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     // Disable interrupts
//     cortex_m::interrupt::disable();
//     
//     // Clear sensitive data if needed
//     // Note: Implement clear_crypto_memory() based on your needs
//     
//     // Reset system
//     cortex_m::peripheral::SCB::sys_reset();
// }
``
```

## No-std Memory Management Patterns

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


use zeroize::{Zeroize, ZeroizeOnDrop};
use core::mem;
use heapless::Vec;

use core::fmt;
use core::result::Result;

use heapless::{Vec, pool::{Pool, Node, Singleton}};

// Static memory pools for different crypto operations
static mut SMALL_BLOCKS: [Node<[u8; 256]>; 16] = [Node::new(); 16];
static mut LARGE_BLOCKS: [Node<[u8; 4096]>; 4] = [Node::new(); 4];
static mut CRYPTO_CONTEXTS: [Node<CryptoContext>; 8] = [Node::new(); 8];

static SMALL_POOL: Pool<[u8; 256]> = Pool::new();
static LARGE_POOL: Pool<[u8; 4096]> = Pool::new();
static CONTEXT_POOL: Pool<CryptoContext> = Pool::new();

// Singleton for global crypto state (only one instance allowed)
static mut GLOBAL_CRYPTO_STATE: [u8; 2048] = [0; 2048];
static CRYPTO_STATE: Singleton<GlobalCryptoState> = Singleton::new();

fn init_memory_pools() {
    unsafe {
        SMALL_POOL.grow(&mut SMALL_BLOCKS);
        LARGE_POOL.grow(&mut LARGE_BLOCKS);
        CONTEXT_POOL.grow(&mut CRYPTO_CONTEXTS);
    }
}

// Memory-efficient crypto operations using pools
fn encrypt_with_pool(plaintext: &[u8]) -> Result<heapless::Vec<u8, 4096>, CryptoError> {
    // Choose appropriate pool based on data size
    if plaintext.len() <= 256 {
        let mut buffer = SMALL_POOL.alloc().ok_or(CryptoError::OutOfMemory)?;
        buffer[..plaintext.len()].copy_from_slice(plaintext);
        
        // Perform in-place encryption
        encrypt_in_place(&mut buffer[..plaintext.len()])?;
        
        // Copy result to heapless Vec
        let mut result = heapless::Vec::new();
        result.extend_from_slice(&buffer[..plaintext.len()])
            .map_err(|_| CryptoError::BufferTooSmall)?;
        
        Ok(result)
    } else if plaintext.len() <= 4096 {
        let mut buffer = LARGE_POOL.alloc().ok_or(CryptoError::OutOfMemory)?;
        buffer[..plaintext.len()].copy_from_slice(plaintext);
        
        encrypt_in_place(&mut buffer[..plaintext.len()])?;
        
        let mut result = heapless::Vec::new();
        result.extend_from_slice(&buffer[..plaintext.len()])
            .map_err(|_| CryptoError::BufferTooSmall)?;
        
        Ok(result)
    } else {
        Err(CryptoError::DataTooLarge)
    }
}

// Stack-based collections for crypto state management
#[derive(ZeroizeOnDrop)]
struct CryptoSession {
    active_keys: heapless::Vec<[u8; 32], 8, 32>,        // Max 8 active keys
    message_queue: heapless::Vec<CryptoMessage, 32, 32>, // Max 32 queued messages
    nonce_counters: heapless::Vec<u64, 8, 32>,           // Nonce counter per key
    session_id: u32,
}

impl CryptoSession {
    fn new(session_id: u32) -> Self {
        Self {
            active_keys: Vec::new(),
            message_queue: Vec::new(),
            nonce_counters: Vec::new(),
            session_id,
        }
    }
    
    fn add_key(&mut self, key: [u8; 32]) -> Result<usize, CryptoError> {
        let key_index = self.active_keys.len();
        
        self.active_keys.push(key)
            .map_err(|_| CryptoError::TooManyKeys)?;
        self.nonce_counters.push(0)
            .map_err(|_| CryptoError::TooManyKeys)?;
            
        Ok(key_index)
    }
    
    fn encrypt_message(&mut self, key_index: usize, plaintext: &[u8]) -> Result<heapless::Vec<u8, 4096, 32>, CryptoError> {
        let key = self.active_keys.get(key_index).ok_or(CryptoError::InvalidKeyIndex)?;
        let nonce_counter = self.nonce_counters.get_mut(key_index).ok_or(CryptoError::InvalidKeyIndex)?;
        
        // Increment nonce counter (prevents reuse)
        *nonce_counter += 1;
        let nonce_bytes = nonce_counter.to_le_bytes();
        
        // Perform encryption with automatic nonce management
        chacha20poly1305_encrypt(key, &nonce_bytes, plaintext)
    }
}

// Compile-time memory layout with const generics
struct CryptoWorkspace<const BUFFER_SIZE: usize, const KEY_COUNT: usize> {
    buffers: [[u8; BUFFER_SIZE]; 4],
    keys: [[u8; 32]; KEY_COUNT],
    nonces: [[u8; 12]; KEY_COUNT],
    in_use: [bool; 4], // Track buffer usage
}

impl<const BUFFER_SIZE: usize, const KEY_COUNT: usize> CryptoWorkspace<BUFFER_SIZE, KEY_COUNT> {
    const fn new() -> Self {
        Self {
            buffers: [[0; BUFFER_SIZE]; 4],
            keys: [[0; 32]; KEY_COUNT],
            nonces: [[0; 12]; KEY_COUNT],
            in_use: [false; 4],
        }
    }
    
    fn get_free_buffer(&mut self) -> Option<(usize, &mut [u8; BUFFER_SIZE])> {
        for (i, in_use) in self.in_use.iter_mut().enumerate() {
            if !*in_use {
                *in_use = true;
                return Some((i, &mut self.buffers[i]));
            }
        }
        None
    }
    
    fn release_buffer(&mut self, index: usize) {
        if index < 4 {
            self.in_use[index] = false;
            // Zeroize buffer on release for security
            self.buffers[index].zeroize();
        }
    }
}

// Different workspace configurations for different applications
type SmallCryptoWorkspace = CryptoWorkspace<256, 2>;   // 2KB buffers, 2 keys
type LargeCryptoWorkspace = CryptoWorkspace<4096, 8>;  // 16KB buffers, 8 keys

static mut CRYPTO_WS: SmallCryptoWorkspace = SmallCryptoWorkspace::new();
```

## No-std Error Handling and Result Types

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

// Custom error types that work in no-std
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoError {
    InvalidKey,
    InvalidNonce,
    EncryptionFailed,
    DecryptionFailed,
    AuthenticationFailed,
    OutOfMemory,
    BufferTooSmall,
    DataTooLarge,
    TooManyKeys,
    InvalidKeyIndex,
    HardwareError(u32),
}

// No-std compatible Result type
pub type CryptoResult<T> = core::result::Result<T, CryptoError>;

// Error handling without core::error::Error trait
impl CryptoError {
    pub fn description(&self) -> &'static str {
        match self {
            CryptoError::InvalidKey => "Invalid cryptographic key",
            CryptoError::InvalidNonce => "Invalid nonce value",
            CryptoError::EncryptionFailed => "Encryption operation failed",
            CryptoError::DecryptionFailed => "Decryption operation failed",
            CryptoError::AuthenticationFailed => "Authentication verification failed",
            CryptoError::OutOfMemory => "Insufficient memory available",
            CryptoError::BufferTooSmall => "Buffer too small for operation",
            CryptoError::DataTooLarge => "Data exceeds maximum size",
            CryptoError::TooManyKeys => "Maximum number of keys exceeded",
            CryptoError::InvalidKeyIndex => "Invalid key index specified",
            CryptoError::HardwareError(_) => "Hardware crypto accelerator error",
        }
    }
    
    pub fn is_recoverable(&self) -> bool {
        match self {
            CryptoError::OutOfMemory | CryptoError::TooManyKeys => true,
            _ => false,
        }
    }
}

// No-std compatible formatting
impl core::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.description())
    }
}
```