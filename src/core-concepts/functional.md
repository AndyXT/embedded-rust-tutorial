# 3.5 Functional Programming and Data Processing {functional-programming-and-data-processing}

Rust's functional programming features provide powerful abstractions for data processing while maintaining zero-cost performance. For embedded cryptography engineers, these features enable more expressive and safer mathematical operations, efficient data transformations, and flexible algorithm customization within no_std constraints.

<details>
<summary><strong>▶️ Section Overview</strong> - What you'll learn</summary>

This section covers:
- **Mathematical operations** - Overflow protection and checked arithmetic for crypto safety
- **Iterator patterns** - Zero-cost data processing abstractions in embedded contexts
- **Closures** - Algorithm customization and callback patterns
- **Functional programming in no_std** - Applying functional concepts within embedded constraints

**Prerequisites:** Understanding of basic Rust syntax, ownership, and embedded constraints

**Related Sections:**
- → [Constant-Time Implementations](../cryptography/constant-time.md) - Apply functional patterns for timing-safe crypto
- → [Secure Coding Patterns](../cryptography/secure-patterns.md) - Use iterators and closures for safer crypto code
- → [Advanced Type System Features](./advanced-types.md) - Combine with enums and traits for powerful abstractions

</details>

## Mathematical Operations with Safety

Rust provides explicit control over mathematical operations, preventing the silent overflow bugs that plague C crypto implementations.

<details>
<summary><strong>▶️ C vs Rust Mathematical Operations</strong> - See the safety improvements</summary>

**C Approach - Silent Overflow:**
```c
// C arithmetic - silent overflow can break crypto
uint32_t crypto_multiply(uint32_t a, uint32_t b) {
    return a * b;  // Silent overflow! Result wraps around
}

// Vulnerable key derivation
uint64_t derive_key_schedule(uint32_t base_key, uint32_t round) {
    uint64_t expanded = base_key * round * 0x9E3779B9;  // May overflow
    return expanded ^ (expanded >> 32);
}

// Timing-critical operations with potential overflow
void process_crypto_block(uint8_t* data, size_t len, uint32_t multiplier) {
    for (size_t i = 0; i < len; i++) {
        // BUG: i * multiplier may overflow, causing buffer overrun
        uint32_t offset = i * multiplier;
        if (offset < len) {  // Check may be bypassed by overflow
            data[offset] ^= 0xAA;
        }
    }
}
```

**Rust Approach - Explicit Overflow Handling:**



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

// Rust arithmetic - explicit overflow control
fn crypto_multiply(a: u32, b: u32) -> Result<u32, CryptoError> {
    a.checked_mul(b).ok_or(CryptoError::ArithmeticOverflow)
}

// Safe key derivation with overflow detection
fn derive_key_schedule(base_key: u32, round: u32) -> Result<u64, CryptoError> {
    let multiplied = base_key
        .checked_mul(round)
        .ok_or(CryptoError::ArithmeticOverflow)?;
    
    let expanded = (multiplied as u64)
        .checked_mul(0x9E3779B9)
        .ok_or(CryptoError::ArithmeticOverflow)?;
    
    Ok(expanded ^ (expanded >> 32))
}

// Safe block processing with bounds checking
fn process_crypto_block(data: &mut [u8], multiplier: u32) -> Result<(), CryptoError> {
    for (i, byte) in data.iter_mut().enumerate() {
        let offset = (i as u32)
            .checked_mul(multiplier)
            .ok_or(CryptoError::ArithmeticOverflow)?;
        
        if (offset as usize) < data.len() {
            *byte ^= 0xAA;
        }
    }
    Ok(())
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

</details>

**Crypto-Safe Arithmetic Operations:**

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use subtle::{Choice, ConstantTimeEq, ConditionallySelectable};

#[derive(Debug)]
pub struct CryptoError(&'static str);


use subtle::{Choice, ConstantTimeEq};
use core::mem;
use core::fmt;

use core::result::Result;

use core::num::Wrapping;

// Different arithmetic modes for different crypto needs
#[derive(Debug, Clone, Copy)]
pub enum ArithmeticMode {
    Checked,    // Panic on overflow (debug builds)
    Saturating, // Clamp to max/min values
    Wrapping,   // Explicit wrapping behavior
}

// Generic crypto arithmetic with explicit overflow behavior
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
                Ok(Wrapping(a).0.wrapping_add(b))
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
                Ok(Wrapping(a).0.wrapping_mul(b))
            },
        }
    }
    
    // Crypto-specific operations
    pub fn rotate_left(&self, value: u32, n: u32) -> u32 {
        value.rotate_left(n)
    }
    
    pub fn rotate_right(&self, value: u32, n: u32) -> u32 {
        value.rotate_right(n)
    }
    
    // Constant-time operations for crypto
    pub fn constant_time_select(&self, condition: bool, a: u32, b: u32) -> u32 {
        // Use subtle crate for constant-time selection
        use subtle::{Choice, ConditionallySelectable};
        let choice = Choice::from(condition as u8);
        u32::conditional_select(&b, &a, choice)
    }
}

// Example: AES S-box calculation with safe arithmetic
fn aes_sbox_transform(input: u8, math: &CryptoMath<u32>) -> Result<u8, CryptoError> {
    let x = input as u32;
    
    // Galois field operations with overflow protection
    let squared = math.mul(x, x)?;
    let fourth = math.mul(squared, squared)?;
    let result = math.add(fourth, x)?;
    
    Ok((result & 0xFF) as u8)
}

// Modular arithmetic for crypto operations
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
```

## Iterator Patterns for Zero-Cost Data Processing

Rust's iterator system provides zero-cost abstractions that compile to the same assembly as hand-written loops, making them perfect for embedded crypto operations.

<details>
<summary><strong>▶️ C Loops vs Rust Iterators</strong> - See the performance and safety benefits</summary>

**C Approach - Manual Loop Management:**
```c
// C approach - manual indexing, potential for errors
void xor_buffers(uint8_t* dst, const uint8_t* src1, const uint8_t* src2, size_t len) {
    for (size_t i = 0; i < len; i++) {
        dst[i] = src1[i] ^ src2[i];  // Potential buffer overrun
    }
}

// Complex data processing with multiple loops
void process_crypto_data(uint8_t* data, size_t len, const uint8_t* key, size_t key_len) {
    // First pass: XOR with key
    for (size_t i = 0; i < len; i++) {
        data[i] ^= key[i % key_len];
    }
    
    // Second pass: byte substitution
    for (size_t i = 0; i < len; i++) {
        data[i] = sbox[data[i]];
    }
    
    // Third pass: permutation
    uint8_t temp[256];  // Fixed size buffer - potential overflow
    for (size_t i = 0; i < len && i < 256; i++) {
        temp[i] = data[permutation_table[i]];
    }
    memcpy(data, temp, len);
}
```

**Rust Approach - Iterator Chains:**

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;

#[derive(Debug)]
pub struct CryptoError(&'static str);


use core::mem;
use core::fmt;

use core::result::Result;

// Rust iterators - bounds-safe, zero-cost abstractions
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

// Complex data processing with iterator chains
fn process_crypto_data(
    data: &mut [u8], 
    key: &[u8], 
    sbox: &[u8; 256],
    permutation_table: &[usize]
) -> Result<(), CryptoError> {
    if permutation_table.len() != data.len() {
        return Err(CryptoError::InvalidPermutationTable);
    }
    
    // All operations in a single iterator chain - compiler optimizes to single loop
    let processed: Result<heapless::Vec<u8, 256>, _> = data
        .iter()
        .enumerate()
        .map(|(i, &byte)| {
            // XOR with key (cycling)
            let keyed = byte ^ key[i % key.len()];
            
            // S-box substitution
            let substituted = sbox[keyed as usize];
            
            // Permutation
            let perm_index = permutation_table[i];
            if perm_index >= data.len() {
                return Err(CryptoError::InvalidPermutationIndex);
            }
            
            Ok(substituted)
        })
        .collect();
    
    let processed = processed?;
    data[..processed.len()].copy_from_slice(&processed);
    Ok(())
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

</details>

**Advanced Iterator Patterns for Crypto:**

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;

#[derive(Debug)]
pub struct CryptoError(&'static str);


use zeroize::{Zeroize, ZeroizeOnDrop};
use core::mem;
use core::fmt;

use core::result::Result;
use heapless::Vec;


// Custom iterator for crypto block processing
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

// Parallel processing simulation for crypto operations
fn parallel_block_encrypt(
    data: &mut [u8], 
    key: &[u8; 32], 
    block_size: usize
) -> Result<(), CryptoError> {
    // Process blocks using iterator - compiler can optimize for vectorization
    data.chunks_exact_mut(block_size)
        .try_for_each(|block| -> Result<(), CryptoError> {
            encrypt_block_aes(block, key)
        })?;
    
    // Handle remaining bytes if any
    let remainder_start = (data.len() / block_size) * block_size;
    if remainder_start < data.len() {
        let remainder = &mut data[remainder_start..];
        encrypt_block_aes_partial(remainder, key)?;
    }
    
    Ok(())
}

// Functional data transformation pipeline
fn crypto_pipeline(input: &[u8]) -> Result<heapless::Vec<u8, 1024, 32>, CryptoError> {
    input
        .iter()
        .copied()
        // Step 1: Add entropy
        .map(|byte| byte.wrapping_add(0x5A))
        // Step 2: Bit rotation
        .map(|byte| byte.rotate_left(3))
        // Step 3: XOR with pattern
        .enumerate()
        .map(|(i, byte)| byte ^ ((i as u8).wrapping_mul(0x33)))
        // Step 4: S-box substitution
        .map(|byte| AES_SBOX[byte as usize])
        // Step 5: Collect with bounds checking
        .collect::<Result<heapless::Vec<u8, 1024, 32>, _>>()
        .map_err(|_| CryptoError::BufferTooSmall)
}

// Streaming crypto operations with iterators
struct StreamCipher<'a> {
    key_stream: core::iter::Cycle<core::slice::Iter<'a, u8>>,
}

impl<'a> StreamCipher<'a> {
    fn new(key: &'a [u8]) -> Self {
        Self {
            key_stream: key.iter().cycle(),
        }
    }
    
    fn encrypt(&mut self, data: &mut [u8]) {
        data.iter_mut()
            .zip(&mut self.key_stream)
            .for_each(|(data_byte, &key_byte)| {
                *data_byte ^= key_byte;
            });
    }
}

// Iterator adapters for crypto-specific operations
// These patterns complement the secure coding techniques in 
// → [Constant-Time Implementations](../cryptography/constant-time.md)
trait CryptoIteratorExt: Iterator {
    // Constant-time processing to prevent timing attacks
    fn constant_time_map<F, B>(self, f: F) -> ConstantTimeMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B;
    
    // Secure aggregation that clears intermediate values
    fn secure_fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
        B: zeroize::Zeroize;
}

impl<I: Iterator> CryptoIteratorExt for I {
    fn constant_time_map<F, B>(self, f: F) -> ConstantTimeMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B,
    {
        ConstantTimeMap { iter: self, f }
    }
    
    fn secure_fold<B, F>(self, mut init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
        B: zeroize::Zeroize,
    {
        for item in self {
            init = f(init, item);
        }
        init
    }
}

pub struct ConstantTimeMap<I, F> {
    iter: I,
    f: F,
}

impl<I, F, B> Iterator for ConstantTimeMap<I, F>
where
    I: Iterator,
    F: FnMut(I::Item) -> B,
{
    type Item = B;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(&mut self.f)
    }
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

## Closures for Algorithm Customization

Closures provide a safe alternative to C function pointers, enabling flexible algorithm customization while maintaining zero-cost abstractions.

<details>
<summary><strong>▶️ C Function Pointers vs Rust Closures</strong> - See the safety and flexibility improvements</summary>

**C Approach - Function Pointers:**
```c
// C function pointers - type unsafe, potential for errors
typedef uint8_t (*transform_fn_t)(uint8_t input, void* context);
typedef int (*compare_fn_t)(const void* a, const void* b);

// Vulnerable callback system
struct crypto_processor {
    transform_fn_t transform;
    void* transform_context;
    compare_fn_t compare;
};

// Easy to misuse - wrong function signatures, null pointers
int process_with_callback(uint8_t* data, size_t len, struct crypto_processor* proc) {
    if (!proc || !proc->transform) {
        return -1;  // Runtime error detection
    }
    
    for (size_t i = 0; i < len; i++) {
        // BUG: What if transform_context is wrong type?
        data[i] = proc->transform(data[i], proc->transform_context);
    }
    
    // BUG: What if compare function expects different data types?
    qsort(data, len, sizeof(uint8_t), proc->compare);
    return 0;
}
```

**Rust Approach - Type-Safe Closures:**

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

// Rust closures - type safe, zero-cost abstractions
fn process_with_closure<F, C>(
    data: &mut [u8], 
    mut transform: F,
    mut compare: C
) -> Result<(), CryptoError>
where
    F: FnMut(u8) -> u8,
    C: FnMut(&u8, &u8) -> core::cmp::Ordering,
{
    // Transform data - compiler knows exact types
    for byte in data.iter_mut() {
        *byte = transform(*byte);
    }
    
    // Sort with custom comparison - type safe
    data.sort_by(|a, b| compare(a, b));
    
    Ok(())
}

// Usage with closures capturing context safely
fn example_usage() -> Result<(), CryptoError> {
    let mut data = [1u8, 3, 2, 5, 4];
    let key = 0x42u8;
    let reverse_order = true;
    
    process_with_closure(
        &mut data,
        |byte| byte ^ key,  // Closure captures key safely
        |a, b| {            // Closure captures reverse_order safely
            if reverse_order {
                b.cmp(a)
            } else {
                a.cmp(b)
            }
        }
    )?;
    
    Ok(())
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

</details>

**Advanced Closure Patterns for Crypto:**

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


use zeroize::{Zeroize, ZeroizeOnDrop};

// Secure closure context that auto-zeroizes
#[derive(ZeroizeOnDrop)]
struct SecureCryptoContext {
    key: [u8; 32],
    nonce: [u8; 12],
    counter: u64,
}

impl SecureCryptoContext {
    fn new(key: [u8; 32], nonce: [u8; 12]) -> Self {
        Self {
            key,
            nonce,
            counter: 0,
        }
    }
    
    // Create closure that captures secure context
    fn create_encryptor(&mut self) -> impl FnMut(&mut [u8]) -> Result<(), CryptoError> + '_ {
        move |data: &mut [u8]| -> Result<(), CryptoError> {
            // Increment counter for each encryption
            self.counter = self.counter
                .checked_add(1)
                .ok_or(CryptoError::CounterOverflow)?;
            
            // Create unique nonce for this operation
            let mut full_nonce = [0u8; 16];
            full_nonce[..12].copy_from_slice(&self.nonce);
            full_nonce[12..].copy_from_slice(&self.counter.to_le_bytes()[..4]);
            
            // Perform encryption with captured context
            chacha20_encrypt(&self.key, &full_nonce, data)
        }
    }
}

// Higher-order functions for crypto algorithm composition
fn compose_crypto_operations<F1, F2, E>(
    first: F1,
    second: F2,
) -> impl FnMut(&mut [u8]) -> Result<(), E>
where
    F1: FnMut(&mut [u8]) -> Result<(), E>,
    F2: FnMut(&mut [u8]) -> Result<(), E>,
{
    let mut first = first;
    let mut second = second;
    
    move |data: &mut [u8]| -> Result<(), E> {
        first(data)?;
        second(data)?;
        Ok(())
    }
}

// Crypto operation builder using closures
struct CryptoOperationBuilder<F> {
    operation: F,
}

impl CryptoOperationBuilder<()> {
    fn new() -> Self {
        Self { operation: () }
    }
}

impl<F> CryptoOperationBuilder<F>
where
    F: FnMut(&mut [u8]) -> Result<(), CryptoError>,
{
    fn then<G>(self, next_op: G) -> CryptoOperationBuilder<impl FnMut(&mut [u8]) -> Result<(), CryptoError>>
    where
        G: FnMut(&mut [u8]) -> Result<(), CryptoError>,
    {
        let mut prev_op = self.operation;
        let mut next_op = next_op;
        
        CryptoOperationBuilder {
            operation: move |data: &mut [u8]| -> Result<(), CryptoError> {
                prev_op(data)?;
                next_op(data)?;
                Ok(())
            }
        }
    }
    
    fn execute(mut self, data: &mut [u8]) -> Result<(), CryptoError> {
        (self.operation)(data)
    }
}

// Example: Building complex crypto pipeline with closures
fn build_crypto_pipeline() -> Result<(), CryptoError> {
    let mut data = [1u8, 2, 3, 4, 5, 6, 7, 8];
    let key = [0x42u8; 32];
    let sbox = &AES_SBOX;
    
    // Build operation pipeline using closures
    let result = CryptoOperationBuilder::new()
        .then(|data| {
            // XOR with key
            for (i, byte) in data.iter_mut().enumerate() {
                *byte ^= key[i % key.len()];
            }
            Ok(())
        })
        .then(|data| {
            // S-box substitution
            for byte in data.iter_mut() {
                *byte = sbox[*byte as usize];
            }
            Ok(())
        })
        .then(|data| {
            // Bit rotation
            for byte in data.iter_mut() {
                *byte = byte.rotate_left(3);
            }
            Ok(())
        })
        .execute(&mut data);
    
    result
}

// Callback-based event system for crypto operations
trait CryptoEventHandler {
    fn on_key_generated(&mut self, key_id: u32, key_type: KeyType);
    fn on_encryption_complete(&mut self, operation_id: u64, result: Result<(), CryptoError>);
    fn on_error(&mut self, error: CryptoError);
}

// Generic crypto processor with closure-based callbacks
struct CryptoProcessor<H>
where
    H: CryptoEventHandler,
{
    handler: H,
    operation_counter: u64,
}

impl<H> CryptoProcessor<H>
where
    H: CryptoEventHandler,
{
    fn new(handler: H) -> Self {
        Self {
            handler,
            operation_counter: 0,
        }
    }
    
    fn encrypt_with_callback<F>(
        &mut self,
        data: &mut [u8],
        encrypt_fn: F,
    ) -> Result<(), CryptoError>
    where
        F: FnOnce(&mut [u8]) -> Result<(), CryptoError>,
    {
        self.operation_counter += 1;
        let operation_id = self.operation_counter;
        
        let result = encrypt_fn(data);
        
        // Notify handler of completion
        self.handler.on_encryption_complete(operation_id, result.clone());
        
        if let Err(ref error) = result {
            self.handler.on_error(error.clone());
        }
        
        result
    }
}
```

## Functional Programming in No-std Environments

Rust's functional programming features work seamlessly in no_std embedded environments, providing powerful abstractions without heap allocation.

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;

#[derive(Debug)]
pub struct CryptoError(&'static str);


use core::mem;
use heapless::Vec;

use core::fmt;
use core::result::Result;

use heapless::{Vec, FnvIndexMap};
use core::iter;

// Functional data structures for embedded crypto
#[derive(Clone)]
struct ImmutableCryptoState<const N: usize> {
    keys: [u8; N],
    counters: [u64; 8],
    flags: u32,
}

impl<const N: usize> ImmutableCryptoState<N> {
    fn new(keys: [u8; N]) -> Self {
        Self {
            keys,
            counters: [0; 8],
            flags: 0,
        }
    }
    
    // Functional update - returns new state without modifying original
    fn with_counter(mut self, index: usize, value: u64) -> Result<Self, CryptoError> {
        if index >= self.counters.len() {
            return Err(CryptoError::InvalidCounterIndex);
        }
        self.counters[index] = value;
        Ok(self)
    }
    
    fn with_flag(mut self, flag: u32) -> Self {
        self.flags |= flag;
        self
    }
    
    // Functional transformation
    fn transform_keys<F>(mut self, f: F) -> Self
    where
        F: Fn(u8) -> u8,
    {
        for key_byte in &mut self.keys {
            *key_byte = f(*key_byte);
        }
        self
    }
}

// Monadic error handling for crypto operations
#[derive(Debug, Clone)]
enum CryptoResult<T> {
    Success(T),
    Warning(T, &'static str),
    Error(CryptoError),
}

impl<T> CryptoResult<T> {
    fn map<U, F>(self, f: F) -> CryptoResult<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            CryptoResult::Success(value) => CryptoResult::Success(f(value)),
            CryptoResult::Warning(value, msg) => CryptoResult::Warning(f(value), msg),
            CryptoResult::Error(err) => CryptoResult::Error(err),
        }
    }
    
    fn and_then<U, F>(self, f: F) -> CryptoResult<U>
    where
        F: FnOnce(T) -> CryptoResult<U>,
    {
        match self {
            CryptoResult::Success(value) => f(value),
            CryptoResult::Warning(value, msg) => {
                match f(value) {
                    CryptoResult::Success(new_value) => CryptoResult::Warning(new_value, msg),
                    CryptoResult::Warning(new_value, _) => CryptoResult::Warning(new_value, msg),
                    CryptoResult::Error(err) => CryptoResult::Error(err),
                }
            },
            CryptoResult::Error(err) => CryptoResult::Error(err),
        }
    }
}

// Functional crypto pipeline with monadic composition
fn functional_crypto_pipeline(input: &[u8]) -> CryptoResult<heapless::Vec<u8, 256, 32>> {
    let initial_state = ImmutableCryptoState::<32>::new([0x42; 32]);
    
    // Functional composition of crypto operations
    let result = CryptoResult::Success(input.to_vec())
        .and_then(|data| {
            // Validate input
            if data.len() > 256 {
                CryptoResult::Error(CryptoError::DataTooLarge)
            } else if data.is_empty() {
                CryptoResult::Warning(data, "Empty input data")
            } else {
                CryptoResult::Success(data)
            }
        })
        .map(|mut data| {
            // Transform data functionally
            for byte in &mut data {
                *byte = byte.wrapping_add(0x33);
            }
            data
        })
        .and_then(|data| {
            // Apply crypto transformation
            let transformed_state = initial_state
                .with_counter(0, data.len() as u64)
                .map_err(|e| CryptoError::InvalidState)?
                .with_flag(0x01)
                .transform_keys(|k| k.rotate_left(1));
            
            let mut result = Vec::new();
            for (i, &byte) in data.iter().enumerate() {
                let key_byte = transformed_state.keys[i % transformed_state.keys.len()];
                result.push(byte ^ key_byte).map_err(|_| CryptoError::BufferTooSmall)?;
            }
            
            CryptoResult::Success(result)
        });
    
    result
}

// Lazy evaluation for crypto computations
struct LazyCryptoComputation<F> {
    computation: Option<F>,
}

impl<F, T> LazyCryptoComputation<F>
where
    F: FnOnce() -> T,
{
    fn new(computation: F) -> Self {
        Self {
            computation: Some(computation),
        }
    }
    
    fn evaluate(mut self) -> T {
        (self.computation.take().unwrap())()
    }
}

// Example: Lazy key derivation
fn create_lazy_key_derivation(master_key: [u8; 32], salt: [u8; 16]) -> LazyCryptoComputation<impl FnOnce() -> [u8; 32]> {
    LazyCryptoComputation::new(move || {
        // Expensive key derivation only computed when needed
        let mut derived_key = [0u8; 32];
        for i in 0..32 {
            derived_key[i] = master_key[i] ^ salt[i % 16] ^ (i as u8);
        }
        derived_key
    })
}
```

**→ Next:** [Memory Model Differences](../core-concepts/memory-model.md) - Understanding Rust's memory model for embedded systems

**Related Crypto Applications:**
- → [Constant-Time Implementations](../cryptography/constant-time.md) - Apply functional patterns for timing-safe operations
- → [Key Management and Zeroization](../cryptography/key-management.md) - Use iterators for secure data processing
```