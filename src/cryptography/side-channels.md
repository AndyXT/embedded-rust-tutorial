# 5.5 Side-Channel Mitigations

Protecting against side-channel attacks is crucial for embedded cryptographic implementations. Side-channel attacks exploit information leaked through execution time, power consumption, electromagnetic emissions, or cache behavior. This section demonstrates Rust-specific techniques for mitigating these vulnerabilities.

## Understanding Side-Channel Vulnerabilities

Side-channel attacks can extract sensitive information by analyzing:

- **Timing variations**: Different execution paths reveal secret-dependent information
- **Power consumption**: Operations on different data consume different amounts of power
- **Electromagnetic emissions**: Hardware operations emit characteristic EM signatures
- **Cache behavior**: Memory access patterns can leak information about secret data

## Power Analysis Protection

Power analysis attacks monitor the power consumption of cryptographic devices to extract secret keys. Rust provides several techniques to mitigate these attacks:

```rust
#![no_std]
#![no_main]

use panic_halt as _;
// Helper functions for Cortex-R5 bare metal examples
#[cfg(feature = "embedded")]
mod cortex_r5_helpers {
    use core::ptr;
    
    /// Simple cycle counter for timing (implementation depends on your specific Cortex-R5 setup)
    pub fn get_cycle_count() -> u32 {
        // This is a placeholder - implement based on your specific Cortex-R5 configuration
        // You might use PMU (Performance Monitoring Unit) or a timer peripheral
        0 // Placeholder
    }
    
    /// Memory barrier for ensuring ordering in crypto operations
    pub fn memory_barrier() {
        unsafe {
            core::arch::asm!("dmb sy", options(nostack, preserves_flags));
        }
    }
    
    /// Constant-time conditional move (basic implementation)
    pub fn conditional_move(condition: bool, a: u8, b: u8) -> u8 {
        let mask = if condition { 0xFF } else { 0x00 };
        (a & mask) | (b & !mask)
    }
}

#[cfg(feature = "embedded")]
use cortex_r5_helpers::*;```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};


use core::mem;
use core::fmt;

use core::result::Result;

use rand_core::RngCore;
use core::hint::black_box;

// Randomized execution timing to mask power patterns
fn randomized_delay(rng: &mut impl RngCore) {
    let delay_cycles = rng.next_u32() & 0xFF; // 0-255 cycles
    for _ in 0..delay_cycles {
        cortex_m::asm::nop();
        black_box(()); // Prevent optimization
    }
}

// Power-balanced operations using dummy computations
fn power_balanced_aes_sbox(input: u8, sbox: &[u8; 256], rng: &mut impl RngCore) -> u8 {
    let real_output = sbox[input as usize];
    
    // Perform dummy lookups to balance power consumption
    let dummy_indices = [
        rng.next_u32() as u8,
        rng.next_u32() as u8,
        rng.next_u32() as u8,
    ];
    
    for &dummy_idx in &dummy_indices {
        let _dummy_output = sbox[dummy_idx as usize];
        black_box(_dummy_output); // Ensure operation isn't optimized away
    }
    
    real_output
}

// Blinded scalar multiplication for ECC operations
fn blinded_scalar_multiply(
    scalar: &[u8; 32], 
    point: &EccPoint, 
    rng: &mut impl RngCore
) -> EccPoint {
    // Generate random blinding factor
    let mut blind = [0u8; 32];
    rng.fill_bytes(&mut blind);
    
    // Blind the scalar: blinded_scalar = scalar + blind
    let blinded_scalar = scalar_add(scalar, &blind);
    
    // Perform blinded operation: blinded_result = blinded_scalar * point
    let blinded_result = scalar_multiply(&blinded_scalar, point);
    
    // Remove blinding: result = blinded_result - blind * point
    let blind_point = scalar_multiply(&blind, point);
    point_subtract(&blinded_result, &blind_point)
}

// Power-analysis resistant key expansion
fn power_resistant_key_expansion(
    master_key: &[u8; 32],
    rng: &mut impl RngCore
) -> [[u8; 16]; 15] {
    let mut round_keys = [[0u8; 16]; 15];
    
    // Add random delays between key expansion rounds
    for i in 0..15 {
        randomized_delay(rng);
        
        // Perform key expansion with power balancing
        expand_round_key(master_key, i, &mut round_keys[i]);
        
        // Add dummy operations to mask the real computation
        let mut dummy_key = [0u8; 16];
        let dummy_round = rng.next_u32() as usize % 15;
        expand_round_key(master_key, dummy_round, &mut dummy_key);
        black_box(dummy_key);
    }
    
    round_keys
}

// Secure memory clearing with power analysis resistance
fn secure_power_resistant_clear(data: &mut [u8], rng: &mut impl RngCore) {
    // Multiple clearing passes with random data
    for _ in 0..3 {
        // Fill with random data first
        rng.fill_bytes(data);
        black_box(&data[0]); // Ensure write happens
        
        // Then clear to zeros
        data.fill(0);
        black_box(&data[0]); // Ensure write happens
        
        // Add random delay
        randomized_delay(rng);
    }
}
```

## Electromagnetic (EM) Analysis Protection

EM analysis attacks monitor electromagnetic emissions from cryptographic devices:

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

// EM-resistant operations using noise injection
struct EmResistantCrypto {
    noise_generator: ChaCha20Rng,
    dummy_operations: u32,
}

impl EmResistantCrypto {
    fn new(seed: [u8; 32]) -> Self {
        Self {
            noise_generator: ChaCha20Rng::from_seed(seed),
            dummy_operations: 0,
        }
    }
    
    // Inject EM noise through dummy computations
    fn inject_em_noise(&mut self) {
        let noise_level = self.noise_generator.next_u32() & 0x1F; // 0-31 operations
        
        for _ in 0..noise_level {
            // Perform dummy arithmetic operations
            let a = self.noise_generator.next_u32();
            let b = self.noise_generator.next_u32();
            let _result = a.wrapping_mul(b).wrapping_add(0xDEADBEEF);
            black_box(_result);
            
            self.dummy_operations += 1;
        }
    }
    
    // EM-resistant AES encryption
    fn em_resistant_aes_encrypt(
        &mut self,
        key: &[u8; 32],
        plaintext: &[u8; 16],
        ciphertext: &mut [u8; 16]
    ) -> Result<(), CryptoError> {
        // Inject noise before operation
        self.inject_em_noise();
        
        // Perform actual encryption with interleaved noise
        let mut state = *plaintext;
        let round_keys = aes_key_expansion(key);
        
        for (round, round_key) in round_keys.iter().enumerate() {
            // Add round key
            for i in 0..16 {
                state[i] ^= round_key[i];
            }
            
            // Inject noise between operations
            if round % 2 == 0 {
                self.inject_em_noise();
            }
            
            // SubBytes, ShiftRows, MixColumns (except last round)
            if round < 14 {
                aes_sub_bytes(&mut state);
                self.inject_em_noise();
                
                aes_shift_rows(&mut state);
                self.inject_em_noise();
                
                aes_mix_columns(&mut state);
            } else {
                aes_sub_bytes(&mut state);
                aes_shift_rows(&mut state);
            }
        }
        
        // Final noise injection
        self.inject_em_noise();
        
        *ciphertext = state;
        Ok(())
    }
    
    // EM-resistant table lookups
    fn em_resistant_table_lookup(
        &mut self,
        table: &[u8; 256],
        index: u8
    ) -> u8 {
        let real_result = table[index as usize];
        
        // Perform multiple dummy lookups to mask the real one
        let dummy_count = (self.noise_generator.next_u32() & 0x7) + 1; // 1-8 dummy lookups
        
        for _ in 0..dummy_count {
            let dummy_index = self.noise_generator.next_u32() as u8;
            let _dummy_result = table[dummy_index as usize];
            black_box(_dummy_result);
        }
        
        real_result
    }
}

// Hardware-level EM protection using register manipulation
fn em_hardware_protection() {
    unsafe {
        // Toggle unused GPIO pins to create EM noise
        let gpio_base = 0xFF0A0000 as *mut u32;
        let noise_pattern = 0xAAAA5555u32;
        
        for _ in 0..10 {
            core::ptr::write_volatile(gpio_base, noise_pattern);
            core::ptr::write_volatile(gpio_base, !noise_pattern);
            cortex_m::asm::nop();
        }
    }
}
```

## Cache Timing Attack Mitigations

Cache timing attacks exploit variations in memory access times to infer secret information:

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};


use core::mem;
use core::fmt;

use core::result::Result;

use core::hint::black_box;

// Cache-resistant table lookups
fn cache_resistant_lookup(table: &[u8; 256], index: u8) -> u8 {
    let mut result = 0u8;
    
    // Access every table entry to maintain consistent cache behavior
    for (i, &value) in table.iter().enumerate() {
        let mask = ((i as u8 == index) as u8).wrapping_neg();
        result |= value & mask;
        
        // Ensure memory access happens and isn't optimized away
        black_box(value);
    }
    
    result
}

// Cache-resistant AES S-box implementation
const AES_SBOX: [u8; 256] = [
    // Standard AES S-box values...
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    // ... (complete S-box)
];

fn cache_resistant_sbox(input: u8) -> u8 {
    cache_resistant_lookup(&AES_SBOX, input)
}

// Multi-dimensional cache-resistant lookup
fn cache_resistant_matrix_lookup(
    matrix: &[[u8; 16]; 16],
    row: u8,
    col: u8
) -> u8 {
    let mut result = 0u8;
    
    // Access all matrix elements to prevent cache-based leakage
    for (r, row_data) in matrix.iter().enumerate() {
        let row_mask = ((r as u8 == row) as u8).wrapping_neg();
        
        for (c, &value) in row_data.iter().enumerate() {
            let col_mask = ((c as u8 == col) as u8).wrapping_neg();
            let select_mask = row_mask & col_mask;
            
            result |= value & select_mask;
            black_box(value); // Ensure access happens
        }
    }
    
    result
}

// Cache-line aligned data structures to control cache behavior
#[repr(align(64))] // Align to cache line boundary
struct CacheAlignedCryptoState {
    state: [u8; 16],
    round_keys: [[u8; 16]; 15],
    _padding: [u8; 48], // Pad to multiple of cache line size
}

impl CacheAlignedCryptoState {
    fn new() -> Self {
        Self {
            state: [0; 16],
            round_keys: [[0; 16]; 15],
            _padding: [0; 48],
        }
    }
    
    // Cache-aware AES implementation
    fn cache_aware_aes_encrypt(&mut self, plaintext: &[u8; 16]) -> [u8; 16] {
        self.state.copy_from_slice(plaintext);
        
        // Process rounds with cache-conscious access patterns
        for round in 0..14 {
            // Ensure round key is in cache
            black_box(&self.round_keys[round]);
            
            // Perform round operations
            self.add_round_key(round);
            
            if round < 13 {
                self.cache_resistant_sub_bytes();
                self.shift_rows();
                self.mix_columns();
            } else {
                self.cache_resistant_sub_bytes();
                self.shift_rows();
            }
        }
        
        self.state
    }
    
    fn cache_resistant_sub_bytes(&mut self) {
        for byte in &mut self.state {
            *byte = cache_resistant_sbox(*byte);
        }
    }
    
    fn add_round_key(&mut self, round: usize) {
        for i in 0..16 {
            self.state[i] ^= self.round_keys[round][i];
        }
    }
    
    fn shift_rows(&mut self) {
        // Standard AES ShiftRows operation
        let temp = self.state[1];
        self.state[1] = self.state[5];
        self.state[5] = self.state[9];
        self.state[9] = self.state[13];
        self.state[13] = temp;
        
        // Continue for other rows...
    }
    
    fn mix_columns(&mut self) {
        // Standard AES MixColumns operation
        for col in 0..4 {
            let base = col * 4;
            let a = self.state[base];
            let b = self.state[base + 1];
            let c = self.state[base + 2];
            let d = self.state[base + 3];
            
            self.state[base] = gf_mul(a, 2) ^ gf_mul(b, 3) ^ c ^ d;
            self.state[base + 1] = a ^ gf_mul(b, 2) ^ gf_mul(c, 3) ^ d;
            self.state[base + 2] = a ^ b ^ gf_mul(c, 2) ^ gf_mul(d, 3);
            self.state[base + 3] = gf_mul(a, 3) ^ b ^ c ^ gf_mul(d, 2);
        }
    }
}

// Galois field multiplication for AES
fn gf_mul(a: u8, b: u8) -> u8 {
    let mut result = 0;
    let mut temp_a = a;
    let mut temp_b = b;
    
    while temp_b != 0 {
        if temp_b & 1 != 0 {
            result ^= temp_a;
        }
        
        let high_bit = temp_a & 0x80;
        temp_a <<= 1;
        if high_bit != 0 {
            temp_a ^= 0x1B; // AES irreducible polynomial
        }
        
        temp_b >>= 1;
    }
    
    result
}
```

## Fault Injection Attack Protection

Fault injection attacks attempt to corrupt computations to extract secret information:

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

// Redundant computation for fault detection
fn fault_resistant_aes_encrypt(
    key: &[u8; 32],
    plaintext: &[u8; 16],
    ciphertext: &mut [u8; 16]
) -> Result<(), CryptoError> {
    // Perform encryption twice with independent implementations
    let result1 = aes_encrypt_implementation_a(key, plaintext)?;
    let result2 = aes_encrypt_implementation_b(key, plaintext)?;
    
    // Compare results to detect faults
    if result1 != result2 {
        // Clear outputs on fault detection
        ciphertext.fill(0);
        return Err(CryptoError::FaultDetected);
    }
    
    *ciphertext = result1;
    Ok(())
}

// Checksum-based fault detection
fn checksum_protected_operation(data: &mut [u8]) -> Result<(), CryptoError> {
    // Calculate initial checksum
    let initial_checksum = calculate_checksum(data);
    
    // Perform operation
    crypto_operation(data);
    
    // Verify checksum after operation
    let final_checksum = calculate_checksum(data);
    let expected_checksum = transform_checksum(initial_checksum);
    
    if final_checksum != expected_checksum {
        // Fault detected - clear data and return error
        data.fill(0);
        return Err(CryptoError::FaultDetected);
    }
    
    Ok(())
}

fn calculate_checksum(data: &[u8]) -> u32 {
    data.iter().fold(0u32, |acc, &byte| {
        acc.wrapping_mul(31).wrapping_add(byte as u32)
    })
}

fn transform_checksum(checksum: u32) -> u32 {
    // Transform checksum based on expected operation
    checksum.wrapping_mul(0x9E3779B9) // Golden ratio constant
}

// Hardware-based fault detection using watchdog timers
struct FaultDetectionContext {
    watchdog_timer: WatchdogTimer,
    operation_start_time: u32,
    max_operation_time: u32,
}

impl FaultDetectionContext {
    fn new(max_time_ms: u32) -> Self {
        Self {
            watchdog_timer: WatchdogTimer::new(),
            operation_start_time: 0,
            max_operation_time: max_time_ms,
        }
    }
    
    fn start_protected_operation(&mut self) {
        self.operation_start_time = get_system_time_ms();
        self.watchdog_timer.start(self.max_operation_time);
    }
    
    fn check_operation_integrity(&mut self) -> Result<(), CryptoError> {
        let current_time = get_system_time_ms();
        let elapsed = current_time - self.operation_start_time;
        
        // Check for timing anomalies that might indicate fault injection
        if elapsed > self.max_operation_time {
            return Err(CryptoError::TimingAnomalyDetected);
        }
        
        // Check watchdog status
        if self.watchdog_timer.has_expired() {
            return Err(CryptoError::WatchdogExpired);
        }
        
        Ok(())
    }
    
    fn finish_protected_operation(&mut self) {
        self.watchdog_timer.stop();
    }
}

// Memory protection against fault injection
fn memory_protected_crypto_operation(
    sensitive_data: &mut [u8],
    operation: impl Fn(&mut [u8]) -> Result<(), CryptoError>
) -> Result<(), CryptoError> {
    // Create multiple copies for comparison
    let mut copy1 = sensitive_data.to_vec();
    let mut copy2 = sensitive_data.to_vec();
    let mut copy3 = sensitive_data.to_vec();
    
    // Perform operation on all copies
    operation(&mut copy1)?;
    operation(&mut copy2)?;
    operation(&mut copy3)?;
    
    // Majority voting to detect and correct single faults
    for i in 0..sensitive_data.len() {
        let votes = [copy1[i], copy2[i], copy3[i]];
        
        // Count votes for each value
        let mut vote_count = [0u8; 256];
        for &vote in &votes {
            vote_count[vote as usize] += 1;
        }
        
        // Find majority value
        let mut majority_value = 0u8;
        let mut max_votes = 0u8;
        
        for (value, &count) in vote_count.iter().enumerate() {
            if count > max_votes {
                max_votes = count;
                majority_value = value as u8;
            }
        }
        
        // If no clear majority, fault detected
        if max_votes < 2 {
            sensitive_data.fill(0);
            return Err(CryptoError::FaultDetected);
        }
        
        sensitive_data[i] = majority_value;
    }
    
    // Clear temporary copies
    copy1.zeroize();
    copy2.zeroize();
    copy3.zeroize();
    
    Ok(())
}
```

## Comprehensive Side-Channel Protection Framework

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

// Complete side-channel resistant crypto framework
struct SideChannelResistantCrypto {
    em_protection: EmResistantCrypto,
    fault_detection: FaultDetectionContext,
    cache_aligned_state: CacheAlignedCryptoState,
    rng: ChaCha20Rng,
}

impl SideChannelResistantCrypto {
    fn new(seed: [u8; 32]) -> Self {
        Self {
            em_protection: EmResistantCrypto::new(seed),
            fault_detection: FaultDetectionContext::new(1000), // 1 second max
            cache_aligned_state: CacheAlignedCryptoState::new(),
            rng: ChaCha20Rng::from_seed(seed),
        }
    }
    
    // Comprehensive protected AES encryption
    fn protected_aes_encrypt(
        &mut self,
        key: &[u8; 32],
        plaintext: &[u8; 16],
        ciphertext: &mut [u8; 16]
    ) -> Result<(), CryptoError> {
        // Start fault detection
        self.fault_detection.start_protected_operation();
        
        // Inject EM noise
        self.em_protection.inject_em_noise();
        
        // Perform cache-resistant encryption with redundancy
        let result1 = self.cache_aligned_state.cache_aware_aes_encrypt(plaintext);
        
        // Check for timing anomalies
        self.fault_detection.check_operation_integrity()?;
        
        // Second encryption for fault detection
        let result2 = self.em_protection.em_resistant_aes_encrypt(key, plaintext, ciphertext)?;
        
        // Verify results match
        if result1 != *ciphertext {
            ciphertext.fill(0);
            return Err(CryptoError::FaultDetected);
        }
        
        // Final EM noise injection
        self.em_protection.inject_em_noise();
        
        // Finish fault detection
        self.fault_detection.finish_protected_operation();
        
        Ok(())
    }
    
    // Protected key derivation
    fn protected_key_derivation(
        &mut self,
        master_key: &[u8; 32],
        context: &[u8],
        derived_key: &mut [u8; 32]
    ) -> Result<(), CryptoError> {
        self.fault_detection.start_protected_operation();
        
        // Use HKDF with side-channel protections
        let mut hkdf_context = HkdfContext::new();
        
        // Extract phase with power balancing
        randomized_delay(&mut self.rng);
        hkdf_context.extract(master_key)?;
        
        self.em_protection.inject_em_noise();
        
        // Expand phase with cache resistance
        hkdf_context.expand_cache_resistant(context, derived_key)?;
        
        self.fault_detection.check_operation_integrity()?;
        
        // Verify derivation with redundant computation
        let mut verification_key = [0u8; 32];
        hkdf_context.expand_cache_resistant(context, &mut verification_key)?;
        
        if derived_key != &verification_key {
            derived_key.fill(0);
            verification_key.zeroize();
            return Err(CryptoError::FaultDetected);
        }
        
        verification_key.zeroize();
        self.fault_detection.finish_protected_operation();
        
        Ok(())
    }
}

// Usage example with comprehensive protection
fn comprehensive_crypto_example() -> Result<(), CryptoError> {
    let seed = [0x42u8; 32]; // In practice, use secure random seed
    let mut protected_crypto = SideChannelResistantCrypto::new(seed);
    
    let master_key = [0x12u8; 32];
    let plaintext = [0x34u8; 16];
    let mut ciphertext = [0u8; 16];
    
    // Perform protected encryption
    protected_crypto.protected_aes_encrypt(&master_key, &plaintext, &mut ciphertext)?;
    
    defmt::info!("Protected encryption completed successfully");
    
    // Derive session key with protection
    let mut session_key = [0u8; 32];
    protected_crypto.protected_key_derivation(&master_key, b"session", &mut session_key)?;
    
    defmt::info!("Protected key derivation completed successfully");
    
    // Clear sensitive data
    session_key.zeroize();
    
    Ok(())
}
```