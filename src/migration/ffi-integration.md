# 6.2 FFI Integration with C Libraries {ffi-integration-with-c-libraries}

Interfacing with existing C cryptographic libraries during migration requires careful attention to safety and proper resource management. This section provides comprehensive examples for common integration scenarios.

**Advanced Integration Techniques:**
- → [Advanced Type System Features](../core-concepts/advanced-types.md) - Use enums and traits to wrap C APIs safely
- → [Error Handling Without Exceptions](../core-concepts/error-handling.md) - Convert C error codes to Result types

## Understanding FFI Basics

Foreign Function Interface (FFI) allows Rust code to call C functions and vice versa. Here's how the connection works:

### Declaring External Functions

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;

// Declare C functions that exist in external libraries
extern "C" {
    // Function from libc
    fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
    
    // Function from a crypto library
    fn crypto_hash_sha256(out: *mut u8, input: *const u8, len: usize) -> i32;
}

// Safe wrapper around unsafe C function
fn safe_sha256(input: &[u8]) -> Result<[u8; 32], &'static str> {
    let mut hash = [0u8; 32];
    
    // Call the C function within unsafe block
    let result = unsafe {
        crypto_hash_sha256(
            hash.as_mut_ptr(),
            input.as_ptr(),
            input.len()
        )
    };
    
    if result == 0 {
        Ok(hash)
    } else {
        Err("Hash computation failed")
    }
}

#[entry]
fn main() -> ! {
    let data = b"Hello, FFI!";
    let _ = safe_sha256(data);
    loop {}
}
```

### Linking C Libraries

To use external C libraries, you need to tell the Rust compiler how to link them:

```toml
# Cargo.toml
[dependencies]
# Your Rust dependencies

[build-dependencies]
cc = "1.0"  # For compiling C code

# Link to system libraries
[target.'cfg(target_os = "linux")'.dependencies]
libc = "0.2"
```

```rust
// build.rs - Compiles C code and links libraries
fn main() {
    // Tell cargo to link a system library
    println!("cargo:rustc-link-lib=crypto");
    
    // Tell cargo where to find the library
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    
    // Compile C source files
    cc::Build::new()
        .file("src/crypto_wrapper.c")
        .include("src")
        .compile("crypto_wrapper");
}
```

## Safe FFI Wrapper Patterns

### Pattern 1: Resource Management with RAII

This pattern shows how to safely wrap C library resources (like OpenSSL contexts) with Rust's ownership system to ensure proper cleanup.

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use zeroize::{Zeroize, ZeroizeOnDrop};
use core::ffi::{c_int, c_void, c_uchar};

// External C functions - OpenSSL example
extern "C" {
    fn EVP_CIPHER_CTX_new() -> *mut c_void;
    fn EVP_CIPHER_CTX_free(ctx: *mut c_void);
    fn EVP_EncryptInit_ex(
        ctx: *mut c_void,
        cipher: *const c_void,
        engine: *mut c_void,
        key: *const c_uchar,
        iv: *const c_uchar,
    ) -> c_int;
    fn EVP_EncryptUpdate(
        ctx: *mut c_void,
        out: *mut c_uchar,
        outl: *mut c_int,
        input: *const c_uchar,
        inl: c_int,
    ) -> c_int;
    fn EVP_EncryptFinal_ex(
        ctx: *mut c_void,
        out: *mut c_uchar,
        outl: *mut c_int,
    ) -> c_int;
    fn EVP_aes_256_cbc() -> *const c_void;
    fn EVP_aes_256_gcm() -> *const c_void;
}

#[derive(Debug)]
pub enum CryptoError {
    InitializationFailed,
    EncryptionFailed,
    DecryptionFailed,
    InvalidInput,
    InsufficientBuffer,
}

// Safe Rust wrapper with automatic resource cleanup
pub struct OpensslAes {
    ctx: *mut c_void,
    _key: [u8; 32], // Keep key for zeroization
}

impl OpensslAes {
    pub fn new_cbc(key: &[u8; 32], iv: &[u8; 16]) -> Result<Self, CryptoError> {
        let ctx = unsafe { EVP_CIPHER_CTX_new() };
        if ctx.is_null() {
            return Err(CryptoError::InitializationFailed);
        }
        
        let result = unsafe {
            EVP_EncryptInit_ex(
                ctx,
                EVP_aes_256_cbc(),
                core::ptr::null_mut(),
                key.as_ptr(),
                iv.as_ptr(),
            )
        };
        
        if result != 1 {
            unsafe { EVP_CIPHER_CTX_free(ctx) };
            return Err(CryptoError::InitializationFailed);
        }
        
        Ok(Self { 
            ctx,
            _key: *key, // Store for secure cleanup
        })
    }
    
    pub fn encrypt(&mut self, plaintext: &[u8], ciphertext: &mut [u8]) -> Result<usize, CryptoError> {
        if ciphertext.len() < plaintext.len() + 16 {
            return Err(CryptoError::InsufficientBuffer);
        }
        
        let mut outlen: c_int = 0;
        let mut total_len = 0usize;
        
        // Update phase
        let result = unsafe {
            EVP_EncryptUpdate(
                self.ctx,
                ciphertext.as_mut_ptr(),
                &mut outlen,
                plaintext.as_ptr(),
                plaintext.len() as c_int,
            )
        };
        
        if result != 1 {
            return Err(CryptoError::EncryptionFailed);
        }
        
        total_len += outlen as usize;
        
        // Final phase
        let result = unsafe {
            EVP_EncryptFinal_ex(
                self.ctx,
                ciphertext.as_mut_ptr().add(total_len),
                &mut outlen,
            )
        };
        
        if result != 1 {
            return Err(CryptoError::EncryptionFailed);
        }
        
        total_len += outlen as usize;
        Ok(total_len)
    }
}

impl Drop for OpensslAes {
    fn drop(&mut self) {
        unsafe {
            EVP_CIPHER_CTX_free(self.ctx);
        }
        // _key is automatically zeroized by ZeroizeOnDrop
    }
}

// Implement ZeroizeOnDrop for secure key cleanup
impl ZeroizeOnDrop for OpensslAes {}

#[entry]
fn main() -> ! {
    let key = [0u8; 32];
    let iv = [0u8; 16];
    let plaintext = b"Hello, OpenSSL!";
    let mut ciphertext = [0u8; 64];
    
    match OpensslAes::new_cbc(&key, &iv) {
        Ok(mut aes) => {
            let _ = aes.encrypt(plaintext, &mut ciphertext);
        }
        Err(_) => {
            // Handle error
        }
    }
    
    loop {}
}
```

### Pattern 2: HMAC Integration with Error Handling

This pattern demonstrates how to wrap HMAC functions with proper error handling and secure key management.

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use zeroize::{Zeroize, ZeroizeOnDrop};
use heapless::Vec;
use heapless::consts::U64;
use core::ffi::{c_int, c_void, c_uchar};

// External HMAC functions
extern "C" {
    fn HMAC_CTX_new() -> *mut c_void;
    fn HMAC_CTX_free(ctx: *mut c_void);
    fn HMAC_Init_ex(
        ctx: *mut c_void,
        key: *const c_void,
        len: c_int,
        md: *const c_void,
        engine: *mut c_void,
    ) -> c_int;
    fn HMAC_Update(ctx: *mut c_void, data: *const c_uchar, len: usize) -> c_int;
    fn HMAC_Final(ctx: *mut c_void, md: *mut c_uchar, len: *mut u32) -> c_int;
    fn EVP_sha256() -> *const c_void;
}

#[derive(Debug)]
enum CryptoError {
    InitializationFailed,
    UpdateFailed,
    FinalizeFailed,
}

pub struct OpensslHmac {
    ctx: *mut c_void,
    _key: Vec<u8, U64>, // Variable-length key storage
}

impl OpensslHmac {
    pub fn new_sha256(key: &[u8]) -> Result<Self, CryptoError> {
        let ctx = unsafe { HMAC_CTX_new() };
        if ctx.is_null() {
            return Err(CryptoError::InitializationFailed);
        }
        
        let result = unsafe {
            HMAC_Init_ex(
                ctx,
                key.as_ptr() as *const c_void,
                key.len() as c_int,
                EVP_sha256(),
                core::ptr::null_mut(),
            )
        };
        
        if result != 1 {
            unsafe { HMAC_CTX_free(ctx) };
            return Err(CryptoError::InitializationFailed);
        }
        
        let mut key_vec = Vec::new();
        key_vec.extend_from_slice(key).map_err(|_| {
            unsafe { HMAC_CTX_free(ctx) };
            CryptoError::InitializationFailed
        })?;
        
        Ok(Self {
            ctx,
            _key: key_vec,
        })
    }
    
    pub fn update(&mut self, data: &[u8]) -> Result<(), CryptoError> {
        let result = unsafe {
            HMAC_Update(self.ctx, data.as_ptr(), data.len())
        };
        
        if result == 1 {
            Ok(())
        } else {
            Err(CryptoError::UpdateFailed)
        }
    }
    
    pub fn finalize(self) -> Result<[u8; 32], CryptoError> {
        let mut mac = [0u8; 32];
        let mut mac_len = 32u32;
        
        let result = unsafe {
            HMAC_Final(self.ctx, mac.as_mut_ptr(), &mut mac_len)
        };
        
        if result == 1 && mac_len == 32 {
            Ok(mac)
        } else {
            Err(CryptoError::FinalizeFailed)
        }
    }
}

impl Drop for OpensslHmac {
    fn drop(&mut self) {
        unsafe {
            HMAC_CTX_free(self.ctx);
        }
        self._key.zeroize();
    }
}

#[entry]
fn main() -> ! {
    let key = b"secret_key";
    let message = b"message to authenticate";
    
    match OpensslHmac::new_sha256(key) {
        Ok(mut hmac) => {
            if hmac.update(message).is_ok() {
                match hmac.finalize() {
                    Ok(mac) => {
                        // Use MAC
                    }
                    Err(_) => {
                        // Handle error
                    }
                }
            }
        }
        Err(_) => {
            // Handle error
        }
    }
    
    loop {}
}
```

## Integration with mbedTLS

### Complete mbedTLS Integration Example

mbedTLS is a popular embedded crypto library. Here's how to integrate it with Rust:

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use zeroize::{Zeroize, ZeroizeOnDrop};
use core::ffi::{c_int, c_void, c_uchar};

// mbedTLS FFI bindings
extern "C" {
    // AES functions
    fn mbedtls_aes_init(ctx: *mut c_void);
    fn mbedtls_aes_free(ctx: *mut c_void);
    fn mbedtls_aes_setkey_enc(ctx: *mut c_void, key: *const c_uchar, keysize: u32) -> c_int;
    fn mbedtls_aes_crypt_cbc(
        ctx: *mut c_void,
        mode: c_int,
        length: usize,
        iv: *mut c_uchar,
        input: *const c_uchar,
        output: *mut c_uchar,
    ) -> c_int;
}

const MBEDTLS_AES_ENCRYPT: c_int = 1;
const MBEDTLS_AES_DECRYPT: c_int = 0;

// mbedTLS context sizes (platform-specific)
// These need to match your mbedTLS configuration
const MBEDTLS_AES_CONTEXT_SIZE: usize = 288;

#[derive(Debug)]
enum CryptoError {
    InitializationFailed,
    EncryptionFailed,
    InvalidInput,
}

// Safe wrapper around mbedTLS AES
pub struct MbedTlsAes {
    ctx: [u8; MBEDTLS_AES_CONTEXT_SIZE],
    _key: [u8; 32],
}

impl MbedTlsAes {
    pub fn new(key: &[u8; 32]) -> Result<Self, CryptoError> {
        let mut aes = Self {
            ctx: [0; MBEDTLS_AES_CONTEXT_SIZE],
            _key: *key,
        };
        
        unsafe {
            mbedtls_aes_init(aes.ctx.as_mut_ptr() as *mut c_void);
            
            let result = mbedtls_aes_setkey_enc(
                aes.ctx.as_mut_ptr() as *mut c_void,
                key.as_ptr(),
                256, // Key size in bits
            );
            
            if result != 0 {
                mbedtls_aes_free(aes.ctx.as_mut_ptr() as *mut c_void);
                return Err(CryptoError::InitializationFailed);
            }
        }
        
        Ok(aes)
    }
    
    pub fn encrypt_cbc(&mut self, iv: &mut [u8; 16], plaintext: &[u8], ciphertext: &mut [u8]) -> Result<(), CryptoError> {
        if plaintext.len() != ciphertext.len() || plaintext.len() % 16 != 0 {
            return Err(CryptoError::InvalidInput);
        }
        
        let result = unsafe {
            mbedtls_aes_crypt_cbc(
                self.ctx.as_mut_ptr() as *mut c_void,
                MBEDTLS_AES_ENCRYPT,
                plaintext.len(),
                iv.as_mut_ptr(),
                plaintext.as_ptr(),
                ciphertext.as_mut_ptr(),
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            Err(CryptoError::EncryptionFailed)
        }
    }
}

impl Drop for MbedTlsAes {
    fn drop(&mut self) {
        unsafe {
            mbedtls_aes_free(self.ctx.as_mut_ptr() as *mut c_void);
        }
    }
}

impl ZeroizeOnDrop for MbedTlsAes {}

#[entry]
fn main() -> ! {
    let key = [0u8; 32];
    let mut iv = [0u8; 16];
    let plaintext = [0u8; 32]; // Must be multiple of 16
    let mut ciphertext = [0u8; 32];
    
    match MbedTlsAes::new(&key) {
        Ok(mut aes) => {
            let _ = aes.encrypt_cbc(&mut iv, &plaintext, &mut ciphertext);
        }
        Err(_) => {
            // Handle error
        }
    }
    
    loop {}
}
```

### mbedTLS Random Number Generator

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use core::ffi::{c_int, c_void, c_uchar};

// Random number generation functions
extern "C" {
    fn mbedtls_ctr_drbg_init(ctx: *mut c_void);
    fn mbedtls_ctr_drbg_free(ctx: *mut c_void);
    fn mbedtls_ctr_drbg_seed(
        ctx: *mut c_void,
        entropy_func: *mut c_void,
        entropy: *mut c_void,
        custom: *const c_uchar,
        len: usize,
    ) -> c_int;
    fn mbedtls_ctr_drbg_random(data: *mut c_void, output: *mut c_uchar, len: usize) -> c_int;
    
    // Entropy functions
    fn mbedtls_entropy_init(ctx: *mut c_void);
    fn mbedtls_entropy_free(ctx: *mut c_void);
    fn mbedtls_entropy_func(data: *mut c_void, output: *mut c_uchar, len: usize) -> c_int;
}

const MBEDTLS_CTR_DRBG_CONTEXT_SIZE: usize = 352;
const MBEDTLS_ENTROPY_CONTEXT_SIZE: usize = 1024;

#[derive(Debug)]
enum RngError {
    InitializationFailed,
    GenerationFailed,
}

// Random number generator wrapper
pub struct MbedTlsRng {
    ctr_drbg_ctx: [u8; MBEDTLS_CTR_DRBG_CONTEXT_SIZE],
    entropy_ctx: [u8; MBEDTLS_ENTROPY_CONTEXT_SIZE],
}

impl MbedTlsRng {
    pub fn new() -> Result<Self, RngError> {
        let mut rng = Self {
            ctr_drbg_ctx: [0; MBEDTLS_CTR_DRBG_CONTEXT_SIZE],
            entropy_ctx: [0; MBEDTLS_ENTROPY_CONTEXT_SIZE],
        };
        
        unsafe {
            // Initialize entropy source
            mbedtls_entropy_init(rng.entropy_ctx.as_mut_ptr() as *mut c_void);
            
            // Initialize DRBG
            mbedtls_ctr_drbg_init(rng.ctr_drbg_ctx.as_mut_ptr() as *mut c_void);
            
            // Seed the DRBG with entropy
            let result = mbedtls_ctr_drbg_seed(
                rng.ctr_drbg_ctx.as_mut_ptr() as *mut c_void,
                Some(mbedtls_entropy_func),
                rng.entropy_ctx.as_mut_ptr() as *mut c_void,
                b"Rust-mbedTLS-RNG".as_ptr(),
                16,
            );
            
            if result != 0 {
                mbedtls_ctr_drbg_free(rng.ctr_drbg_ctx.as_mut_ptr() as *mut c_void);
                mbedtls_entropy_free(rng.entropy_ctx.as_mut_ptr() as *mut c_void);
                return Err(RngError::InitializationFailed);
            }
        }
        
        Ok(rng)
    }
    
    pub fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RngError> {
        let result = unsafe {
            mbedtls_ctr_drbg_random(
                self.ctr_drbg_ctx.as_mut_ptr() as *mut c_void,
                dest.as_mut_ptr(),
                dest.len(),
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            Err(RngError::GenerationFailed)
        }
    }
}

impl Drop for MbedTlsRng {
    fn drop(&mut self) {
        unsafe {
            mbedtls_ctr_drbg_free(self.ctr_drbg_ctx.as_mut_ptr() as *mut c_void);
            mbedtls_entropy_free(self.entropy_ctx.as_mut_ptr() as *mut c_void);
        }
    }
}

#[entry]
fn main() -> ! {
    match MbedTlsRng::new() {
        Ok(mut rng) => {
            let mut random_bytes = [0u8; 32];
            let _ = rng.fill_bytes(&mut random_bytes);
        }
        Err(_) => {
            // Handle error
        }
    }
    
    loop {}
}
```

## Hardware Crypto Library Integration

### Xilinx CSU (Crypto Services Unit) Integration

Many embedded platforms provide hardware crypto acceleration. Here's how to integrate with Xilinx's hardware crypto:

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use zeroize::{Zeroize, ZeroizeOnDrop};
use core::ffi::{c_int, c_void};

// Xilinx CSU FFI bindings for hardware crypto acceleration
extern "C" {
    fn XCsuDma_Initialize(instance_ptr: *mut c_void, config_ptr: *const c_void) -> c_int;
    fn XCsuDma_Transfer(
        instance_ptr: *mut c_void,
        channel: u32,
        src: u64,
        dst: u64,
        size: u32,
        endianness: u8,
    ) -> c_int;
    fn XCsuDma_WaitForDone(instance_ptr: *mut c_void, channel: u32) -> c_int;
    
    // AES hardware functions
    fn XSecure_AesInitialize(instance_ptr: *mut c_void, csu_dma_ptr: *mut c_void) -> c_int;
    fn XSecure_AesWriteKey(
        instance_ptr: *mut c_void,
        key_sel: u32,
        key_size: u32,
        key_addr: u64,
    ) -> c_int;
    fn XSecure_AesEncryptData(
        instance_ptr: *mut c_void,
        src: u64,
        dst: u64,
        size: u32,
        iv_addr: u64,
    ) -> c_int;
}

const XSECURE_CSU_AES_KEY_SRC_KUP: u32 = 0;
const XSECURE_CSU_AES_KEY_SIZE_256: u32 = 0;

// Hardware context sizes (Xilinx-specific)
const XCSU_DMA_CONTEXT_SIZE: usize = 1024;
const XSECURE_AES_CONTEXT_SIZE: usize = 512;

#[derive(Debug)]
enum CryptoError {
    InitializationFailed,
    EncryptionFailed,
    InvalidInput,
}

pub struct XilinxHardwareCrypto {
    csu_dma_ctx: [u8; XCSU_DMA_CONTEXT_SIZE],
    aes_ctx: [u8; XSECURE_AES_CONTEXT_SIZE],
    key_buffer: [u8; 32], // Aligned key storage
}

impl XilinxHardwareCrypto {
    pub fn new() -> Result<Self, CryptoError> {
        let mut crypto = Self {
            csu_dma_ctx: [0; XCSU_DMA_CONTEXT_SIZE],
            aes_ctx: [0; XSECURE_AES_CONTEXT_SIZE],
            key_buffer: [0; 32],
        };
        
        unsafe {
            // Initialize CSU DMA
            let dma_result = XCsuDma_Initialize(
                crypto.csu_dma_ctx.as_mut_ptr() as *mut c_void,
                core::ptr::null(), // Use default config
            );
            
            if dma_result != 0 {
                return Err(CryptoError::InitializationFailed);
            }
            
            // Initialize AES engine
            let aes_result = XSecure_AesInitialize(
                crypto.aes_ctx.as_mut_ptr() as *mut c_void,
                crypto.csu_dma_ctx.as_mut_ptr() as *mut c_void,
            );
            
            if aes_result != 0 {
                return Err(CryptoError::InitializationFailed);
            }
        }
        
        Ok(crypto)
    }
    
    pub fn set_key(&mut self, key: &[u8; 32]) -> Result<(), CryptoError> {
        self.key_buffer.copy_from_slice(key);
        
        let result = unsafe {
            XSecure_AesWriteKey(
                self.aes_ctx.as_mut_ptr() as *mut c_void,
                XSECURE_CSU_AES_KEY_SRC_KUP,
                XSECURE_CSU_AES_KEY_SIZE_256,
                self.key_buffer.as_ptr() as u64,
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            Err(CryptoError::InitializationFailed)
        }
    }
    
    pub fn encrypt_hardware(&mut self, plaintext: &[u8], ciphertext: &mut [u8], iv: &[u8; 16]) -> Result<(), CryptoError> {
        if plaintext.len() != ciphertext.len() || plaintext.len() % 16 != 0 {
            return Err(CryptoError::InvalidInput);
        }
        
        let result = unsafe {
            XSecure_AesEncryptData(
                self.aes_ctx.as_mut_ptr() as *mut c_void,
                plaintext.as_ptr() as u64,
                ciphertext.as_mut_ptr() as u64,
                plaintext.len() as u32,
                iv.as_ptr() as u64,
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            Err(CryptoError::EncryptionFailed)
        }
    }
}

impl Drop for XilinxHardwareCrypto {
    fn drop(&mut self) {
        // Hardware cleanup if needed
        // Zeroize key buffer
        self.key_buffer.zeroize();
    }
}

impl ZeroizeOnDrop for XilinxHardwareCrypto {}

#[entry]
fn main() -> ! {
    match XilinxHardwareCrypto::new() {
        Ok(mut hw_crypto) => {
            let key = [0u8; 32];
            let iv = [0u8; 16];
            let plaintext = [0u8; 64];
            let mut ciphertext = [0u8; 64];
            
            if hw_crypto.set_key(&key).is_ok() {
                let _ = hw_crypto.encrypt_hardware(&plaintext, &mut ciphertext, &iv);
            }
        }
        Err(_) => {
            // Fall back to software crypto
        }
    }
    
    loop {}
}
```

## Build System Integration

Integrating C libraries requires proper configuration of the build system to handle compilation, linking, and binding generation.

### Cargo.toml Configuration for FFI

```toml
[package]
name = "crypto-ffi-integration"
version = "0.1.0"
edition = "2021"

[dependencies]
zeroize = { version = "1.6", default-features = false }
cortex-r-rt = "0.7"
panic-halt = "0.2"

[build-dependencies]
cc = "1.0"       # For compiling C code
bindgen = "0.65" # For generating Rust bindings

# Feature flags for different crypto backends
[features]
default = ["software-crypto"]
software-crypto = []
openssl = []
mbedtls = []
xilinx-hardware = []

# Link to crypto libraries based on features
[target.'cfg(feature = "openssl")'.dependencies]
openssl-sys = "0.9"

[target.'cfg(feature = "mbedtls")'.dependencies]
mbedtls-sys = "2.28"

# Target-specific dependencies
[target.'cfg(target_arch = "arm")'.dependencies]
heapless = "0.7"
```

### build.rs for Automatic Binding Generation

```rust
// build.rs - Standard Rust build script (runs on host)
use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    
    // Configure based on features
    #[cfg(feature = "openssl")]
    build_openssl_bindings();
    
    #[cfg(feature = "mbedtls")]
    build_mbedtls_bindings();
    
    #[cfg(feature = "xilinx-hardware")]
    build_xilinx_bindings();
    
    // Link platform-specific libraries
    match target.as_str() {
        t if t.contains("armv7r") => {
            // Tell cargo to link Xilinx crypto library
            println!("cargo:rustc-link-lib=static=xilinx_crypto");
            println!("cargo:rustc-link-search=native=/opt/xilinx/lib");
        }
        t if t.contains("thumbv7em") => {
            // STM32 hardware crypto
            println!("cargo:rustc-link-lib=static=stm32_crypto");
        }
        _ => {}
    }
    
    // Rerun build script if headers change
    println!("cargo:rerun-if-changed=wrapper.h");
}

#[cfg(feature = "openssl")]
fn build_openssl_bindings() {
    let bindings = bindgen::Builder::default()
        .header("wrapper_openssl.h")
        // Don't generate bindings for system headers
        .allowlist_function("EVP_.*")
        .allowlist_function("HMAC_.*")
        .allowlist_type("EVP_.*")
        // Generate bindings that work in no_std
        .use_core()
        .ctypes_prefix("cty")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate OpenSSL bindings");
    
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("openssl_bindings.rs"))
        .expect("Couldn't write OpenSSL bindings!");
}

#[cfg(feature = "mbedtls")]
fn build_mbedtls_bindings() {
    // Compile mbedTLS wrapper if needed
    cc::Build::new()
        .file("src/mbedtls_wrapper.c")
        .include("/usr/local/include")
        .flag("-DMBEDTLS_CONFIG_FILE=\"mbedtls_config.h\"")
        .compile("mbedtls_wrapper");
    
    let bindings = bindgen::Builder::default()
        .header("wrapper_mbedtls.h")
        .allowlist_function("mbedtls_.*")
        .allowlist_type("mbedtls_.*")
        .use_core()
        .ctypes_prefix("cty")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate mbedTLS bindings");
    
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("mbedtls_bindings.rs"))
        .expect("Couldn't write mbedTLS bindings!");
}

#[cfg(feature = "xilinx-hardware")]
fn build_xilinx_bindings() {
    // Compile C wrapper for Xilinx BSP
    cc::Build::new()
        .file("src/xilinx_wrapper.c")
        .include("/opt/xilinx/include")
        .flag("-DARMR5")
        .flag("-DUSE_AMP=1")
        .compile("xilinx_wrapper");
    
    let bindings = bindgen::Builder::default()
        .header("wrapper_xilinx.h")
        .clang_arg("-I/opt/xilinx/include")
        .allowlist_function("XSecure_.*")
        .allowlist_function("XCsuDma_.*")
        .allowlist_type("XSecure.*")
        .allowlist_type("XCsuDma.*")
        .use_core()
        .ctypes_prefix("cty")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate Xilinx bindings");
    
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("xilinx_bindings.rs"))
        .expect("Couldn't write Xilinx bindings!");
}
```

## Best Practices and Common Patterns

### Creating Wrapper Headers

For successful FFI integration, you need wrapper headers that expose only the functions you need:

```c
// wrapper_mbedtls.h - Wrapper header for mbedTLS
#ifndef WRAPPER_MBEDTLS_H
#define WRAPPER_MBEDTLS_H

#include <stdint.h>
#include <stddef.h>

// Forward declarations to avoid including full mbedTLS headers
typedef struct mbedtls_aes_context mbedtls_aes_context;
typedef struct mbedtls_ctr_drbg_context mbedtls_ctr_drbg_context;
typedef struct mbedtls_entropy_context mbedtls_entropy_context;

// Only declare the functions we actually use
void mbedtls_aes_init(mbedtls_aes_context *ctx);
void mbedtls_aes_free(mbedtls_aes_context *ctx);
int mbedtls_aes_setkey_enc(mbedtls_aes_context *ctx, const unsigned char *key, unsigned int keybits);
int mbedtls_aes_crypt_cbc(mbedtls_aes_context *ctx, int mode, size_t length,
                          unsigned char iv[16], const unsigned char *input,
                          unsigned char *output);

#endif // WRAPPER_MBEDTLS_H
```

### Complete Hybrid Crypto System Example

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use heapless::Vec;
use heapless::consts::U256;

#[derive(Debug)]
struct CryptoError(&'static str);

// Types from previous examples (stubs for this example)
struct OpensslAes {
    _key: [u8; 32],
}

struct MbedTlsAes {
    _key: [u8; 32],
}

struct XilinxHardwareCrypto {
    _key: [u8; 32],
}

struct OpensslHmac;

struct MbedTlsRng;

// Stub implementations
impl OpensslAes {
    fn new_cbc(_key: &[u8; 32], _iv: &[u8; 16]) -> Result<Self, CryptoError> {
        Ok(Self { _key: *_key })
    }
    
    fn encrypt(&mut self, _plaintext: &[u8], _ciphertext: &mut [u8]) -> Result<usize, CryptoError> {
        Ok(0)
    }
}

impl MbedTlsAes {
    fn new(_key: &[u8; 32]) -> Result<Self, CryptoError> {
        Ok(Self { _key: *_key })
    }
    
    fn encrypt_cbc(&mut self, _iv: &mut [u8; 16], _plaintext: &[u8], _ciphertext: &mut [u8]) -> Result<(), CryptoError> {
        Ok(())
    }
}

impl XilinxHardwareCrypto {
    fn new() -> Result<Self, CryptoError> {
        Ok(Self { _key: [0u8; 32] })
    }
    
    fn set_key(&mut self, key: &[u8; 32]) -> Result<(), CryptoError> {
        self._key = *key;
        Ok(())
    }
    
    fn encrypt_hardware(&mut self, _plaintext: &[u8], _ciphertext: &mut [u8], _iv: &[u8; 16]) -> Result<(), CryptoError> {
        Ok(())
    }
}

impl OpensslHmac {
    fn new_sha256(_key: &[u8]) -> Result<Self, CryptoError> {
        Ok(Self)
    }
    
    fn update(&mut self, _data: &[u8]) -> Result<(), CryptoError> {
        Ok(())
    }
    
    fn finalize(self) -> Result<[u8; 32], CryptoError> {
        Ok([0u8; 32])
    }
}

impl MbedTlsRng {
    fn new() -> Result<Self, CryptoError> {
        Ok(Self)
    }
    
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), CryptoError> {
        for (i, byte) in dest.iter_mut().enumerate() {
            *byte = (i * 17) as u8;
        }
        Ok(())
    }
}

// Complete hybrid crypto system using multiple backends
pub struct HybridCryptoSystem {
    openssl_aes: Option<OpensslAes>,
    mbedtls_aes: Option<MbedTlsAes>,
    hardware_aes: Option<XilinxHardwareCrypto>,
    openssl_hmac: Option<OpensslHmac>,
    rng: MbedTlsRng,
}

impl HybridCryptoSystem {
    pub fn new() -> Result<Self, CryptoError> {
        let rng = MbedTlsRng::new()?;
        
        let mut system = Self {
            openssl_aes: None,
            mbedtls_aes: None,
            hardware_aes: None,
            openssl_hmac: None,
            rng,
        };
        
        // Initialize available crypto engines
        #[cfg(feature = "xilinx-hardware")]
        {
            system.hardware_aes = XilinxHardwareCrypto::new().ok();
        }
        
        #[cfg(feature = "openssl")]
        {
            // OpenSSL might not be available in embedded
            system.openssl_aes = None;
        }
        
        #[cfg(feature = "mbedtls")]
        {
            // mbedTLS is common in embedded systems
            let key = [0u8; 32];
            system.mbedtls_aes = MbedTlsAes::new(&key).ok();
        }
        
        Ok(system)
    }
    
    pub fn encrypt_with_best_available(
        &mut self, 
        key: &[u8; 32], 
        plaintext: &[u8]
    ) -> Result<Vec<u8, U256>, CryptoError> {
        let mut iv = [0u8; 16];
        self.rng.fill_bytes(&mut iv)?;
        
        let mut ciphertext = [0u8; 256];
        let ciphertext_slice = &mut ciphertext[..plaintext.len()];
        
        // Priority: Hardware > mbedTLS > OpenSSL
        let encrypted_len = if let Some(ref mut hw_crypto) = self.hardware_aes {
            hw_crypto.set_key(key)?;
            hw_crypto.encrypt_hardware(plaintext, ciphertext_slice, &iv)?;
            plaintext.len()
        } else if let Some(ref mut mbedtls) = self.mbedtls_aes {
            mbedtls.encrypt_cbc(&mut iv, plaintext, ciphertext_slice)?;
            plaintext.len()
        } else if let Some(ref mut openssl) = self.openssl_aes {
            openssl.encrypt(plaintext, ciphertext_slice)?
        } else {
            return Err(CryptoError("No crypto backend available"));
        };
        
        // Build result: IV || Ciphertext
        let mut result = Vec::new();
        result.extend_from_slice(&iv)
            .map_err(|_| CryptoError("Buffer full"))?;
        result.extend_from_slice(&ciphertext_slice[..encrypted_len])
            .map_err(|_| CryptoError("Buffer full"))?;
        
        Ok(result)
    }
    
    pub fn generate_hmac(&mut self, key: &[u8], data: &[u8]) -> Result<[u8; 32], CryptoError> {
        // Create new HMAC context for each operation
        let mut hmac = OpensslHmac::new_sha256(key)?;
        hmac.update(data)?;
        hmac.finalize()
    }
    
    pub fn get_random_bytes(&mut self, dest: &mut [u8]) -> Result<(), CryptoError> {
        self.rng.fill_bytes(dest)
    }
}

#[entry]
fn main() -> ! {
    match HybridCryptoSystem::new() {
        Ok(mut crypto) => {
            let key = [0x42u8; 32];
            let plaintext = b"Hello from hybrid crypto!";
            
            match crypto.encrypt_with_best_available(&key, plaintext) {
                Ok(ciphertext) => {
                    // Successfully encrypted with best available backend
                }
                Err(_) => {
                    // Handle encryption error
                }
            }
        }
        Err(_) => {
            // Handle initialization error
        }
    }
    
    loop {}
}
```

### Safety Guidelines for FFI

1. **Always validate pointers from C**:
   ```rust
   if ptr.is_null() {
       return Err(CryptoError("Null pointer from C library"));
   }
   ```

2. **Use RAII wrappers** to ensure cleanup
3. **Validate all return codes** from C functions
4. **Never trust sizes** from external sources
5. **Use `#[repr(C)]`** for structs shared with C
6. **Document safety requirements** for unsafe blocks

### Debugging FFI Issues

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use core::ffi::{c_void, c_int};

// Debug helpers for FFI development
#[cfg(debug_assertions)]
fn debug_ffi_call(name: &str, result: c_int) {
    if result != 0 {
        // In production, use proper logging
        // For now, just track the error
        unsafe {
            static mut LAST_ERROR: &str = "";
            static mut LAST_CODE: c_int = 0;
            LAST_ERROR = name;
            LAST_CODE = result;
        }
    }
}

// Example with debug tracking
extern "C" {
    fn some_crypto_function(ctx: *mut c_void) -> c_int;
}

fn call_crypto_function(ctx: *mut c_void) -> Result<(), &'static str> {
    let result = unsafe { some_crypto_function(ctx) };
    
    #[cfg(debug_assertions)]
    debug_ffi_call("some_crypto_function", result);
    
    if result == 0 {
        Ok(())
    } else {
        Err("Crypto function failed")
    }
}

#[entry]
fn main() -> ! {
    // Example usage
    let ctx = core::ptr::null_mut();
    let _ = call_crypto_function(ctx);
    
    loop {}
}
```

## Summary and Best Practices

When integrating C cryptographic libraries with Rust in embedded systems:

1. **Safety First**: Always wrap unsafe FFI calls in safe Rust APIs
2. **Resource Management**: Use RAII patterns to ensure proper cleanup
3. **Error Handling**: Convert C error codes to Rust Result types
4. **Memory Safety**: Validate all pointers and sizes from C code
5. **Build Integration**: Use build.rs for automatic binding generation
6. **Feature Flags**: Support multiple backends with Cargo features
7. **Testing**: Thoroughly test FFI boundaries with various inputs
8. **Documentation**: Document all safety requirements and assumptions

The examples in this chapter demonstrate how to safely integrate existing C crypto libraries while maintaining Rust's safety guarantees in resource-constrained embedded environments.