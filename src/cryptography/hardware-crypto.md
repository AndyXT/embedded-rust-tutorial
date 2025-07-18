# 5.4 Hardware Crypto Acceleration

Leveraging hardware crypto acceleration while maintaining portability and security is crucial for embedded cryptographic implementations. This section demonstrates how to create flexible abstractions that can utilize hardware acceleration when available while falling back to software implementations.

## Generic Hardware Abstraction

Creating a generic trait allows for seamless switching between hardware and software implementations:




```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use sha2::{Sha256, Digest};
use aes::{Aes256, cipher::{KeyInit, BlockEncrypt, BlockDecrypt}};

#[derive(Debug)]
pub struct CryptoError(&'static str);

// Stub function for HMAC
fn hmac_sha256(_key: &[u8; 32], _message: &[u8]) -> Result<[u8; 32], CryptoError> {
    Ok([0u8; 32])
}


use core::mem;
use core::fmt;

use core::result::Result;

// Generic trait for crypto acceleration
trait CryptoAccelerator {
    type Error;
    
    fn aes_encrypt(&mut self, key: &[u8; 32], plaintext: &[u8], ciphertext: &mut [u8]) -> Result<(), Self::Error>;
    fn aes_decrypt(&mut self, key: &[u8; 32], ciphertext: &[u8], plaintext: &mut [u8]) -> Result<(), Self::Error>;
    fn sha256(&mut self, data: &[u8], hash: &mut [u8; 32]) -> Result<(), Self::Error>;
    fn hmac_sha256(&mut self, key: &[u8], data: &[u8], mac: &mut [u8; 32]) -> Result<(), Self::Error>;
    fn random_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error>;
}

// Hardware implementation for Xilinx
struct XilinxCryptoAccel {
    aes_engine: XilinxAesEngine,
    hash_engine: XilinxHashEngine,
    rng_engine: XilinxRngEngine,
}

impl XilinxCryptoAccel {
    fn new() -> Result<Self, HardwareError> {
        let aes_engine = XilinxAesEngine::initialize()?;
        let hash_engine = XilinxHashEngine::initialize()?;
        let rng_engine = XilinxRngEngine::initialize()?;
        
        Ok(Self {
            aes_engine,
            hash_engine,
            rng_engine,
        })
    }
    
    fn is_available() -> bool {
        // Check if hardware crypto is available
        unsafe {
            let crypto_base = 0xFFCA0000 as *const u32;
            let status = core::ptr::read_volatile(crypto_base);
            (status & 0x1) != 0 // Check ready bit
        }
    }
}

impl CryptoAccelerator for XilinxCryptoAccel {
    type Error = HardwareError;
    
    fn aes_encrypt(&mut self, key: &[u8; 32], plaintext: &[u8], ciphertext: &mut [u8]) -> Result<(), Self::Error> {
        self.aes_engine.load_key(key)?;
        self.aes_engine.encrypt(plaintext, ciphertext)?;
        Ok(())
    }
    
    fn aes_decrypt(&mut self, key: &[u8; 32], ciphertext: &[u8], plaintext: &mut [u8]) -> Result<(), Self::Error> {
        self.aes_engine.load_key(key)?;
        self.aes_engine.decrypt(ciphertext, plaintext)?;
        Ok(())
    }
    
    fn sha256(&mut self, data: &[u8], hash: &mut [u8; 32]) -> Result<(), Self::Error> {
        self.hash_engine.hash_sha256(data, hash)
    }
    
    fn hmac_sha256(&mut self, key: &[u8], data: &[u8], mac: &mut [u8; 32]) -> Result<(), Self::Error> {
        self.hash_engine.hmac_sha256(key, data, mac)
    }
    
    fn random_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        self.rng_engine.fill_bytes(dest)
    }
}

// Software fallback implementation
struct SoftwareCrypto {
    rng: ChaCha20Rng,
}

impl SoftwareCrypto {
    fn new() -> Result<Self, CryptoError> {
        // Initialize with entropy from system
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed).map_err(|_| CryptoError::RngFailed)?;
        
        Ok(Self {
            rng: ChaCha20Rng::from_seed(seed),
        })
    }
}

impl CryptoAccelerator for SoftwareCrypto {
    type Error = CryptoError;
    
    fn aes_encrypt(&mut self, key: &[u8; 32], plaintext: &[u8], ciphertext: &mut [u8]) -> Result<(), Self::Error> {
        use aes::Aes256;
        use aes::cipher::{BlockEncrypt, KeyInit};
        
        let cipher = Aes256::new_from_slice(key).map_err(|_| CryptoError::InvalidKey)?;
        
        // Process in 16-byte blocks
        for (plain_chunk, cipher_chunk) in plaintext.chunks(16).zip(ciphertext.chunks_mut(16)) {
            let mut block = [0u8; 16];
            block[..plain_chunk.len()].copy_from_slice(plain_chunk);
            
            let mut block = aes::cipher::generic_array::GenericArray::from(block);
            cipher.encrypt_block(&mut block);
            
            cipher_chunk.copy_from_slice(&block[..cipher_chunk.len()]);
        }
        
        Ok(())
    }
    
    fn aes_decrypt(&mut self, key: &[u8; 32], ciphertext: &[u8], plaintext: &mut [u8]) -> Result<(), Self::Error> {
        use aes::cipher::{BlockDecrypt, KeyInit};
        
        let cipher = Aes256::new_from_slice(key).map_err(|_| CryptoError::InvalidKey)?;
        
        // Process in 16-byte blocks
        for (cipher_chunk, plain_chunk) in ciphertext.chunks(16).zip(plaintext.chunks_mut(16)) {
            let mut block = [0u8; 16];
            block[..cipher_chunk.len()].copy_from_slice(cipher_chunk);
            
            let mut block = aes::cipher::generic_array::GenericArray::from(block);
            cipher.decrypt_block(&mut block);
            
            plain_chunk.copy_from_slice(&block[..plain_chunk.len()]);
        }
        
        Ok(())
    }
    
    fn sha256(&mut self, data: &[u8], hash: &mut [u8; 32]) -> Result<(), Self::Error> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hash.copy_from_slice(&hasher.finalize());
        Ok(())
    }
    
    fn hmac_sha256(&mut self, key: &[u8], data: &[u8], mac: &mut [u8; 32]) -> Result<(), Self::Error> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        type HmacSha256 = Hmac<Sha256>;
        let mut hmac = HmacSha256::new_from_slice(key).map_err(|_| CryptoError::InvalidKey)?;
        hmac.update(data);
        mac.copy_from_slice(&hmac.finalize().into_bytes());
        Ok(())
    }
    
    fn random_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        use rand_core::RngCore;
        self.rng.fill_bytes(dest);
        Ok(())
    }
}

// Adaptive crypto engine that chooses best available implementation
enum CryptoEngine {
    Hardware(XilinxCryptoAccel),
    Software(SoftwareCrypto),
}

impl CryptoEngine {
    fn new() -> Result<Self, Box<dyn core::error::Error>> {
        // Try to initialize hardware acceleration first
        if XilinxCryptoAccel::is_available() {
            if let Ok(hw_accel) = XilinxCryptoAccel::new() {
                return Ok(CryptoEngine::Hardware(hw_accel));
            }
        }
        
        // Fall back to software implementation
        let sw_crypto = SoftwareCrypto::new()?;
        Ok(CryptoEngine::Software(sw_crypto))
    }
    
    fn encrypt_aes(&mut self, key: &[u8; 32], plaintext: &[u8], ciphertext: &mut [u8]) -> Result<(), Box<dyn core::error::Error>> {
        match self {
            CryptoEngine::Hardware(hw) => hw.aes_encrypt(key, plaintext, ciphertext).map_err(|e| Box::new(e) as Box<dyn core::error::Error>),
            CryptoEngine::Software(sw) => sw.aes_encrypt(key, plaintext, ciphertext).map_err(|e| Box::new(e) as Box<dyn core::error::Error>),
        }
    }
    
    fn decrypt_aes(&mut self, key: &[u8; 32], ciphertext: &[u8], plaintext: &mut [u8]) -> Result<(), Box<dyn core::error::Error>> {
        match self {
            CryptoEngine::Hardware(hw) => hw.aes_decrypt(key, ciphertext, plaintext).map_err(|e| Box::new(e) as Box<dyn core::error::Error>),
            CryptoEngine::Software(sw) => sw.aes_decrypt(key, ciphertext, plaintext).map_err(|e| Box::new(e) as Box<dyn core::error::Error>),
        }
    }
    
    fn hash_sha256(&mut self, data: &[u8], hash: &mut [u8; 32]) -> Result<(), Box<dyn core::error::Error>> {
        match self {
            CryptoEngine::Hardware(hw) => hw.sha256(data, hash).map_err(|e| Box::new(e) as Box<dyn core::error::Error>),
            CryptoEngine::Software(sw) => sw.sha256(data, hash).map_err(|e| Box::new(e) as Box<dyn core::error::Error>),
        }
    }
    
    fn is_hardware_accelerated(&self) -> bool {
        matches!(self, CryptoEngine::Hardware(_))
    }
}
```

## Hardware-Specific Implementations

### Xilinx Ultrascale+ Crypto Engine

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use aes::{Aes256, cipher::{KeyInit, BlockEncrypt, BlockDecrypt}};

// Stub function for HMAC
fn hmac_sha256(_key: &[u8; 32], _message: &[u8]) -> Result<[u8; 32], CryptoError> {
    Ok([0u8; 32])
}


use core::mem;
use core::fmt;

use core::result::Result;

// Xilinx-specific hardware register definitions
const XILINX_CRYPTO_BASE: usize = 0xFFCA0000;
const AES_KEY_OFFSET: usize = 0x10;
const AES_IV_OFFSET: usize = 0x20;
const AES_SRC_OFFSET: usize = 0x30;
const AES_DST_OFFSET: usize = 0x40;
const AES_CTRL_OFFSET: usize = 0x00;
const AES_STATUS_OFFSET: usize = 0x04;

struct XilinxAesEngine {
    base_addr: *mut u32,
}

impl XilinxAesEngine {
    fn initialize() -> Result<Self, HardwareError> {
        let base_addr = XILINX_CRYPTO_BASE as *mut u32;
        
        // Check if AES engine is available
        let status = unsafe { core::ptr::read_volatile(base_addr.add(AES_STATUS_OFFSET / 4)) };
        if (status & 0x1) == 0 {
            return Err(HardwareError::NotAvailable);
        }
        
        Ok(Self { base_addr })
    }
    
    fn load_key(&mut self, key: &[u8; 32]) -> Result<(), HardwareError> {
        // Load 256-bit key into hardware registers
        let key_ptr = self.base_addr.add(AES_KEY_OFFSET / 4);
        
        for (i, chunk) in key.chunks(4).enumerate() {
            let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            unsafe {
                core::ptr::write_volatile(key_ptr.add(i), word);
            }
        }
        
        // Wait for key loading to complete
        self.wait_ready()?;
        Ok(())
    }
    
    fn encrypt(&mut self, plaintext: &[u8], ciphertext: &mut [u8]) -> Result<(), HardwareError> {
        if plaintext.len() != ciphertext.len() {
            return Err(HardwareError::InvalidLength);
        }
        
        // Process data in 16-byte blocks
        for (plain_chunk, cipher_chunk) in plaintext.chunks(16).zip(ciphertext.chunks_mut(16)) {
            self.encrypt_block(plain_chunk, cipher_chunk)?;
        }
        
        Ok(())
    }
    
    fn decrypt(&mut self, ciphertext: &[u8], plaintext: &mut [u8]) -> Result<(), HardwareError> {
        if plaintext.len() != ciphertext.len() {
            return Err(HardwareError::InvalidLength);
        }
        
        // Process data in 16-byte blocks
        for (cipher_chunk, plain_chunk) in ciphertext.chunks(16).zip(plaintext.chunks_mut(16)) {
            self.decrypt_block(cipher_chunk, plain_chunk)?;
        }
        
        Ok(())
    }
    
    fn encrypt_block(&mut self, plaintext: &[u8], ciphertext: &mut [u8]) -> Result<(), HardwareError> {
        // Load plaintext into source register
        let src_ptr = self.base_addr.add(AES_SRC_OFFSET / 4);
        let mut padded_input = [0u8; 16];
        padded_input[..plaintext.len()].copy_from_slice(plaintext);
        
        for (i, chunk) in padded_input.chunks(4).enumerate() {
            let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            unsafe {
                core::ptr::write_volatile(src_ptr.add(i), word);
            }
        }
        
        // Start encryption
        let ctrl_ptr = self.base_addr.add(AES_CTRL_OFFSET / 4);
        unsafe {
            core::ptr::write_volatile(ctrl_ptr, 0x1); // Start encrypt
        }
        
        // Wait for completion
        self.wait_ready()?;
        
        // Read result from destination register
        let dst_ptr = self.base_addr.add(AES_DST_OFFSET / 4);
        let mut result = [0u8; 16];
        
        for i in 0..4 {
            let word = unsafe { core::ptr::read_volatile(dst_ptr.add(i)) };
            let bytes = word.to_le_bytes();
            result[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
        }
        
        ciphertext.copy_from_slice(&result[..ciphertext.len()]);
        Ok(())
    }
    
    fn decrypt_block(&mut self, ciphertext: &[u8], plaintext: &mut [u8]) -> Result<(), HardwareError> {
        // Similar to encrypt_block but with decrypt control bit
        let src_ptr = self.base_addr.add(AES_SRC_OFFSET / 4);
        let mut padded_input = [0u8; 16];
        padded_input[..ciphertext.len()].copy_from_slice(ciphertext);
        
        for (i, chunk) in padded_input.chunks(4).enumerate() {
            let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            unsafe {
                core::ptr::write_volatile(src_ptr.add(i), word);
            }
        }
        
        // Start decryption
        let ctrl_ptr = self.base_addr.add(AES_CTRL_OFFSET / 4);
        unsafe {
            core::ptr::write_volatile(ctrl_ptr, 0x2); // Start decrypt
        }
        
        // Wait for completion
        self.wait_ready()?;
        
        // Read result
        let dst_ptr = self.base_addr.add(AES_DST_OFFSET / 4);
        let mut result = [0u8; 16];
        
        for i in 0..4 {
            let word = unsafe { core::ptr::read_volatile(dst_ptr.add(i)) };
            let bytes = word.to_le_bytes();
            result[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
        }
        
        plaintext.copy_from_slice(&result[..plaintext.len()]);
        Ok(())
    }
    
    fn wait_ready(&self) -> Result<(), HardwareError> {
        let status_ptr = self.base_addr.add(AES_STATUS_OFFSET / 4);
        let mut timeout = 10000;
        
        while timeout > 0 {
            let status = unsafe { core::ptr::read_volatile(status_ptr) };
            if (status & 0x2) != 0 { // Ready bit
                return Ok(());
            }
            timeout -= 1;
            cortex_m::asm::nop();
        }
        
        Err(HardwareError::Timeout)
    }
}

// Hash engine implementation
struct XilinxHashEngine {
    base_addr: *mut u32,
}

impl XilinxHashEngine {
    fn initialize() -> Result<Self, HardwareError> {
        let base_addr = (XILINX_CRYPTO_BASE + 0x1000) as *mut u32;
        
        // Check if hash engine is available
        let status = unsafe { core::ptr::read_volatile(base_addr) };
        if (status & 0x1) == 0 {
            return Err(HardwareError::NotAvailable);
        }
        
        Ok(Self { base_addr })
    }
    
    fn hash_sha256(&mut self, data: &[u8], hash: &mut [u8; 32]) -> Result<(), HardwareError> {
        // Initialize SHA-256 context
        self.sha256_init()?;
        
        // Process data in chunks
        for chunk in data.chunks(64) {
            self.sha256_update(chunk)?;
        }
        
        // Finalize and get result
        self.sha256_finalize(hash)?;
        Ok(())
    }
    
    fn hmac_sha256(&mut self, key: &[u8], data: &[u8], mac: &mut [u8; 32]) -> Result<(), HardwareError> {
        // HMAC implementation using hardware SHA-256
        let mut ipad = [0x36u8; 64];
        let mut opad = [0x5Cu8; 64];
        
        // Prepare key
        let mut key_buf = [0u8; 64];
        if key.len() > 64 {
            self.hash_sha256(key, &mut key_buf[..32].try_into().unwrap())?;
        } else {
            key_buf[..key.len()].copy_from_slice(key);
        }
        
        // XOR key with pads
        for i in 0..64 {
            ipad[i] ^= key_buf[i];
            opad[i] ^= key_buf[i];
        }
        
        // Inner hash: SHA256(ipad || data)
        self.sha256_init()?;
        self.sha256_update(&ipad)?;
        self.sha256_update(data)?;
        let mut inner_hash = [0u8; 32];
        self.sha256_finalize(&mut inner_hash)?;
        
        // Outer hash: SHA256(opad || inner_hash)
        self.sha256_init()?;
        self.sha256_update(&opad)?;
        self.sha256_update(&inner_hash)?;
        self.sha256_finalize(mac)?;
        
        Ok(())
    }
    
    fn sha256_init(&mut self) -> Result<(), HardwareError> {
        // Initialize SHA-256 hardware context
        unsafe {
            core::ptr::write_volatile(self.base_addr, 0x1); // Init command
        }
        self.wait_hash_ready()
    }
    
    fn sha256_update(&mut self, data: &[u8]) -> Result<(), HardwareError> {
        // Process data block
        let data_ptr = self.base_addr.add(4);
        
        // Copy data to hardware buffer (pad if necessary)
        let mut padded = [0u8; 64];
        padded[..data.len()].copy_from_slice(data);
        
        for (i, chunk) in padded.chunks(4).enumerate() {
            let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            unsafe {
                core::ptr::write_volatile(data_ptr.add(i), word);
            }
        }
        
        // Start processing
        unsafe {
            core::ptr::write_volatile(self.base_addr, 0x2); // Update command
        }
        
        self.wait_hash_ready()
    }
    
    fn sha256_finalize(&mut self, hash: &mut [u8; 32]) -> Result<(), HardwareError> {
        // Finalize hash computation
        unsafe {
            core::ptr::write_volatile(self.base_addr, 0x3); // Finalize command
        }
        
        self.wait_hash_ready()?;
        
        // Read hash result
        let result_ptr = self.base_addr.add(20);
        for i in 0..8 {
            let word = unsafe { core::ptr::read_volatile(result_ptr.add(i)) };
            let bytes = word.to_be_bytes(); // SHA-256 uses big-endian
            hash[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
        }
        
        Ok(())
    }
    
    fn wait_hash_ready(&self) -> Result<(), HardwareError> {
        let status_ptr = self.base_addr.add(1);
        let mut timeout = 10000;
        
        while timeout > 0 {
            let status = unsafe { core::ptr::read_volatile(status_ptr) };
            if (status & 0x1) != 0 { // Ready bit
                return Ok(());
            }
            timeout -= 1;
            cortex_m::asm::nop();
        }
        
        Err(HardwareError::Timeout)
    }
}
```

## Performance Optimization and Benchmarking

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};


use core::mem;
use core::fmt;

use core::result::Result;

// Performance measurement utilities
struct CryptoPerformance {
    hw_aes_cycles: u32,
    sw_aes_cycles: u32,
    hw_sha256_cycles: u32,
    sw_sha256_cycles: u32,
}

impl CryptoPerformance {
    fn benchmark_crypto_engine(engine: &mut CryptoEngine) -> Self {
        let mut perf = Self {
            hw_aes_cycles: 0,
            sw_aes_cycles: 0,
            hw_sha256_cycles: 0,
            sw_sha256_cycles: 0,
        };
        
        // Benchmark AES encryption
        let key = [0u8; 32];
        let plaintext = [0u8; 1024];
        let mut ciphertext = [0u8; 1024];
        
        let start = cortex_m::peripheral::DWT::cycle_count();
        let _ = engine.encrypt_aes(&key, &plaintext, &mut ciphertext);
        let end = cortex_m::peripheral::DWT::cycle_count();
        
        if engine.is_hardware_accelerated() {
            perf.hw_aes_cycles = end.wrapping_sub(start);
        } else {
            perf.sw_aes_cycles = end.wrapping_sub(start);
        }
        
        // Benchmark SHA-256
        let data = [0u8; 1024];
        let mut hash = [0u8; 32];
        
        let start = cortex_m::peripheral::DWT::cycle_count();
        let _ = engine.hash_sha256(&data, &mut hash);
        let end = cortex_m::peripheral::DWT::cycle_count();
        
        if engine.is_hardware_accelerated() {
            perf.hw_sha256_cycles = end.wrapping_sub(start);
        } else {
            perf.sw_sha256_cycles = end.wrapping_sub(start);
        }
        
        perf
    }
    
    fn print_results(&self) {
        if self.hw_aes_cycles > 0 {
            defmt::info!("Hardware AES (1KB): {} cycles", self.hw_aes_cycles);
        }
        if self.sw_aes_cycles > 0 {
            defmt::info!("Software AES (1KB): {} cycles", self.sw_aes_cycles);
        }
        if self.hw_sha256_cycles > 0 {
            defmt::info!("Hardware SHA-256 (1KB): {} cycles", self.hw_sha256_cycles);
        }
        if self.sw_sha256_cycles > 0 {
            defmt::info!("Software SHA-256 (1KB): {} cycles", self.sw_sha256_cycles);
        }
    }
}

// Usage example with performance monitoring
fn crypto_performance_example() -> Result<(), Box<dyn core::error::Error>> {
    let mut crypto_engine = CryptoEngine::new()?;
    
    defmt::info!("Crypto engine initialized: {}", 
        if crypto_engine.is_hardware_accelerated() { "Hardware" } else { "Software" });
    
    // Run performance benchmark
    let perf = CryptoPerformance::benchmark_crypto_engine(&mut crypto_engine);
    perf.print_results();
    
    // Use crypto engine for actual work
    let key = [0x42u8; 32];
    let message = b"Hello, hardware crypto!";
    let mut encrypted = vec![0u8; message.len()];
    
    crypto_engine.encrypt_aes(&key, message, &mut encrypted)?;
    defmt::info!("Encryption completed successfully");
    
    Ok(())
}
```

## Error Handling and Fallback Strategies

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

#[derive(Debug)]
enum HardwareError {
    NotAvailable,
    Timeout,
    InvalidLength,
    KeyLoadFailed,
    OperationFailed,
}

impl core::fmt::Display for HardwareError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            HardwareError::NotAvailable => write!(f, "Hardware crypto not available"),
            HardwareError::Timeout => write!(f, "Hardware operation timeout"),
            HardwareError::InvalidLength => write!(f, "Invalid data length"),
            HardwareError::KeyLoadFailed => write!(f, "Failed to load key"),
            HardwareError::OperationFailed => write!(f, "Crypto operation failed"),
        }
    }
}

impl core::error::Error for HardwareError {}

// Robust crypto engine with automatic fallback
struct RobustCryptoEngine {
    primary: Option<XilinxCryptoAccel>,
    fallback: SoftwareCrypto,
    hw_failure_count: u32,
    max_failures: u32,
}

impl RobustCryptoEngine {
    fn new() -> Result<Self, CryptoError> {
        let primary = if XilinxCryptoAccel::is_available() {
            XilinxCryptoAccel::new().ok()
        } else {
            None
        };
        
        let fallback = SoftwareCrypto::new()?;
        
        Ok(Self {
            primary,
            fallback,
            hw_failure_count: 0,
            max_failures: 3,
        })
    }
    
    fn encrypt_aes_robust(&mut self, key: &[u8; 32], plaintext: &[u8], ciphertext: &mut [u8]) -> Result<(), CryptoError> {
        // Try hardware first if available and not failed too many times
        if let Some(ref mut hw) = self.primary {
            if self.hw_failure_count < self.max_failures {
                match hw.aes_encrypt(key, plaintext, ciphertext) {
                    Ok(()) => {
                        self.hw_failure_count = 0; // Reset on success
                        return Ok(());
                    }
                    Err(_) => {
                        self.hw_failure_count += 1;
                        // Fall through to software fallback
                    }
                }
            }
        }
        
        // Use software fallback
        self.fallback.aes_encrypt(key, plaintext, ciphertext)
    }
    
    fn is_hardware_healthy(&self) -> bool {
        self.primary.is_some() && self.hw_failure_count < self.max_failures
    }
    
    fn reset_hardware(&mut self) -> Result<(), CryptoError> {
        if XilinxCryptoAccel::is_available() {
            self.primary = XilinxCryptoAccel::new().ok();
            self.hw_failure_count = 0;
            Ok(())
        } else {
            Err(CryptoError::HardwareNotAvailable)
        }
    }
}
```