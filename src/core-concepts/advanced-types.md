# 3.4 Advanced Type System Features {advanced-type-system-features}

Rust's advanced type system provides powerful abstractions that go far beyond C's capabilities. For embedded cryptography engineers, these features enable safer, more expressive, and more maintainable code while maintaining zero-cost abstractions.

<details>
<summary><strong>▶️ Section Overview</strong> - What you'll learn</summary>

This section covers:
- **Enums with data** - Pattern matching vs C switch statements
- **Traits** - Safe alternatives to C function pointers
- **Methods** - Better code organization than C function naming
- **Crypto-specific applications** - State machines, error handling, algorithm abstractions

**Prerequisites:** Understanding of basic Rust syntax and ownership concepts

**Related Sections:**
- → [Error Handling Without Exceptions](../core-concepts/error-handling.md) - See how enums enable Result<T, E> pattern
- → [Secure Coding Patterns](../cryptography/secure-patterns.md) - Apply type system features for crypto safety

</details>

#### Advanced Enums and Pattern Matching

Rust enums are far more powerful than C enums. They can carry data, enabling type-safe state machines and error handling that's impossible in C.

<details>
<summary><strong>▶️ C vs Rust Enum Comparison</strong> - See the fundamental differences</summary>

**C Approach - Error-Prone:**
```c
// C enum - just integers, no data
typedef enum {
    CRYPTO_STATE_IDLE,
    CRYPTO_STATE_KEY_EXCHANGE,
    CRYPTO_STATE_ENCRYPTED,
    CRYPTO_STATE_ERROR
} crypto_state_t;

// Separate variables for state data - easy to get out of sync
typedef struct {
    crypto_state_t state;
    uint8_t* key_data;        // Only valid in KEY_EXCHANGE state
    size_t key_len;           // Only valid in KEY_EXCHANGE state
    uint32_t error_code;      // Only valid in ERROR state
    char* error_msg;          // Only valid in ERROR state
} crypto_context_t;

// Error-prone state handling
int process_crypto_message(crypto_context_t* ctx, uint8_t* data, size_t len) {
    switch (ctx->state) {
        case CRYPTO_STATE_IDLE:
            // BUG: What if key_data is not NULL? Memory leak!
            return start_key_exchange(ctx, data, len);
            
        case CRYPTO_STATE_KEY_EXCHANGE:
            // BUG: What if key_data is NULL? Segfault!
            return complete_key_exchange(ctx, ctx->key_data, ctx->key_len);
            
        case CRYPTO_STATE_ERROR:
            // BUG: Forgot to check if error_msg is valid
            printf("Error: %s\n", ctx->error_msg);  // Potential crash
            return -1;
            
        default:
            return -1;  // BUG: Forgot to handle ENCRYPTED state
    }
}
```

**Rust Approach - Type-Safe:**
```rust
// Rust enum - each variant can carry different data
#[derive(Debug)]
enum CryptoState {
    Idle,
    KeyExchange { 
        key_data: Vec<u8>, 
        algorithm: KeyExchangeAlgorithm 
    },
    Encrypted { 
        session_key: SecureKey<32>,
        cipher: CipherType 
    },
    Error { 
        code: CryptoErrorCode, 
        message: &'static str 
    },
}

// Impossible to access wrong data - compiler enforces correctness
fn process_crypto_message(state: CryptoState, data: &[u8]) -> Result<CryptoState, CryptoError> {
    match state {
        CryptoState::Idle => {
            // Can only access data that exists in this state
            let key_exchange = start_key_exchange(data)?;
            Ok(CryptoState::KeyExchange { 
                key_data: key_exchange.key_material,
                algorithm: key_exchange.algorithm 
            })
        },
        
        CryptoState::KeyExchange { key_data, algorithm } => {
            // Compiler guarantees key_data exists and is the right type
            let session_key = complete_key_exchange(&key_data, algorithm)?;
            Ok(CryptoState::Encrypted { 
                session_key,
                cipher: CipherType::Aes256Gcm 
            })
        },
        
        CryptoState::Encrypted { session_key, cipher } => {
            // Process encrypted message with guaranteed valid session key
            let decrypted = decrypt_message(&session_key, cipher, data)?;
            process_decrypted_data(&decrypted)?;
            Ok(state)  // Stay in encrypted state
        },
        
        CryptoState::Error { code, message } => {
            // Compiler guarantees error data exists
            log::error!("Crypto error {}: {}", code as u32, message);
            Err(CryptoError::StateMachine(code))
        }
        
        // Compiler error if we forget any state - impossible to miss cases!
    }
}
```

</details>

**Crypto State Machine Example:**

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug)]
enum TlsHandshakeState {
    ClientHello,
    ServerHello { 
        server_random: [u8; 32],
        cipher_suite: CipherSuite,
        compression: CompressionMethod,
    },
    Certificate { 
        cert_chain: Vec<Certificate>,
        server_key_exchange: Option<ServerKeyExchange>,
    },
    ClientKeyExchange { 
        pre_master_secret: PreMasterSecret,
        client_cert: Option<Certificate>,
    },
    Finished {
        master_secret: MasterSecret,
        client_verify_data: [u8; 12],
        server_verify_data: [u8; 12],
    },
}

#[derive(ZeroizeOnDrop)]
struct PreMasterSecret([u8; 48]);

#[derive(ZeroizeOnDrop)]
struct MasterSecret([u8; 48]);

impl TlsHandshakeState {
    fn process_message(self, message: TlsMessage) -> Result<Self, TlsError> {
        match (self, message) {
            // Type-safe state transitions
            (TlsHandshakeState::ClientHello, TlsMessage::ServerHello(hello)) => {
                Ok(TlsHandshakeState::ServerHello {
                    server_random: hello.random,
                    cipher_suite: hello.cipher_suite,
                    compression: hello.compression_method,
                })
            },
            
            (TlsHandshakeState::ServerHello { server_random, cipher_suite, .. }, 
             TlsMessage::Certificate(cert)) => {
                // Validate certificate chain
                validate_certificate_chain(&cert.certificates)?;
                
                Ok(TlsHandshakeState::Certificate {
                    cert_chain: cert.certificates,
                    server_key_exchange: None,
                })
            },
            
            // Invalid state transitions caught at compile time
            (TlsHandshakeState::ClientHello, TlsMessage::Certificate(_)) => {
                Err(TlsError::InvalidStateTransition)
            },
            
            // Compiler ensures all valid combinations are handled
            _ => Err(TlsError::UnexpectedMessage),
        }
    }
}
```

**Pattern Matching vs C Switch:**

```rust
// Rust pattern matching - much more powerful than C switch
fn handle_crypto_result(result: CryptoResult) -> Action {
    match result {
        // Match specific values
        CryptoResult::Success => Action::Continue,
        
        // Match with conditions (guards)
        CryptoResult::Retry(count) if count < 3 => Action::RetryOperation,
        CryptoResult::Retry(_) => Action::Abort,
        
        // Extract and use data from enum variants
        CryptoResult::KeyRotation { old_key, new_key } => {
            secure_key_transition(old_key, new_key);
            Action::Continue
        },
        
        // Match ranges
        CryptoResult::ErrorCode(code @ 100..=199) => {
            log::warn!("Recoverable error: {}", code);
            Action::Retry
        },
        
        // Match nested structures
        CryptoResult::ProtocolError { 
            phase: HandshakePhase::KeyExchange,
            error: KeyExchangeError::InvalidSignature 
        } => {
            security_log("Invalid signature during key exchange");
            Action::Abort
        },
        
        // Catch-all with variable binding
        CryptoResult::ProtocolError { phase, error } => {
            log::error!("Protocol error in {:?}: {:?}", phase, error);
            Action::Abort
        },
    }
}
```

#### Traits - Safe Function Pointer Alternatives

Traits provide a safe, zero-cost alternative to C function pointers, enabling polymorphism without runtime overhead.

<details>
<summary><strong>▶️ C Function Pointers vs Rust Traits</strong> - See the safety improvements</summary>

**C Approach - Unsafe:**
```c
// C function pointers - no type safety, easy to misuse
typedef int (*crypto_hash_fn)(const uint8_t* data, size_t len, uint8_t* output);
typedef int (*crypto_cipher_fn)(const uint8_t* key, const uint8_t* input, 
                               size_t len, uint8_t* output);

// Easy to mix up function pointers - runtime errors
typedef struct {
    crypto_hash_fn hash;      // Could accidentally assign cipher function!
    crypto_cipher_fn encrypt;
    crypto_cipher_fn decrypt;
    size_t key_size;
    size_t block_size;
} crypto_suite_t;

// Runtime errors possible
int process_data(crypto_suite_t* suite, uint8_t* data, size_t len) {
    uint8_t hash[32];
    
    // BUG: What if hash function expects different output size?
    int result = suite->hash(data, len, hash);  // Potential buffer overflow
    if (result != 0) return result;
    
    // BUG: What if encrypt function is NULL?
    return suite->encrypt(suite->key, data, len, data);  // Potential crash
}
```

**Rust Approach - Type-Safe:**
```rust
// Trait defines interface - compiler enforces correctness
trait CryptoHash {
    type Output: AsRef<[u8]>;
    const OUTPUT_SIZE: usize;
    
    fn hash(&self, data: &[u8]) -> Self::Output;
    fn hash_into(&self, data: &[u8], output: &mut [u8]) -> Result<(), CryptoError>;
}

trait BlockCipher {
    const KEY_SIZE: usize;
    const BLOCK_SIZE: usize;
    
    fn encrypt_block(&self, key: &[u8], block: &mut [u8]) -> Result<(), CryptoError>;
    fn decrypt_block(&self, key: &[u8], block: &mut [u8]) -> Result<(), CryptoError>;
}

// Implementations are type-safe
struct Sha256;
impl CryptoHash for Sha256 {
    type Output = [u8; 32];
    const OUTPUT_SIZE: usize = 32;
    
    fn hash(&self, data: &[u8]) -> Self::Output {
        // Implementation guaranteed to return correct size
        sha256_implementation(data)
    }
    
    fn hash_into(&self, data: &[u8], output: &mut [u8]) -> Result<(), CryptoError> {
        if output.len() != Self::OUTPUT_SIZE {
            return Err(CryptoError::InvalidOutputSize);
        }
        let result = self.hash(data);
        output.copy_from_slice(&result);
        Ok(())
    }
}

struct Aes256;
impl BlockCipher for Aes256 {
    const KEY_SIZE: usize = 32;
    const BLOCK_SIZE: usize = 16;
    
    fn encrypt_block(&self, key: &[u8], block: &mut [u8]) -> Result<(), CryptoError> {
        if key.len() != Self::KEY_SIZE || block.len() != Self::BLOCK_SIZE {
            return Err(CryptoError::InvalidSize);
        }
        aes256_encrypt_block(key, block);
        Ok(())
    }
    
    fn decrypt_block(&self, key: &[u8], block: &mut [u8]) -> Result<(), CryptoError> {
        if key.len() != Self::KEY_SIZE || block.len() != Self::BLOCK_SIZE {
            return Err(CryptoError::InvalidSize);
        }
        aes256_decrypt_block(key, block);
        Ok(())
    }
}

// Generic functions work with any implementation - zero runtime cost
fn process_data<H: CryptoHash, C: BlockCipher>(
    hasher: &H, 
    cipher: &C, 
    key: &[u8], 
    data: &mut [u8]
) -> Result<H::Output, CryptoError> {
    // Compiler enforces correct sizes at compile time
    if key.len() != C::KEY_SIZE {
        return Err(CryptoError::InvalidKeySize);
    }
    
    // Type system guarantees hash output is correct size
    let hash = hasher.hash(data);
    
    // Process data in blocks
    for chunk in data.chunks_exact_mut(C::BLOCK_SIZE) {
        cipher.encrypt_block(key, chunk)?;
    }
    
    Ok(hash)
}
```

</details>

**Crypto Algorithm Abstraction:**

```rust
// Generic crypto operations using traits
trait AuthenticatedEncryption {
    type Key: AsRef<[u8]> + Zeroize;
    type Nonce: AsRef<[u8]>;
    type Tag: AsRef<[u8]>;
    
    const KEY_SIZE: usize;
    const NONCE_SIZE: usize;
    const TAG_SIZE: usize;
    
    fn encrypt(
        &self,
        key: &Self::Key,
        nonce: &Self::Nonce,
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<(Vec<u8>, Self::Tag), CryptoError>;
    
    fn decrypt(
        &self,
        key: &Self::Key,
        nonce: &Self::Nonce,
        ciphertext: &[u8],
        aad: &[u8],
        tag: &Self::Tag,
    ) -> Result<Vec<u8>, CryptoError>;
}

// AES-GCM implementation
struct AesGcm;

impl AuthenticatedEncryption for AesGcm {
    type Key = SecureKey<32>;
    type Nonce = [u8; 12];
    type Tag = [u8; 16];
    
    const KEY_SIZE: usize = 32;
    const NONCE_SIZE: usize = 12;
    const TAG_SIZE: usize = 16;
    
    fn encrypt(
        &self,
        key: &Self::Key,
        nonce: &Self::Nonce,
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<(Vec<u8>, Self::Tag), CryptoError> {
        // Hardware-accelerated AES-GCM if available
        #[cfg(feature = "hw-crypto")]
        {
            hw_aes_gcm_encrypt(key.as_bytes(), nonce, plaintext, aad)
        }
        #[cfg(not(feature = "hw-crypto"))]
        {
            sw_aes_gcm_encrypt(key.as_bytes(), nonce, plaintext, aad)
        }
    }
    
    fn decrypt(
        &self,
        key: &Self::Key,
        nonce: &Self::Nonce,
        ciphertext: &[u8],
        aad: &[u8],
        tag: &Self::Tag,
    ) -> Result<Vec<u8>, CryptoError> {
        #[cfg(feature = "hw-crypto")]
        {
            hw_aes_gcm_decrypt(key.as_bytes(), nonce, ciphertext, aad, tag)
        }
        #[cfg(not(feature = "hw-crypto"))]
        {
            sw_aes_gcm_decrypt(key.as_bytes(), nonce, ciphertext, aad, tag)
        }
    }
}

// ChaCha20-Poly1305 implementation
struct ChaCha20Poly1305;

impl AuthenticatedEncryption for ChaCha20Poly1305 {
    type Key = SecureKey<32>;
    type Nonce = [u8; 12];
    type Tag = [u8; 16];
    
    const KEY_SIZE: usize = 32;
    const NONCE_SIZE: usize = 12;
    const TAG_SIZE: usize = 16;
    
    fn encrypt(
        &self,
        key: &Self::Key,
        nonce: &Self::Nonce,
        plaintext: &[u8],
        aad: &[u8],
    ) -> Result<(Vec<u8>, Self::Tag), CryptoError> {
        chacha20_poly1305_encrypt(key.as_bytes(), nonce, plaintext, aad)
    }
    
    fn decrypt(
        &self,
        key: &Self::Key,
        nonce: &Self::Nonce,
        ciphertext: &[u8],
        aad: &[u8],
        tag: &Self::Tag,
    ) -> Result<Vec<u8>, CryptoError> {
        chacha20_poly1305_decrypt(key.as_bytes(), nonce, ciphertext, aad, tag)
    }
}

// Generic secure communication using any AEAD algorithm
fn secure_send<A: AuthenticatedEncryption>(
    aead: &A,
    key: &A::Key,
    message: &[u8],
    connection_id: u64,
) -> Result<Vec<u8>, CryptoError> {
    // Generate unique nonce from connection ID and counter
    let nonce = generate_nonce::<A>(connection_id)?;
    
    // Additional authenticated data
    let aad = connection_id.to_be_bytes();
    
    // Encrypt with authentication
    let (ciphertext, tag) = aead.encrypt(key, &nonce, message, &aad)?;
    
    // Package for transmission
    let mut packet = Vec::with_capacity(
        A::NONCE_SIZE + ciphertext.len() + A::TAG_SIZE
    );
    packet.extend_from_slice(nonce.as_ref());
    packet.extend_from_slice(&ciphertext);
    packet.extend_from_slice(tag.as_ref());
    
    Ok(packet)
}
```

#### Methods and Associated Functions

Rust's method system provides better organization than C's function naming conventions, with clear ownership semantics.

<details>
<summary><strong>▶️ C Function Naming vs Rust Methods</strong> - See the organizational improvements</summary>

**C Approach - Naming Conventions:**
```c
// C relies on naming conventions - easy to get wrong
typedef struct {
    uint8_t key[32];
    uint32_t counter;
    uint8_t nonce[12];
} aes_gcm_context_t;

// Function naming tries to indicate ownership/usage
int aes_gcm_context_init(aes_gcm_context_t* ctx, const uint8_t* key);
int aes_gcm_context_set_nonce(aes_gcm_context_t* ctx, const uint8_t* nonce);
int aes_gcm_context_encrypt(aes_gcm_context_t* ctx, const uint8_t* plaintext, 
                           size_t len, uint8_t* ciphertext, uint8_t* tag);
void aes_gcm_context_destroy(aes_gcm_context_t* ctx);

// Easy to misuse - no compiler enforcement
int crypto_operation() {
    aes_gcm_context_t ctx;
    
    // BUG: Forgot to initialize!
    aes_gcm_context_encrypt(&ctx, data, len, output, tag);  // Undefined behavior
    
    // BUG: Double destroy
    aes_gcm_context_destroy(&ctx);
    aes_gcm_context_destroy(&ctx);  // Undefined behavior
    
    return 0;
}
```

**Rust Approach - Method Organization:**
```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
pub struct AesGcmContext {
    key: [u8; 32],
    counter: u32,
    nonce: [u8; 12],
}

impl AesGcmContext {
    // Associated function (like static method in C++)
    pub fn new(key: [u8; 32]) -> Self {
        Self {
            key,
            counter: 0,
            nonce: [0; 12],
        }
    }
    
    // Method taking &mut self - exclusive access
    pub fn set_nonce(&mut self, nonce: [u8; 12]) {
        self.nonce = nonce;
        self.counter = 0;  // Reset counter for new nonce
    }
    
    // Method taking &self - shared access
    pub fn encrypt(&self, plaintext: &[u8], aad: &[u8]) -> Result<(Vec<u8>, [u8; 16]), CryptoError> {
        if self.counter == u32::MAX {
            return Err(CryptoError::NonceExhausted);
        }
        
        // Use current nonce and counter
        let mut full_nonce = [0u8; 16];
        full_nonce[..12].copy_from_slice(&self.nonce);
        full_nonce[12..].copy_from_slice(&self.counter.to_be_bytes());
        
        aes_gcm_encrypt(&self.key, &full_nonce, plaintext, aad)
    }
    
    // Method taking self - consumes the context
    pub fn finalize(self) -> [u8; 32] {
        // Return final state, context is consumed and zeroized
        self.key
    }
}

// Impossible to misuse - compiler enforces correct usage
fn crypto_operation() -> Result<(), CryptoError> {
    let key = generate_key();
    let mut ctx = AesGcmContext::new(key);  // Must initialize
    
    ctx.set_nonce(generate_nonce());
    let (ciphertext, tag) = ctx.encrypt(b"Hello", b"AAD")?;
    
    // ctx automatically zeroized when dropped - no manual cleanup needed
    Ok(())
}
```

</details>

**Crypto Context Management:**

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
pub struct CryptoSession {
    session_key: [u8; 32],
    send_counter: u64,
    recv_counter: u64,
    cipher_suite: CipherSuite,
}

impl CryptoSession {
    // Constructor - ensures proper initialization
    pub fn establish(
        master_key: &[u8; 32],
        peer_public_key: &[u8; 32],
        cipher_suite: CipherSuite,
    ) -> Result<Self, CryptoError> {
        let session_key = derive_session_key(master_key, peer_public_key)?;
        
        Ok(Self {
            session_key,
            send_counter: 0,
            recv_counter: 0,
            cipher_suite,
        })
    }
    
    // Method for sending - updates internal state
    pub fn encrypt_message(&mut self, message: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let nonce = self.generate_send_nonce()?;
        let ciphertext = self.cipher_suite.encrypt(&self.session_key, &nonce, message)?;
        
        self.send_counter += 1;
        Ok(ciphertext)
    }
    
    // Method for receiving - validates counter
    pub fn decrypt_message(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let (nonce, payload) = extract_nonce_and_payload(ciphertext)?;
        let counter = extract_counter_from_nonce(&nonce)?;
        
        // Replay protection
        if counter <= self.recv_counter {
            return Err(CryptoError::ReplayAttack);
        }
        
        let plaintext = self.cipher_suite.decrypt(&self.session_key, &nonce, payload)?;
        self.recv_counter = counter;
        
        Ok(plaintext)
    }
    
    // Associated function for key derivation
    pub fn derive_session_key(
        master_key: &[u8; 32],
        peer_key: &[u8; 32],
    ) -> Result<[u8; 32], CryptoError> {
        // ECDH key exchange
        let shared_secret = ecdh_compute(master_key, peer_key)?;
        
        // HKDF key derivation
        let mut session_key = [0u8; 32];
        hkdf_expand(&shared_secret, b"session_key", &mut session_key)?;
        
        Ok(session_key)
    }
    
    // Private helper method
    fn generate_send_nonce(&self) -> Result<[u8; 12], CryptoError> {
        if self.send_counter == u64::MAX {
            return Err(CryptoError::CounterExhausted);
        }
        
        let mut nonce = [0u8; 12];
        nonce[4..].copy_from_slice(&self.send_counter.to_be_bytes());
        Ok(nonce)
    }
}

// Usage is clear and safe
fn secure_communication() -> Result<(), CryptoError> {
    let master_key = load_master_key()?;
    let peer_key = exchange_public_keys()?;
    
    let mut session = CryptoSession::establish(&master_key, &peer_key, CipherSuite::Aes256Gcm)?;
    
    // Send message
    let message = b"Secret message";
    let encrypted = session.encrypt_message(message)?;
    send_to_peer(&encrypted)?;
    
    // Receive response
    let response = receive_from_peer()?;
    let decrypted = session.decrypt_message(&response)?;
    
    // Session automatically cleaned up when dropped
    Ok(())
}
```

#### Crypto-Specific Error Handling with Enums

Advanced enums enable sophisticated error handling that's impossible in C. This builds directly on the foundational error handling patterns covered in [Error Handling Without Exceptions](../core-concepts/error-handling.md):

```rust
#[derive(Debug, Clone)]
pub enum CryptoError {
    // Simple error cases
    InvalidKeySize,
    InvalidNonceSize,
    InvalidTagSize,
    
    // Errors with additional data
    AuthenticationFailed { 
        expected_tag: [u8; 16], 
        received_tag: [u8; 16] 
    },
    
    // Nested error information
    KeyDerivation { 
        phase: KeyDerivationPhase,
        underlying: Box<CryptoError> 
    },
    
    // Protocol-specific errors
    TlsError(TlsError),
    IkeError(IkeError),
    
    // Hardware errors with context
    HardwareError { 
        device: HardwareDevice,
        register: u32,
        expected: u32,
        actual: u32,
    },
}

#[derive(Debug, Clone)]
pub enum KeyDerivationPhase {
    EcdhCompute,
    HkdfExtract,
    HkdfExpand,
    KeyValidation,
}

#[derive(Debug, Clone)]
pub enum HardwareDevice {
    CryptoAccelerator,
    TrueRandomGenerator,
    SecureStorage,
}

impl CryptoError {
    // Methods on error types for better handling
    pub fn is_recoverable(&self) -> bool {
        match self {
            CryptoError::InvalidKeySize | 
            CryptoError::InvalidNonceSize | 
            CryptoError::InvalidTagSize => false,
            
            CryptoError::AuthenticationFailed { .. } => false,
            
            CryptoError::HardwareError { device, .. } => {
                // Some hardware errors are recoverable
                matches!(device, HardwareDevice::TrueRandomGenerator)
            },
            
            CryptoError::KeyDerivation { underlying, .. } => {
                underlying.is_recoverable()
            },
            
            _ => true,
        }
    }
    
    pub fn security_level(&self) -> SecurityLevel {
        match self {
            CryptoError::AuthenticationFailed { .. } => SecurityLevel::Critical,
            CryptoError::HardwareError { device: HardwareDevice::SecureStorage, .. } => SecurityLevel::Critical,
            CryptoError::TlsError(_) => SecurityLevel::High,
            _ => SecurityLevel::Medium,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

// Pattern matching enables sophisticated error handling
fn handle_crypto_error(error: CryptoError) -> RecoveryAction {
    match error {
        // Handle authentication failures with detailed logging
        CryptoError::AuthenticationFailed { expected_tag, received_tag } => {
            security_log(SecurityLevel::Critical, &format!(
                "Authentication failed - expected: {:02x?}, received: {:02x?}",
                expected_tag, received_tag
            ));
            RecoveryAction::Abort
        },
        
        // Handle hardware errors based on device type
        CryptoError::HardwareError { device, register, expected, actual } => {
            match device {
                HardwareDevice::CryptoAccelerator => {
                    log::error!("Crypto accelerator error at register 0x{:08x}: expected 0x{:08x}, got 0x{:08x}", 
                               register, expected, actual);
                    RecoveryAction::FallbackToSoftware
                },
                HardwareDevice::TrueRandomGenerator => {
                    log::warn!("RNG error, retrying with different source");
                    RecoveryAction::RetryWithFallback
                },
                HardwareDevice::SecureStorage => {
                    security_log(SecurityLevel::Critical, "Secure storage compromised");
                    RecoveryAction::EmergencyShutdown
                },
            }
        },
        
        // Handle nested errors recursively
        CryptoError::KeyDerivation { phase, underlying } => {
            log::error!("Key derivation failed in phase {:?}", phase);
            handle_crypto_error(*underlying)
        },
        
        // Simple errors
        error if error.is_recoverable() => RecoveryAction::Retry,
        _ => RecoveryAction::Abort,
    }
}

#[derive(Debug)]
enum RecoveryAction {
    Continue,
    Retry,
    RetryWithFallback,
    FallbackToSoftware,
    Abort,
    EmergencyShutdown,
}
```

**→ Next:** [Functional Programming and Data Processing](../core-concepts/functional.md) - Mathematical operations, iterators, and closures for embedded crypto

**Migration Applications:**
- → [Incremental Migration Strategies](../migration/strategies.md) - Apply type system features when migrating from C
- → [FFI Integration with C Libraries](../migration/ffi-integration.md) - Use enums and traits to wrap C APIs safely

---