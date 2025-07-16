# 3.1 Ownership and Memory Management

<details>
<summary><strong>▶️ Ownership Rules Summary</strong> - The three fundamental rules</summary>

#### The Three Rules of Ownership
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

#### Ownership in Embedded Crypto Context

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

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
    
    fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Exclusive mutable access prevents concurrent key usage
        self.nonce_counter += 1;
        
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
```

#### Borrowing Rules for Crypto Operations

```rust
// Immutable borrowing - safe concurrent reads
fn verify_multiple_signatures(
    public_key: &[u8; 32],     // Immutable borrow
    messages: &[&[u8]],        // Multiple immutable borrows allowed
    signatures: &[[u8; 64]]
) -> Vec<bool> {
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
```

#### Memory Management Patterns for Embedded

```rust
#![no_std]

use heapless::Vec;

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

// Static allocation for global crypto state
static mut GLOBAL_CRYPTO_CTX: Option<CryptoContext> = None;

fn init_global_crypto(key: [u8; 32]) {
    unsafe {
        GLOBAL_CRYPTO_CTX = Some(CryptoContext::new(key));
    }
}

// Heapless collections for message queues
fn message_queue_example() -> Result<(), CryptoError> {
    let mut encrypted_messages: Vec<[u8; 32], 16> = Vec::new();
    
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
```