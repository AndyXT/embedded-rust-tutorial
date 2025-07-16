#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;

// Test mathematical operations with overflow protection for crypto safety
#[derive(Debug, Clone, Copy)]
pub enum ArithmeticMode {
    Checked,
    Saturating,
    Wrapping,
}

#[derive(Debug, Clone)]
pub enum CryptoError {
    ArithmeticOverflow,
    InvalidModulus,
    TestFailed,
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
    
    pub fn rotate_left(&self, value: u32, n: u32) -> u32 {
        value.rotate_left(n)
    }
    
    pub fn rotate_right(&self, value: u32, n: u32) -> u32 {
        value.rotate_right(n)
    }
}

// Test modular arithmetic for crypto operations
pub fn mod_exp(base: u64, exp: u64, modulus: u64) -> Result<u64, CryptoError> {
    if modulus == 0 {
        return Err(CryptoError::InvalidModulus);
    }
    
    let mut result = 1u64;
    let mut base = base % modulus;
    let mut exp = exp;
    
    while exp > 0 {
        if exp & 1 == 1 {
            result = result
                .checked_mul(base)
                .ok_or(CryptoError::ArithmeticOverflow)?
                % modulus;
        }
        
        exp >>= 1;
        base = base
            .checked_mul(base)
            .ok_or(CryptoError::ArithmeticOverflow)?
            % modulus;
    }
    
    Ok(result)
}

// Test safe crypto key derivation
fn derive_key_schedule(base_key: u32, round: u32) -> Result<u64, CryptoError> {
    let multiplied = base_key
        .checked_mul(round)
        .ok_or(CryptoError::ArithmeticOverflow)?;
    
    let expanded = (multiplied as u64)
        .checked_mul(0x9E3779B9)
        .ok_or(CryptoError::ArithmeticOverflow)?;
    
    Ok(expanded ^ (expanded >> 32))
}

// Test AES S-box calculation with safe arithmetic
fn aes_sbox_transform(input: u8, math: &CryptoMath<u32>) -> Result<u8, CryptoError> {
    let x = input as u32;
    
    // Galois field operations with overflow protection
    let squared = math.mul(x, x)?;
    let fourth = math.mul(squared, squared)?;
    let result = math.add(fourth, x)?;
    
    Ok((result & 0xFF) as u8)
}

// Test safe block processing with bounds checking
fn process_crypto_block(data: &mut [u8], multiplier: u32) -> Result<(), CryptoError> {
    let data_len = data.len();
    for (i, byte) in data.iter_mut().enumerate() {
        let offset = (i as u32)
            .checked_mul(multiplier)
            .ok_or(CryptoError::ArithmeticOverflow)?;
        
        if (offset as usize) < data_len {
            *byte ^= 0xAA;
        }
    }
    Ok(())
}

#[entry]
fn main() -> ! {
    // Test 1: Checked arithmetic mode
    let checked_math = CryptoMath::new(ArithmeticMode::Checked);
    
    // Test normal operations
    let result1 = checked_math.add(100, 200).unwrap();
    let _result2 = checked_math.mul(result1, 2).unwrap();
    
    // Test overflow detection
    let overflow_test = checked_math.mul(u32::MAX, 2);
    match overflow_test {
        Err(CryptoError::ArithmeticOverflow) => {
            // Expected overflow detected - good!
        },
        _ => {
            // Test failed - should have detected overflow
            loop { cortex_m::asm::wfe(); }
        }
    }
    
    // Test 2: Saturating arithmetic mode
    let saturating_math = CryptoMath::new(ArithmeticMode::Saturating);
    let saturated_result = saturating_math.mul(u32::MAX, 2).unwrap();
    if saturated_result != u32::MAX {
        // Test failed - should have saturated to MAX
        loop { cortex_m::asm::wfe(); }
    }
    
    // Test 3: Wrapping arithmetic mode
    let wrapping_math = CryptoMath::new(ArithmeticMode::Wrapping);
    let wrapped_result = wrapping_math.add(u32::MAX, 1).unwrap();
    if wrapped_result != 0 {
        // Test failed - should have wrapped to 0
        loop { cortex_m::asm::wfe(); }
    }
    
    // Test 4: Rotation operations
    let rotated_left = checked_math.rotate_left(0x12345678, 8);
    let rotated_right = checked_math.rotate_right(rotated_left, 8);
    if rotated_right != 0x12345678 {
        // Test failed - rotation should be reversible
        loop { cortex_m::asm::wfe(); }
    }
    
    // Test 5: Modular exponentiation
    let mod_result = mod_exp(2, 10, 1000).unwrap();
    if mod_result != 24 {  // 2^10 % 1000 = 1024 % 1000 = 24
        // Test failed - incorrect modular exponentiation
        loop { cortex_m::asm::wfe(); }
    }
    
    // Test 6: Key derivation with overflow protection
    let key_schedule = derive_key_schedule(0x12345678, 1).unwrap();
    if key_schedule == 0 {
        // Test failed - key schedule should not be zero
        loop { cortex_m::asm::wfe(); }
    }
    
    // Test 7: AES S-box transformation
    let sbox_result = aes_sbox_transform(0x53, &checked_math).unwrap();
    if sbox_result == 0x53 {
        // Test failed - S-box should transform the input
        loop { cortex_m::asm::wfe(); }
    }
    
    // Test 8: Safe block processing
    let mut test_data = [1u8, 2, 3, 4, 5, 6, 7, 8];
    let process_result = process_crypto_block(&mut test_data, 1);
    if process_result.is_err() {
        // Test failed - should succeed with multiplier 1
        loop { cortex_m::asm::wfe(); }
    }
    
    // Test 9: Overflow detection in block processing
    let mut test_data2 = [1u8, 2, 3, 4];
    let overflow_process = process_crypto_block(&mut test_data2, u32::MAX);
    match overflow_process {
        Err(CryptoError::ArithmeticOverflow) => {
            // Expected overflow detected - good!
        },
        _ => {
            // Test failed - should have detected overflow
            loop { cortex_m::asm::wfe(); }
        }
    }
    
    // All tests passed!
    loop {
        cortex_m::asm::nop();
    }
}