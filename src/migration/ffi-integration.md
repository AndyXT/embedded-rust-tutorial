# 6.2 FFI Integration with C Libraries {ffi-integration-with-c-libraries}

Interfacing with existing C cryptographic libraries during migration requires careful attention to safety and proper resource management. This section provides comprehensive examples for common integration scenarios.

**Advanced Integration Techniques:**
- → [Advanced Type System Features](../core-concepts/advanced-types.md) - Use enums and traits to wrap C APIs safely
- → [Error Handling Without Exceptions](../core-concepts/error-handling.md) - Convert C error codes to Result types

#### Safe FFI Wrapper Patterns

**Pattern 1: Resource Management with RAII**

```rust
// Safe wrapper for OpenSSL or similar C crypto library
use core::ffi::{c_int, c_void, c_char, c_uchar};
use zeroize::ZeroizeOnDrop;

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
    
    // HMAC functions
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
    
    pub fn new_gcm(key: &[u8; 32], iv: &[u8; 12]) -> Result<Self, CryptoError> {
        let ctx = unsafe { EVP_CIPHER_CTX_new() };
        if ctx.is_null() {
            return Err(CryptoError::InitializationFailed);
        }
        
        let result = unsafe {
            EVP_EncryptInit_ex(
                ctx,
                EVP_aes_256_gcm(),
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
            _key: *key,
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
                ciphertext.as_mut_ptr().offset(total_len as isize),
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
        // _key is automatically zeroized by compiler-generated Drop
    }
}

// Implement ZeroizeOnDrop for secure key cleanup
impl ZeroizeOnDrop for OpensslAes {}
```

**Pattern 2: HMAC Integration with Error Handling**

```rust
pub struct OpensslHmac {
    ctx: *mut c_void,
    _key: Vec<u8>, // Variable-length key storage
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
        
        Ok(Self {
            ctx,
            _key: key.to_vec(), // Store key for secure cleanup
        })
    }
    
    pub fn update(&mut self, data: &[u8]) -> Result<(), CryptoError> {
        let result = unsafe {
            HMAC_Update(self.ctx, data.as_ptr(), data.len())
        };
        
        if result == 1 {
            Ok(())
        } else {
            Err(CryptoError::EncryptionFailed)
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
            Err(CryptoError::EncryptionFailed)
        }
    }
}

impl Drop for OpensslHmac {
    fn drop(&mut self) {
        unsafe {
            HMAC_CTX_free(self.ctx);
        }
        // Zeroize key material
        use zeroize::Zeroize;
        self._key.zeroize();
    }
}
```

#### Integration with mbedTLS

**Complete mbedTLS Integration Example**

```rust
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
    
    // Random number generation
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
}

const MBEDTLS_AES_ENCRYPT: c_int = 1;
const MBEDTLS_AES_DECRYPT: c_int = 0;

// mbedTLS context sizes (platform-specific)
const MBEDTLS_AES_CONTEXT_SIZE: usize = 288;
const MBEDTLS_CTR_DRBG_CONTEXT_SIZE: usize = 352;
const MBEDTLS_ENTROPY_CONTEXT_SIZE: usize = 1024;

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
        // _key automatically zeroized
    }
}

impl ZeroizeOnDrop for MbedTlsAes {}

// Random number generator wrapper
pub struct MbedTlsRng {
    ctr_drbg_ctx: [u8; MBEDTLS_CTR_DRBG_CONTEXT_SIZE],
    entropy_ctx: [u8; MBEDTLS_ENTROPY_CONTEXT_SIZE],
}

impl MbedTlsRng {
    pub fn new() -> Result<Self, CryptoError> {
        let mut rng = Self {
            ctr_drbg_ctx: [0; MBEDTLS_CTR_DRBG_CONTEXT_SIZE],
            entropy_ctx: [0; MBEDTLS_ENTROPY_CONTEXT_SIZE],
        };
        
        unsafe {
            mbedtls_entropy_init(rng.entropy_ctx.as_mut_ptr() as *mut c_void);
            mbedtls_ctr_drbg_init(rng.ctr_drbg_ctx.as_mut_ptr() as *mut c_void);
            
            let result = mbedtls_ctr_drbg_seed(
                rng.ctr_drbg_ctx.as_mut_ptr() as *mut c_void,
                core::ptr::null_mut(), // Use default entropy function
                rng.entropy_ctx.as_mut_ptr() as *mut c_void,
                b"Rust-mbedTLS-RNG".as_ptr(),
                17,
            );
            
            if result != 0 {
                mbedtls_ctr_drbg_free(rng.ctr_drbg_ctx.as_mut_ptr() as *mut c_void);
                mbedtls_entropy_free(rng.entropy_ctx.as_mut_ptr() as *mut c_void);
                return Err(CryptoError::InitializationFailed);
            }
        }
        
        Ok(rng)
    }
    
    pub fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), CryptoError> {
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
            Err(CryptoError::EncryptionFailed)
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
```

#### Hardware Crypto Library Integration

**Xilinx CSU (Crypto Services Unit) Integration**

```rust
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
        use zeroize::Zeroize;
        self.key_buffer.zeroize();
    }
}

impl ZeroizeOnDrop for XilinxHardwareCrypto {}
```

#### Build System Integration

**Cargo.toml Configuration for FFI**

```toml
[package]
name = "crypto-ffi-integration"
version = "0.1.0"
edition = "2021"

[dependencies]
zeroize = { version = "1.6", default-features = false }

[build-dependencies]
cc = "1.0"
bindgen = "0.65"

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
```

**build.rs for Automatic Binding Generation**

```rust
// build.rs - Automatic FFI binding generation
use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    
    // Configure based on features
    if cfg!(feature = "openssl") {
        build_openssl_bindings();
    }
    
    if cfg!(feature = "mbedtls") {
        build_mbedtls_bindings();
    }
    
    if cfg!(feature = "xilinx-hardware") {
        build_xilinx_bindings();
    }
    
    // Link platform-specific libraries
    match target.as_str() {
        t if t.contains("armv7r") => {
            println!("cargo:rustc-link-lib=static=xilinx_crypto");
            println!("cargo:rustc-link-search=native=/opt/xilinx/lib");
        }
        _ => {}
    }
}

fn build_openssl_bindings() {
    let bindings = bindgen::Builder::default()
        .header("wrapper_openssl.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate OpenSSL bindings");
    
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("openssl_bindings.rs"))
        .expect("Couldn't write OpenSSL bindings!");
}

fn build_mbedtls_bindings() {
    let bindings = bindgen::Builder::default()
        .header("wrapper_mbedtls.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate mbedTLS bindings");
    
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("mbedtls_bindings.rs"))
        .expect("Couldn't write mbedTLS bindings!");
}

fn build_xilinx_bindings() {
    cc::Build::new()
        .file("src/xilinx_wrapper.c")
        .include("/opt/xilinx/include")
        .compile("xilinx_wrapper");
    
    let bindings = bindgen::Builder::default()
        .header("wrapper_xilinx.h")
        .clang_arg("-I/opt/xilinx/include")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate Xilinx bindings");
    
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("xilinx_bindings.rs"))
        .expect("Couldn't write Xilinx bindings!");
}
```

#### Usage Examples and Best Practices

**Hybrid System Example**

```rust
// Complete example using multiple crypto backends
pub struct HybridCryptoSystem {
    software_aes: Option<SoftwareAes>,
    hardware_aes: Option<XilinxHardwareCrypto>,
    openssl_hmac: Option<OpensslHmac>,
    rng: MbedTlsRng,
}

impl HybridCryptoSystem {
    pub fn new() -> Result<Self, CryptoError> {
        let mut system = Self {
            software_aes: None,
            hardware_aes: None,
            openssl_hmac: None,
            rng: MbedTlsRng::new()?,
        };
        
        // Initialize available crypto engines
        #[cfg(feature = "xilinx-hardware")]
        {
            system.hardware_aes = XilinxHardwareCrypto::new().ok();
        }
        
        #[cfg(feature = "openssl")]
        {
            // Initialize software fallback
            system.software_aes = SoftwareAes::new().ok();
        }
        
        Ok(system)
    }
    
    pub fn encrypt_with_best_available(&mut self, key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut iv = [0u8; 16];
        self.rng.fill_bytes(&mut iv)?;
        
        let mut ciphertext = vec![0u8; plaintext.len()];
        
        // Prefer hardware acceleration when available
        if let Some(ref mut hw_crypto) = self.hardware_aes {
            hw_crypto.set_key(key)?;
            hw_crypto.encrypt_hardware(plaintext, &mut ciphertext, &iv)?;
        } else if let Some(ref mut sw_crypto) = self.software_aes {
            sw_crypto.encrypt_cbc(&mut iv, plaintext, &mut ciphertext)?;
        } else {
            return Err(CryptoError::InitializationFailed);
        }
        
        // Prepend IV to ciphertext
        let mut result = Vec::with_capacity(16 + ciphertext.len());
        result.extend_from_slice(&iv);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    pub fn generate_hmac(&mut self, key: &[u8], data: &[u8]) -> Result<[u8; 32], CryptoError> {
        if self.openssl_hmac.is_none() {
            self.openssl_hmac = Some(OpensslHmac::new_sha256(key)?);
        }
        
        if let Some(ref mut hmac) = self.openssl_hmac {
            hmac.update(data)?;
            // Note: This consumes the HMAC context
            let hmac_ctx = self.openssl_hmac.take().unwrap();
            hmac_ctx.finalize()
        } else {
            Err(CryptoError::InitializationFailed)
        }
    }
}
```