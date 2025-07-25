# 5.2 Constant-Time Implementations {constant-time-implementations}

Constant-time implementations are crucial for preventing timing attacks in cryptographic code. Rust provides excellent tools and libraries for implementing constant-time operations, with compile-time guarantees and runtime verification capabilities.

**Enhanced by Functional Programming:**
- → [Functional Programming and Data Processing](../core-concepts/functional.md) - Use iterator patterns for constant-time data processing

## Understanding Side-Channel Vulnerabilities

Side-channel attacks exploit information leaked through execution time, power consumption, or electromagnetic emissions. In embedded systems, timing attacks are particularly dangerous.

### Vulnerable vs Secure MAC Verification

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use subtle::ConstantTimeEq;

// VULNERABLE: Early return leaks timing information
fn vulnerable_mac_verify(expected: &[u8], received: &[u8]) -> bool {
    if expected.len() != received.len() {
        return false; // Different timing for length mismatch
    }
    
    for (a, b) in expected.iter().zip(received.iter()) {
        if a != b {
            return false; // Early return reveals position of first difference
        }
    }
    true
}

// SECURE: Constant-time comparison
fn secure_mac_verify(expected: &[u8], received: &[u8]) -> bool {
    if expected.len() != received.len() {
        return false;
    }
    
    // Always takes same time regardless of input differences
    expected.ct_eq(received).into()
}

#[entry]
fn main() -> ! {
    let mac1 = [0u8; 32];
    let mac2 = [0u8; 32];
    
    // Vulnerable comparison - timing depends on data
    let _ = vulnerable_mac_verify(&mac1, &mac2);
    
    // Secure comparison - constant time
    let _ = secure_mac_verify(&mac1, &mac2);
    
    loop {}
}
```

## Using the `subtle` Crate for Constant-Time Operations

The `subtle` crate provides cryptographically secure constant-time operations.

### MAC Verification with Constant-Time Comparison

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use subtle::{Choice, ConstantTimeEq};

#[derive(Debug)]
struct CryptoError(&'static str);

// Stub function for HMAC
fn hmac_sha256(_key: &[u8; 32], _message: &[u8]) -> Result<[u8; 32], CryptoError> {
    Ok([0u8; 32])
}

// Constant-time MAC verification with comprehensive example
fn verify_hmac_constant_time(
    key: &[u8; 32],
    message: &[u8],
    expected_mac: &[u8; 32],
    received_mac: &[u8; 32]
) -> Result<bool, CryptoError> {
    // Compute expected MAC
    let computed_mac = hmac_sha256(key, message)?;
    
    // Constant-time comparison - always takes same time
    let mac_valid = computed_mac.ct_eq(received_mac);
    
    // Also verify against provided expected MAC
    let expected_valid = expected_mac.ct_eq(received_mac);
    
    // Both must match for verification to succeed
    Ok((mac_valid & expected_valid).into())
}

#[entry]
fn main() -> ! {
    let key = [0u8; 32];
    let message = b"test message";
    let expected = [0u8; 32];
    let received = [0u8; 32];
    
    let _ = verify_hmac_constant_time(&key, message, &expected, &received);
    
    loop {}
}
```

### Conditional Key Selection

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use subtle::{Choice, ConditionallySelectable};

// Constant-time conditional key selection
fn conditional_key_selection(
    condition: bool, 
    key_a: &[u8; 32], 
    key_b: &[u8; 32]
) -> [u8; 32] {
    let choice = Choice::from(condition as u8);
    let mut result = [0u8; 32];
    
    for i in 0..32 {
        result[i] = u8::conditional_select(&key_a[i], &key_b[i], choice);
    }
    
    result
}

#[entry]
fn main() -> ! {
    let key1 = [0xAAu8; 32];
    let key2 = [0xBBu8; 32];
    
    // Select key based on condition without branching
    let selected = conditional_key_selection(true, &key1, &key2);
    
    loop {}
}
```

### Constant-Time Table Lookups

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use subtle::{Choice, ConditionallySelectable};

// Example AES S-box (first 16 bytes for brevity)
const AES_SBOX: [u8; 256] = {
    let mut sbox = [0u8; 256];
    sbox[0] = 0x63; sbox[1] = 0x7c; sbox[2] = 0x77; sbox[3] = 0x7b;
    sbox[4] = 0xf2; sbox[5] = 0x6b; sbox[6] = 0x6f; sbox[7] = 0xc5;
    sbox[8] = 0x30; sbox[9] = 0x01; sbox[10] = 0x67; sbox[11] = 0x2b;
    sbox[12] = 0xfe; sbox[13] = 0xd7; sbox[14] = 0xab; sbox[15] = 0x76;
    // Remaining values would be filled in production
    sbox
};

// Constant-time table lookup - critical for S-box implementations
fn constant_time_sbox_lookup(index: u8) -> u8 {
    let mut result = 0u8;
    
    for (i, &value) in AES_SBOX.iter().enumerate() {
        let choice = Choice::from((i as u8 == index) as u8);
        result = u8::conditional_select(&result, &value, choice);
    }
    
    result
}

// Example matrix for MixColumns operation
const MIX_COLUMNS_MATRIX: [[u8; 16]; 16] = [[0; 16]; 16];

// Constant-time multi-dimensional table lookup
fn constant_time_matrix_lookup(row: u8, col: u8) -> u8 {
    let mut result = 0u8;
    
    for (r, row_data) in MIX_COLUMNS_MATRIX.iter().enumerate() {
        let row_choice = Choice::from((r as u8 == row) as u8);
        
        for (c, &value) in row_data.iter().enumerate() {
            let col_choice = Choice::from((c as u8 == col) as u8);
            let select_choice = row_choice & col_choice;
            
            result = u8::conditional_select(&result, &value, select_choice);
        }
    }
    
    result
}

#[entry]
fn main() -> ! {
    // Example S-box lookup
    let index = 0x42;
    let value = constant_time_sbox_lookup(index);
    
    // Example matrix lookup
    let element = constant_time_matrix_lookup(3, 7);
    
    loop {}
}
```

## Advanced Constant-Time Patterns

### Constant-Time Scalar Arithmetic

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

// Constant-time modular arithmetic for ECC operations
struct ConstantTimeScalar([u64; 4]); // 256-bit scalar

impl ConstantTimeScalar {
    fn new(value: [u64; 4]) -> Self {
        Self(value)
    }
    
    // Constant-time conditional negation
    fn conditional_negate(&mut self, choice: Choice) {
        let mask = u64::conditional_select(&0, &u64::MAX, choice);
        
        // XOR with mask (0 or all 1s)
        for limb in &mut self.0 {
            *limb ^= mask;
        }
        
        // Add 1 if negating (two's complement)
        let mut carry = u64::from(u8::from(choice));
        for limb in &mut self.0 {
            let (new_limb, new_carry) = limb.overflowing_add(carry);
            *limb = new_limb;
            carry = new_carry as u64;
        }
    }
    
    // Constant-time conditional addition
    fn conditional_add(&mut self, other: &Self, choice: Choice) {
        let mut carry = 0u64;
        
        for (a, &b) in self.0.iter_mut().zip(other.0.iter()) {
            let addend = u64::conditional_select(&0, &b, choice);
            let (sum, c1) = a.overflowing_add(addend);
            let (final_sum, c2) = sum.overflowing_add(carry);
            
            *a = final_sum;
            carry = (c1 as u64) + (c2 as u64);
        }
    }
    
    // Constant-time comparison
    fn ct_eq(&self, other: &Self) -> Choice {
        let mut acc = 0u64;
        for (a, b) in self.0.iter().zip(other.0.iter()) {
            acc |= a ^ b;
        }
        Choice::from((acc == 0) as u8)
    }
    
    // Constant-time greater than comparison
    fn ct_gt(&self, other: &Self) -> Choice {
        let mut borrow = 0u64;
        let mut result = 0u64;
        
        for (&a, &b) in self.0.iter().zip(other.0.iter()) {
            let (diff, b1) = a.overflowing_sub(b);
            let (final_diff, b2) = diff.overflowing_sub(borrow);
            
            result |= final_diff;
            borrow = (b1 as u64) + (b2 as u64);
        }
        
        Choice::from((borrow == 0 && result != 0) as u8)
    }
}

#[entry]
fn main() -> ! {
    let mut scalar1 = ConstantTimeScalar::new([1, 0, 0, 0]);
    let scalar2 = ConstantTimeScalar::new([2, 0, 0, 0]);
    
    // Conditional operations without branching
    scalar1.conditional_negate(Choice::from(1));
    scalar1.conditional_add(&scalar2, Choice::from(1));
    
    let _ = scalar1.ct_eq(&scalar2);
    let _ = scalar1.ct_gt(&scalar2);
    
    loop {}
}
```

### Constant-Time Array Operations

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeLess};

// Constant-time conditional swap for sorting/selection algorithms
fn conditional_swap_arrays<const N: usize>(
    condition: Choice,
    a: &mut [u8; N],
    b: &mut [u8; N]
) {
    for (x, y) in a.iter_mut().zip(b.iter_mut()) {
        let temp = *x;
        *x = u8::conditional_select(x, y, condition);
        *y = u8::conditional_select(y, &temp, condition);
    }
}

// Constant-time binary search in sorted array
fn constant_time_binary_search(
    sorted_array: &[[u8; 32]],
    target: &[u8; 32]
) -> (Choice, usize) {
    let mut found = Choice::from(0);
    let mut result_index = 0usize;
    
    let mut left = 0;
    let mut right = sorted_array.len();
    
    while left < right {
        let mid = (left + right) / 2;
        let mid_value = &sorted_array[mid];
        
        let equal = target.ct_eq(mid_value);
        let less = target.ct_less(mid_value);
        
        // Update found flag if equal
        found = found | equal;
        
        // Update result index if equal
        result_index = usize::conditional_select(&result_index, &mid, equal);
        
        // Update search bounds
        let go_left = less;
        let go_right = !less & !equal;
        
        left = usize::conditional_select(&left, &(mid + 1), go_right);
        right = usize::conditional_select(&right, &mid, go_left);
    }
    
    (found, result_index)
}

#[entry]
fn main() -> ! {
    let mut arr1 = [0xAAu8; 32];
    let mut arr2 = [0xBBu8; 32];
    
    // Conditional swap without branching
    conditional_swap_arrays(Choice::from(1), &mut arr1, &mut arr2);
    
    // Constant-time search
    let sorted = [[0u8; 32], [1u8; 32], [2u8; 32]];
    let target = [1u8; 32];
    let (found, index) = constant_time_binary_search(&sorted, &target);
    
    loop {}
}
```

## Manual Constant-Time Implementations

For cases where the `subtle` crate isn't available or you need custom operations.

### Basic Constant-Time Operations

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;

// Manual constant-time byte operations
fn constant_time_memcmp_manual(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    
    // Convert to bool in constant time
    result == 0
}

fn constant_time_select_byte_manual(condition: bool, a: u8, b: u8) -> u8 {
    // Create mask: true -> 0xFF, false -> 0x00
    let mask = (condition as u8).wrapping_neg();
    (a & mask) | (b & !mask)
}

#[entry]
fn main() -> ! {
    let data1 = [0xAAu8; 32];
    let data2 = [0xAAu8; 32];
    
    // Manual constant-time comparison
    let equal = constant_time_memcmp_manual(&data1, &data2);
    
    // Manual constant-time selection
    let selected = constant_time_select_byte_manual(true, 0xAA, 0xBB);
    
    loop {}
}
```

### Arithmetic Without Branching

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;

// Constant-time find maximum without branching
fn constant_time_max_u32(a: u32, b: u32) -> u32 {
    let diff = a.wrapping_sub(b);
    let mask = (diff >> 31).wrapping_neg(); // Extract sign bit
    
    // If a >= b, mask is 0x00000000, else 0xFFFFFFFF
    (a & !mask) | (b & mask)
}

// Constant-time absolute value
fn constant_time_abs_i32(x: i32) -> u32 {
    let mask = (x >> 31) as u32; // Sign extension: 0x00000000 or 0xFFFFFFFF
    ((x as u32) ^ mask).wrapping_sub(mask)
}

// Constant-time conditional increment
fn constant_time_conditional_increment(value: &mut u32, condition: bool) {
    let increment = condition as u32;
    *value = value.wrapping_add(increment);
}

#[entry]
fn main() -> ! {
    // Constant-time maximum
    let max = constant_time_max_u32(100, 200);
    
    // Constant-time absolute value
    let abs = constant_time_abs_i32(-42);
    
    // Conditional increment
    let mut counter = 0u32;
    constant_time_conditional_increment(&mut counter, true);
    
    loop {}
}
```

## Embedded-Specific Constant-Time Considerations

### Protecting Against Compiler Optimizations

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use core::hint::black_box;

// Disable compiler optimizations that might break constant-time properties
fn protected_constant_time_operation(secret: &[u8; 32], public: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    
    // Use black_box to prevent compiler from optimizing away operations
    for i in 0..32 {
        let secret_byte = black_box(secret[i]);
        let public_byte = black_box(public[i]);
        
        result[i] = secret_byte ^ public_byte;
        
        // Prevent compiler from reordering operations
        black_box(&result[i]);
    }
    
    result
}

// Memory access patterns that resist cache timing attacks
fn cache_resistant_lookup(table: &[u8; 256], index: u8) -> u8 {
    let mut result = 0u8;
    
    // Access every table entry to maintain consistent cache behavior
    for (i, &value) in table.iter().enumerate() {
        let mask = ((i as u8 == index) as u8).wrapping_neg();
        result |= value & mask;
        
        // Ensure memory access happens
        black_box(value);
    }
    
    result
}

#[entry]
fn main() -> ! {
    let secret = [0x42u8; 32];
    let public = [0xAAu8; 32];
    
    // Protected XOR operation
    let result = protected_constant_time_operation(&secret, &public);
    
    // Cache-resistant table lookup
    let table = [0u8; 256];
    let value = cache_resistant_lookup(&table, 42);
    
    loop {}
}
```

### Hardware vs Software Constant-Time

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;

// Stub types for demonstration
type KeySchedule = [[u8; 16]; 11];

fn aes_key_expansion_ct(key: &[u8; 32]) -> KeySchedule {
    [[0u8; 16]; 11] // Stub implementation
}

fn aes_round_ct(state: &mut [u8; 16], round_key: &[u8; 16]) {
    // Stub implementation
    for (s, &k) in state.iter_mut().zip(round_key.iter()) {
        *s ^= k;
    }
}

// Constant-time operations with hardware considerations
#[cfg(feature = "hw_crypto")]
fn hardware_constant_time_aes(key: &[u8; 32], plaintext: &[u8; 16]) -> [u8; 16] {
    // Use hardware AES if available - inherently constant-time
    extern "C" {
        fn hardware_aes_encrypt(key: &[u8; 32], plaintext: &[u8; 16], output: &mut [u8; 16]);
    }
    
    let mut ciphertext = [0u8; 16];
    unsafe {
        hardware_aes_encrypt(key, plaintext, &mut ciphertext);
    }
    ciphertext
}

#[cfg(not(feature = "hw_crypto"))]
fn software_constant_time_aes(key: &[u8; 32], plaintext: &[u8; 16]) -> [u8; 16] {
    // Software implementation with constant-time guarantees
    let mut state = *plaintext;
    let key_schedule = aes_key_expansion_ct(key);
    
    // All operations must be constant-time
    for round_key in key_schedule.iter() {
        aes_round_ct(&mut state, round_key);
    }
    
    state
}

#[entry]
fn main() -> ! {
    let key = [0u8; 32];
    let plaintext = [0u8; 16];
    
    #[cfg(feature = "hw_crypto")]
    let ciphertext = hardware_constant_time_aes(&key, &plaintext);
    
    #[cfg(not(feature = "hw_crypto"))]
    let ciphertext = software_constant_time_aes(&key, &plaintext);
    
    loop {}
}
```

### Timing Validation in Debug Mode

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::entry;
use heapless::Vec;
use heapless::consts::U1000;

// Timing measurement for validation (debug builds only)
#[cfg(debug_assertions)]
fn validate_constant_time_property<F>(operation: F, iterations: usize) 
where 
    F: Fn() -> ()
{
    // Note: This requires cortex-m with DWT support
    // For Cortex-R5, use PMU cycle counter instead
    
    let mut timings = Vec::<u32, U1000>::new();
    
    for _ in 0..iterations {
        // For Cortex-R5, use performance monitoring unit
        let start = read_cycle_counter();
        operation();
        let end = read_cycle_counter();
        
        let _ = timings.push(end.wrapping_sub(start));
    }
    
    // Analyze timing variance
    let mean = timings.iter().sum::<u32>() / timings.len() as u32;
    let variance = timings.iter()
        .map(|&t| {
            let diff = (t as i64) - (mean as i64);
            diff * diff
        })
        .sum::<i64>() / timings.len() as i64;
    
    // Low variance indicates constant-time behavior
    assert!(variance < 100, "High timing variance detected: {}", variance);
}

// Stub function for cycle counter
fn read_cycle_counter() -> u32 {
    // In real implementation, read from PMU CCNT register
    0
}

#[entry]
fn main() -> ! {
    #[cfg(debug_assertions)]
    {
        // Validate constant-time property of an operation
        validate_constant_time_property(|| {
            let a = [0u8; 32];
            let b = [0u8; 32];
            let _ = constant_time_compare(&a, &b);
        }, 1000);
    }
    
    loop {}
}

fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    
    result == 0
}
```