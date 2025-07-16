# 1.7 Critical Differences and Gotchas

#### ⚠️ Memory Management Gotchas

| Issue | C Behavior | Rust Behavior | Solution |
|-------|------------|---------------|----------|
| **Use after free** | Undefined behavior, potential exploit | Compile error | Ownership system prevents |
| **Double free** | Undefined behavior, crash | Compile error | Automatic memory management |
| **Buffer overflow** | Silent corruption or crash | Panic in debug, bounds checking | Use slices, not raw pointers |
| **Null pointer dereference** | Segmentation fault | Compile error | Use `Option<T>` instead of nullable pointers |
| **Data races** | Undefined behavior | Compile error | Ownership prevents shared mutable access |
| **Memory leaks** | Silent resource exhaustion | Compile-time prevention | RAII and Drop trait |
| **Dangling pointers** | Undefined behavior | Compile error | Lifetime system prevents |

**⚠️ Critical Example - Use After Free Prevention:**

```rust
// C code that compiles but has use-after-free bug:
// uint8_t* create_key() {
//     uint8_t key[32] = {0};
//     return key;  // Returns pointer to stack memory!
// }

// Rust equivalent - won't compile:
fn create_key() -> &[u8; 32] {
    let key = [0u8; 32];
    &key  // ERROR: borrowed value does not live long enough
}

// Correct Rust approach:
fn create_key() -> [u8; 32] {
    [0u8; 32]  // Return owned value, not reference
}

// Or use heap allocation:
fn create_key_heap() -> Box<[u8; 32]> {
    Box::new([0u8; 32])  // Heap-allocated, properly owned
}
```

#### ⚠️ Crypto-Specific Gotchas

| Issue | C Risk | Rust Protection | Implementation |
|-------|--------|-----------------|----------------|
| **Key material not cleared** | Memory disclosure | Automatic via `Drop` | Use `ZeroizeOnDrop` trait |
| **Timing attacks** | Manual constant-time coding | Library support | Use `subtle` crate |
| **Side-channel leaks** | Compiler optimizations | Explicit control | Use `volatile` operations |
| **Integer overflow** | Silent wraparound | Panic or explicit | Use checked arithmetic |
| **Uninitialized memory** | Information disclosure | Compile error | All memory must be initialized |
| **Key reuse** | Manual tracking required | Type system enforcement | Use session-specific key types |
| **Nonce reuse** | Catastrophic failure | Compile-time prevention | Use linear types for nonces |

**⚠️ Critical Example - Automatic Key Zeroization:**

```rust
// C code - easy to forget zeroization:
// void process_message(uint8_t* key) {
//     uint8_t session_key[32];
//     derive_key(key, session_key);
//     encrypt_message(session_key, message);
//     // BUG: Forgot to clear session_key!
//     return;
// }

// Rust equivalent - automatic zeroization:
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
struct SessionKey([u8; 32]);

fn process_message(master_key: &[u8; 32]) {
    let session_key = SessionKey(derive_key(master_key));
    encrypt_message(&session_key.0, message);
    // session_key automatically zeroized when dropped
}
```

**⚠️ Critical Example - Constant-Time Operations:**

```rust
// C code - vulnerable to timing attacks:
// int verify_mac(uint8_t* expected, uint8_t* actual, size_t len) {
//     for (size_t i = 0; i < len; i++) {
//         if (expected[i] != actual[i]) {
//             return 0;  // Early return leaks timing info!
//         }
//     }
//     return 1;
// }

// Rust equivalent - constant-time comparison:
use subtle::ConstantTimeEq;

fn verify_mac(expected: &[u8], actual: &[u8]) -> bool {
    expected.ct_eq(actual).into()  // Always takes same time
}
```

#### ⚠️ Embedded-Specific Gotchas

| Issue | C Behavior | Rust Behavior | Workaround |
|-------|------------|---------------|------------|
| **Stack overflow** | Silent corruption | Stack overflow detection | Use `#[no_mangle]` for stack checking |
| **Interrupt safety** | Manual critical sections | Compile-time checking | Use `cortex_m::interrupt::Mutex` |
| **Hardware register access** | Direct pointer access | Requires `unsafe` | Use PAC/HAL crates |
| **Linker script integration** | Manual memory layout | Explicit configuration | Use `memory.x` and `build.rs` |
| **Real-time constraints** | Manual timing analysis | No built-in support | Use RTIC framework |
| **Global mutable state** | Direct access | Requires `unsafe` or synchronization | Use `Mutex` or `RefCell` |
| **Interrupt handlers** | Function pointers | Attribute-based | Use `#[interrupt]` attribute |

**⚠️ Critical Example - Interrupt-Safe Global State:**

```rust
// C code - race condition possible:
// volatile uint32_t crypto_status = 0;
// 
// void main_loop() {
//     crypto_status = PROCESSING;  // Race with interrupt!
//     start_crypto_operation();
// }
// 
// void CRYPTO_IRQ() {
//     crypto_status = COMPLETE;    // Race with main loop!
// }

// Rust equivalent - compile-time safety:
use cortex_m::interrupt::Mutex;
use core::cell::RefCell;

static CRYPTO_STATUS: Mutex<RefCell<CryptoStatus>> = 
    Mutex::new(RefCell::new(CryptoStatus::Idle));

#[derive(Clone, Copy)]
enum CryptoStatus {
    Idle,
    Processing,
    Complete,
}

fn main_loop() {
    cortex_m::interrupt::free(|cs| {
        *CRYPTO_STATUS.borrow(cs).borrow_mut() = CryptoStatus::Processing;
    });
    start_crypto_operation();
}

#[interrupt]
fn CRYPTO_IRQ() {
    cortex_m::interrupt::free(|cs| {
        *CRYPTO_STATUS.borrow(cs).borrow_mut() = CryptoStatus::Complete;
    });
}
```

**⚠️ Critical Example - Safe Hardware Register Access:**

```rust
// C code - direct register access:
// #define CRYPTO_BASE 0x40000000
// #define CRYPTO_CTRL (*(volatile uint32_t*)(CRYPTO_BASE + 0x00))
// #define CRYPTO_DATA (*(volatile uint32_t*)(CRYPTO_BASE + 0x04))
// 
// void start_encryption() {
//     CRYPTO_CTRL = 0x01;  // Enable
//     CRYPTO_DATA = key_data;  // Potential race conditions
// }

// Rust equivalent - type-safe register access:
use cortex_m::peripheral::Peripherals;

// Using PAC (Peripheral Access Crate)
fn start_encryption(crypto: &mut CRYPTO) {
    crypto.ctrl.write(|w| w.enable().set_bit());
    crypto.data.write(|w| unsafe { w.bits(key_data) });
}

// Or using raw pointers with explicit unsafe:
fn start_encryption_raw() {
    const CRYPTO_BASE: *mut u32 = 0x4000_0000 as *mut u32;
    
    unsafe {
        // Explicit unsafe block makes risks visible
        core::ptr::write_volatile(CRYPTO_BASE, 0x01);
        core::ptr::write_volatile(CRYPTO_BASE.offset(1), key_data);
    }
}
```

#### ⚠️ Type System and Functional Programming Gotchas

| Issue | C Approach | Rust Advantage | Implementation |
|-------|------------|----------------|----------------|
| **Enum safety** | Switch statements can miss cases | Exhaustive pattern matching | Compiler enforces all cases handled |
| **Function pointer safety** | Null function pointers possible | Traits guarantee implementation | Use trait bounds for safety |
| **Data with enums** | Separate enum + union/struct | Enums carry typed data | `enum Result<T, E> { Ok(T), Err(E) }` |
| **Iterator performance** | Manual loop optimization | Zero-cost abstractions | Iterators compile to same code as loops |
| **Closure capture** | Manual context passing | Automatic environment capture | Closures capture by value/reference automatically |
| **Method organization** | Global function namespace | Associated functions and methods | Methods grouped with types |
| **Mathematical safety** | Silent overflow/underflow | Explicit overflow handling | Use `checked_add()`, `saturating_mul()` |

**⚠️ Critical Example - Enum Pattern Matching Safety:**

```rust
// C code - easy to miss cases:
// enum crypto_state { IDLE, ENCRYPTING, DECRYPTING, ERROR };
// 
// void handle_state(enum crypto_state state) {
//     switch (state) {
//         case IDLE:
//             start_operation();
//             break;
//         case ENCRYPTING:
//             continue_encrypt();
//             break;
//         // BUG: Missing DECRYPTING and ERROR cases!
//     }
// }

// Rust equivalent - compiler enforces exhaustiveness:
#[derive(Debug)]
enum CryptoState {
    Idle,
    Encrypting { progress: u8 },
    Decrypting { bytes_left: usize },
    Error(CryptoError),
}

fn handle_state(state: CryptoState) {
    match state {
        CryptoState::Idle => start_operation(),
        CryptoState::Encrypting { progress } => {
            continue_encrypt(progress);
        }
        CryptoState::Decrypting { bytes_left } => {
            continue_decrypt(bytes_left);
        }
        CryptoState::Error(err) => {
            handle_crypto_error(err);
        }
        // Compiler error if any case is missing!
    }
}
```

**⚠️ Critical Example - Iterator Safety and Performance:**

```rust
// C code - manual bounds checking and potential errors:
// void process_crypto_data(uint8_t* data, size_t len) {
//     for (size_t i = 0; i < len; i++) {
//         if (i >= MAX_BUFFER) break;  // Manual bounds check
//         data[i] = transform_byte(data[i]);
//         // Risk: off-by-one errors, buffer overflows
//     }
// }

// Rust equivalent - automatic bounds checking, zero-cost:
fn process_crypto_data(data: &mut [u8]) {
    data.iter_mut()
        .take(MAX_BUFFER)  // Safe limiting
        .for_each(|byte| *byte = transform_byte(*byte));
    // Compiles to same assembly as manual loop, but safe
}

// Mathematical operations with overflow protection:
fn safe_crypto_math(a: u32, b: u32) -> Option<u32> {
    a.checked_mul(b)  // Returns None on overflow
     .and_then(|result| result.checked_add(42))
}
```

**⚠️ Critical Example - Trait-Based Polymorphism vs Function Pointers:**

```rust
// C code - unsafe function pointers:
// typedef int (*crypto_func)(uint8_t*, size_t);
// 
// struct crypto_engine {
//     crypto_func encrypt;  // Could be NULL!
//     crypto_func decrypt;
// };
// 
// int use_crypto(struct crypto_engine* engine, uint8_t* data, size_t len) {
//     if (engine->encrypt == NULL) return -1;  // Runtime check
//     return engine->encrypt(data, len);
// }

// Rust equivalent - compile-time safety:
trait CryptoEngine {
    fn encrypt(&self, data: &mut [u8]) -> Result<(), CryptoError>;
    fn decrypt(&self, data: &mut [u8]) -> Result<(), CryptoError>;
}

struct AesEngine { /* ... */ }
impl CryptoEngine for AesEngine {
    fn encrypt(&self, data: &mut [u8]) -> Result<(), CryptoError> {
        // Implementation guaranteed to exist
        aes_encrypt_impl(data)
    }
    
    fn decrypt(&self, data: &mut [u8]) -> Result<(), CryptoError> {
        aes_decrypt_impl(data)
    }
}

fn use_crypto<T: CryptoEngine>(engine: &T, data: &mut [u8]) -> Result<(), CryptoError> {
    engine.encrypt(data)  // No null check needed - guaranteed to exist
}
```

#### ⚠️ Common Migration Pitfalls

**1. String Handling Differences:**

```rust
// C approach:
// char buffer[256];
// strcpy(buffer, "Hello");
// strcat(buffer, " World");

// Rust approach - different string types:
let mut buffer = [0u8; 256];  // Byte array, not string
let hello = b"Hello";         // Byte string literal
let world = b" World";        // Byte string literal

// For actual strings:
use heapless::String;
let mut message: String<256> = String::new();
message.push_str("Hello").unwrap();
message.push_str(" World").unwrap();
```

**2. Array Initialization Differences:**

```rust
// C approach:
// uint8_t buffer[1024] = {0};  // Zero-initialized

// Rust approach:
let buffer = [0u8; 1024];     // Zero-initialized
// Or:
let mut buffer: [u8; 1024] = [0; 1024];

// For uninitialized arrays (unsafe):
let mut buffer: [u8; 1024] = unsafe { 
    core::mem::MaybeUninit::uninit().assume_init() 
};
// Must initialize before use!
```

**3. Function Pointer Differences:**

```rust
// C approach:
// typedef int (*crypto_func_t)(uint8_t*, size_t);
// crypto_func_t encrypt_fn = aes_encrypt;

// Rust approach:
type CryptoFn = fn(&mut [u8]) -> Result<(), CryptoError>;
let encrypt_fn: CryptoFn = aes_encrypt;

// Or using closures:
let encrypt_fn = |data: &mut [u8]| -> Result<(), CryptoError> {
    aes_encrypt(data)
};
```