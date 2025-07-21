# 4.4 Static Memory Management

This section consolidates all static memory management patterns for deterministic embedded crypto applications, providing comprehensive coverage of memory pool management, compile-time allocation, and secure memory handling.

## Advanced Static Memory Pool Management

Embedded systems require deterministic memory allocation. This example shows how to use static memory pools for crypto operations.

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use heapless::pool::{Pool, Node};
use heapless::Vec;
use heapless::consts::*;
use core::mem::MaybeUninit;
use zeroize::Zeroize;

// Comprehensive static memory pools for different crypto operations
static mut TINY_BLOCKS: [Node<[u8; 64]>; 32] = [Node::new(); 32];      // For keys, nonces
static mut SMALL_BLOCKS: [Node<[u8; 256]>; 16] = [Node::new(); 16];    // For small messages
static mut MEDIUM_BLOCKS: [Node<[u8; 1024]>; 8] = [Node::new(); 8];    // For medium messages
static mut LARGE_BLOCKS: [Node<[u8; 4096]>; 4] = [Node::new(); 4];     // For large messages
static mut CRYPTO_CONTEXTS: [Node<CryptoContext>; 8] = [Node::new(); 8]; // For crypto contexts

// Memory pools with different characteristics
static TINY_POOL: Pool<[u8; 64]> = Pool::new();
static SMALL_POOL: Pool<[u8; 256]> = Pool::new();
static MEDIUM_POOL: Pool<[u8; 1024]> = Pool::new();
static LARGE_POOL: Pool<[u8; 4096]> = Pool::new();
static CONTEXT_POOL: Pool<CryptoContext> = Pool::new();

// Singleton for global crypto state management
static mut GLOBAL_CRYPTO_STATE_MEM: MaybeUninit<GlobalCryptoState> = MaybeUninit::uninit();
static GLOBAL_CRYPTO_STATE: Singleton<GlobalCryptoState> = Singleton::new();

// Memory pool initialization with error handling
fn init_static_memory_pools() -> Result<(), MemoryError> {
    unsafe {
        TINY_POOL.grow(&mut TINY_BLOCKS);
        SMALL_POOL.grow(&mut SMALL_BLOCKS);
        MEDIUM_POOL.grow(&mut MEDIUM_BLOCKS);
        LARGE_POOL.grow(&mut LARGE_BLOCKS);
        CONTEXT_POOL.grow(&mut CRYPTO_CONTEXTS);
    }
    
    // Initialize global crypto state
    let global_state = GlobalCryptoState::new();
    GLOBAL_CRYPTO_STATE.spawn(global_state)
        .map_err(|_| MemoryError::SingletonAlreadyInitialized)?;
    
    Ok(())
}

// Smart memory allocation based on data size and usage pattern
fn allocate_crypto_buffer(size: usize, usage: BufferUsage) -> Result<CryptoBuffer, MemoryError> {
    match (size, usage) {
        (0..=64, BufferUsage::Key | BufferUsage::Nonce) => {
            let buffer = TINY_POOL.alloc().ok_or(MemoryError::TinyPoolExhausted)?;
            Ok(CryptoBuffer::Tiny(buffer))
        }
        (0..=256, BufferUsage::SmallMessage) => {
            let buffer = SMALL_POOL.alloc().ok_or(MemoryError::SmallPoolExhausted)?;
            Ok(CryptoBuffer::Small(buffer))
        }
        (257..=1024, BufferUsage::MediumMessage) => {
            let buffer = MEDIUM_POOL.alloc().ok_or(MemoryError::MediumPoolExhausted)?;
            Ok(CryptoBuffer::Medium(buffer))
        }
        (1025..=4096, BufferUsage::LargeMessage) => {
            let buffer = LARGE_POOL.alloc().ok_or(MemoryError::LargePoolExhausted)?;
            Ok(CryptoBuffer::Large(buffer))
        }
        _ => Err(MemoryError::SizeNotSupported),
    }
}

// Secure crypto buffer with automatic zeroization
enum CryptoBuffer {
    Tiny(heapless::pool::Box<[u8; 64]>),
    Small(heapless::pool::Box<[u8; 256]>),
    Medium(heapless::pool::Box<[u8; 1024]>),
    Large(heapless::pool::Box<[u8; 4096]>),
}

impl CryptoBuffer {
    fn as_mut_slice(&mut self) -> &mut [u8] {
        match self {
            CryptoBuffer::Tiny(buf) => buf.as_mut(),
            CryptoBuffer::Small(buf) => buf.as_mut(),
            CryptoBuffer::Medium(buf) => buf.as_mut(),
            CryptoBuffer::Large(buf) => buf.as_mut(),
        }
    }
    
    fn capacity(&self) -> usize {
        match self {
            CryptoBuffer::Tiny(_) => 64,
            CryptoBuffer::Small(_) => 256,
            CryptoBuffer::Medium(_) => 1024,
            CryptoBuffer::Large(_) => 4096,
        }
    }
}

impl Drop for CryptoBuffer {
    fn drop(&mut self) {
        // Secure zeroization before returning to pool
        self.as_mut_slice().zeroize();
    }
}

#[derive(Clone, Copy)]
enum BufferUsage {
    Key,
    Nonce,
    SmallMessage,
    MediumMessage,
    LargeMessage,
}

#[derive(Debug, Clone, Copy)]
enum MemoryError {
    TinyPoolExhausted,
    SmallPoolExhausted,
    MediumPoolExhausted,
    LargePoolExhausted,
    ContextPoolExhausted,
    SizeNotSupported,
    SingletonAlreadyInitialized,
}

// Memory-efficient crypto operations with automatic pool selection
fn encrypt_with_optimal_memory(plaintext: &[u8]) -> Result<heapless::Vec<u8, 4096>, CryptoError> {
    // Determine optimal buffer size (add padding for encryption overhead)
    let required_size = plaintext.len() + 16; // AES block size padding
    let usage = match plaintext.len() {
        0..=240 => BufferUsage::SmallMessage,
        241..=1008 => BufferUsage::MediumMessage,
        _ => BufferUsage::LargeMessage,
    };
    
    // Allocate appropriate buffer
    let mut buffer = allocate_crypto_buffer(required_size, usage)
        .map_err(|_| CryptoError::OutOfMemory)?;
    
    // Copy plaintext to buffer
    let buf_slice = buffer.as_mut_slice();
    buf_slice[..plaintext.len()].copy_from_slice(plaintext);
    
    // Perform in-place encryption
    let ciphertext_len = encrypt_in_place(&mut buf_slice[..plaintext.len()])?;
    
    // Copy result to return vector
    let mut result = heapless::Vec::new();
    result.extend_from_slice(&buf_slice[..ciphertext_len])
        .map_err(|_| CryptoError::BufferTooSmall)?;
    
    // Buffer automatically zeroized and returned to pool when dropped
    Ok(result)
}
```

// Stub implementations for crypto operations
fn encrypt_in_place(data: &mut [u8]) -> Result<usize, CryptoError> {
    // Simple XOR encryption for demonstration
    for byte in data.iter_mut() {
        *byte ^= 0xAA;
    }
    Ok(data.len())
}

#[derive(Debug)]
struct CryptoContext {
    key: [u8; 32],
}

impl CryptoContext {
    const fn new() -> Self {
        Self { key: [0u8; 32] }
    }
}

struct GlobalCryptoState {
    initialized: bool,
}

impl GlobalCryptoState {
    fn new() -> Self {
        Self { initialized: true }
    }
}

#[entry]
fn main() -> ! {
    // Initialize memory pools
    let _ = init_static_memory_pools();
    
    // Example encryption
    let plaintext = b"Hello, static memory!";
    match encrypt_with_optimal_memory(plaintext) {
        Ok(ciphertext) => {
            // Use encrypted data
        }
        Err(_) => {
            // Handle error
        }
    }
    
    loop {}
}
```

## Compile-Time Memory Layout with Security Features

This pattern shows how to create a secure workspace with compile-time memory allocation.

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use zeroize::{Zeroize, ZeroizeOnDrop};

// Advanced compile-time memory layout with security considerations
#[derive(ZeroizeOnDrop)]
struct SecureCryptoWorkspace<const BUFFER_SIZE: usize, const KEY_COUNT: usize, const SESSION_COUNT: usize> {
    // Data buffers with different access patterns
    input_buffers: [[u8; BUFFER_SIZE]; 2],     // Double buffering for input
    output_buffers: [[u8; BUFFER_SIZE]; 2],    // Double buffering for output
    scratch_buffer: [u8; BUFFER_SIZE],         // Temporary calculations
    
    // Key material with automatic zeroization
    session_keys: [[u8; 32]; KEY_COUNT],
    nonces: [[u8; 12]; KEY_COUNT],
    key_schedules: [[u32; 60]; KEY_COUNT],     // Expanded AES keys
    
    // Session management
    active_sessions: [SessionInfo; SESSION_COUNT],
    session_states: [SessionState; SESSION_COUNT],
    
    // Buffer management state
    input_buffer_active: usize,
    output_buffer_active: usize,
    buffer_locks: [bool; 4],                   // Track buffer usage
    
    // Security state
    key_generation_counter: u64,
    last_key_rotation: u64,
    security_violations: u32,
}

impl<const BUFFER_SIZE: usize, const KEY_COUNT: usize, const SESSION_COUNT: usize> 
    SecureCryptoWorkspace<BUFFER_SIZE, KEY_COUNT, SESSION_COUNT> {
    
    const fn new() -> Self {
        Self {
            input_buffers: [[0; BUFFER_SIZE]; 2],
            output_buffers: [[0; BUFFER_SIZE]; 2],
            scratch_buffer: [0; BUFFER_SIZE],
            session_keys: [[0; 32]; KEY_COUNT],
            nonces: [[0; 12]; KEY_COUNT],
            key_schedules: [[0; 60]; KEY_COUNT],
            active_sessions: [SessionInfo::new(); SESSION_COUNT],
            session_states: [SessionState::Idle; SESSION_COUNT],
            input_buffer_active: 0,
            output_buffer_active: 0,
            buffer_locks: [false; 4],
            key_generation_counter: 0,
            last_key_rotation: 0,
            security_violations: 0,
        }
    }
    
    fn get_input_buffer(&mut self) -> Result<(usize, &mut [u8; BUFFER_SIZE]), WorkspaceError> {
        let inactive_buffer = 1 - self.input_buffer_active;
        if !self.buffer_locks[inactive_buffer] {
            self.buffer_locks[inactive_buffer] = true;
            Ok((inactive_buffer, &mut self.input_buffers[inactive_buffer]))
        } else {
            Err(WorkspaceError::NoBufferAvailable)
        }
    }
    
    fn get_output_buffer(&mut self) -> Result<(usize, &mut [u8; BUFFER_SIZE]), WorkspaceError> {
        let inactive_buffer = 1 - self.output_buffer_active;
        if !self.buffer_locks[inactive_buffer + 2] {
            self.buffer_locks[inactive_buffer + 2] = true;
            Ok((inactive_buffer, &mut self.output_buffers[inactive_buffer]))
        } else {
            Err(WorkspaceError::NoBufferAvailable)
        }
    }
    
    fn release_buffer(&mut self, buffer_id: usize) {
        if buffer_id < 4 {
            self.buffer_locks[buffer_id] = false;
            
            // Zeroize buffer on release for security
            match buffer_id {
                0 => self.input_buffers[0].zeroize(),
                1 => self.input_buffers[1].zeroize(),
                2 => self.output_buffers[0].zeroize(),
                3 => self.output_buffers[1].zeroize(),
                _ => {}
            }
        }
    }
    
    fn allocate_session(&mut self) -> Result<usize, WorkspaceError> {
        for (i, state) in self.session_states.iter_mut().enumerate() {
            if matches!(state, SessionState::Idle) {
                *state = SessionState::Allocated;
                self.active_sessions[i] = SessionInfo::new();
                return Ok(i);
            }
        }
        Err(WorkspaceError::NoSessionAvailable)
    }
    
    fn setup_session_key(&mut self, session_id: usize, key: &[u8; 32]) -> Result<(), WorkspaceError> {
        if session_id >= SESSION_COUNT {
            return Err(WorkspaceError::InvalidSessionId);
        }
        
        // Copy key and expand for AES
        self.session_keys[session_id].copy_from_slice(key);
        aes_key_expansion(key, &mut self.key_schedules[session_id]);
        
        // Generate unique nonce for this session
        self.key_generation_counter += 1;
        let nonce_seed = self.key_generation_counter.to_le_bytes();
        self.nonces[session_id][..8].copy_from_slice(&nonce_seed);
        
        self.session_states[session_id] = SessionState::KeyLoaded;
        Ok(())
    }
    
    fn encrypt_session_data(&mut self, session_id: usize, data: &[u8]) -> Result<usize, WorkspaceError> {
        if session_id >= SESSION_COUNT || !matches!(self.session_states[session_id], SessionState::KeyLoaded) {
            return Err(WorkspaceError::InvalidSessionState);
        }
        
        if data.len() > BUFFER_SIZE - 16 { // Reserve space for authentication tag
            return Err(WorkspaceError::DataTooLarge);
        }
        
        // Get buffers for operation
        let (input_id, input_buf) = self.get_input_buffer()?;
        let (output_id, output_buf) = self.get_output_buffer()?;
        
        // Copy data to input buffer
        input_buf[..data.len()].copy_from_slice(data);
        
        // Perform encryption using session key
        let ciphertext_len = aes_gcm_encrypt(
            &self.session_keys[session_id],
            &self.nonces[session_id],
            &input_buf[..data.len()],
            &mut output_buf[..data.len() + 16],
        )?;
        
        // Update nonce to prevent reuse
        increment_nonce(&mut self.nonces[session_id]);
        
        // Release input buffer (automatically zeroized)
        self.release_buffer(input_id);
        
        // Keep output buffer locked for caller to retrieve
        Ok(ciphertext_len)
    }
    
    fn check_security_state(&mut self) -> SecurityStatus {
        let current_time = get_current_time();
        
        // Check if key rotation is needed
        if current_time - self.last_key_rotation > KEY_ROTATION_INTERVAL {
            return SecurityStatus::KeyRotationRequired;
        }
        
        // Check for security violations
        if self.security_violations > MAX_SECURITY_VIOLATIONS {
            return SecurityStatus::SecurityViolation;
        }
        
        SecurityStatus::Secure
    }
}

#[derive(Clone, Copy)]
struct SessionInfo {
    creation_time: u64,
    message_count: u32,
    last_activity: u64,
}

impl SessionInfo {
    const fn new() -> Self {
        Self {
            creation_time: 0,
            message_count: 0,
            last_activity: 0,
        }
    }
}

#[derive(Clone, Copy)]
enum SessionState {
    Idle,
    Allocated,
    KeyLoaded,
    Active,
    Expired,
}

#[derive(Debug)]
enum WorkspaceError {
    NoBufferAvailable,
    NoSessionAvailable,
    InvalidSessionId,
    InvalidSessionState,
    DataTooLarge,
    EncryptionFailed,
}

enum SecurityStatus {
    Secure,
    KeyRotationRequired,
    SecurityViolation,
}

// Predefined workspace configurations for different applications
type MicroCryptoWorkspace = SecureCryptoWorkspace<128, 2, 2>;    // 256B buffers, 2 keys, 2 sessions
type SmallCryptoWorkspace = SecureCryptoWorkspace<512, 4, 4>;    // 1KB buffers, 4 keys, 4 sessions  
type MediumCryptoWorkspace = SecureCryptoWorkspace<2048, 8, 8>;  // 4KB buffers, 8 keys, 8 sessions
type LargeCryptoWorkspace = SecureCryptoWorkspace<8192, 16, 16>; // 16KB buffers, 16 keys, 16 sessions

// Global workspace instance (choose based on application requirements)
static mut CRYPTO_WORKSPACE: MediumCryptoWorkspace = MediumCryptoWorkspace::new();

// Safe wrapper for global workspace access
fn with_crypto_workspace<F, R>(f: F) -> R 
where 
    F: FnOnce(&mut MediumCryptoWorkspace) -> R,
{
    cortex_m::interrupt::free(|_| {
        unsafe { f(&mut CRYPTO_WORKSPACE) }
    })
}

// Constants for security policy
const KEY_ROTATION_INTERVAL: u64 = 3600; // 1 hour in seconds
const MAX_SECURITY_VIOLATIONS: u32 = 5;
```

// Stub functions for crypto operations
fn aes_key_expansion(key: &[u8; 32], schedule: &mut [u32; 60]) {
    // Simple stub - in real implementation would expand AES key
    for (i, chunk) in key.chunks_exact(4).enumerate() {
        schedule[i] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
}

fn aes_gcm_encrypt(
    _key: &[u8; 32],
    _nonce: &[u8; 12],
    _plaintext: &[u8],
    output: &mut [u8],
) -> Result<usize, WorkspaceError> {
    // Stub implementation
    Ok(output.len())
}

fn increment_nonce(nonce: &mut [u8; 12]) {
    // Simple increment for demonstration
    for byte in nonce.iter_mut().rev() {
        if *byte == 255 {
            *byte = 0;
        } else {
            *byte += 1;
            break;
        }
    }
}

fn get_current_time() -> u64 {
    // Stub - would read from RTC or timer
    0
}

#[entry]
fn main() -> ! {
    // Example usage of secure workspace
    with_crypto_workspace(|workspace| {
        // Allocate a session
        if let Ok(session_id) = workspace.allocate_session() {
            let key = [0x42u8; 32];
            let _ = workspace.setup_session_key(session_id, &key);
            
            // Encrypt some data
            let data = b"Secret message";
            let _ = workspace.encrypt_session_data(session_id, data);
        }
    });
    
    loop {}
}
```

## Memory Layout Optimization for Crypto Performance

Cache-aligned memory layouts improve crypto performance by reducing cache misses.

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;

#[derive(Debug)]
struct CryptoError(&'static str);

// Memory layout optimized for crypto operations and cache performance
#[repr(C, align(32))] // Align to cache line boundary
struct OptimizedCryptoLayout {
    // Hot data - frequently accessed, keep together
    active_key: [u8; 32],
    current_nonce: [u8; 12],
    message_counter: u64,
    
    // Padding to next cache line
    _pad1: [u8; 20],
    
    // Working buffers - align to AES block boundaries
    #[repr(align(16))]
    aes_input_block: [u8; 16],
    #[repr(align(16))]
    aes_output_block: [u8; 16],
    
    // Key schedule - used during encryption setup
    #[repr(align(16))]
    expanded_key: [u32; 60],
    
    // Cold data - less frequently accessed
    backup_keys: [[u8; 32]; 4],
    session_metadata: [SessionMetadata; 8],
    
    // Statistics and monitoring (coldest data)
    operation_count: u64,
    error_count: u32,
    last_maintenance: u64,
}

impl OptimizedCryptoLayout {
    const fn new() -> Self {
        Self {
            active_key: [0; 32],
            current_nonce: [0; 12],
            message_counter: 0,
            _pad1: [0; 20],
            aes_input_block: [0; 16],
            aes_output_block: [0; 16],
            expanded_key: [0; 60],
            backup_keys: [[0; 32]; 4],
            session_metadata: [SessionMetadata::new(); 8],
            operation_count: 0,
            error_count: 0,
            last_maintenance: 0,
        }
    }
    
    // Fast path encryption using optimized layout
    fn fast_encrypt_block(&mut self, plaintext: &[u8; 16]) -> Result<[u8; 16], CryptoError> {
        // Input and output blocks are cache-aligned for optimal performance
        self.aes_input_block.copy_from_slice(plaintext);
        
        // Use expanded key (already in cache from previous operations)
        aes_encrypt_block_optimized(
            &mut self.aes_input_block,
            &mut self.aes_output_block,
            &self.expanded_key,
        )?;
        
        // Increment message counter (hot data, likely in cache)
        self.message_counter += 1;
        
        Ok(self.aes_output_block)
    }
}

#[derive(Clone, Copy)]
struct SessionMetadata {
    session_id: u32,
    creation_time: u64,
    last_used: u64,
    message_count: u32,
}

impl SessionMetadata {
    const fn new() -> Self {
        Self {
            session_id: 0,
            creation_time: 0,
            last_used: 0,
            message_count: 0,
        }
    }
}

// Static allocation with optimal memory layout
static mut OPTIMIZED_CRYPTO: OptimizedCryptoLayout = OptimizedCryptoLayout::new();

// Stub function for AES encryption
fn aes_encrypt_block_optimized(
    input: &mut [u8; 16],
    output: &mut [u8; 16],
    _key_schedule: &[u32; 60],
) -> Result<(), CryptoError> {
    // Simple XOR for demonstration
    for i in 0..16 {
        output[i] = input[i] ^ 0xAA;
    }
    Ok(())
}

// Safe wrapper for accessing optimized crypto layout
fn with_optimized_crypto<F, R>(f: F) -> R
where
    F: FnOnce(&mut OptimizedCryptoLayout) -> R,
{
    // Use critical section for thread safety
    critical_section::with(|_| unsafe { f(&mut OPTIMIZED_CRYPTO) })
}

#[entry]
fn main() -> ! {
    // Example usage
    with_optimized_crypto(|crypto| {
        let plaintext = [0x42u8; 16];
        match crypto.fast_encrypt_block(&plaintext) {
            Ok(ciphertext) => {
                // Use encrypted block
            }
            Err(_) => {
                // Handle error
            }
        }
    });
    
    loop {}
}
```