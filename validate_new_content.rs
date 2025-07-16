#![no_std]
#![no_main]

// Test all new content examples for embedded compilation
// This validates enum, trait, iterator, closure, and mathematical operation examples

use panic_halt as _;
use cortex_m_rt::entry;
use heapless::Vec;
use zeroize::Zeroize;

// Test 1: Advanced Enum Examples
#[derive(Debug)]
enum CryptoState {
    Idle,
    KeyExchange { 
        key_data: Vec<u8, 256>, 
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

#[derive(Debug, Clone, Copy)]
enum KeyExchangeAlgorithm {
    Ecdh,
    Rsa,
}

#[derive(Debug, Clone, Copy)]
enum CipherType {
    Aes256Gcm,
    ChaCha20Poly1305,
}

#[derive(Debug, Clone, Copy)]
enum CryptoErrorCode {
    InvalidKey = 1,
    InvalidNonce = 2,
    AuthenticationFailed = 3,
}

#[derive(Debug)]
struct SecureKey<const N: usize> {
    key: [u8; N],
}

impl<const N: usize> Drop for SecureKey<N> {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

impl<const N: usize> SecureKey<N> {
    fn new(key_material: [u8; N]) -> Self {
        Self { key: key_material }
    }
    
    fn as_bytes(&self) -> &[u8; N] {
        &self.key
    }
}

// Test enum pattern matching
fn process_crypto_state(state: CryptoState) -> Result<CryptoState, CryptoError> {
    match state {
        CryptoState::Idle => {
            let key_data = Vec::new();
            Ok(CryptoState::KeyExchange { 
                key_data,
                algorithm: KeyExchangeAlgorithm::Ecdh 
            })
        },
        
        CryptoState::KeyExchange { key_data: _, algorithm: _ } => {
            let session_key = SecureKey::new([0u8; 32]);
            Ok(CryptoState::Encrypted { 
                session_key,
                cipher: CipherType::Aes256Gcm 
            })
        },
        
        CryptoState::Encrypted { session_key: _, cipher: _ } => {
            // Process with session key - return new state since we can't move
            let new_key = SecureKey::new([0u8; 32]);
            Ok(CryptoState::Encrypted {
                session_key: new_key,
                cipher: CipherType::Aes256Gcm
            })
        },
        
        CryptoState::Error { code, message: _ } => {
            Err(CryptoError::StateMachine(code))
        }
    }
}

// Test 2: Trait Examples
trait CryptoHash {
    type Output: AsRef<[u8]>;
    const OUTPUT_SIZE: usize;
    
    fn hash(&self, data: &[u8]) -> Self::Output;
}

trait BlockCipher {
    const KEY_SIZE: usize;
    const BLOCK_SIZE: usize;
    
    fn encrypt_block(&self, key: &[u8], block: &mut [u8]) -> Result<(), CryptoError>;
    fn decrypt_block(&self, key: &[u8], block: &mut [u8]) -> Result<(), CryptoError>;
}

// Implementations
struct Sha256;
impl CryptoHash for Sha256 {
    type Output = [u8; 32];
    const OUTPUT_SIZE: usize = 32;
    
    fn hash(&self, data: &[u8]) -> Self::Output {
        // Mock implementation for testing
        let mut result = [0u8; 32];
        for (i, &byte) in data.iter().enumerate().take(32) {
            result[i] = byte.wrapping_add(i as u8);
        }
        result
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
        
        // Mock encryption for testing
        for (i, byte) in block.iter_mut().enumerate() {
            *byte ^= key[i % key.len()];
        }
        Ok(())
    }
    
    fn decrypt_block(&self, key: &[u8], block: &mut [u8]) -> Result<(), CryptoError> {
        // Mock decryption (same as encryption for XOR)
        self.encrypt_block(key, block)
    }
}

// Generic function using traits
fn process_data<H: CryptoHash, C: BlockCipher>(
    hasher: &H, 
    cipher: &C, 
    key: &[u8], 
    data: &mut [u8]
) -> Result<H::Output, CryptoError> {
    if key.len() != C::KEY_SIZE {
        return Err(CryptoError::InvalidKeySize);
    }
    
    let hash = hasher.hash(data);
    
    // Process data in blocks
    for chunk in data.chunks_exact_mut(C::BLOCK_SIZE) {
        cipher.encrypt_block(key, chunk)?;
    }
    
    Ok(hash)
}

// Test 3: Mathematical Operations with Safety
#[derive(Debug, Clone, Copy)]
pub enum ArithmeticMode {
    Checked,
    Saturating,
    Wrapping,
}

pub struct CryptoMath<T> {
    mode: ArithmeticMode,
    _phantom: core::marker::PhantomData<T>,
}

impl CryptoMath<u32> {
    pub fn new(mode: ArithmeticMode) -> Self {
        Self {
            mode,
            _phantom: core::marker::PhantomData,
        }
    }
    
    pub fn add(&self, a: u32, b: u32) -> Result<u32, CryptoError> {
        match self.mode {
            ArithmeticMode::Checked => {
                a.checked_add(b).ok_or(CryptoError::ArithmeticOverflow)
            },
            ArithmeticMode::Saturating => {
                Ok(a.saturating_add(b))
            },
            ArithmeticMode::Wrapping => {
                Ok(a.wrapping_add(b))
            },
        }
    }
    
    pub fn mul(&self, a: u32, b: u32) -> Result<u32, CryptoError> {
        match self.mode {
            ArithmeticMode::Checked => {
                a.checked_mul(b).ok_or(CryptoError::ArithmeticOverflow)
            },
            ArithmeticMode::Saturating => {
                Ok(a.saturating_mul(b))
            },
            ArithmeticMode::Wrapping => {
                Ok(a.wrapping_mul(b))
            },
        }
    }
}

// Test safe arithmetic
fn test_safe_arithmetic() -> Result<(), CryptoError> {
    let math = CryptoMath::new(ArithmeticMode::Checked);
    
    let result1 = math.add(100, 200)?;
    let _result2 = math.mul(result1, 2)?;
    
    // Test overflow detection
    match math.mul(u32::MAX, 2) {
        Err(CryptoError::ArithmeticOverflow) => {
            // Expected overflow detected
        },
        _ => return Err(CryptoError::TestFailed),
    }
    
    Ok(())
}

// Test 4: Iterator Patterns
fn xor_buffers(dst: &mut [u8], src1: &[u8], src2: &[u8]) -> Result<(), CryptoError> {
    if dst.len() != src1.len() || src1.len() != src2.len() {
        return Err(CryptoError::BufferSizeMismatch);
    }
    
    dst.iter_mut()
        .zip(src1.iter())
        .zip(src2.iter())
        .for_each(|((d, s1), s2)| *d = s1 ^ s2);
    
    Ok(())
}

// Iterator for crypto block processing
struct CryptoBlockIterator<'a> {
    data: &'a [u8],
    block_size: usize,
    position: usize,
}

impl<'a> CryptoBlockIterator<'a> {
    fn new(data: &'a [u8], block_size: usize) -> Self {
        Self {
            data,
            block_size,
            position: 0,
        }
    }
}

impl<'a> Iterator for CryptoBlockIterator<'a> {
    type Item = &'a [u8];
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.data.len() {
            return None;
        }
        
        let end = core::cmp::min(self.position + self.block_size, self.data.len());
        let block = &self.data[self.position..end];
        self.position = end;
        
        Some(block)
    }
}

// Test iterator patterns
fn test_iterators() -> Result<(), CryptoError> {
    let mut dst = [0u8; 8];
    let src1 = [1u8, 2, 3, 4, 5, 6, 7, 8];
    let src2 = [8u8, 7, 6, 5, 4, 3, 2, 1];
    
    xor_buffers(&mut dst, &src1, &src2)?;
    
    // Test block iterator
    let data = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut block_count = 0;
    
    for _block in CryptoBlockIterator::new(&data, 4) {
        block_count += 1;
        // Process block
    }
    
    if block_count != 3 {  // 4+4+2 bytes = 3 blocks
        return Err(CryptoError::TestFailed);
    }
    
    Ok(())
}

// Test 5: Closure Examples
fn process_with_closure<F>(
    data: &mut [u8], 
    mut transform: F
) -> Result<(), CryptoError>
where
    F: FnMut(u8) -> u8,
{
    for byte in data.iter_mut() {
        *byte = transform(*byte);
    }
    Ok(())
}

// Secure closure context
struct SecureCryptoContext {
    key: [u8; 32],
    counter: u64,
}

impl Drop for SecureCryptoContext {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

impl SecureCryptoContext {
    fn new(key: [u8; 32]) -> Self {
        Self {
            key,
            counter: 0,
        }
    }
    
    fn create_encryptor(&mut self) -> impl FnMut(&mut [u8]) -> Result<(), CryptoError> + '_ {
        move |data: &mut [u8]| -> Result<(), CryptoError> {
            self.counter = self.counter
                .checked_add(1)
                .ok_or(CryptoError::CounterOverflow)?;
            
            // Simple XOR encryption for testing
            for (i, byte) in data.iter_mut().enumerate() {
                *byte ^= self.key[i % self.key.len()];
            }
            
            Ok(())
        }
    }
}

// Test closures
fn test_closures() -> Result<(), CryptoError> {
    let mut data = [1u8, 2, 3, 4, 5];
    let key = 0x42u8;
    
    // Test simple closure
    process_with_closure(&mut data, |byte| byte ^ key)?;
    
    // Test secure context closure
    let mut context = SecureCryptoContext::new([0x33u8; 32]);
    let mut encryptor = context.create_encryptor();
    
    encryptor(&mut data)?;
    
    Ok(())
}

// Error types for testing
#[derive(Debug, Clone)]
pub enum CryptoError {
    InvalidKeySize,
    InvalidSize,
    BufferSizeMismatch,
    ArithmeticOverflow,
    CounterOverflow,
    StateMachine(CryptoErrorCode),
    TestFailed,
}

// Main test function
#[entry]
fn main() -> ! {
    // Test all new content examples
    let test_results = [
        test_safe_arithmetic(),
        test_iterators(),
        test_closures(),
    ];
    
    // Test enum pattern matching
    let initial_state = CryptoState::Idle;
    let _processed_state = process_crypto_state(initial_state);
    
    // Test trait usage
    let hasher = Sha256;
    let cipher = Aes256;
    let key = [0x42u8; 32];
    let mut data = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    
    let _hash_result = process_data(&hasher, &cipher, &key, &mut data);
    
    // Check all test results
    let all_passed = test_results.iter().all(|result| result.is_ok());
    
    if all_passed {
        // All tests passed - indicate success
        loop {
            cortex_m::asm::nop();
        }
    } else {
        // Some tests failed - indicate failure
        loop {
            cortex_m::asm::wfe();
        }
    }
}