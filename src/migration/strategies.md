# 6.1 Incremental Migration Strategies {incremental-migration-strategies}

Migrating large cryptographic codebases requires careful planning and incremental approaches. This section provides step-by-step strategies for systematic migration from C to Rust.

**Key Migration Concepts:**
- → [Advanced Type System Features](../core-concepts/advanced-types.md) - Use enums and traits to improve C designs
- → [Functional Programming and Data Processing](../core-concepts/functional.md) - Replace C loops with safer iterators
- → [Error Handling Without Exceptions](../core-concepts/error-handling.md) - Migrate from error codes to Result types

#### Step 1: Assessment and Planning

**Migration Assessment Checklist:**

1. **Identify Dependencies**
   - Map all crypto modules and their interdependencies
   - Identify external library dependencies (OpenSSL, mbedTLS, etc.)
   - Document hardware-specific code (register access, DMA, interrupts)
   - Catalog test vectors and validation procedures

2. **Risk Assessment**
   - Identify critical security components that cannot fail
   - Determine acceptable downtime for migration phases
   - Plan rollback strategies for each migration step
   - Document compliance requirements (FIPS, Common Criteria)

3. **Migration Order Planning**
   ```

   Phase 1: Leaf Modules (crypto primitives)
   Phase 2: Utility Functions (key derivation, random number generation)
   Phase 3: Protocol Implementations (TLS, IPSec, etc.)
   Phase 4: Application Integration
   Phase 5: Hardware Abstraction Layer
   ```

#### Step 2: Module-by-Module Migration

**Starting with Crypto Primitives (Lowest Risk)**




```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use aes::{Aes256, cipher::{KeyInit, BlockEncrypt, BlockDecrypt}};

#[derive(Debug)]
pub struct CryptoError(&'static str);


use core::mem;
use core::fmt;
use core::result::Result;


// Step 2a: Create Rust implementation of AES
// Original C: aes.c, aes.h
// New Rust: crypto/aes.rs
// This example demonstrates advanced type system features from
// → [Advanced Type System Features](../core-concepts/advanced-types.md)

use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
pub struct AesContext {
    key_schedule: [u32; 60],
    rounds: usize,
}

impl AesContext {
    pub fn new(key: &[u8]) -> Result<Self, CryptoError> {
        let mut ctx = Self {
            key_schedule: [0; 60],
            rounds: match key.len() {
                16 => 10,
                24 => 12,
                32 => 14,
                _ => return Err(CryptoError::InvalidKeySize),
            },
        };
        
        aes_key_expansion(key, &mut ctx.key_schedule, ctx.rounds)?;
        Ok(ctx)
    }
    
    pub fn encrypt_block(&self, input: &[u8; 16]) -> [u8; 16] {
        let mut output = [0u8; 16];
        aes_encrypt_block(input, &mut output, &self.key_schedule, self.rounds);
        output
    }
    
    pub fn decrypt_block(&self, input: &[u8; 16]) -> [u8; 16] {
        let mut output = [0u8; 16];
        aes_decrypt_block(input, &mut output, &self.key_schedule, self.rounds);
        output
    }
}

// Step 2b: Create C-compatible interface for gradual migration
#[no_mangle]
pub extern "C" fn aes_context_new(key: *const u8, key_len: usize) -> *mut AesContext {
    if key.is_null() || key_len == 0 {
        return core::ptr::null_mut();
    }
    
    let key_slice = unsafe { core::slice::from_raw_parts(key, key_len) };
    
    match AesContext::new(key_slice) {
        Ok(ctx) => Box::into_raw(Box::new(ctx)),
        Err(_) => core::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn aes_context_free(ctx: *mut AesContext) {
    if !ctx.is_null() {
        unsafe {
            let _ = Box::from_raw(ctx); // Automatic zeroization via Drop
        }
    }
}

#[no_mangle]
pub extern "C" fn aes_encrypt_block_c(
    ctx: *const AesContext,
    input: *const u8,
    output: *mut u8,
) -> i32 {
    if ctx.is_null() || input.is_null() || output.is_null() {
        return -1;
    }
    
    unsafe {
        let ctx = &*ctx;
        let input_block = core::slice::from_raw_parts(input, 16);
        let output_block = core::slice::from_raw_parts_mut(output, 16);
        
        let input_array: [u8; 16] = match input_block.try_into() {
            Ok(arr) => arr,
            Err(_) => return -1,
        };
        
        let result = ctx.encrypt_block(&input_array);
        output_block.copy_from_slice(&result);
    }
    
    0 // Success
}

// Step 2c: Update C code to use new Rust implementation
// In your C files, replace direct AES calls:
/*
// Old C code:
AES_KEY aes_key;
AES_set_encrypt_key(key, 256, &aes_key);
AES_encrypt(plaintext, ciphertext, &aes_key);

// New C code using Rust backend:
void* aes_ctx = aes_context_new(key, 32);
if (aes_ctx) {
    aes_encrypt_block_c(aes_ctx, plaintext, ciphertext);
    aes_context_free(aes_ctx);
}
*/
```

#### Step 3: Protocol-Level Migration

**Migrating Higher-Level Protocols While Keeping Proven Crypto**

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;
use sha2::{Sha256, Digest};

// Stub function for HMAC
fn hmac_sha256(_key: &[u8; 32], _message: &[u8]) -> Result<[u8; 32], CryptoError> {
    Ok([0u8; 32])
}


use zeroize::{Zeroize, ZeroizeOnDrop};
use core::mem;
use heapless::Vec;

use core::fmt;
use core::result::Result;

// Step 3a: Define external C crypto functions still in use
extern "C" {
    fn c_aes_gcm_encrypt(
        key: *const u8, key_len: usize,
        iv: *const u8, iv_len: usize,
        plaintext: *const u8, plaintext_len: usize,
        ciphertext: *mut u8,
        tag: *mut u8
    ) -> i32;
    
    fn c_hmac_sha256(
        key: *const u8, key_len: usize,
        data: *const u8, data_len: usize,
        mac: *mut u8
    ) -> i32;
    
    fn c_ecdsa_sign(
        private_key: *const u8,
        message_hash: *const u8,
        signature: *mut u8
    ) -> i32;
}

// Step 3b: Implement protocol logic in Rust using C crypto
#[derive(Debug)]
pub enum TlsError {
    EncryptionFailed,
    MacFailed,
    SignatureFailed,
    InvalidState,
}

pub struct HybridTlsConnection {
    state: TlsState,
    session_keys: SessionKeys,
    sequence_number: u64,
}

impl HybridTlsConnection {
    pub fn new(master_secret: &[u8; 48]) -> Result<Self, TlsError> {
        let session_keys = derive_session_keys(master_secret)?;
        
        Ok(Self {
            state: TlsState::Connected,
            session_keys,
            sequence_number: 0,
        })
    }
    
    pub fn encrypt_record(&mut self, record_type: u8, data: &[u8]) -> Result<heapless::Vec<u8, 32>, TlsError> {
        // Step 3c: Use Rust for protocol logic, C for crypto operations
        let sequence_bytes = self.sequence_number.to_be_bytes();
        let mut iv = [0u8; 12];
        iv[4..].copy_from_slice(&sequence_bytes);
        
        let mut ciphertext = vec![0u8; data.len()];
        let mut tag = [0u8; 16];
        
        // Use C crypto function temporarily during migration
        let result = unsafe {
            c_aes_gcm_encrypt(
                self.session_keys.encryption_key.as_ptr(),
                self.session_keys.encryption_key.len(),
                iv.as_ptr(),
                iv.len(),
                data.as_ptr(),
                data.len(),
                ciphertext.as_mut_ptr(),
                tag.as_mut_ptr(),
            )
        };
        
        if result != 0 {
            return Err(TlsError::EncryptionFailed);
        }
        
        // Rust-based record formatting (gradually migrating)
        let mut record = Vec::with_capacity(5 + ciphertext.len() + tag.len());
        record.push(record_type);
        record.extend_from_slice(&[0x03, 0x03]); // TLS 1.2
        record.extend_from_slice(&((ciphertext.len() + tag.len()) as u16).to_be_bytes());
        record.extend_from_slice(&ciphertext);
        record.extend_from_slice(&tag);
        
        self.sequence_number += 1;
        Ok(record)
    }
    
    pub fn decrypt_record(&mut self, record: &[u8]) -> Result<heapless::Vec<u8, 32>, TlsError> {
        if record.len() < 21 { // Minimum record size
            return Err(TlsError::InvalidState);
        }
        
        let payload_len = u16::from_be_bytes([record[3], record[4]]) as usize;
        let ciphertext = &record[5..5 + payload_len - 16];
        let tag = &record[5 + payload_len - 16..5 + payload_len];
        
        // Implementation continues with C crypto calls...
        // This allows testing new protocol logic with proven crypto
        
        Ok(vec![]) // Placeholder
    }
}

// Step 3d: Session key derivation in pure Rust (lower risk)
fn derive_session_keys(master_secret: &[u8; 48]) -> Result<SessionKeys, TlsError> {
    
    let mut hasher = Sha256::new();
    hasher.update(b"key expansion");
    hasher.update(master_secret);
    let key_material = hasher.finalize();
    
    Ok(SessionKeys {
        encryption_key: key_material[0..32].try_into().unwrap(),
        mac_key: key_material[32..64].try_into().unwrap(),
    })
}

#[derive(ZeroizeOnDrop)]
struct SessionKeys {
    encryption_key: [u8; 32],
    mac_key: [u8; 32],
}

#[derive(Debug, PartialEq)]
enum TlsState {
    Handshake,
    Connected,
    Closed,
}
```

#### Step 4: Application Integration Migration

**Migrating Application Logic to Use New Rust Interfaces**

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;


use core::mem;
use heapless::Vec;

use core::fmt;
use core::result::Result;

// Step 4a: Create high-level application interface
pub struct SecureCommunicationManager {
    connections: heapless::FnvIndexMap<u32, HybridTlsConnection, 16>,
    next_connection_id: u32,
}

impl SecureCommunicationManager {
    pub fn new() -> Self {
        Self {
            connections: heapless::FnvIndexMap::new(),
            next_connection_id: 1,
        }
    }
    
    pub fn create_connection(&mut self, master_secret: &[u8; 48]) -> Result<u32, TlsError> {
        let connection = HybridTlsConnection::new(master_secret)?;
        let connection_id = self.next_connection_id;
        
        self.connections.insert(connection_id, connection)
            .map_err(|_| TlsError::InvalidState)?;
        
        self.next_connection_id += 1;
        Ok(connection_id)
    }
    
    pub fn send_data(&mut self, connection_id: u32, data: &[u8]) -> Result<heapless::Vec<u8, 32>, TlsError> {
        let connection = self.connections.get_mut(&connection_id)
            .ok_or(TlsError::InvalidState)?;
        
        connection.encrypt_record(0x17, data) // Application data
    }
    
    pub fn receive_data(&mut self, connection_id: u32, record: &[u8]) -> Result<heapless::Vec<u8, 32>, TlsError> {
        let connection = self.connections.get_mut(&connection_id)
            .ok_or(TlsError::InvalidState)?;
        
        connection.decrypt_record(record)
    }
}

// Step 4b: C interface for existing application code
#[no_mangle]
pub extern "C" fn secure_comm_manager_new() -> *mut SecureCommunicationManager {
    Box::into_raw(Box::new(SecureCommunicationManager::new()))
}

#[no_mangle]
pub extern "C" fn secure_comm_create_connection(
    manager: *mut SecureCommunicationManager,
    master_secret: *const u8,
) -> i32 {
    if manager.is_null() || master_secret.is_null() {
        return -1;
    }
    
    unsafe {
        let manager = &mut *manager;
        let secret_slice = core::slice::from_raw_parts(master_secret, 48);
        let secret_array: [u8; 48] = match secret_slice.try_into() {
            Ok(arr) => arr,
            Err(_) => return -1,
        };
        
        match manager.create_connection(&secret_array) {
            Ok(id) => id as i32,
            Err(_) => -1,
        }
    }
}

// Step 4c: Update existing C application code
/*
// Old C application code:
static tls_context_t* tls_ctx;

void app_init() {
    tls_ctx = tls_context_create();
}

int app_send_message(uint8_t* data, size_t len) {
    return tls_encrypt_and_send(tls_ctx, data, len);
}

// New C application code using Rust backend:
static void* secure_manager;
static int connection_id;

void app_init() {
    secure_manager = secure_comm_manager_new();
    uint8_t master_secret[48] = {...};
    connection_id = secure_comm_create_connection(secure_manager, master_secret);
}

int app_send_message(uint8_t* data, size_t len) {
    return secure_comm_send_data(secure_manager, connection_id, data, len);
}
*/
```

#### Step 5: Hardware Abstraction Layer Migration

**Final Step: Migrating Hardware-Specific Code**

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};
use aes::{Aes256, cipher::{KeyInit, BlockEncrypt, BlockDecrypt}};


use core::mem;
use core::fmt;

use core::result::Result;

// Step 5a: Create Rust HAL for crypto hardware
pub struct CryptoHardware {
    base_addr: *mut u32,
}

impl CryptoHardware {
    pub unsafe fn new(base_addr: usize) -> Self {
        Self {
            base_addr: base_addr as *mut u32,
        }
    }
    
    pub fn aes_hardware_encrypt(&self, key: &[u8; 32], plaintext: &[u8; 16]) -> [u8; 16] {
        unsafe {
            // Load key into hardware registers
            for (i, chunk) in key.chunks(4).enumerate() {
                let key_word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                core::ptr::write_volatile(self.base_addr.offset(i as isize), key_word);
            }
            
            // Load plaintext
            for (i, chunk) in plaintext.chunks(4).enumerate() {
                let data_word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                core::ptr::write_volatile(self.base_addr.offset(8 + i as isize), data_word);
            }
            
            // Start encryption
            core::ptr::write_volatile(self.base_addr.offset(16), 0x01);
            
            // Wait for completion
            while core::ptr::read_volatile(self.base_addr.offset(17)) & 0x01 == 0 {
                cortex_m::asm::nop();
            }
            
            // Read result
            let mut result = [0u8; 16];
            for (i, chunk) in result.chunks_mut(4).enumerate() {
                let word = core::ptr::read_volatile(self.base_addr.offset(18 + i as isize));
                chunk.copy_from_slice(&word.to_le_bytes());
            }
            
            result
        }
    }
}

// Step 5b: Integration with existing crypto implementations
impl AesContext {
    pub fn encrypt_block_hw(&self, input: &[u8; 16], hw: &CryptoHardware) -> [u8; 16] {
        // Use hardware acceleration when available
        if let Some(key_bytes) = self.key_schedule_as_bytes() {
            hw.aes_hardware_encrypt(&key_bytes, input)
        } else {
            // Fallback to software implementation
            self.encrypt_block(input)
        }
    }
    
    fn key_schedule_as_bytes(&self) -> Option<[u8; 32]> {
        // Convert key schedule back to original key if possible
        // This is a simplified example - real implementation would be more complex
        None // Placeholder
    }
}
```

#### Migration Validation Checklist

**After Each Migration Step:**

1. **Functional Testing**
   - All existing test vectors pass
   - Cross-validation between C and Rust implementations
   - Performance benchmarks meet requirements

2. **Security Validation**
   - Side-channel analysis shows no timing leaks
   - Memory is properly zeroized
   - No new attack surfaces introduced

3. **Integration Testing**
   - Existing applications continue to work
   - No regressions in functionality
   - Error handling works correctly

4. **Rollback Verification**
   - Can revert to previous implementation if needed
   - Rollback procedure tested and documented
   - Data compatibility maintained
```