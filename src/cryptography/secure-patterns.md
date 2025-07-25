# 5.1 Secure Coding Patterns {secure-coding-patterns}

Rust provides unique advantages for secure cryptographic implementations through its type system, memory safety guarantees, and zero-cost abstractions. This section demonstrates Rust-specific security patterns that prevent common vulnerabilities found in C crypto implementations.

**Enhanced by Advanced Features:**
- → [Advanced Type System Features](../core-concepts/advanced-types.md) - Use enums and traits for type-safe crypto protocols
- → [Functional Programming and Data Processing](../core-concepts/functional.md) - Apply iterators for safer data processing

## Memory Safety Advantages for Cryptography

Rust eliminates entire classes of vulnerabilities that plague C cryptographic implementations.

### Buffer Overflow Prevention

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;

// Stub types for demonstration
struct CryptoError;

// Buffer overflow prevention - automatic bounds checking
fn safe_crypto_buffer_operations() -> Result<(), CryptoError> {
    let mut plaintext = [0u8; 256];
    let mut ciphertext = [0u8; 256];
    
    // This would panic in debug mode, return error in release
    // No silent buffer overflow like in C
    let safe_slice = &plaintext[..200]; // Always bounds-checked
    
    // Compile-time size verification
    let key: [u8; 32] = [0; 32];  // Exactly 32 bytes, enforced by type system
    let nonce: [u8; 12] = [0; 12]; // Exactly 12 bytes for ChaCha20Poly1305
    
    // No accidental key/nonce size mismatches
    encrypt_chacha20poly1305(&key, &nonce, safe_slice, &mut ciphertext[..200])?;
    
    Ok(())
}

// Stub function for example
fn encrypt_chacha20poly1305(
    _key: &[u8; 32], 
    _nonce: &[u8; 12], 
    _input: &[u8], 
    _output: &mut [u8]
) -> Result<(), CryptoError> {
    Ok(())
}

#[entry]
fn main() -> ! {
    // Example usage
    let _ = safe_crypto_buffer_operations();
    loop {}
}
```

### Secure Key Lifecycle Management

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
struct MasterKey([u8; 32]);

#[derive(Zeroize)]
#[zeroize(drop)]
struct SessionKeys {
    encryption_key: [u8; 32],
    mac_key: [u8; 32],
}

// Use-after-free prevention through ownership
fn secure_key_lifecycle() {
    let master_key = generate_master_key();
    let session_keys = derive_session_keys(&master_key);
    
    // master_key automatically zeroized when it goes out of scope
    // No risk of accessing freed key material
    
    use_session_keys(session_keys);
    // session_keys automatically zeroized here
}

// Stub functions for demonstration
fn generate_master_key() -> MasterKey {
    MasterKey([0u8; 32])
}

fn derive_session_keys(_master: &MasterKey) -> SessionKeys {
    SessionKeys {
        encryption_key: [0u8; 32],
        mac_key: [0u8; 32],
    }
}

fn use_session_keys(_keys: SessionKeys) {
    // Keys are consumed and will be zeroized when dropped
}

#[entry]
fn main() -> ! {
    secure_key_lifecycle();
    loop {}
}
```

### Double-Free Prevention

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;

struct CryptoContext {
    key_schedule: [u32; 60],
    rounds: u8,
}

impl CryptoContext {
    fn new() -> Self {
        Self {
            key_schedule: [0u32; 60],
            rounds: 14,
        }
    }
}

// Double-free prevention - impossible in safe Rust
fn no_double_free_crypto_contexts() {
    let crypto_ctx = CryptoContext::new();
    process_with_context(crypto_ctx); // Ownership transferred
    // crypto_ctx can't be used again - compile error prevents double-free
}

fn process_with_context(_ctx: CryptoContext) {
    // Context is owned here and will be dropped when function exits
}

#[entry]
fn main() -> ! {
    no_double_free_crypto_contexts();
    loop {}
}
```

## Automatic Key Zeroization Patterns

Rust's `Drop` trait provides automatic, guaranteed cleanup of sensitive material.

### Session Keys with Automatic Zeroization

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use zeroize::{Zeroize, ZeroizeOnDrop};
use heapless::Vec;
use heapless::consts::U64;

#[derive(Debug)]
struct CryptoError(&'static str);

// Automatic zeroization for all key material
#[derive(ZeroizeOnDrop)]
struct SessionKeys {
    encryption_key: [u8; 32],
    mac_key: [u8; 32],
    iv: [u8; 16],
}

impl SessionKeys {
    fn derive_from_master(master_key: &[u8; 32], context: &[u8]) -> Self {
        let mut keys = Self {
            encryption_key: [0; 32],
            mac_key: [0; 32],
            iv: [0; 16],
        };
        
        // HKDF key derivation
        hkdf_expand(master_key, context, &mut keys.encryption_key, b"encrypt");
        hkdf_expand(master_key, context, &mut keys.mac_key, b"mac");
        hkdf_expand(master_key, context, &mut keys.iv, b"iv");
        
        keys
    }
    
    fn encrypt_and_mac(&self, plaintext: &[u8]) -> Result<Vec<u8, U64>, CryptoError> {
        // Encrypt-then-MAC construction
        let ciphertext = aes_gcm_encrypt(&self.encryption_key, &self.iv, plaintext)?;
        let mac = hmac_sha256(&self.mac_key, &ciphertext)?;
        
        let mut result = Vec::new();
        result.extend_from_slice(&ciphertext).map_err(|_| CryptoError("Buffer full"))?;
        result.extend_from_slice(&mac).map_err(|_| CryptoError("Buffer full"))?;
        Ok(result)
    }
}

// Stub functions for crypto operations
fn hkdf_expand(_master: &[u8; 32], _context: &[u8], output: &mut [u8], _label: &[u8]) {
    // Fill output with deterministic data for example
    for (i, byte) in output.iter_mut().enumerate() {
        *byte = i as u8;
    }
}

fn aes_gcm_encrypt(_key: &[u8; 32], _iv: &[u8; 16], _plaintext: &[u8]) -> Result<[u8; 32], CryptoError> {
    Ok([0u8; 32])
}

fn hmac_sha256(_key: &[u8; 32], _message: &[u8]) -> Result<[u8; 32], CryptoError> {
    Ok([0u8; 32])
}

#[entry]
fn main() -> ! {
    let master_key = [0u8; 32];
    let _keys = SessionKeys::derive_from_master(&master_key, b"session_1");
    loop {}
}
```

### Secure Buffer with Automatic Cleanup

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug)]
struct CryptoError(&'static str);

// Secure wrapper for sensitive data with automatic cleanup
#[derive(ZeroizeOnDrop)]
struct SecureBuffer<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> SecureBuffer<N> {
    fn new() -> Self {
        Self {
            data: [0; N],
            len: 0,
        }
    }
    
    fn write(&mut self, bytes: &[u8]) -> Result<(), CryptoError> {
        if bytes.len() > N - self.len {
            return Err(CryptoError("Buffer too small"));
        }
        
        self.data[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }
    
    fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }
    
    fn clear(&mut self) {
        self.data.zeroize();
        self.len = 0;
    }
}

// Usage example - automatic cleanup guaranteed
fn secure_session_example() -> Result<(), CryptoError> {
    let mut secure_buffer = SecureBuffer::<1024>::new();
    secure_buffer.write(b"secret message")?;
    
    // Process the buffer
    process_secret_data(secure_buffer.as_slice());
    
    // secure_buffer automatically zeroized here
    // No risk of sensitive data remaining in memory
    Ok(())
}

fn process_secret_data(_data: &[u8]) {
    // Process the secret data
}

#[entry]
fn main() -> ! {
    let _ = secure_session_example();
    loop {}
}
```

## Type-Safe Protocol State Machines

Rust's type system can enforce protocol correctness at compile time, preventing state machine violations.

### Protocol State Machine Implementation

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use core::marker::PhantomData;
use heapless::Vec;
use heapless::consts::U128;

#[derive(Debug)]
struct CryptoError(&'static str);

// Stub types for demonstration
struct Socket;
impl Socket {
    fn new() -> Self { Socket }
    fn send(&self, _data: &[u8]) -> Result<(), CryptoError> { Ok(()) }
    fn receive(&self) -> Result<Vec<u8, U128>, CryptoError> { 
        Ok(Vec::new()) 
    }
}

struct SessionKeys {
    encryption_key: [u8; 32],
    mac_key: [u8; 32],
}

impl SessionKeys {
    fn encrypt_and_mac(&self, _data: &[u8]) -> Result<Vec<u8, U128>, CryptoError> {
        Ok(Vec::new())
    }
    
    fn decrypt_and_verify(&self, _data: &[u8]) -> Result<Vec<u8, U128>, CryptoError> {
        let mut result = Vec::new();
        // Add 8 bytes for sequence number plus some data
        for i in 0..16 {
            result.push(i as u8).map_err(|_| CryptoError("Buffer full"))?;
        }
        Ok(result)
    }
}

// Protocol states as types
struct Uninitialized;
struct HandshakeInProgress;
struct SessionEstablished;
struct Terminated;

// State machine enforced by type system
struct SecureChannel<State> {
    socket: Socket,
    session_keys: Option<SessionKeys>,
    sequence_number: u64,
    state: PhantomData<State>,
}

impl SecureChannel<Uninitialized> {
    fn new(socket: Socket) -> Self {
        Self {
            socket,
            session_keys: None,
            sequence_number: 0,
            state: PhantomData,
        }
    }
    
    // Can only start handshake from uninitialized state
    fn start_handshake(self) -> SecureChannel<HandshakeInProgress> {
        SecureChannel {
            socket: self.socket,
            session_keys: None,
            sequence_number: 0,
            state: PhantomData,
        }
    }
}

impl SecureChannel<HandshakeInProgress> {
    // Can only complete handshake from in-progress state
    fn complete_handshake(self, keys: SessionKeys) -> SecureChannel<SessionEstablished> {
        SecureChannel {
            socket: self.socket,
            session_keys: Some(keys),
            sequence_number: 0,
            state: PhantomData,
        }
    }
    
    // Can abort handshake and return to uninitialized
    fn abort_handshake(self) -> SecureChannel<Uninitialized> {
        SecureChannel {
            socket: self.socket,
            session_keys: None,
            sequence_number: 0,
            state: PhantomData,
        }
    }
}

impl SecureChannel<SessionEstablished> {
    // Can only send/receive in established state
    fn send_encrypted(&mut self, data: &[u8]) -> Result<(), CryptoError> {
        let keys = self.session_keys.as_ref().unwrap();
        
        // Include sequence number to prevent replay attacks
        let mut message = Vec::<u8, U128>::new();
        message.extend_from_slice(&self.sequence_number.to_be_bytes())
            .map_err(|_| CryptoError("Buffer full"))?;
        message.extend_from_slice(data)
            .map_err(|_| CryptoError("Buffer full"))?;
        
        let encrypted = keys.encrypt_and_mac(&message)?;
        self.socket.send(&encrypted)?;
        
        self.sequence_number += 1;
        Ok(())
    }
    
    fn receive_encrypted(&mut self) -> Result<Vec<u8, U128>, CryptoError> {
        let encrypted = self.socket.receive()?;
        let keys = self.session_keys.as_ref().unwrap();
        let decrypted = keys.decrypt_and_verify(&encrypted)?;
        
        // Verify sequence number
        if decrypted.len() < 8 {
            return Err(CryptoError("Invalid message"));
        }
        
        let mut seq_bytes = [0u8; 8];
        seq_bytes.copy_from_slice(&decrypted[..8]);
        let received_seq = u64::from_be_bytes(seq_bytes);
        
        if received_seq != self.sequence_number {
            return Err(CryptoError("Replay attack detected"));
        }
        
        self.sequence_number += 1;
        
        let mut result = Vec::new();
        result.extend_from_slice(&decrypted[8..])
            .map_err(|_| CryptoError("Buffer full"))?;
        Ok(result)
    }
    
    // Can terminate session
    fn terminate(self) -> SecureChannel<Terminated> {
        SecureChannel {
            socket: self.socket,
            session_keys: None, // Keys automatically zeroized
            sequence_number: 0,
            state: PhantomData,
        }
    }
}

impl SecureChannel<Terminated> {
    // Can only restart from terminated state
    fn restart(self) -> SecureChannel<Uninitialized> {
        SecureChannel {
            socket: self.socket,
            session_keys: None,
            sequence_number: 0,
            state: PhantomData,
        }
    }
}

#[entry]
fn main() -> ! {
    // Compile-time protocol enforcement example
    let socket = Socket::new();
    let channel = SecureChannel::new(socket);
    
    // This compiles - valid state transition
    let channel = channel.start_handshake();
    
    // This would NOT compile - can't send in handshake state
    // channel.send_encrypted(b"data"); // Compile error!
    
    let keys = SessionKeys {
        encryption_key: [0u8; 32],
        mac_key: [0u8; 32],
    };
    let mut channel = channel.complete_handshake(keys);
    
    // Now this compiles - valid in established state
    let _ = channel.send_encrypted(b"secure data");
    
    loop {}
}
```

## Rust-Specific Security Advantages

Rust provides several unique security advantages over C for cryptographic implementations.

### No Null Pointer Vulnerabilities

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use heapless::Vec;
use heapless::consts::U32;

#[derive(Debug)]
struct CryptoError(&'static str);

// No null pointer dereferences
fn safe_key_handling(key: Option<&[u8; 32]>) -> Result<Vec<u8, U32>, CryptoError> {
    // Compiler forces explicit handling of None case
    let key = key.ok_or(CryptoError("No key provided"))?;
    
    // No risk of null pointer dereference
    encrypt_with_key(key)
}

fn encrypt_with_key(key: &[u8; 32]) -> Result<Vec<u8, U32>, CryptoError> {
    let mut result = Vec::new();
    // Simple example: XOR with key
    for &byte in key.iter() {
        result.push(byte ^ 0xAA).map_err(|_| CryptoError("Buffer full"))?;
    }
    Ok(result)
}

#[entry]
fn main() -> ! {
    // Safe handling of optional keys
    let _ = safe_key_handling(None); // Returns error
    let _ = safe_key_handling(Some(&[0u8; 32])); // Works
    loop {}
}
```

### Integer Overflow Protection

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use heapless::Vec;
use heapless::consts::U256;

#[derive(Debug)]
struct CryptoError(&'static str);

// Integer overflow protection
fn safe_buffer_calculations(data_len: usize, overhead: usize) -> Result<usize, CryptoError> {
    // Checked arithmetic prevents integer overflow attacks
    let total_len = data_len.checked_add(overhead)
        .ok_or(CryptoError("Integer overflow"))?;
    
    // Additional bounds check
    if total_len > 256 {
        return Err(CryptoError("Buffer too large"));
    }
    
    Ok(total_len)
}

fn allocate_crypto_buffer(size: usize) -> Result<Vec<u8, U256>, CryptoError> {
    if size > 256 {
        return Err(CryptoError("Size exceeds maximum"));
    }
    
    let mut buffer = Vec::new();
    buffer.resize(size, 0).map_err(|_| CryptoError("Allocation failed"))?;
    Ok(buffer)
}

#[entry]
fn main() -> ! {
    // Safe calculation prevents overflow
    match safe_buffer_calculations(200, 100) {
        Ok(size) => {
            let _ = allocate_crypto_buffer(size);
        }
        Err(_) => {
            // Handle overflow safely
        }
    }
    
    // This would be caught
    let _ = safe_buffer_calculations(usize::MAX, 100);
    
    loop {}
}
```

### Initialization Safety

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;

struct CryptoContext {
    key_schedule: [u32; 60],
    rounds: u8,
    initialized: bool,
}

impl CryptoContext {
    // All fields must be explicitly initialized
    fn new() -> Self {
        Self {
            key_schedule: [0u32; 60],  // Explicitly zeroed
            rounds: 14,
            initialized: false,
        }
    }
    
    fn init_with_key(&mut self, key: &[u8; 32]) {
        // Initialize key schedule
        for (i, chunk) in key.chunks_exact(4).enumerate() {
            self.key_schedule[i] = u32::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3]
            ]);
        }
        self.initialized = true;
    }
    
    fn encrypt(&self, data: &mut [u8]) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("Context not initialized");
        }
        
        // Safe to use - guaranteed initialized
        for byte in data.iter_mut() {
            *byte ^= (self.key_schedule[0] & 0xFF) as u8;
        }
        
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    let mut ctx = CryptoContext::new();
    let mut data = [0u8; 16];
    
    // This would fail - not initialized
    let _ = ctx.encrypt(&mut data);
    
    // Initialize properly
    ctx.init_with_key(&[0u8; 32]);
    
    // Now it works
    let _ = ctx.encrypt(&mut data);
    
    loop {}
}
```

### Thread-Safe Crypto Counters

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use core::sync::atomic::{AtomicU64, Ordering};

struct ThreadSafeCryptoCounter {
    counter: AtomicU64,
}

impl ThreadSafeCryptoCounter {
    const fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }
    
    // Thread-safe counter for nonces/IVs
    fn next_nonce(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
    
    // Get current value without incrementing
    fn current(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
}

// Global counter - safe to access from multiple contexts
static NONCE_COUNTER: ThreadSafeCryptoCounter = ThreadSafeCryptoCounter::new();

#[entry]
fn main() -> ! {
    // Safe concurrent access
    let nonce1 = NONCE_COUNTER.next_nonce(); // 0
    let nonce2 = NONCE_COUNTER.next_nonce(); // 1
    let nonce3 = NONCE_COUNTER.next_nonce(); // 2
    
    loop {}
}
```

### Compile-Time Size Verification

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;

// Const generics for compile-time crypto parameters
struct CryptoEngine<const KEY_SIZE: usize, const BLOCK_SIZE: usize> {
    key: [u8; KEY_SIZE],
    buffer: [u8; BLOCK_SIZE],
}

impl<const KEY_SIZE: usize, const BLOCK_SIZE: usize> CryptoEngine<KEY_SIZE, BLOCK_SIZE> {
    fn new(key: [u8; KEY_SIZE]) -> Self {
        Self {
            key,
            buffer: [0; BLOCK_SIZE],
        }
    }
    
    // Compile-time verification of buffer sizes
    fn process_block(&mut self, input: &[u8; BLOCK_SIZE]) -> [u8; BLOCK_SIZE] {
        // No runtime size checks needed - guaranteed by type system
        self.buffer.copy_from_slice(input);
        self.encrypt_buffer();
        self.buffer
    }
    
    fn encrypt_buffer(&mut self) {
        // Simple XOR encryption for demonstration
        for (i, byte) in self.buffer.iter_mut().enumerate() {
            *byte ^= self.key[i % KEY_SIZE];
        }
    }
}

// Type aliases with specific sizes
type Aes256Engine = CryptoEngine<32, 16>; // AES-256 with 128-bit blocks
type ChaCha20Engine = CryptoEngine<32, 64>; // ChaCha20 with 512-bit blocks

#[entry]
fn main() -> ! {
    let aes_key = [0u8; 32];
    let mut aes_engine = Aes256Engine::new(aes_key);
    
    let plaintext_block = [0u8; 16];
    let ciphertext = aes_engine.process_block(&plaintext_block);
    
    // This would NOT compile - wrong block size
    // let wrong_block = [0u8; 32];
    // aes_engine.process_block(&wrong_block); // Compile error!
    
    loop {}
}
```