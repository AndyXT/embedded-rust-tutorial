# 5.1 Secure Coding Patterns {secure-coding-patterns}

Rust provides unique advantages for secure cryptographic implementations through its type system, memory safety guarantees, and zero-cost abstractions. This section demonstrates Rust-specific security patterns that prevent common vulnerabilities found in C crypto implementations.

**Enhanced by Advanced Features:**
- → [Advanced Type System Features](../core-concepts/advanced-types.md) - Use enums and traits for type-safe crypto protocols
- → [Functional Programming and Data Processing](../core-concepts/functional.md) - Apply iterators for safer data processing

## Memory Safety Advantages for Cryptography

Rust eliminates entire classes of vulnerabilities that plague C cryptographic implementations:

```rust
// Buffer overflow prevention - automatic bounds checking
fn safe_crypto_buffer_operations() {
    let mut plaintext = [0u8; 256];
    let mut ciphertext = [0u8; 256];
    
    // This would panic in debug mode, return error in release
    // No silent buffer overflow like in C
    let safe_slice = &plaintext[..200]; // Always bounds-checked
    
    // Compile-time size verification
    let key: [u8; 32] = [0; 32];  // Exactly 32 bytes, enforced by type system
    let nonce: [u8; 12] = [0; 12]; // Exactly 12 bytes for ChaCha20Poly1305
    
    // No accidental key/nonce size mismatches
    encrypt_chacha20poly1305(&key, &nonce, safe_slice, &mut ciphertext[..200]);
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

// Double-free prevention - impossible in safe Rust
fn no_double_free_crypto_contexts() {
    let crypto_ctx = CryptoContext::new();
    process_with_context(crypto_ctx); // Ownership transferred
    // crypto_ctx can't be used again - compile error prevents double-free
}
```

## Automatic Key Zeroization Patterns

Rust's `Drop` trait provides automatic, guaranteed cleanup of sensitive material:

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

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
    
    fn encrypt_and_mac(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Encrypt-then-MAC construction
        let ciphertext = aes_gcm_encrypt(&self.encryption_key, &self.iv, plaintext)?;
        let mac = hmac_sha256(&self.mac_key, &ciphertext)?;
        
        let mut result = Vec::with_capacity(ciphertext.len() + mac.len());
        result.extend_from_slice(&ciphertext);
        result.extend_from_slice(&mac);
        Ok(result)
    }
}

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
            return Err(CryptoError::BufferTooSmall);
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
fn secure_session_example() {
    let master_key = [0u8; 32]; // From secure key exchange
    let session_keys = SessionKeys::derive_from_master(&master_key, b"session_1");
    
    let mut secure_buffer = SecureBuffer::<1024>::new();
    secure_buffer.write(b"secret message").unwrap();
    
    let encrypted = session_keys.encrypt_and_mac(secure_buffer.as_slice()).unwrap();
    
    // Both session_keys and secure_buffer automatically zeroized here
    // No risk of key material remaining in memory
}
```

## Type-Safe Protocol State Machines

Rust's type system can enforce protocol correctness at compile time, preventing state machine violations:

```rust
use core::marker::PhantomData;

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
        let mut message = Vec::with_capacity(data.len() + 8);
        message.extend_from_slice(&self.sequence_number.to_be_bytes());
        message.extend_from_slice(data);
        
        let encrypted = keys.encrypt_and_mac(&message)?;
        self.socket.send(&encrypted)?;
        
        self.sequence_number += 1;
        Ok(())
    }
    
    fn receive_encrypted(&mut self) -> Result<Vec<u8>, CryptoError> {
        let encrypted = self.socket.receive()?;
        let keys = self.session_keys.as_ref().unwrap();
        let decrypted = keys.decrypt_and_verify(&encrypted)?;
        
        // Verify sequence number
        if decrypted.len() < 8 {
            return Err(CryptoError::InvalidMessage);
        }
        
        let received_seq = u64::from_be_bytes(
            decrypted[..8].try_into().unwrap()
        );
        
        if received_seq != self.sequence_number {
            return Err(CryptoError::ReplayAttack);
        }
        
        self.sequence_number += 1;
        Ok(decrypted[8..].to_vec())
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

// Compile-time protocol enforcement example
fn protocol_state_example() {
    let socket = Socket::new();
    let channel = SecureChannel::new(socket);
    
    // This compiles - valid state transition
    let channel = channel.start_handshake();
    
    // This would NOT compile - can't send in handshake state
    // channel.send_encrypted(b"data"); // Compile error!
    
    let keys = perform_key_exchange();
    let mut channel = channel.complete_handshake(keys);
    
    // Now this compiles - valid in established state
    channel.send_encrypted(b"secure data").unwrap();
}
```

## Rust-Specific Security Advantages

Rust provides several unique security advantages over C for cryptographic implementations:

```rust
// 1. No null pointer dereferences
fn safe_key_handling(key: Option<&[u8; 32]>) -> Result<Vec<u8>, CryptoError> {
    // Compiler forces explicit handling of None case
    let key = key.ok_or(CryptoError::NoKey)?;
    
    // No risk of null pointer dereference
    encrypt_with_key(key)
}

// 2. Integer overflow protection
fn safe_buffer_calculations(data_len: usize, overhead: usize) -> Result<Vec<u8>, CryptoError> {
    // Checked arithmetic prevents integer overflow attacks
    let total_len = data_len.checked_add(overhead)
        .ok_or(CryptoError::IntegerOverflow)?;
    
    let mut buffer = vec![0u8; total_len];
    Ok(buffer)
}

// 3. Initialization safety - no uninitialized memory
fn safe_crypto_context() -> CryptoContext {
    // All fields must be explicitly initialized
    CryptoContext {
        key_schedule: [0u32; 60],  // Explicitly zeroed
        rounds: 14,
        initialized: false,
    }
    // No risk of using uninitialized crypto state
}

// 4. Thread safety for crypto operations
use core::sync::atomic::{AtomicU64, Ordering};

struct ThreadSafeCryptoCounter {
    counter: AtomicU64,
}

impl ThreadSafeCryptoCounter {
    fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }
    
    // Thread-safe counter for nonces/IVs
    fn next_nonce(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}

// 5. Const generics for compile-time crypto parameters
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
        // Implementation details...
    }
}

// Usage with compile-time size verification
type Aes256Engine = CryptoEngine<32, 16>; // AES-256 with 128-bit blocks
type ChaCha20Engine = CryptoEngine<32, 64>; // ChaCha20 with 512-bit blocks

fn compile_time_crypto_safety() {
    let aes_key = [0u8; 32];
    let mut aes_engine = Aes256Engine::new(aes_key);
    
    let plaintext_block = [0u8; 16];
    let ciphertext = aes_engine.process_block(&plaintext_block);
    
    // This would NOT compile - wrong block size
    // let wrong_block = [0u8; 32];
    // aes_engine.process_block(&wrong_block); // Compile error!
}
```