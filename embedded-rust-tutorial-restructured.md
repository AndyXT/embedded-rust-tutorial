# Embedded Rust Tutorial for Cryptography Engineers

*A streamlined guide for experienced embedded cryptography C programmers transitioning to Rust*

---

## Table of Contents

### [1. Quick Reference](#quick-reference)
- [1.1 C to Rust Syntax Mapping](#c-to-rust-syntax-mapping)
- [1.2 Memory and Pointer Patterns](#memory-and-pointer-patterns)
- [1.3 Control Flow and Functions](#control-flow-and-functions)
- [1.4 Error Handling Patterns](#error-handling-patterns)
- [1.5 Crypto-Specific Quick Reference](#crypto-specific-quick-reference)
- [1.6 Embedded-Specific Patterns](#embedded-specific-patterns)
- [1.7 Critical Differences and Gotchas](#critical-differences-and-gotchas)

### [2. Environment Setup](#environment-setup)
- [2.1 Rust Installation and Toolchain](#rust-installation-and-toolchain)
- [2.2 Target Configuration](#target-configuration)
  - [2.2.1 Xilinx Ultrascale+ (Cortex-R5)](#xilinx-ultrascale-cortex-r5)
  - [2.2.2 ARM Cortex-M Series](#arm-cortex-m-series)
  - [2.2.3 Other Embedded Targets](#other-embedded-targets)
- [2.3 Project Structure and Dependencies](#project-structure-and-dependencies)
- [2.4 Build Configuration](#build-configuration)
- [2.5 Verification and Testing](#verification-and-testing)

### [3. Core Language Concepts](#core-language-concepts)
- [3.1 Ownership and Memory Management](#ownership-and-memory-management)
- [3.2 Error Handling Without Exceptions](#error-handling-without-exceptions)
- [3.3 Type System Advantages](#type-system-advantages)
- [3.4 Memory Model Differences](#memory-model-differences)
- [3.5 Safety Guarantees for Crypto](#safety-guarantees-for-crypto)

### [4. Embedded-Specific Patterns](#embedded-specific-patterns)
- [4.1 No-std Programming Essentials](#no-std-programming-essentials)
- [4.2 Hardware Abstraction Patterns](#hardware-abstraction-patterns)
- [4.3 Interrupt Handling](#interrupt-handling)
- [4.4 Static Memory Management](#static-memory-management)
- [4.5 DMA and Hardware Integration](#dma-and-hardware-integration)

### [5. Cryptography Implementation](#cryptography-implementation)
- [5.1 Secure Coding Patterns](#secure-coding-patterns)
- [5.2 Constant-Time Implementations](#constant-time-implementations)
- [5.3 Key Management and Zeroization](#key-management-and-zeroization)
- [5.4 Hardware Crypto Acceleration](#hardware-crypto-acceleration)
- [5.5 Side-Channel Mitigations](#side-channel-mitigations)

### [6. Migration and Integration](#migration-and-integration)
- [6.1 Incremental Migration Strategies](#incremental-migration-strategies)
- [6.2 FFI Integration with C Libraries](#ffi-integration-with-c-libraries)
- [6.3 Testing and Validation](#testing-and-validation)
- [6.4 Debugging and Tooling](#debugging-and-tooling)
- [6.5 Performance Considerations](#performance-considerations)

---

## Introduction

As an experienced embedded cryptography engineer, you understand the critical importance of correctness and security in cryptographic implementations. A single buffer overflow, timing leak, or use-after-free can compromise an entire system. Traditional C development requires constant vigilance against these threats while managing the complexities of embedded systems.

Rust addresses these challenges while maintaining the performance and control essential for cryptographic operations. This tutorial is specifically designed for experienced C programmers who need to become productive in Rust quickly, without wading through basic programming concepts.

**Why this tutorial is different:**
- **Assumes C expertise** - No basic programming explanations
- **Crypto-focused examples** - All examples relevant to cryptographic engineering
- **Quick reference first** - Immediate productivity through comprehensive lookup tables
- **Embedded-specific** - Covers no-std, hardware integration, and real-time constraints
- **Streamlined organization** - Eliminates redundancy, focuses on practical differences

**What you'll gain:**
- Memory safety without performance overhead
- Automatic key material zeroization
- Type-safe protocol state machines
- Compile-time prevention of common crypto vulnerabilities
- Clear boundaries between safe and unsafe code

---

## 1. Quick Reference {#quick-reference}

This section provides immediate lookup for common C patterns and their Rust equivalents. Keep this section handy as your primary reference during transition.

### 1.1 Comprehensive C-to-Rust Syntax Mapping {#comprehensive-c-to-rust-syntax-mapping}

#### Basic Declarations and Types

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `int x = 5;` | `let x = 5i32;` | Immutable by default, explicit type optional |
| `int x; x = 5;` | `let mut x: i32; x = 5;` | Must declare mutability explicitly |
| `const int MAX = 100;` | `const MAX: i32 = 100;` | Compile-time constants, type required |
| `#define SIZE 256` | `const SIZE: usize = 256;` | Type-safe constants instead of macros |
| `static int counter = 0;` | `static mut COUNTER: i32 = 0;` | Global mutable state requires `unsafe` |
| `typedef struct {...} my_t;` | `struct MyStruct {...}` | Rust naming conventions (PascalCase) |
| `enum state { IDLE, BUSY };` | `enum State { Idle, Busy }` | More powerful enums with data |
| `union data { int i; float f; };` | `union Data { i: i32, f: f32 }` | Requires `unsafe` to access |
| `char str[256];` | `let mut str = [0u8; 256];` | Use byte arrays for C-style strings |
| `char* str = "hello";` | `let str = "hello";` | String literals are `&str` |

#### Functions and Control Flow

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `int func(void)` | `fn func() -> i32` | Explicit return type |
| `void func(void)` | `fn func()` | Unit type `()` implied |
| `int* func()` | `fn func() -> Option<i32>` | Use `Option` instead of nullable pointers |
| `if (condition) { ... }` | `if condition { ... }` | No parentheses needed around condition |
| `for (int i = 0; i < n; i++)` | `for i in 0..n` | Iterator-based loops |
| `while (condition)` | `while condition` | Same syntax, no parentheses |
| `switch (value)` | `match value` | More powerful pattern matching |
| `do { ... } while (cond);` | `loop { ... if !cond { break; } }` | No direct equivalent, use loop + break |
| `goto label;` | *Not available* | Use structured control flow instead |

### 1.2 Memory and Pointer Patterns {#memory-and-pointer-patterns}

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `uint8_t buffer[256];` | `let mut buffer = [0u8; 256];` | Stack array, bounds checked |
| `uint8_t* ptr = malloc(size);` | `let mut vec = vec![0u8; size];` | Heap allocation (needs `alloc` feature) |
| `if (ptr != NULL)` | `if let Some(ref_val) = option` | No null pointers, use `Option<T>` |
| `ptr[i]` | `slice[i]` | Bounds checked at runtime |
| `memcpy(dst, src, len)` | `dst.copy_from_slice(src)` | Safe, bounds checked copy |
| `memset(ptr, 0, len)` | `slice.fill(0)` | Safe initialization |
| `free(ptr)` | *automatic* | Memory freed when owner goes out of scope |
| `ptr++` | `ptr = ptr.offset(1)` | Pointer arithmetic requires `unsafe` |

### 1.3 Control Flow and Functions {#control-flow-and-functions}

#### Error Handling Patterns

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `if (result < 0) return -1;` | `result.map_err(\|e\| MyError::from(e))?` | Use `Result<T, E>` type |
| `errno` | `Result<T, Error>` | Errors are values, not global state |
| `goto cleanup;` | Early return with `?` | Automatic error propagation |
| `assert(condition)` | `assert!(condition)` | Debug assertions |
| `abort()` | `panic!("message")` | Controlled program termination |

### 1.4 Error Handling Patterns {#error-handling-patterns}

#### Result Type Usage

```rust
// C style error handling
// int divide(int a, int b, int* result) {
//     if (b == 0) return -1;
//     *result = a / b;
//     return 0;
// }

// Rust equivalent
fn divide(a: i32, b: i32) -> Result<i32, &'static str> {
    if b == 0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}

// Usage with ? operator for error propagation
fn calculate() -> Result<i32, &'static str> {
    let result = divide(10, 2)?;  // Automatically propagates error
    Ok(result * 2)
}
```

### 1.5 Crypto-Specific Quick Reference {#crypto-specific-quick-reference}

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `uint8_t key[32]; /* clear on exit */` | `let key = SecureKey::<32>::new(data);` | Automatic zeroization via `Drop` |
| `if (memcmp(a, b, len) == 0)` | `constant_time_eq(a, b)` | Timing-attack resistant comparison |
| `volatile uint8_t* reg = 0x40000000;` | `unsafe { ptr::read_volatile(reg) }` | Hardware register access |
| `#ifdef CRYPTO_HW_ACCEL` | `#[cfg(feature = "hw-crypto")]` | Conditional compilation |
| `RAND_bytes(buffer, len)` | `rng.fill_bytes(&mut buffer)` | Cryptographically secure RNG |
| `explicit_bzero(key, sizeof(key))` | `key.zeroize()` | Secure memory clearing |

#### Secure Key Management Example

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
struct SecureKey<const N: usize> {
    key: [u8; N],
}

impl<const N: usize> SecureKey<N> {
    fn new(key_material: [u8; N]) -> Self {
        Self { key: key_material }
    }
    
    fn as_bytes(&self) -> &[u8; N] {
        &self.key
    }
}

// Automatically zeroized when dropped
let key = SecureKey::<32>::new(key_data);
// Use key...
// key is automatically zeroized here when it goes out of scope
```

### 1.6 Embedded-Specific Quick Reference {#embedded-specific-quick-reference}

#### Hardware and System Programming

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `static uint8_t buffer[1024];` | `static mut BUFFER: [u8; 1024] = [0; 1024];` | Global mutable state (needs `unsafe`) |
| `const uint8_t table[] = {...};` | `const TABLE: [u8; N] = [...];` | Compile-time constants |
| `__attribute__((section(".crypto")))` | `#[link_section = ".crypto"]` | Custom linker sections |
| `__attribute__((aligned(16)))` | `#[repr(align(16))]` | Memory alignment |
| `ISR(TIMER_vect) { ... }` | `#[interrupt] fn TIMER() { ... }` | Interrupt handlers |
| `cli(); /* critical section */ sei();` | `cortex_m::interrupt::free(\|_\| { ... })` | Critical sections |
| `volatile uint32_t* reg = 0x40000000;` | `let reg = 0x4000_0000 as *mut u32;` | Hardware register pointers |
| `*reg = value;` | `unsafe { ptr::write_volatile(reg, value) }` | Volatile register writes |
| `value = *reg;` | `unsafe { ptr::read_volatile(reg) }` | Volatile register reads |
| `__asm__("nop");` | `core::arch::asm!("nop");` | Inline assembly |

#### Memory Management and Collections

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `uint8_t buffer[256];` | `let mut buffer = [0u8; 256];` | Stack-allocated arrays |
| `static uint8_t pool[4096];` | `static mut POOL: [u8; 4096] = [0; 4096];` | Static memory pools |
| `malloc(size)` | `heapless::Vec::new()` | Heap-free dynamic collections |
| `realloc(ptr, new_size)` | *Not available* | Use fixed-size collections |
| `alloca(size)` | *Not recommended* | Use stack arrays with known size |

#### Crypto Hardware Integration

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `CRYPTO->KEY = key_value;` | `crypto.key.write(\|w\| w.bits(key_value))` | Register field access via PAC |
| `while (!(CRYPTO->STATUS & READY));` | `while !crypto.status.read().ready().bit() {}` | Status polling |
| `CRYPTO->CTRL \|= ENABLE;` | `crypto.ctrl.modify(\|_, w\| w.enable().set_bit())` | Register bit manipulation |
| `DMA_setup(src, dst, len);` | `dma.setup_transfer(src, dst, len)` | DMA configuration |

#### No-std Complete Template

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use cortex_m::interrupt;
use heapless::Vec;

// Global state with proper synchronization
static mut CRYPTO_BUFFER: [u8; 1024] = [0; 1024];
static mut CRYPTO_STATE: Option<CryptoEngine> = None;

#[entry]
fn main() -> ! {
    // Initialize hardware
    let peripherals = init_hardware();
    
    // Initialize crypto engine
    let crypto_engine = CryptoEngine::new(peripherals.crypto);
    
    // Store in global state (unsafe required)
    unsafe {
        CRYPTO_STATE = Some(crypto_engine);
    }
    
    // Enable interrupts
    unsafe { cortex_m::interrupt::enable() };
    
    loop {
        // Main application loop
        process_crypto_operations();
        cortex_m::asm::wfi(); // Wait for interrupt
    }
}

#[interrupt]
fn CRYPTO_IRQ() {
    // Handle crypto hardware interrupt
    interrupt::free(|_| {
        unsafe {
            if let Some(ref mut crypto) = CRYPTO_STATE {
                crypto.handle_interrupt();
            }
        }
    });
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Secure panic handler - clear sensitive data
    unsafe {
        CRYPTO_BUFFER.fill(0);
        if let Some(ref mut crypto) = CRYPTO_STATE {
            crypto.emergency_zeroize();
        }
    }
    
    // Reset system or halt
    cortex_m::peripheral::SCB::sys_reset();
}
```

#### Xilinx-Specific Patterns

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `Xil_DCacheFlush();` | `cortex_r::asm::dsb(); cortex_r::asm::isb();` | Cache operations |
| `XTime_GetTime(&timestamp);` | `timer.get_timestamp()` | Timing functions |
| `XIpiPsu_PollForAck(&ipi, target);` | `ipi.poll_for_ack(target)` | Inter-processor communication |
| `XCsuDma_Transfer(&dma, src, dst, len);` | `csu_dma.transfer(src, dst, len)` | CSU DMA operations |

### 1.7 Critical Differences and Gotchas {#critical-differences-and-gotchas}

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

---

*Continue to [Environment Setup](#environment-setup) for detailed installation and configuration instructions, or jump to any section using the table of contents above.*

---

## Cross-Reference System

This document uses a comprehensive cross-reference system to help you navigate between related concepts:

- **→ See also:** Related information in other sections
- **⚠️ Important:** Critical differences from C that could cause issues
- **💡 Tip:** Practical advice for embedded crypto development
- **🔗 Link:** Direct links to relevant sections
- **📖 Reference:** Links to external documentation

**Navigation Tips:**
- Use Ctrl+F (Cmd+F) to search for specific C patterns or Rust concepts
- Each section is designed to be self-contained for reference use
- Code examples are complete and can be compiled directly
- All crypto examples use production-ready crates and patterns

## 2. Environment Setup {#environment-setup}

Complete setup guide for embedded Rust cryptography development. All redundant instructions consolidated into this single authoritative section.

### 2.1 Rust Installation and Toolchain {#rust-installation-and-toolchain}

```bash
# Install Rust and essential embedded tools
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Core embedded development tools
cargo install cargo-binutils probe-rs --features cli cargo-embed

# Install targets for your hardware (choose what you need)
rustup target add thumbv7em-none-eabihf  # Cortex-M4F/M7F (most common)
rustup target add armv7r-none-eabihf     # Xilinx Cortex-R5F
rustup target add thumbv8m.main-none-eabihf # Cortex-M33F (with TrustZone)
```

### 2.2 Target Configuration {#target-configuration}

Choose your target configuration based on your hardware. Each includes optimized settings for crypto workloads.

#### 2.2.1 Xilinx Ultrascale+ (Cortex-R5) {#xilinx-ultrascale-cortex-r5}

```toml
# .cargo/config.toml - ZynqMP/Versal configuration
[target.armv7r-none-eabihf]
runner = "echo 'Use Xilinx Vitis/XSCT for debugging'"
rustflags = [
  "-C", "link-arg=-Tlink.x",
  "-C", "target-cpu=cortex-r5",
  "-C", "target-feature=+vfp3",
]

[build]
target = "armv7r-none-eabihf"
```

```rust
// memory.x - Optimized for crypto operations
MEMORY
{
  ATCM : ORIGIN = 0x00000000, LENGTH = 64K   /* Fast instruction access */
  BTCM : ORIGIN = 0x00020000, LENGTH = 64K   /* Fast data/stack */
  OCM  : ORIGIN = 0xFFFC0000, LENGTH = 256K  /* Shared crypto workspace */
  DDR  : ORIGIN = 0x00100000, LENGTH = 2G    /* Large buffers */
}

_stack_start = ORIGIN(BTCM) + LENGTH(BTCM);
_crypto_workspace = ORIGIN(OCM);
```

#### 2.2.2 ARM Cortex-M Series {#arm-cortex-m-series}

```toml
# .cargo/config.toml - STM32F4 example
[target.thumbv7em-none-eabihf]
runner = "probe-rs run --chip STM32F411RETx"
rustflags = [
  "-C", "link-arg=-Tlink.x",
  "-C", "target-cpu=cortex-m4",
  "-C", "target-feature=+fp-armv8d16",
]

[build]
target = "thumbv7em-none-eabihf"
```

#### 2.2.3 Other Embedded Targets {#other-embedded-targets}

```bash
# Additional targets for specialized applications
rustup target add riscv32imac-unknown-none-elf  # RISC-V with crypto extensions
rustup target add thumbv6m-none-eabi             # Cortex-M0+ (resource constrained)
```

### 2.3 Project Structure and Dependencies {#project-structure-and-dependencies}

```toml
# Cargo.toml - Essential crypto dependencies
[package]
name = "embedded-crypto-app"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core embedded
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"

# Crypto (no-std)
chacha20poly1305 = { version = "0.10", default-features = false }
aes-gcm = { version = "0.10", default-features = false }
sha2 = { version = "0.10", default-features = false }
subtle = { version = "2.5", default-features = false }
zeroize = { version = "1.6", default-features = false }

# Collections
heapless = "0.7"

[profile.release]
opt-level = "z"
lto = true
panic = "abort"
```

```
project/
├── Cargo.toml
├── .cargo/config.toml
├── memory.x
├── src/
│   ├── main.rs
│   ├── crypto/          # Crypto implementations
│   ├── hardware/        # Hardware abstraction
│   └── protocol/        # Communication protocols
```

### 2.4 Build Configuration {#build-configuration}

```rust
// build.rs - Hardware-specific optimizations
use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    
    // Enable hardware features based on target
    match target.as_str() {
        t if t.contains("thumbv8m") => {
            println!("cargo:rustc-cfg=feature=\"hw_crypto\"");
            println!("cargo:rustc-cfg=feature=\"trustzone\"");
        }
        t if t.contains("cortex-r5") => {
            println!("cargo:rustc-cfg=feature=\"xilinx_r5\"");
        }
        _ => {}
    }
}
```

### 2.5 Verification and Testing {#verification-and-testing}

#### Step-by-Step Setup Verification

```bash
# 1. Verify installation
rustc --version && cargo --version

# 2. Test target compilation
cargo new --bin setup-test && cd setup-test
echo 'cortex-m = "0.7"' >> Cargo.toml
echo 'cortex-m-rt = "0.7"' >> Cargo.toml
echo 'panic-halt = "0.2"' >> Cargo.toml

# 3. Build for your target
cargo build --target thumbv7em-none-eabihf --release

# 4. Verify binary size (should be small)
cargo size --target thumbv7em-none-eabihf --release

# 5. Test crypto compilation
echo 'sha2 = { version = "0.10", default-features = false }' >> Cargo.toml
cargo build --target thumbv7em-none-eabihf --release
```

#### Minimal Verification Application

```rust
// src/main.rs - Setup verification test
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    // Basic functionality test
    let test_data = [0x42u8; 32];
    let mut result = 0u8;
    
    // Simple crypto-like operation
    for &byte in &test_data {
        result ^= byte;
    }
    
    // If this compiles and the result is 0x42, setup is working
    assert_eq!(result, 0x42);
    
    loop {
        cortex_m::asm::wfi();
    }
}
```

**✅ Setup Complete** - Your environment is ready for embedded Rust crypto development.

**→ Next:** [Core Language Concepts](#core-language-concepts) - Essential Rust concepts for crypto development

---

## 3. Core Language Concepts {#core-language-concepts}

This section consolidates the essential Rust concepts that differ significantly from C, with focus on how they benefit cryptographic development. Each concept is explained once, thoroughly, with embedded crypto-specific examples.

### 3.1 Ownership and Memory Management {#ownership-and-memory-management}

Rust's ownership system replaces C's manual memory management with compile-time rules that prevent entire classes of crypto vulnerabilities. This is the single most important concept for C programmers to master.

#### The Three Rules of Ownership

1. **Each value has exactly one owner** - No shared ownership without explicit mechanisms
2. **When the owner goes out of scope, the value is dropped** - Automatic cleanup with Drop trait
3. **Only one mutable reference OR multiple immutable references** - Prevents data races and use-after-free

#### Ownership in Embedded Crypto Context

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
struct CryptoContext {
    session_key: [u8; 32],    // Owned key material
    nonce_counter: u64,       // Owned state
    cipher_state: AesState,   // Owned cipher context
}

impl CryptoContext {
    fn new(key: [u8; 32]) -> Self {
        Self {
            session_key: key,  // key ownership transferred here
            nonce_counter: 0,
            cipher_state: AesState::new(),
        }
    }
    
    fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Exclusive mutable access prevents concurrent key usage
        self.nonce_counter += 1;
        
        // Use owned key material safely
        let nonce = self.nonce_counter.to_le_bytes();
        aes_gcm_encrypt(&self.session_key, &nonce, plaintext)
    }
}

fn secure_communication() -> Result<(), CryptoError> {
    let master_key = generate_session_key()?;
    let mut crypto_ctx = CryptoContext::new(master_key);
    // master_key is moved, can't be accidentally reused
    
    let message1 = crypto_ctx.encrypt(b"first message")?;
    let message2 = crypto_ctx.encrypt(b"second message")?;
    // Nonce automatically incremented, no reuse possible
    
    transmit_messages(&[message1, message2])?;
    
    // crypto_ctx dropped here, session_key automatically zeroized
    Ok(())
}
```

#### Borrowing Rules for Crypto Operations

```rust
// Immutable borrowing - safe concurrent reads
fn verify_multiple_signatures(
    public_key: &[u8; 32],     // Immutable borrow
    messages: &[&[u8]],        // Multiple immutable borrows allowed
    signatures: &[[u8; 64]]
) -> Vec<bool> {
    messages.iter()
        .zip(signatures.iter())
        .map(|(msg, sig)| verify_signature(public_key, msg, sig))
        .collect()
}

// Mutable borrowing - exclusive access for state updates
fn update_crypto_state(ctx: &mut CryptoContext, new_params: CryptoParams) {
    // Exclusive access ensures no race conditions
    ctx.nonce_counter = new_params.starting_nonce;
    ctx.cipher_state.update_key_schedule(&new_params.key);
    // No other code can access ctx during this function
}

// Lifetime management for key derivation
struct KeyDerivationContext<'a> {
    master_key: &'a [u8; 32],  // Borrowed master key
    salt: &'a [u8; 16],        // Borrowed salt
}

impl<'a> KeyDerivationContext<'a> {
    fn derive_key(&self, info: &[u8]) -> [u8; 32] {
        // Compiler ensures master_key and salt remain valid
        hkdf_expand(self.master_key, self.salt, info)
    }
}

// Usage ensures master key outlives derived keys
fn key_hierarchy_example() -> Result<(), CryptoError> {
    let master_key = [0u8; 32];  // Stack allocated
    let salt = [1u8; 16];        // Stack allocated
    
    let kdf_ctx = KeyDerivationContext {
        master_key: &master_key,
        salt: &salt,
    };
    
    let encryption_key = kdf_ctx.derive_key(b"encryption");
    let mac_key = kdf_ctx.derive_key(b"authentication");
    
    // master_key and salt guaranteed valid throughout this scope
    perform_crypto_operations(&encryption_key, &mac_key)?;
    
    Ok(())
}
```

#### Memory Management Patterns for Embedded

```rust
#![no_std]

use heapless::Vec;

// Stack-based crypto operations (preferred in embedded)
fn stack_crypto_operations() -> Result<[u8; 16], CryptoError> {
    let mut plaintext = [0u8; 16];      // Stack allocated
    let key = [0u8; 32];                // Stack allocated
    let mut cipher_state = AesState::new(); // Stack allocated
    
    // All operations use stack memory
    cipher_state.set_key(&key);
    cipher_state.encrypt_block(&mut plaintext)?;
    
    Ok(plaintext)
    // All memory automatically cleaned up
}

// Static allocation for global crypto state
static mut GLOBAL_CRYPTO_CTX: Option<CryptoContext> = None;

fn init_global_crypto(key: [u8; 32]) {
    unsafe {
        GLOBAL_CRYPTO_CTX = Some(CryptoContext::new(key));
    }
}

// Heapless collections for message queues
fn message_queue_example() -> Result<(), CryptoError> {
    let mut encrypted_messages: Vec<[u8; 32], 16> = Vec::new();
    
    for i in 0..10 {
        let plaintext = [i; 16];
        let ciphertext = encrypt_message(&plaintext)?;
        encrypted_messages.push(ciphertext)
            .map_err(|_| CryptoError::QueueFull)?;
    }
    
    // Process messages without heap allocation
    for message in encrypted_messages.iter() {
        transmit_encrypted_message(message)?;
    }
    
    Ok(())
}
```

### 3.2 Error Handling Without Exceptions {#error-handling-without-exceptions}

Rust's explicit error handling with `Result<T, E>` and `Option<T>` prevents silent failures that are catastrophic in cryptographic systems. Every error condition must be explicitly handled.

#### Comprehensive Crypto Error Types

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CryptoError {
    // Key-related errors
    InvalidKeySize,
    WeakKey,
    KeyExpired,
    KeyNotFound,
    
    // Algorithm errors
    InvalidNonce,
    InvalidTag,
    AuthenticationFailed,
    DecryptionFailed,
    
    // Hardware errors
    HardwareNotAvailable,
    HardwareTimeout,
    HardwareError(u32),
    
    // Resource errors
    InsufficientEntropy,
    BufferTooSmall,
    QueueFull,
    OutOfMemory,
    
    // Protocol errors
    InvalidState,
    ProtocolViolation,
    HandshakeFailed,
}

impl CryptoError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            CryptoError::HardwareTimeout | 
            CryptoError::InsufficientEntropy |
            CryptoError::QueueFull => true,
            _ => false,
        }
    }
}

type CryptoResult<T> = Result<T, CryptoError>;
```

#### Error Propagation in Crypto Pipelines

```rust
// Crypto operations with explicit error handling
fn encrypt_aes_gcm(
    key: &[u8; 32], 
    nonce: &[u8; 12], 
    plaintext: &[u8],
    aad: &[u8]
) -> CryptoResult<Vec<u8>> {
    // Validate inputs explicitly
    if key.iter().all(|&b| b == 0) {
        return Err(CryptoError::WeakKey);
    }
    
    if nonce.iter().all(|&b| b == 0) {
        return Err(CryptoError::InvalidNonce);
    }
    
    // Perform encryption with error checking
    let mut cipher = Aes256Gcm::new(key)?;
    let ciphertext = cipher.encrypt(nonce, plaintext, aad)?;
    
    Ok(ciphertext)
}

// Error propagation with ? operator
fn secure_message_processing(message: &[u8]) -> CryptoResult<Vec<u8>> {
    let session_key = derive_session_key()?;     // Propagates key derivation errors
    let nonce = generate_nonce()?;               // Propagates RNG errors
    let aad = compute_aad(&message)?;            // Propagates AAD computation errors
    
    let encrypted = encrypt_aes_gcm(&session_key, &nonce, message, &aad)?;
    
    Ok(encrypted)
}

// Comprehensive error handling
fn handle_crypto_pipeline(messages: &[&[u8]]) -> CryptoResult<Vec<Vec<u8>>> {
    let mut results = Vec::new();
    
    for message in messages {
        match secure_message_processing(message) {
            Ok(encrypted) => {
                results.push(encrypted);
            }
            Err(CryptoError::InsufficientEntropy) => {
                // Recoverable error - wait and retry
                wait_for_entropy()?;
                let encrypted = secure_message_processing(message)?;
                results.push(encrypted);
            }
            Err(CryptoError::HardwareTimeout) => {
                // Fallback to software crypto
                let encrypted = software_encrypt_message(message)?;
                results.push(encrypted);
            }
            Err(e) if e.is_recoverable() => {
                // Generic recoverable error handling
                retry_with_backoff(|| secure_message_processing(message))?;
            }
            Err(e) => {
                // Non-recoverable error
                log_crypto_error(&e);
                return Err(e);
            }
        }
    }
    
    Ok(results)
}
```

#### Option Types for Safe Nullable Crypto State

```rust
// Safe handling of optional crypto contexts
struct CryptoManager {
    active_sessions: heapless::FnvIndexMap<u32, CryptoContext, 16>,
}

impl CryptoManager {
    fn get_session(&mut self, session_id: u32) -> Option<&mut CryptoContext> {
        self.active_sessions.get_mut(&session_id)
    }
    
    fn process_message(&mut self, session_id: u32, message: &[u8]) -> CryptoResult<Vec<u8>> {
        match self.get_session(session_id) {
            Some(ctx) => {
                // Session exists, safe to use
                ctx.encrypt(message)
            }
            None => {
                // No session found, handle gracefully
                Err(CryptoError::InvalidState)
            }
        }
    }
    
    fn cleanup_expired_sessions(&mut self) {
        self.active_sessions.retain(|_, ctx| !ctx.is_expired());
    }
}

// Chaining Option operations
fn find_and_use_key(key_id: u32) -> Option<Vec<u8>> {
    get_key_from_storage(key_id)?     // Returns None if key not found
        .validate()?                  // Returns None if key invalid
        .decrypt_with_master_key()    // Returns None if decryption fails
}
```

#### Error Recovery Patterns for Embedded Crypto

```rust
// Retry with exponential backoff for transient errors
fn retry_with_backoff<F, T>(mut operation: F) -> CryptoResult<T>
where
    F: FnMut() -> CryptoResult<T>,
{
    const MAX_RETRIES: u32 = 3;
    let mut delay_ms = 10;
    
    for attempt in 0..MAX_RETRIES {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if e.is_recoverable() && attempt < MAX_RETRIES - 1 => {
                // Wait before retry
                cortex_m::asm::delay(delay_ms * 1000);
                delay_ms *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
    
    Err(CryptoError::HardwareTimeout)
}

// Graceful degradation for hardware failures
fn encrypt_with_fallback(data: &[u8], key: &[u8; 32]) -> CryptoResult<Vec<u8>> {
    // Try hardware acceleration first
    match hardware_encrypt(data, key) {
        Ok(result) => Ok(result),
        Err(CryptoError::HardwareNotAvailable) => {
            // Fall back to software implementation
            software_encrypt(data, key)
        }
        Err(e) => Err(e), // Propagate other errors
    }
}
```

### 3.3 Type System Advantages for Security {#type-system-advantages}

Rust's type system encodes security invariants at compile time, preventing entire classes of cryptographic vulnerabilities that are common in C implementations.

#### Type-Safe Protocol State Machines

```rust
use core::marker::PhantomData;

// Protocol states as types
struct Uninitialized;
struct HandshakeInProgress;
struct KeyExchangeComplete;
struct SessionEstablished;
struct SessionTerminated;

// TLS connection with compile-time state tracking
struct TlsConnection<State> {
    socket: Socket,
    buffer: [u8; 4096],
    state: PhantomData<State>,
}

impl TlsConnection<Uninitialized> {
    fn new(socket: Socket) -> Self {
        Self { 
            socket, 
            buffer: [0; 4096],
            state: PhantomData 
        }
    }
    
    fn start_handshake(self, config: TlsConfig) -> CryptoResult<TlsConnection<HandshakeInProgress>> {
        // Send ClientHello
        let client_hello = build_client_hello(&config)?;
        self.socket.send(&client_hello)?;
        
        Ok(TlsConnection {
            socket: self.socket,
            buffer: self.buffer,
            state: PhantomData,
        })
    }
}

impl TlsConnection<HandshakeInProgress> {
    fn process_server_hello(mut self, server_hello: &[u8]) -> CryptoResult<TlsConnection<KeyExchangeComplete>> {
        // Validate server hello
        let server_params = parse_server_hello(server_hello)?;
        validate_server_params(&server_params)?;
        
        Ok(TlsConnection {
            socket: self.socket,
            buffer: self.buffer,
            state: PhantomData,
        })
    }
}

impl TlsConnection<KeyExchangeComplete> {
    fn complete_handshake(self, master_secret: &[u8; 48]) -> TlsConnection<SessionEstablished> {
        TlsConnection {
            socket: self.socket,
            buffer: self.buffer,
            state: PhantomData,
        }
    }
}

impl TlsConnection<SessionEstablished> {
    // Only available after successful handshake
    fn send_encrypted(&mut self, data: &[u8]) -> CryptoResult<()> {
        let encrypted = encrypt_application_data(data)?;
        self.socket.send(&encrypted)?;
        Ok(())
    }
    
    fn receive_encrypted(&mut self) -> CryptoResult<Vec<u8>> {
        let encrypted = self.socket.receive(&mut self.buffer)?;
        decrypt_application_data(&encrypted)
    }
    
    fn terminate(self) -> TlsConnection<SessionTerminated> {
        // Send close_notify
        let _ = self.socket.send(&[0x15, 0x03, 0x03, 0x00, 0x02, 0x01, 0x00]);
        
        TlsConnection {
            socket: self.socket,
            buffer: self.buffer,
            state: PhantomData,
        }
    }
}

// Compiler enforces correct state transitions
fn secure_communication_example() -> CryptoResult<()> {
    let socket = Socket::connect("server:443")?;
    let config = TlsConfig::default();
    
    let conn = TlsConnection::new(socket)
        .start_handshake(config)?
        .process_server_hello(&receive_server_hello()?)?
        .complete_handshake(&derive_master_secret()?);
    
    // Can only send encrypted data after handshake is complete
    conn.send_encrypted(b"GET / HTTP/1.1\r\n\r\n")?;
    let response = conn.receive_encrypted()?;
    
    // Properly terminate session
    let _terminated = conn.terminate();
    
    Ok(())
}
```

#### Const Generics for Compile-Time Crypto Parameters

```rust
// Type-safe key sizes with const generics
#[derive(ZeroizeOnDrop)]
struct CryptoKey<const N: usize> {
    key_material: [u8; N],
    algorithm: KeyAlgorithm,
}

impl<const N: usize> CryptoKey<N> {
    fn new(material: [u8; N], algorithm: KeyAlgorithm) -> CryptoResult<Self> {
        // Validate key size matches algorithm
        match (N, algorithm) {
            (16, KeyAlgorithm::Aes128) |
            (32, KeyAlgorithm::Aes256) |
            (32, KeyAlgorithm::ChaCha20) => {
                Ok(Self { key_material: material, algorithm })
            }
            _ => Err(CryptoError::InvalidKeySize),
        }
    }
    
    fn as_bytes(&self) -> &[u8; N] {
        &self.key_material
    }
}

// Type aliases for specific key sizes
type Aes128Key = CryptoKey<16>;
type Aes256Key = CryptoKey<32>;
type ChaCha20Key = CryptoKey<32>;

// Compile-time prevention of key size mismatches
fn encrypt_with_aes256(key: &Aes256Key, plaintext: &[u8]) -> CryptoResult<Vec<u8>> {
    // key is guaranteed to be exactly 32 bytes at compile time
    let cipher = Aes256::new(key.as_bytes());
    cipher.encrypt(plaintext)
}

fn encrypt_with_chacha20(key: &ChaCha20Key, nonce: &[u8; 12], plaintext: &[u8]) -> CryptoResult<Vec<u8>> {
    // key is guaranteed to be exactly 32 bytes at compile time
    let cipher = ChaCha20::new(key.as_bytes(), nonce);
    cipher.encrypt(plaintext)
}

// Const generic arrays for fixed-size crypto operations
struct CryptoBuffer<const SIZE: usize> {
    data: [u8; SIZE],
    used: usize,
}

impl<const SIZE: usize> CryptoBuffer<SIZE> {
    fn new() -> Self {
        Self { data: [0; SIZE], used: 0 }
    }
    
    fn encrypt_in_place<const KEY_SIZE: usize>(
        &mut self, 
        key: &CryptoKey<KEY_SIZE>
    ) -> CryptoResult<()> {
        // Compile-time bounds checking
        if self.used > SIZE {
            return Err(CryptoError::BufferTooSmall);
        }
        
        // Perform in-place encryption
        encrypt_buffer(&mut self.data[..self.used], key.as_bytes())?;
        Ok(())
    }
}

// Usage with compile-time guarantees
fn crypto_operations_example() -> CryptoResult<()> {
    let aes_key = Aes256Key::new([0; 32], KeyAlgorithm::Aes256)?;
    let chacha_key = ChaCha20Key::new([1; 32], KeyAlgorithm::ChaCha20)?;
    
    let mut buffer: CryptoBuffer<1024> = CryptoBuffer::new();
    
    // This works - correct key type and size
    encrypt_with_aes256(&aes_key, b"test data")?;
    buffer.encrypt_in_place(&aes_key)?;
    
    // This would be a compile error - wrong key type
    // encrypt_with_aes256(&chacha_key, b"test data")?;
    
    Ok(())
}
```

#### Newtype Pattern for Domain-Specific Security

```rust
// Distinguish between different types of cryptographic data
#[derive(Clone, Copy)]
struct PlaintextData<'a>(&'a [u8]);

#[derive(Clone, Copy)]
struct CiphertextData<'a>(&'a [u8]);

#[derive(Clone, Copy)]
struct AuthenticatedData<'a>(&'a [u8]);

#[derive(Clone, Copy, ZeroizeOnDrop)]
struct KeyMaterial<'a>(&'a [u8]);

// Prevent mixing up different types of crypto data
impl<'a> PlaintextData<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self(data)
    }
    
    fn as_bytes(&self) -> &[u8] {
        self.0
    }
}

impl<'a> CiphertextData<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self(data)
    }
    
    fn as_bytes(&self) -> &[u8] {
        self.0
    }
}

// Type-safe crypto operations
fn aead_encrypt(
    key: KeyMaterial,
    nonce: &[u8; 12],
    plaintext: PlaintextData,
    aad: AuthenticatedData,
) -> CryptoResult<Vec<u8>> {
    // Function signature prevents mixing up parameters
    let cipher = ChaCha20Poly1305::new(key.0);
    cipher.encrypt(nonce, plaintext.as_bytes(), aad.as_bytes())
}

fn aead_decrypt(
    key: KeyMaterial,
    nonce: &[u8; 12],
    ciphertext: CiphertextData,
    aad: AuthenticatedData,
) -> CryptoResult<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(key.0);
    cipher.decrypt(nonce, ciphertext.as_bytes(), aad.as_bytes())
}

// Usage prevents parameter confusion
fn secure_message_example() -> CryptoResult<()> {
    let key_bytes = [0u8; 32];
    let key = KeyMaterial(&key_bytes);
    let nonce = [1u8; 12];
    
    let message = b"secret message";
    let additional_data = b"public header";
    
    // Type system prevents parameter mix-ups
    let plaintext = PlaintextData::new(message);
    let aad = AuthenticatedData::new(additional_data);
    
    let encrypted = aead_encrypt(key, &nonce, plaintext, aad)?;
    
    // For decryption, must use CiphertextData
    let ciphertext = CiphertextData::new(&encrypted);
    let decrypted = aead_decrypt(key, &nonce, ciphertext, aad)?;
    
    Ok(())
}
```

**→ Next:** [Embedded-Specific Patterns](#embedded-specific-patterns) - Rust patterns for embedded development

---

## 4. Embedded-Specific Patterns {#embedded-specific-patterns}

This section consolidates all Rust patterns specific to embedded development, focusing on no-std programming, hardware interaction, and real-time constraints essential for embedded cryptography. All scattered content has been unified into this comprehensive reference.

### 4.1 No-std Programming Essentials {#no-std-programming-essentials}

No-std programming is fundamental to embedded Rust. This section consolidates all no-std concepts and patterns from across the document into a single authoritative reference.

#### Complete No-std Project Template

```rust
#![no_std]
#![no_main]
#![forbid(unsafe_code)]  // Optional: forbid unsafe except in specific modules

// Essential imports for no-std embedded crypto
use panic_halt as _;
use cortex_m_rt::entry;

// Core library - always available in no-std
use core::{
    mem,                    // Memory utilities (size_of, align_of, etc.)
    ptr,                    // Pointer operations (read_volatile, write_volatile)
    slice,                  // Slice operations
    fmt::Write,             // Formatting without heap allocation
    convert::TryInto,       // Fallible conversions
    ops::{Deref, DerefMut}, // Smart pointer traits
};

// Heapless collections - essential for no-std
use heapless::{
    Vec,                    // Stack-allocated vector
    String,                 // Stack-allocated string  
    FnvIndexMap,           // Hash map without heap
    spsc::{Queue, Producer, Consumer}, // Lock-free queues
    pool::{Pool, Node},     // Memory pools
};

// Crypto dependencies (all no-std compatible)
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use sha2::{Sha256, Digest};
use zeroize::{Zeroize, ZeroizeOnDrop};
use subtle::ConstantTimeEq;

#[entry]
fn main() -> ! {
    // Initialize hardware and crypto systems
    let mut crypto_hw = init_crypto_hardware();
    let mut crypto_ctx = CryptoContext::new();
    
    // Initialize static memory pools
    init_memory_pools();
    
    // Enable interrupts for crypto operations
    unsafe { cortex_m::interrupt::enable() };
    
    // Main application loop
    loop {
        process_crypto_requests(&mut crypto_hw, &mut crypto_ctx);
        handle_crypto_interrupts();
        cortex_m::asm::wfi(); // Wait for interrupt
    }
}

// Secure panic handler that clears sensitive data
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Disable interrupts to prevent further crypto operations
    cortex_m::interrupt::disable();
    
    // Clear all sensitive data before panic
    unsafe {
        clear_crypto_memory();
    }
    
    // Log panic info if possible, then reset
    #[cfg(feature = "panic-log")]
    log_panic_info(info);
    
    // Reset system or halt based on configuration
    #[cfg(feature = "panic-reset")]
    cortex_m::peripheral::SCB::sys_reset();
    
    #[cfg(not(feature = "panic-reset"))]
    loop { cortex_m::asm::wfi(); }
}
```

#### No-std Memory Management Patterns

```rust
use heapless::{Vec, pool::{Pool, Node, Singleton}};

// Static memory pools for different crypto operations
static mut SMALL_BLOCKS: [Node<[u8; 256]>; 16] = [Node::new(); 16];
static mut LARGE_BLOCKS: [Node<[u8; 4096]>; 4] = [Node::new(); 4];
static mut CRYPTO_CONTEXTS: [Node<CryptoContext>; 8] = [Node::new(); 8];

static SMALL_POOL: Pool<[u8; 256]> = Pool::new();
static LARGE_POOL: Pool<[u8; 4096]> = Pool::new();
static CONTEXT_POOL: Pool<CryptoContext> = Pool::new();

// Singleton for global crypto state (only one instance allowed)
static mut GLOBAL_CRYPTO_STATE: [u8; 2048] = [0; 2048];
static CRYPTO_STATE: Singleton<GlobalCryptoState> = Singleton::new();

fn init_memory_pools() {
    unsafe {
        SMALL_POOL.grow(&mut SMALL_BLOCKS);
        LARGE_POOL.grow(&mut LARGE_BLOCKS);
        CONTEXT_POOL.grow(&mut CRYPTO_CONTEXTS);
    }
}

// Memory-efficient crypto operations using pools
fn encrypt_with_pool(plaintext: &[u8]) -> Result<heapless::Vec<u8, 4096>, CryptoError> {
    // Choose appropriate pool based on data size
    if plaintext.len() <= 256 {
        let mut buffer = SMALL_POOL.alloc().ok_or(CryptoError::OutOfMemory)?;
        buffer[..plaintext.len()].copy_from_slice(plaintext);
        
        // Perform in-place encryption
        encrypt_in_place(&mut buffer[..plaintext.len()])?;
        
        // Copy result to heapless Vec
        let mut result = heapless::Vec::new();
        result.extend_from_slice(&buffer[..plaintext.len()])
            .map_err(|_| CryptoError::BufferTooSmall)?;
        
        Ok(result)
    } else if plaintext.len() <= 4096 {
        let mut buffer = LARGE_POOL.alloc().ok_or(CryptoError::OutOfMemory)?;
        buffer[..plaintext.len()].copy_from_slice(plaintext);
        
        encrypt_in_place(&mut buffer[..plaintext.len()])?;
        
        let mut result = heapless::Vec::new();
        result.extend_from_slice(&buffer[..plaintext.len()])
            .map_err(|_| CryptoError::BufferTooSmall)?;
        
        Ok(result)
    } else {
        Err(CryptoError::DataTooLarge)
    }
}

// Stack-based collections for crypto state management
#[derive(ZeroizeOnDrop)]
struct CryptoSession {
    active_keys: Vec<[u8; 32], 8>,        // Max 8 active keys
    message_queue: Vec<CryptoMessage, 32>, // Max 32 queued messages
    nonce_counters: Vec<u64, 8>,           // Nonce counter per key
    session_id: u32,
}

impl CryptoSession {
    fn new(session_id: u32) -> Self {
        Self {
            active_keys: Vec::new(),
            message_queue: Vec::new(),
            nonce_counters: Vec::new(),
            session_id,
        }
    }
    
    fn add_key(&mut self, key: [u8; 32]) -> Result<usize, CryptoError> {
        let key_index = self.active_keys.len();
        
        self.active_keys.push(key)
            .map_err(|_| CryptoError::TooManyKeys)?;
        self.nonce_counters.push(0)
            .map_err(|_| CryptoError::TooManyKeys)?;
            
        Ok(key_index)
    }
    
    fn encrypt_message(&mut self, key_index: usize, plaintext: &[u8]) -> Result<Vec<u8, 4096>, CryptoError> {
        let key = self.active_keys.get(key_index).ok_or(CryptoError::InvalidKeyIndex)?;
        let nonce_counter = self.nonce_counters.get_mut(key_index).ok_or(CryptoError::InvalidKeyIndex)?;
        
        // Increment nonce counter (prevents reuse)
        *nonce_counter += 1;
        let nonce_bytes = nonce_counter.to_le_bytes();
        
        // Perform encryption with automatic nonce management
        chacha20poly1305_encrypt(key, &nonce_bytes, plaintext)
    }
}

// Compile-time memory layout with const generics
struct CryptoWorkspace<const BUFFER_SIZE: usize, const KEY_COUNT: usize> {
    buffers: [[u8; BUFFER_SIZE]; 4],
    keys: [[u8; 32]; KEY_COUNT],
    nonces: [[u8; 12]; KEY_COUNT],
    in_use: [bool; 4], // Track buffer usage
}

impl<const BUFFER_SIZE: usize, const KEY_COUNT: usize> CryptoWorkspace<BUFFER_SIZE, KEY_COUNT> {
    const fn new() -> Self {
        Self {
            buffers: [[0; BUFFER_SIZE]; 4],
            keys: [[0; 32]; KEY_COUNT],
            nonces: [[0; 12]; KEY_COUNT],
            in_use: [false; 4],
        }
    }
    
    fn get_free_buffer(&mut self) -> Option<(usize, &mut [u8; BUFFER_SIZE])> {
        for (i, in_use) in self.in_use.iter_mut().enumerate() {
            if !*in_use {
                *in_use = true;
                return Some((i, &mut self.buffers[i]));
            }
        }
        None
    }
    
    fn release_buffer(&mut self, index: usize) {
        if index < 4 {
            self.in_use[index] = false;
            // Zeroize buffer on release for security
            self.buffers[index].zeroize();
        }
    }
}

// Different workspace configurations for different applications
type SmallCryptoWorkspace = CryptoWorkspace<256, 2>;   // 2KB buffers, 2 keys
type LargeCryptoWorkspace = CryptoWorkspace<4096, 8>;  // 16KB buffers, 8 keys

static mut CRYPTO_WS: SmallCryptoWorkspace = SmallCryptoWorkspace::new();
```

#### No-std Error Handling and Result Types

```rust
// Custom error types that work in no-std
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoError {
    InvalidKey,
    InvalidNonce,
    EncryptionFailed,
    DecryptionFailed,
    AuthenticationFailed,
    OutOfMemory,
    BufferTooSmall,
    DataTooLarge,
    TooManyKeys,
    InvalidKeyIndex,
    HardwareError(u32),
}

// No-std compatible Result type
pub type CryptoResult<T> = core::result::Result<T, CryptoError>;

// Error handling without std::error::Error trait
impl CryptoError {
    pub fn description(&self) -> &'static str {
        match self {
            CryptoError::InvalidKey => "Invalid cryptographic key",
            CryptoError::InvalidNonce => "Invalid nonce value",
            CryptoError::EncryptionFailed => "Encryption operation failed",
            CryptoError::DecryptionFailed => "Decryption operation failed",
            CryptoError::AuthenticationFailed => "Authentication verification failed",
            CryptoError::OutOfMemory => "Insufficient memory available",
            CryptoError::BufferTooSmall => "Buffer too small for operation",
            CryptoError::DataTooLarge => "Data exceeds maximum size",
            CryptoError::TooManyKeys => "Maximum number of keys exceeded",
            CryptoError::InvalidKeyIndex => "Invalid key index specified",
            CryptoError::HardwareError(_) => "Hardware crypto accelerator error",
        }
    }
    
    pub fn is_recoverable(&self) -> bool {
        match self {
            CryptoError::OutOfMemory | CryptoError::TooManyKeys => true,
            _ => false,
        }
    }
}

// No-std compatible formatting
impl core::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.description())
    }
}
```

### 4.2 Hardware Abstraction Patterns {#hardware-abstraction-patterns}

This section consolidates all hardware abstraction patterns for embedded crypto development, providing portable and maintainable approaches to hardware integration.

#### Peripheral Access Crate (PAC) Usage

```rust
// Direct register access using PAC - consolidated from scattered examples
use stm32f4xx_pac as pac;
use cortex_m::interrupt;

struct CryptoPeripheral {
    cryp: pac::CRYP,
    dma: Option<pac::DMA2>,
}

impl CryptoPeripheral {
    fn new(cryp: pac::CRYP, dma: Option<pac::DMA2>) -> Self {
        Self { cryp, dma }
    }
    
    fn configure_aes(&mut self, key: &[u32; 8], mode: AesMode) {
        // Configure crypto peripheral for AES with different modes
        self.cryp.cr.write(|w| {
            match mode {
                AesMode::ECB => w.algomode().aes_ecb(),
                AesMode::CBC => w.algomode().aes_cbc(),
                AesMode::CTR => w.algomode().aes_ctr(),
                AesMode::GCM => w.algomode().aes_gcm(),
            }
            .datatype().bits32()
            .keysize().bits256()
        });
        
        // Load key into hardware registers (all 8 words for 256-bit key)
        let key_regs = [
            &self.cryp.k0lr, &self.cryp.k0rr, &self.cryp.k1lr, &self.cryp.k1rr,
            &self.cryp.k2lr, &self.cryp.k2rr, &self.cryp.k3lr, &self.cryp.k3rr,
        ];
        
        for (reg, &key_word) in key_regs.iter().zip(key.iter()) {
            reg.write(|w| unsafe { w.bits(key_word) });
        }
        
        // Enable crypto peripheral
        self.cryp.cr.modify(|_, w| w.crypen().enabled());
    }
    
    fn encrypt_block(&mut self, input: &[u32; 4]) -> Result<[u32; 4], CryptoError> {
        // Check if peripheral is ready
        if self.cryp.sr.read().busy().bit_is_set() {
            return Err(CryptoError::HardwareBusy);
        }
        
        // Write input data to data input registers
        let input_regs = [&self.cryp.din, &self.cryp.din, &self.cryp.din, &self.cryp.din];
        for (reg, &word) in input_regs.iter().zip(input.iter()) {
            reg.write(|w| unsafe { w.bits(word) });
        }
        
        // Wait for processing complete with timeout
        let mut timeout = 10000;
        while self.cryp.sr.read().busy().bit_is_set() && timeout > 0 {
            timeout -= 1;
            cortex_m::asm::nop();
        }
        
        if timeout == 0 {
            return Err(CryptoError::HardwareTimeout);
        }
        
        // Read output data
        Ok([
            self.cryp.dout.read().bits(),
            self.cryp.dout.read().bits(),
            self.cryp.dout.read().bits(),
            self.cryp.dout.read().bits(),
        ])
    }
    
    fn setup_dma_transfer(&mut self, src: &[u8], dst: &mut [u8]) -> Result<(), CryptoError> {
        if let Some(ref mut dma) = self.dma {
            // Configure DMA for crypto operations
            dma.s0cr.write(|w| {
                w.chsel().bits(2)  // Crypto channel
                 .mburst().single()
                 .pburst().single()
                 .pl().high()
                 .msize().bits32()
                 .psize().bits32()
                 .minc().incremented()
                 .pinc().fixed()
                 .dir().memory_to_peripheral()
            });
            
            // Set addresses and count
            dma.s0par.write(|w| unsafe { w.bits(self.cryp.din.as_ptr() as u32) });
            dma.s0m0ar.write(|w| unsafe { w.bits(src.as_ptr() as u32) });
            dma.s0ndtr.write(|w| w.ndt().bits(src.len() as u16 / 4));
            
            // Enable DMA stream
            dma.s0cr.modify(|_, w| w.en().enabled());
            
            Ok(())
        } else {
            Err(CryptoError::DmaNotAvailable)
        }
    }
}

#[derive(Clone, Copy)]
enum AesMode {
    ECB,
    CBC,
    CTR,
    GCM,
}
```

#### Hardware Abstraction Layer (HAL) Patterns

```rust
// Generic crypto traits for hardware abstraction - consolidated interface
trait BlockCipher {
    type Error;
    type Block: AsRef<[u8]> + AsMut<[u8]>;
    
    fn encrypt_block(&mut self, block: &mut Self::Block) -> Result<(), Self::Error>;
    fn decrypt_block(&mut self, block: &mut Self::Block) -> Result<(), Self::Error>;
    fn block_size(&self) -> usize;
}

trait StreamCipher {
    type Error;
    
    fn encrypt_stream(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;
    fn decrypt_stream(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;
}

trait AuthenticatedCipher {
    type Error;
    type Tag: AsRef<[u8]>;
    
    fn encrypt_and_authenticate(&mut self, 
                               plaintext: &[u8], 
                               aad: &[u8]) -> Result<(Vec<u8>, Self::Tag), Self::Error>;
    fn decrypt_and_verify(&mut self, 
                         ciphertext: &[u8], 
                         aad: &[u8], 
                         tag: &Self::Tag) -> Result<Vec<u8>, Self::Error>;
}

// Hardware implementation with comprehensive error handling
struct HardwareAes {
    peripheral: CryptoPeripheral,
    mode: AesMode,
}

impl HardwareAes {
    fn new(peripheral: CryptoPeripheral, mode: AesMode) -> Self {
        Self { peripheral, mode }
    }
    
    fn set_key(&mut self, key: &[u8; 32]) -> Result<(), CryptoError> {
        // Convert bytes to words for hardware
        let mut key_words = [0u32; 8];
        for (i, chunk) in key.chunks_exact(4).enumerate() {
            key_words[i] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        
        self.peripheral.configure_aes(&key_words, self.mode);
        Ok(())
    }
}

impl BlockCipher for HardwareAes {
    type Error = CryptoError;
    type Block = [u8; 16];
    
    fn encrypt_block(&mut self, block: &mut [u8; 16]) -> Result<(), Self::Error> {
        // Convert bytes to words for hardware
        let mut words = [0u32; 4];
        for (i, chunk) in block.chunks_exact(4).enumerate() {
            words[i] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        
        // Use hardware encryption
        let result = self.peripheral.encrypt_block(&words)?;
        
        // Convert back to bytes
        for (i, &word) in result.iter().enumerate() {
            let bytes = word.to_le_bytes();
            block[i*4..(i+1)*4].copy_from_slice(&bytes);
        }
        
        Ok(())
    }
    
    fn decrypt_block(&mut self, block: &mut [u8; 16]) -> Result<(), Self::Error> {
        // Hardware decryption implementation
        // Similar to encrypt but with decryption mode
        self.peripheral.cryp.cr.modify(|_, w| w.algodir().decrypt());
        self.encrypt_block(block)?;
        self.peripheral.cryp.cr.modify(|_, w| w.algodir().encrypt());
        Ok(())
    }
    
    fn block_size(&self) -> usize {
        16
    }
}

// Software fallback implementation
struct SoftwareAes {
    key_schedule: [u32; 60],
    rounds: usize,
}

impl SoftwareAes {
    fn new(key: &[u8; 32]) -> Self {
        let mut key_schedule = [0u32; 60];
        let rounds = aes_key_expansion(key, &mut key_schedule);
        Self { key_schedule, rounds }
    }
}

impl BlockCipher for SoftwareAes {
    type Error = CryptoError;
    type Block = [u8; 16];
    
    fn encrypt_block(&mut self, block: &mut [u8; 16]) -> Result<(), Self::Error> {
        aes_encrypt_block(block, &self.key_schedule, self.rounds);
        Ok(())
    }
    
    fn decrypt_block(&mut self, block: &mut [u8; 16]) -> Result<(), Self::Error> {
        aes_decrypt_block(block, &self.key_schedule, self.rounds);
        Ok(())
    }
    
    fn block_size(&self) -> usize {
        16
    }
}

// Unified crypto engine with runtime hardware detection
pub struct CryptoEngine {
    aes_impl: AesImplementation,
    hardware_available: bool,
}

enum AesImplementation {
    Hardware(HardwareAes),
    Software(SoftwareAes),
}

impl CryptoEngine {
    pub fn new() -> Self {
        // Detect hardware crypto availability at runtime
        let hardware_available = detect_crypto_hardware();
        
        if hardware_available {
            // Initialize hardware crypto
            let peripheral = initialize_crypto_peripheral();
            let hw_aes = HardwareAes::new(peripheral, AesMode::ECB);
            Self {
                aes_impl: AesImplementation::Hardware(hw_aes),
                hardware_available: true,
            }
        } else {
            // Fallback to software implementation
            let sw_aes = SoftwareAes::new(&[0u8; 32]); // Placeholder key
            Self {
                aes_impl: AesImplementation::Software(sw_aes),
                hardware_available: false,
            }
        }
    }
    
    pub fn set_key(&mut self, key: &[u8; 32]) -> Result<(), CryptoError> {
        match &mut self.aes_impl {
            AesImplementation::Hardware(hw) => hw.set_key(key),
            AesImplementation::Software(sw) => {
                *sw = SoftwareAes::new(key);
                Ok(())
            }
        }
    }
    
    pub fn encrypt_block(&mut self, block: &mut [u8; 16]) -> Result<(), CryptoError> {
        match &mut self.aes_impl {
            AesImplementation::Hardware(hw) => hw.encrypt_block(block),
            AesImplementation::Software(sw) => sw.encrypt_block(block),
        }
    }
    
    pub fn is_hardware_accelerated(&self) -> bool {
        self.hardware_available
    }
}

// Hardware detection and initialization functions
fn detect_crypto_hardware() -> bool {
    // Platform-specific hardware detection
    #[cfg(feature = "stm32f4")]
    {
        // Check if CRYP peripheral is available
        true // Simplified for example
    }
    #[cfg(feature = "xilinx_r5")]
    {
        // Check for Xilinx crypto engines
        true
    }
    #[cfg(not(any(feature = "stm32f4", feature = "xilinx_r5")))]
    {
        false
    }
}

fn initialize_crypto_peripheral() -> CryptoPeripheral {
    // Platform-specific peripheral initialization
    #[cfg(feature = "stm32f4")]
    {
        let dp = pac::Peripherals::take().unwrap();
        CryptoPeripheral::new(dp.CRYP, Some(dp.DMA2))
    }
    #[cfg(not(feature = "stm32f4"))]
    {
        panic!("Hardware crypto not supported on this platform")
    }
}
```

#### Cross-Platform Hardware Abstraction

```rust
// Cross-platform crypto hardware abstraction
pub trait CryptoHardware {
    type Error;
    
    fn aes_encrypt(&mut self, key: &[u8; 32], block: &mut [u8; 16]) -> Result<(), Self::Error>;
    fn aes_decrypt(&mut self, key: &[u8; 32], block: &mut [u8; 16]) -> Result<(), Self::Error>;
    fn sha256(&mut self, data: &[u8]) -> Result<[u8; 32], Self::Error>;
    fn random_bytes(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error>;
}

// STM32 implementation
#[cfg(feature = "stm32")]
pub struct Stm32CryptoHardware {
    cryp: pac::CRYP,
    hash: pac::HASH,
    rng: pac::RNG,
}

#[cfg(feature = "stm32")]
impl CryptoHardware for Stm32CryptoHardware {
    type Error = CryptoError;
    
    fn aes_encrypt(&mut self, key: &[u8; 32], block: &mut [u8; 16]) -> Result<(), Self::Error> {
        // STM32-specific AES implementation
        todo!("Implement STM32 AES")
    }
    
    fn sha256(&mut self, data: &[u8]) -> Result<[u8; 32], Self::Error> {
        // STM32 HASH peripheral implementation
        todo!("Implement STM32 SHA256")
    }
    
    fn random_bytes(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        // STM32 RNG implementation
        todo!("Implement STM32 RNG")
    }
    
    fn aes_decrypt(&mut self, key: &[u8; 32], block: &mut [u8; 16]) -> Result<(), Self::Error> {
        todo!("Implement STM32 AES decrypt")
    }
}

// Xilinx implementation
#[cfg(feature = "xilinx")]
pub struct XilinxCryptoHardware {
    aes_engine: XilinxAes,
    sha_engine: XilinxSha,
    trng: XilinxTrng,
}

#[cfg(feature = "xilinx")]
impl CryptoHardware for XilinxCryptoHardware {
    type Error = CryptoError;
    
    fn aes_encrypt(&mut self, key: &[u8; 32], block: &mut [u8; 16]) -> Result<(), Self::Error> {
        self.aes_engine.encrypt(key, block)
    }
    
    fn sha256(&mut self, data: &[u8]) -> Result<[u8; 32], Self::Error> {
        self.sha_engine.hash_sha256(data)
    }
    
    fn random_bytes(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.trng.fill_bytes(buffer)
    }
    
    fn aes_decrypt(&mut self, key: &[u8; 32], block: &mut [u8; 16]) -> Result<(), Self::Error> {
        self.aes_engine.decrypt(key, block)
    }
}
```

### 4.3 Interrupt Handling {#interrupt-handling}

This section consolidates all interrupt handling patterns for embedded crypto applications, covering both basic interrupt handlers and advanced real-time frameworks.

#### Safe Interrupt Handling Fundamentals

```rust
use cortex_m_rt::interrupt;
use cortex_m::interrupt::{Mutex, free};
use core::cell::RefCell;
use heapless::spsc::{Queue, Producer, Consumer};

// Shared state between main code and interrupts - consolidated pattern
type SharedCryptoState = Mutex<RefCell<Option<CryptoContext>>>;
static CRYPTO_STATE: SharedCryptoState = Mutex::new(RefCell::new(None));

// Lock-free message queue for crypto operations
static mut CRYPTO_QUEUE_STORAGE: Queue<CryptoMessage, 32> = Queue::new();
static mut CRYPTO_PRODUCER: Option<Producer<CryptoMessage, 32>> = None;
static mut CRYPTO_CONSUMER: Option<Consumer<CryptoMessage, 32>> = None;

// Initialize interrupt-safe communication
fn init_crypto_interrupts() {
    let (producer, consumer) = unsafe { CRYPTO_QUEUE_STORAGE.split() };
    unsafe {
        CRYPTO_PRODUCER = Some(producer);
        CRYPTO_CONSUMER = Some(consumer);
    }
}

// Comprehensive crypto interrupt messages
#[derive(Clone, Copy)]
enum CryptoMessage {
    EncryptionComplete(CryptoResult),
    DecryptionComplete(CryptoResult),
    KeyExpired,
    HardwareError(u32),
    DmaTransferComplete,
    NonceCounterOverflow,
    AuthenticationFailed,
}

#[derive(Clone, Copy)]
struct CryptoResult {
    operation_id: u32,
    status: CryptoStatus,
    data_length: usize,
}

#[derive(Clone, Copy)]
enum CryptoStatus {
    Success,
    Failed,
    Timeout,
    InvalidInput,
}

// Hardware crypto interrupt handler
#[interrupt]
fn CRYPTO_IRQ() {
    free(|cs| {
        if let Some(ref mut ctx) = CRYPTO_STATE.borrow(cs).borrow_mut().as_mut() {
            // Handle different crypto hardware events
            let hw_status = ctx.get_hardware_status();
            
            match hw_status {
                HardwareStatus::EncryptionComplete => {
                    let result = ctx.get_encryption_result();
                    let message = CryptoMessage::EncryptionComplete(result);
                    
                    // Send to main thread via lock-free queue
                    unsafe {
                        if let Some(ref mut producer) = CRYPTO_PRODUCER {
                            let _ = producer.enqueue(message);
                        }
                    }
                }
                HardwareStatus::DecryptionComplete => {
                    let result = ctx.get_decryption_result();
                    let message = CryptoMessage::DecryptionComplete(result);
                    
                    unsafe {
                        if let Some(ref mut producer) = CRYPTO_PRODUCER {
                            let _ = producer.enqueue(message);
                        }
                    }
                }
                HardwareStatus::Error(error_code) => {
                    let message = CryptoMessage::HardwareError(error_code);
                    unsafe {
                        if let Some(ref mut producer) = CRYPTO_PRODUCER {
                            let _ = producer.enqueue(message);
                        }
                    }
                    
                    // Clear hardware error state
                    ctx.clear_error_state();
                }
                HardwareStatus::Idle => {
                    // No action needed
                }
            }
        }
    });
}

// Timer interrupt for crypto operations timing and key management
#[interrupt]
fn TIM2() {
    free(|cs| {
        if let Some(ref mut ctx) = CRYPTO_STATE.borrow(cs).borrow_mut().as_mut() {
            // Check for nonce counter overflow (critical security check)
            if ctx.is_nonce_counter_near_overflow() {
                let message = CryptoMessage::NonceCounterOverflow;
                unsafe {
                    if let Some(ref mut producer) = CRYPTO_PRODUCER {
                        let _ = producer.enqueue(message);
                    }
                }
            }
            
            // Check for key expiration
            if ctx.is_key_expired() {
                let message = CryptoMessage::KeyExpired;
                unsafe {
                    if let Some(ref mut producer) = CRYPTO_PRODUCER {
                        let _ = producer.enqueue(message);
                    }
                }
            }
            
            // Update timing-sensitive crypto state
            ctx.update_timing_state();
        }
    });
}

// DMA interrupt for large crypto operations
#[interrupt]
fn DMA1_STREAM0() {
    free(|cs| {
        if let Some(ref mut ctx) = CRYPTO_STATE.borrow(cs).borrow_mut().as_mut() {
            if ctx.is_dma_transfer_complete() {
                let message = CryptoMessage::DmaTransferComplete;
                unsafe {
                    if let Some(ref mut producer) = CRYPTO_PRODUCER {
                        let _ = producer.enqueue(message);
                    }
                }
                
                // Process completed DMA crypto operation
                ctx.process_dma_result();
            }
        }
    });
}

// Main thread crypto interrupt processing
fn process_crypto_interrupts() -> Result<(), CryptoError> {
    unsafe {
        if let Some(ref mut consumer) = CRYPTO_CONSUMER {
            while let Some(message) = consumer.dequeue() {
                match message {
                    CryptoMessage::EncryptionComplete(result) => {
                        handle_encryption_result(result)?;
                    }
                    CryptoMessage::DecryptionComplete(result) => {
                        handle_decryption_result(result)?;
                    }
                    CryptoMessage::KeyExpired => {
                        regenerate_session_keys()?;
                    }
                    CryptoMessage::HardwareError(error_code) => {
                        handle_hardware_error(error_code)?;
                    }
                    CryptoMessage::DmaTransferComplete => {
                        finalize_dma_crypto_operation()?;
                    }
                    CryptoMessage::NonceCounterOverflow => {
                        // Critical: Must regenerate keys immediately
                        emergency_key_regeneration()?;
                    }
                    CryptoMessage::AuthenticationFailed => {
                        handle_authentication_failure()?;
                    }
                }
            }
        }
    }
    Ok(())
}
```

#### RTIC Framework for Real-Time Crypto

```rust
// Advanced real-time crypto using RTIC framework
#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1])]
mod app {
    use super::*;
    use rtic::time::duration::*;
    
    #[shared]
    struct Shared {
        crypto_context: CryptoContext,
        session_keys: SessionKeys,
        crypto_statistics: CryptoStatistics,
    }
    
    #[local]
    struct Local {
        crypto_hardware: CryptoPeripheral,
        timer: Timer<TIM2>,
        dma_controller: DmaController,
        led: Led,
    }
    
    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let dp = ctx.device;
        
        // Initialize crypto hardware with comprehensive setup
        let crypto_hw = CryptoPeripheral::new(dp.CRYP, Some(dp.DMA2));
        let timer = Timer::new(dp.TIM2);
        let dma = DmaController::new(dp.DMA1);
        let led = Led::new(dp.GPIOA.pa5);
        
        // Initialize crypto context with secure defaults
        let crypto_ctx = CryptoContext::new_secure();
        let session_keys = SessionKeys::generate_initial();
        let stats = CryptoStatistics::new();
        
        // Schedule periodic key rotation
        key_rotation::spawn_after(Seconds(300u32)).ok(); // Every 5 minutes
        
        (
            Shared {
                crypto_context: crypto_ctx,
                session_keys,
                crypto_statistics: stats,
            },
            Local {
                crypto_hardware: crypto_hw,
                timer,
                dma_controller: dma,
                led,
            },
            init::Monotonics(),
        )
    }
    
    // Highest priority - crypto hardware interrupt
    #[task(binds = CRYP, priority = 4, shared = [crypto_context, crypto_statistics], local = [crypto_hardware])]
    fn crypto_interrupt(mut ctx: crypto_interrupt::Context) {
        let hw = ctx.local.crypto_hardware;
        
        (ctx.shared.crypto_context, ctx.shared.crypto_statistics).lock(|crypto_ctx, stats| {
            if hw.is_encryption_ready() {
                let result = hw.get_encryption_result();
                crypto_ctx.process_encryption_result(result);
                stats.record_encryption_complete();
                
                // Spawn low-priority task to handle result
                crypto_result_handler::spawn(result).ok();
            }
            
            if hw.is_decryption_ready() {
                let result = hw.get_decryption_result();
                crypto_ctx.process_decryption_result(result);
                stats.record_decryption_complete();
                
                crypto_result_handler::spawn(result).ok();
            }
            
            if hw.has_error() {
                let error = hw.get_error();
                stats.record_error(error);
                
                // Handle error immediately at high priority
                crypto_error_handler::spawn(error).ok();
            }
        });
    }
    
    // High priority - DMA completion
    #[task(binds = DMA1_STREAM0, priority = 3, shared = [crypto_context])]
    fn dma_interrupt(mut ctx: dma_interrupt::Context) {
        ctx.shared.crypto_context.lock(|crypto_ctx| {
            crypto_ctx.handle_dma_completion();
        });
        
        // Spawn task to process DMA result
        dma_result_handler::spawn().ok();
    }
    
    // Medium priority - key management and security
    #[task(priority = 2, shared = [session_keys, crypto_context, crypto_statistics])]
    fn key_rotation(mut ctx: key_rotation::Context) {
        (ctx.shared.session_keys, ctx.shared.crypto_context, ctx.shared.crypto_statistics).lock(
            |keys, crypto_ctx, stats| {
                // Generate new session keys
                let new_keys = SessionKeys::generate_rotated(&keys);
                
                // Update crypto context with new keys
                crypto_ctx.update_keys(&new_keys);
                
                // Securely zeroize old keys
                *keys = new_keys;
                
                stats.record_key_rotation();
            }
        );
        
        // Schedule next key rotation
        key_rotation::spawn_after(Seconds(300u32)).ok();
    }
    
    // Medium priority - nonce management
    #[task(priority = 2, shared = [crypto_context])]
    fn nonce_overflow_handler(mut ctx: nonce_overflow_handler::Context) {
        ctx.shared.crypto_context.lock(|crypto_ctx| {
            // Critical: Reset nonce counter and regenerate keys
            crypto_ctx.handle_nonce_overflow();
        });
        
        // Force immediate key rotation
        key_rotation::spawn().ok();
    }
    
    // Low priority - crypto result processing
    #[task(priority = 1, capacity = 8)]
    fn crypto_result_handler(_ctx: crypto_result_handler::Context, result: CryptoResult) {
        // Process crypto results without blocking high-priority operations
        match result.status {
            CryptoStatus::Success => {
                transmit_crypto_result(result);
            }
            CryptoStatus::Failed => {
                log_crypto_failure(result);
                retry_crypto_operation(result);
            }
            CryptoStatus::Timeout => {
                handle_crypto_timeout(result);
            }
            CryptoStatus::InvalidInput => {
                handle_invalid_input(result);
            }
        }
    }
    
    // Medium priority - error handling
    #[task(priority = 2, shared = [crypto_statistics], local = [led])]
    fn crypto_error_handler(mut ctx: crypto_error_handler::Context, error: u32) {
        // Visual indication of crypto error
        ctx.local.led.set_high();
        
        ctx.shared.crypto_statistics.lock(|stats| {
            stats.record_critical_error(error);
        });
        
        // Handle different error types
        match error {
            0x01 => handle_key_error(),
            0x02 => handle_hardware_fault(),
            0x03 => handle_dma_error(),
            _ => handle_unknown_error(error),
        }
        
        // Clear error indication after handling
        ctx.local.led.set_low();
    }
    
    // Low priority - DMA result processing
    #[task(priority = 1)]
    fn dma_result_handler(_ctx: dma_result_handler::Context) {
        // Process completed DMA crypto operations
        finalize_dma_crypto_operation();
    }
    
    // Lowest priority - statistics and monitoring
    #[task(priority = 0, shared = [crypto_statistics])]
    fn statistics_reporter(mut ctx: statistics_reporter::Context) {
        ctx.shared.crypto_statistics.lock(|stats| {
            report_crypto_statistics(stats);
            stats.reset_periodic_counters();
        });
        
        // Schedule next statistics report
        statistics_reporter::spawn_after(Seconds(60u32)).ok();
    }
}

// Supporting structures for RTIC crypto application
#[derive(ZeroizeOnDrop)]
struct SessionKeys {
    encryption_key: [u8; 32],
    mac_key: [u8; 32],
    key_id: u32,
    creation_time: u64,
}

impl SessionKeys {
    fn generate_initial() -> Self {
        Self {
            encryption_key: generate_random_key(),
            mac_key: generate_random_key(),
            key_id: 1,
            creation_time: get_current_time(),
        }
    }
    
    fn generate_rotated(&self) -> Self {
        Self {
            encryption_key: generate_random_key(),
            mac_key: generate_random_key(),
            key_id: self.key_id + 1,
            creation_time: get_current_time(),
        }
    }
}

struct CryptoStatistics {
    encryptions_completed: u32,
    decryptions_completed: u32,
    errors_encountered: u32,
    key_rotations: u32,
    last_reset_time: u64,
}

impl CryptoStatistics {
    fn new() -> Self {
        Self {
            encryptions_completed: 0,
            decryptions_completed: 0,
            errors_encountered: 0,
            key_rotations: 0,
            last_reset_time: get_current_time(),
        }
    }
    
    fn record_encryption_complete(&mut self) {
        self.encryptions_completed += 1;
    }
    
    fn record_decryption_complete(&mut self) {
        self.decryptions_completed += 1;
    }
    
    fn record_error(&mut self, _error: u32) {
        self.errors_encountered += 1;
    }
    
    fn record_key_rotation(&mut self) {
        self.key_rotations += 1;
    }
    
    fn record_critical_error(&mut self, _error: u32) {
        self.errors_encountered += 1;
        // Additional critical error handling
    }
    
    fn reset_periodic_counters(&mut self) {
        self.last_reset_time = get_current_time();
        // Reset counters that should be periodic
    }
}
```

#### Interrupt Priority and Timing Considerations

```rust
// Interrupt priority configuration for crypto applications
const CRYPTO_HW_PRIORITY: u8 = 0;      // Highest - hardware crypto
const DMA_PRIORITY: u8 = 1;             // High - DMA completion
const TIMER_PRIORITY: u8 = 2;           // Medium - timing and key management
const COMM_PRIORITY: u8 = 3;            // Lower - communication
const BACKGROUND_PRIORITY: u8 = 4;      // Lowest - background tasks

// Configure interrupt priorities for optimal crypto performance
fn configure_crypto_interrupt_priorities() {
    unsafe {
        // Set crypto hardware interrupt to highest priority
        cortex_m::peripheral::NVIC::set_priority(
            stm32f4xx_pac::Interrupt::CRYP, 
            CRYPTO_HW_PRIORITY
        );
        
        // Set DMA interrupt to high priority
        cortex_m::peripheral::NVIC::set_priority(
            stm32f4xx_pac::Interrupt::DMA1_STREAM0, 
            DMA_PRIORITY
        );
        
        // Set timer interrupt for key management
        cortex_m::peripheral::NVIC::set_priority(
            stm32f4xx_pac::Interrupt::TIM2, 
            TIMER_PRIORITY
        );
    }
}

// Critical section helpers for crypto operations
fn with_crypto_interrupts_disabled<F, R>(f: F) -> R 
where 
    F: FnOnce() -> R,
{
    cortex_m::interrupt::free(|_| f())
}

// Atomic operations for crypto state
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

static CRYPTO_OPERATION_COUNT: AtomicU32 = AtomicU32::new(0);
static CRYPTO_HARDWARE_BUSY: AtomicBool = AtomicBool::new(false);

fn start_crypto_operation() -> Result<u32, CryptoError> {
    // Atomically check and set hardware busy flag
    if CRYPTO_HARDWARE_BUSY.compare_exchange(
        false, 
        true, 
        Ordering::Acquire, 
        Ordering::Relaxed
    ).is_err() {
        return Err(CryptoError::HardwareBusy);
    }
    
    // Increment operation counter
    let op_id = CRYPTO_OPERATION_COUNT.fetch_add(1, Ordering::Relaxed);
    Ok(op_id)
}

fn complete_crypto_operation() {
    // Clear hardware busy flag
    CRYPTO_HARDWARE_BUSY.store(false, Ordering::Release);
}
```

### 4.4 Static Memory Management {#static-memory-management}

Effective static memory management is essential for deterministic embedded crypto applications.

#### Static Allocation Patterns

```rust
use heapless::pool::{Pool, Node, Singleton};

// Static memory pools for different crypto operations
static mut SMALL_BLOCKS: [Node<[u8; 256]>; 8] = [Node::new(); 8];
static mut LARGE_BLOCKS: [Node<[u8; 4096]>; 2] = [Node::new(); 2];

static SMALL_POOL: Pool<[u8; 256]> = Pool::new();
static LARGE_POOL: Pool<[u8; 4096]> = Pool::new();

// Singleton for crypto context (only one instance allowed)
static mut CRYPTO_CONTEXT_MEM: [u8; 1024] = [0; 1024];
static CRYPTO_CONTEXT: Singleton<CryptoContext> = Singleton::new();

fn init_static_memory() {
    // Initialize memory pools
    unsafe {
        SMALL_POOL.grow(&mut SMALL_BLOCKS);
        LARGE_POOL.grow(&mut LARGE_BLOCKS);
    }
    
    // Initialize singleton crypto context
    let ctx = CryptoContext::new();
    CRYPTO_CONTEXT.spawn(ctx).ok();
}

// Memory-efficient crypto operations
fn encrypt_with_static_memory(data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    // Choose appropriate pool based on data size
    if data.len() <= 256 {
        let mut buffer = SMALL_POOL.alloc().ok_or(CryptoError::OutOfMemory)?;
        buffer[..data.len()].copy_from_slice(data);
        
        // Perform in-place encryption
        encrypt_in_place(&mut buffer[..data.len()])?;
        
        // Copy result (buffer automatically returned to pool when dropped)
        Ok(buffer[..data.len()].to_vec())
    } else if data.len() <= 4096 {
        let mut buffer = LARGE_POOL.alloc().ok_or(CryptoError::OutOfMemory)?;
        // Similar process for large buffers
        todo!("Implement large buffer encryption")
    } else {
        Err(CryptoError::DataTooLarge)
    }
}
```

#### Compile-Time Memory Layout

```rust
// Use const generics for compile-time memory layout
struct CryptoWorkspace<const BUFFER_SIZE: usize, const KEY_COUNT: usize> {
    buffers: [[u8; BUFFER_SIZE]; 4],
    keys: [[u8; 32]; KEY_COUNT],
    nonces: [[u8; 12]; KEY_COUNT],
}

impl<const BUFFER_SIZE: usize, const KEY_COUNT: usize> CryptoWorkspace<BUFFER_SIZE, KEY_COUNT> {
    const fn new() -> Self {
        Self {
            buffers: [[0; BUFFER_SIZE]; 4],
            keys: [[0; 32]; KEY_COUNT],
            nonces: [[0; 12]; KEY_COUNT],
        }
    }
    
    fn get_buffer(&mut self, index: usize) -> Option<&mut [u8; BUFFER_SIZE]> {
        self.buffers.get_mut(index)
    }
    
    fn get_key_pair(&self, index: usize) -> Option<(&[u8; 32], &[u8; 12])> {
        if index < KEY_COUNT {
            Some((&self.keys[index], &self.nonces[index]))
        } else {
            None
        }
    }
}

// Different configurations for different applications
type SmallCryptoWorkspace = CryptoWorkspace<256, 2>;   // 2KB total
type LargeCryptoWorkspace = CryptoWorkspace<4096, 8>;  // 16KB+ total

static mut CRYPTO_WS: SmallCryptoWorkspace = SmallCryptoWorkspace::new();
```

### 4.5 DMA and Hardware Integration {#dma-and-hardware-integration}

Direct Memory Access (DMA) is crucial for high-performance crypto operations without CPU intervention.

#### DMA-Safe Memory Management

```rust
use cortex_m::singleton;

// DMA-safe buffer allocation
fn allocate_dma_buffer() -> Option<&'static mut [u8; 1024]> {
    // Singleton ensures only one DMA buffer exists
    singleton!(: [u8; 1024] = [0; 1024])
}

// DMA crypto operations
struct DmaCrypto {
    dma_channel: DmaChannel,
    crypto_peripheral: CryptoPeripheral,
}

impl DmaCrypto {
    fn encrypt_async(&mut self, data: &'static mut [u8]) -> Result<(), DmaError> {
        // Configure DMA for crypto operation
        self.dma_channel.configure(
            data.as_ptr() as u32,           // Source address
            self.crypto_peripheral.data_register(), // Destination
            data.len(),                     // Transfer count
            DmaDirection::MemoryToPeripheral,
        );
        
        // Start DMA transfer
        self.dma_channel.start();
        
        // Crypto operation will complete via DMA interrupt
        Ok(())
    }
    
    fn is_complete(&self) -> bool {
        self.dma_channel.is_complete()
    }
    
    fn get_result(&mut self) -> Option<&'static mut [u8]> {
        if self.is_complete() {
            // Return processed buffer
            Some(self.dma_channel.get_buffer())
        } else {
            None
        }
    }
}

// Interrupt handler for DMA completion
#[interrupt]
fn DMA1_STREAM0() {
    static mut DMA_CRYPTO: Option<DmaCrypto> = None;
    
    if let Some(ref mut crypto) = DMA_CRYPTO {
        if crypto.is_complete() {
            // Process completed crypto operation
            if let Some(result) = crypto.get_result() {
                handle_crypto_result(result);
            }
        }
    }
}
```

**→ Next:** [Cryptography Implementation](#cryptography-implementation) - Secure coding patterns and crypto-specific implementations

---

*This completes the core structure and navigation framework. The document now has:*

1. **Clear hierarchical structure** with 6 main sections
2. **Comprehensive table of contents** with deep linking
3. **Cross-reference system** with navigation hints
4. **Consolidated content** eliminating redundancy
5. **Quick reference section** for immediate productivity
6. **Embedded-specific focus** throughout

*The remaining sections (5 and 6) would continue with the same structure and cross-referencing approach.*## 5. C
ryptography Implementation {#cryptography-implementation}

This section focuses on implementing cryptographic algorithms and protocols in Rust, with emphasis on security, performance, and embedded constraints.

### 5.1 Secure Coding Patterns {#secure-coding-patterns}

Rust provides unique advantages for secure cryptographic implementations through its type system and memory safety guarantees.

#### Automatic Key Zeroization

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
struct SessionKeys {
    encryption_key: [u8; 32],
    mac_key: [u8; 32],
    iv: [u8; 16],
}

impl SessionKeys {
    fn derive_from_master(master_key: &[u8; 32], context: &[u8]) -> Self {
        // Key derivation implementation
        let mut keys = Self {
            encryption_key: [0; 32],
            mac_key: [0; 32],
            iv: [0; 16],
        };
        
        // HKDF or similar key derivation
        hkdf_expand(master_key, context, &mut keys.encryption_key, b"encrypt");
        hkdf_expand(master_key, context, &mut keys.mac_key, b"mac");
        hkdf_expand(master_key, context, &mut keys.iv, b"iv");
        
        keys
    }
    
    fn encrypt_and_mac(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Use keys for encryption and authentication
        let ciphertext = aes_encrypt(&self.encryption_key, &self.iv, plaintext)?;
        let mac = hmac_sha256(&self.mac_key, &ciphertext)?;
        
        // Combine ciphertext and MAC
        let mut result = Vec::with_capacity(ciphertext.len() + mac.len());
        result.extend_from_slice(&ciphertext);
        result.extend_from_slice(&mac);
        Ok(result)
    }
}

// Keys automatically zeroized when SessionKeys is dropped
fn secure_session_example() {
    let master_key = [0u8; 32]; // From secure key exchange
    let session_keys = SessionKeys::derive_from_master(&master_key, b"session_1");
    
    let encrypted = session_keys.encrypt_and_mac(b"secret message").unwrap();
    
    // session_keys automatically zeroized here
}
```

#### Type-Safe Protocol States

```rust
// Use phantom types to enforce protocol state machine
use core::marker::PhantomData;

struct Uninitialized;
struct HandshakeInProgress;
struct SessionEstablished;

struct SecureChannel<State> {
    socket: Socket,
    session_keys: Option<SessionKeys>,
    state: PhantomData<State>,
}

impl SecureChannel<Uninitialized> {
    fn new(socket: Socket) -> Self {
        Self {
            socket,
            session_keys: None,
            state: PhantomData,
        }
    }
    
    fn start_handshake(self) -> SecureChannel<HandshakeInProgress> {
        SecureChannel {
            socket: self.socket,
            session_keys: None,
            state: PhantomData,
        }
    }
}

impl SecureChannel<HandshakeInProgress> {
    fn complete_handshake(mut self, keys: SessionKeys) -> SecureChannel<SessionEstablished> {
        self.session_keys = Some(keys);
        SecureChannel {
            socket: self.socket,
            session_keys: self.session_keys,
            state: PhantomData,
        }
    }
}

impl SecureChannel<SessionEstablished> {
    fn send_encrypted(&mut self, data: &[u8]) -> Result<(), CryptoError> {
        let keys = self.session_keys.as_ref().unwrap();
        let encrypted = keys.encrypt_and_mac(data)?;
        self.socket.send(&encrypted)?;
        Ok(())
    }
    
    fn receive_encrypted(&mut self) -> Result<Vec<u8>, CryptoError> {
        let encrypted = self.socket.receive()?;
        let keys = self.session_keys.as_ref().unwrap();
        keys.decrypt_and_verify(&encrypted)
    }
}
```

### 5.2 Constant-Time Implementations {#constant-time-implementations}

Constant-time implementations are crucial for preventing timing attacks in cryptographic code.

#### Using the `subtle` Crate

```rust
use subtle::{Choice, ConstantTimeEq, ConditionallySelectable};

// Constant-time comparison
fn verify_mac_constant_time(expected: &[u8], received: &[u8]) -> bool {
    if expected.len() != received.len() {
        return false;
    }
    
    // Use constant-time comparison to prevent timing attacks
    expected.ct_eq(received).into()
}

// Constant-time conditional selection
fn conditional_key_selection(condition: bool, key_a: &[u8; 32], key_b: &[u8; 32]) -> [u8; 32] {
    let choice = Choice::from(condition as u8);
    let mut result = [0u8; 32];
    
    for i in 0..32 {
        result[i] = u8::conditional_select(&key_a[i], &key_b[i], choice);
    }
    
    result
}

// Constant-time array lookup
fn constant_time_lookup(table: &[[u8; 32]], index: usize) -> [u8; 32] {
    let mut result = [0u8; 32];
    
    for (i, entry) in table.iter().enumerate() {
        let choice = Choice::from((i == index) as u8);
        for j in 0..32 {
            result[j] = u8::conditional_select(&result[j], &entry[j], choice);
        }
    }
    
    result
}

// Constant-time modular arithmetic
struct ConstantTimeScalar([u64; 4]); // 256-bit scalar

impl ConstantTimeScalar {
    fn conditional_negate(&mut self, choice: Choice) {
        // Constant-time conditional negation
        let mask = u64::conditional_select(&0, &u64::MAX, choice);
        
        for limb in &mut self.0 {
            *limb ^= mask;
        }
        
        // Add 1 if negating (two's complement)
        let mut carry = choice.into() as u64;
        for limb in &mut self.0 {
            let (new_limb, new_carry) = limb.overflowing_add(carry);
            *limb = new_limb;
            carry = new_carry as u64;
        }
    }
}
```

#### Manual Constant-Time Patterns

```rust
// Constant-time byte array operations
fn constant_time_memcmp(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    
    result == 0
}

fn constant_time_select_byte(condition: bool, a: u8, b: u8) -> u8 {
    let mask = (condition as u8).wrapping_neg(); // 0x00 or 0xFF
    (a & mask) | (b & !mask)
}

// Constant-time conditional swap
fn conditional_swap(condition: bool, a: &mut [u8], b: &mut [u8]) {
    assert_eq!(a.len(), b.len());
    
    let mask = (condition as u8).wrapping_neg();
    
    for (x, y) in a.iter_mut().zip(b.iter_mut()) {
        let temp = *x;
        *x = (*x & !mask) | (*y & mask);
        *y = (*y & !mask) | (temp & mask);
    }
}
```

### 5.3 Key Management and Zeroization {#key-management-and-zeroization}

Proper key management is critical for cryptographic security, especially in embedded systems.

#### Hierarchical Key Derivation

```rust
use hkdf::Hkdf;
use sha2::Sha256;

#[derive(ZeroizeOnDrop)]
struct KeyHierarchy {
    master_key: [u8; 32],
    derived_keys: heapless::FnvIndexMap<&'static str, [u8; 32], 8>,
}

impl KeyHierarchy {
    fn new(master_key: [u8; 32]) -> Self {
        Self {
            master_key,
            derived_keys: heapless::FnvIndexMap::new(),
        }
    }
    
    fn derive_key(&mut self, purpose: &'static str) -> Result<&[u8; 32], CryptoError> {
        if self.derived_keys.contains_key(purpose) {
            return Ok(&self.derived_keys[purpose]);
        }
        
        let hk = Hkdf::<Sha256>::new(None, &self.master_key);
        let mut derived = [0u8; 32];
        hk.expand(purpose.as_bytes(), &mut derived)
            .map_err(|_| CryptoError::KeyDerivationFailed)?;
        
        self.derived_keys.insert(purpose, derived)
            .map_err(|_| CryptoError::TooManyKeys)?;
        
        Ok(&self.derived_keys[purpose])
    }
    
    fn get_encryption_key(&mut self) -> Result<&[u8; 32], CryptoError> {
        self.derive_key("encryption")
    }
    
    fn get_mac_key(&mut self) -> Result<&[u8; 32], CryptoError> {
        self.derive_key("authentication")
    }
    
    fn rotate_master_key(&mut self, new_master: [u8; 32]) {
        // Clear all derived keys
        self.derived_keys.clear();
        
        // Update master key (old one automatically zeroized)
        self.master_key = new_master;
    }
}
```

#### Secure Random Number Generation

```rust
use rand_core::{RngCore, CryptoRng};

// Hardware RNG wrapper
struct HardwareRng {
    rng_peripheral: RngPeripheral,
}

impl RngCore for HardwareRng {
    fn next_u32(&mut self) -> u32 {
        // Wait for hardware RNG to be ready
        while !self.rng_peripheral.is_ready() {
            cortex_m::asm::nop();
        }
        
        self.rng_peripheral.read_random()
    }
    
    fn next_u64(&mut self) -> u64 {
        let low = self.next_u32() as u64;
        let high = self.next_u32() as u64;
        (high << 32) | low
    }
    
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(4) {
            let random = self.next_u32();
            let bytes = random.to_le_bytes();
            
            for (i, &byte) in bytes.iter().enumerate() {
                if i < chunk.len() {
                    chunk[i] = byte;
                }
            }
        }
    }
    
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for HardwareRng {}

// Key generation using hardware RNG
fn generate_session_keys(rng: &mut impl CryptoRng) -> SessionKeys {
    let mut master_key = [0u8; 32];
    rng.fill_bytes(&mut master_key);
    
    SessionKeys::derive_from_master(&master_key, b"session")
}
```

### 5.4 Hardware Crypto Acceleration {#hardware-crypto-acceleration}

Leveraging hardware crypto acceleration while maintaining portability and security.

#### Generic Hardware Abstraction

```rust
// Generic trait for crypto acceleration
trait CryptoAccelerator {
    type Error;
    
    fn aes_encrypt(&mut self, key: &[u8; 32], plaintext: &[u8], ciphertext: &mut [u8]) -> Result<(), Self::Error>;
    fn aes_decrypt(&mut self, key: &[u8; 32], ciphertext: &[u8], plaintext: &mut [u8]) -> Result<(), Self::Error>;
    fn sha256(&mut self, data: &[u8], hash: &mut [u8; 32]) -> Result<(), Self::Error>;
}

// Hardware implementation for Xilinx
struct XilinxCryptoAccel {
    aes_engine: XilinxAesEngine,
    hash_engine: XilinxHashEngine,
}

impl CryptoAccelerator for XilinxCryptoAccel {
    type Error = HardwareError;
    
    fn aes_encrypt(&mut self, key: &[u8; 32], plaintext: &[u8], ciphertext: &mut [u8]) -> Result<(), Self::Error> {
        self.aes_engine.load_key(key)?;
        self.aes_engine.encrypt(plaintext, ciphertext)?;
        Ok(())
    }
    
    fn sha256(&mut self, data: &[u8], hash: &mut [u8; 32]) -> Result<(), Self::Error> {
        self.hash_engine.hash_sha256(data, hash)
    }
    
    // ... other implementations
}

// Software fallback
struct SoftwareCrypto;

impl CryptoAccelerator for SoftwareCrypto {
    type Error = CryptoError;
    
    fn aes_encrypt(&mut self, key: &[u8; 32], plaintext: &[u8], ciphertext: &mut [u8]) -> Result<(), Self::Error> {
        // Use software AES implementation
        software_aes_encrypt(key, plaintext, ciphertext)
    }
    
    fn sha256(&mut self, data: &[u8], hash: &mut [u8; 32]) -> Result<(), Self::Error> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hash.copy_from_slice(&hasher.finalize());
        Ok(())
    }
    
    // ... other implementations
}

// Adaptive crypto engine
enum CryptoEngine {
    Hardware(XilinxCryptoAccel),
    Software(SoftwareCrypto),
}

impl CryptoEngine {
    fn new() -> Self {
        // Try to initialize hardware acceleration
        if let Ok(hw_accel) = XilinxCryptoAccel::new() {
            CryptoEngine::Hardware(hw_accel)
        } else {
            CryptoEngine::Software(SoftwareCrypto)
        }
    }
    
    fn encrypt_aes(&mut self, key: &[u8; 32], plaintext: &[u8], ciphertext: &mut [u8]) -> Result<(), Box<dyn core::error::Error>> {
        match self {
            CryptoEngine::Hardware(hw) => hw.aes_encrypt(key, plaintext, ciphertext).map_err(|e| Box::new(e) as Box<dyn core::error::Error>),
            CryptoEngine::Software(sw) => sw.aes_encrypt(key, plaintext, ciphertext).map_err(|e| Box::new(e) as Box<dyn core::error::Error>),
        }
    }
}
```

### 5.5 Side-Channel Mitigations {#side-channel-mitigations}

Protecting against side-channel attacks is crucial for embedded cryptographic implementations.

#### Power Analysis Protection

```rust
// Randomized execution timing
fn randomized_delay(rng: &mut impl RngCore) {
    let delay_cycles = rng.next_u32() & 0xFF; // 0-255 cycles
    for _ in 0..delay_cycles {
        cortex_m::asm::nop();
    }
}

// Blinded operations
fn blinded_scalar_multiply(scalar: &[u8; 32], point: &EccPoint, rng: &mut impl RngCore) -> EccPoint {
    // Generate random blinding factor
    let mut blind = [0u8; 32];
    rng.fill_bytes(&mut blind);
    
    // Blind the scalar
    let blinded_scalar = scalar_add(scalar, &blind);
    
    // Perform blinded operation
    let blinded_result = scalar_multiply(&blinded_scalar, point);
    
    // Remove blinding
    let blind_point = scalar_multiply(&blind, point);
    point_subtract(&blinded_result, &blind_point)
}

// Constant-time table lookups with dummy operations
fn protected_table_lookup(table: &[[u8; 32]], index: usize, rng: &mut impl RngCore) -> [u8; 32] {
    let mut result = [0u8; 32];
    
    // Perform dummy operations to mask the real lookup
    let dummy_index = rng.next_u32() as usize % table.len();
    let mut dummy_result = [0u8; 32];
    
    for (i, entry) in table.iter().enumerate() {
        let real_choice = Choice::from((i == index) as u8);
        let dummy_choice = Choice::from((i == dummy_index) as u8);
        
        // Real lookup
        for j in 0..32 {
            result[j] = u8::conditional_select(&result[j], &entry[j], real_choice);
        }
        
        // Dummy lookup (result discarded)
        for j in 0..32 {
            dummy_result[j] = u8::conditional_select(&dummy_result[j], &entry[j], dummy_choice);
        }
    }
    
    result
}
```

---

## 6. Migration and Integration {#migration-and-integration}

This section provides comprehensive guidance for migrating from C to Rust and integrating Rust crypto code with existing systems.

### 6.1 Incremental Migration Strategies {#incremental-migration-strategies}

Migrating large cryptographic codebases requires careful planning and incremental approaches.

#### Module-by-Module Migration

```rust
// Start with leaf modules - crypto primitives
// Original C: aes.c, aes.h
// New Rust: crypto/aes.rs

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
        
        aes_key_expansion(key, &mut ctx.key_schedule, ctx.rounds);
        Ok(ctx)
    }
    
    pub fn encrypt_block(&self, input: &[u8; 16], output: &mut [u8; 16]) {
        aes_encrypt_block(input, output, &self.key_schedule, self.rounds);
    }
}

// C-compatible interface for gradual migration
#[no_mangle]
pub extern "C" fn aes_context_new(key: *const u8, key_len: usize) -> *mut AesContext {
    if key.is_null() {
        return core::ptr::null_mut();
    }
    
    let key_slice = unsafe { core::slice::from_raw_parts(key, key_len) };
    
    match AesContext::new(key_slice) {
        Ok(ctx) => Box::into_raw(Box::new(ctx)),
        Err(_) => core::ptr::null_mut(),
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
        
        let input_array: [u8; 16] = input_block.try_into().unwrap();
        let mut output_array = [0u8; 16];
        
        ctx.encrypt_block(&input_array, &mut output_array);
        output_block.copy_from_slice(&output_array);
    }
    
    0 // Success
}
```

#### Protocol-Level Migration

```rust
// Migrate high-level protocols while keeping C crypto primitives
// This allows testing new protocol logic with proven crypto

extern "C" {
    fn c_aes_encrypt(key: *const u8, plaintext: *const u8, ciphertext: *mut u8, len: usize) -> i32;
    fn c_hmac_sha256(key: *const u8, data: *const u8, data_len: usize, mac: *mut u8) -> i32;
}

// Rust protocol implementation using C crypto
struct HybridTlsConnection {
    state: TlsState,
    session_keys: SessionKeys,
}

impl HybridTlsConnection {
    fn encrypt_record(&self, record_type: u8, data: &[u8]) -> Result<Vec<u8>, TlsError> {
        let mut ciphertext = vec![0u8; data.len()];
        
        // Use C crypto function temporarily
        let result = unsafe {
            c_aes_encrypt(
                self.session_keys.encryption_key.as_ptr(),
                data.as_ptr(),
                ciphertext.as_mut_ptr(),
                data.len(),
            )
        };
        
        if result != 0 {
            return Err(TlsError::EncryptionFailed);
        }
        
        // Rust-based MAC calculation (gradually migrating)
        let mac = self.calculate_mac(record_type, &ciphertext)?;
        
        ciphertext.extend_from_slice(&mac);
        Ok(ciphertext)
    }
    
    fn calculate_mac(&self, record_type: u8, data: &[u8]) -> Result<[u8; 32], TlsError> {
        // This could be pure Rust or hybrid C/Rust
        let mut mac = [0u8; 32];
        
        let result = unsafe {
            c_hmac_sha256(
                self.session_keys.mac_key.as_ptr(),
                data.as_ptr(),
                data.len(),
                mac.as_mut_ptr(),
            )
        };
        
        if result == 0 {
            Ok(mac)
        } else {
            Err(TlsError::MacFailed)
        }
    }
}
```

### 6.2 FFI Integration with C Libraries {#ffi-integration-with-c-libraries}

Interfacing with existing C cryptographic libraries during migration.

#### Safe FFI Wrappers

```rust
// Safe wrapper for OpenSSL or similar C crypto library
use core::ffi::{c_int, c_void, c_char};

// External C functions
extern "C" {
    fn EVP_CIPHER_CTX_new() -> *mut c_void;
    fn EVP_CIPHER_CTX_free(ctx: *mut c_void);
    fn EVP_EncryptInit_ex(
        ctx: *mut c_void,
        cipher: *const c_void,
        engine: *mut c_void,
        key: *const u8,
        iv: *const u8,
    ) -> c_int;
    fn EVP_EncryptUpdate(
        ctx: *mut c_void,
        out: *mut u8,
        outl: *mut c_int,
        input: *const u8,
        inl: c_int,
    ) -> c_int;
    fn EVP_aes_256_cbc() -> *const c_void;
}

// Safe Rust wrapper
pub struct OpensslAes {
    ctx: *mut c_void,
}

impl OpensslAes {
    pub fn new(key: &[u8; 32], iv: &[u8; 16]) -> Result<Self, CryptoError> {
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
        
        Ok(Self { ctx })
    }
    
    pub fn encrypt(&mut self, plaintext: &[u8], ciphertext: &mut [u8]) -> Result<usize, CryptoError> {
        let mut outlen: c_int = 0;
        
        let result = unsafe {
            EVP_EncryptUpdate(
                self.ctx,
                ciphertext.as_mut_ptr(),
                &mut outlen,
                plaintext.as_ptr(),
                plaintext.len() as c_int,
            )
        };
        
        if result == 1 {
            Ok(outlen as usize)
        } else {
            Err(CryptoError::EncryptionFailed)
        }
    }
}

impl Drop for OpensslAes {
    fn drop(&mut self) {
        unsafe {
            EVP_CIPHER_CTX_free(self.ctx);
        }
    }
}

// Usage in hybrid system
fn hybrid_encryption_example() -> Result<Vec<u8>, CryptoError> {
    let key = [0u8; 32];
    let iv = [0u8; 16];
    let plaintext = b"Hello, hybrid world!";
    
    let mut openssl_aes = OpensslAes::new(&key, &iv)?;
    let mut ciphertext = vec![0u8; plaintext.len() + 16]; // Extra space for padding
    
    let len = openssl_aes.encrypt(plaintext, &mut ciphertext)?;
    ciphertext.truncate(len);
    
    Ok(ciphertext)
}
```

### 6.3 Testing and Validation {#testing-and-validation}

Comprehensive testing strategies for cryptographic code migration.

#### Test Vector Validation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // NIST test vectors for AES
    const AES_TEST_VECTORS: &[(&[u8], &[u8], &[u8])] = &[
        (
            // Key, Plaintext, Expected Ciphertext
            &[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
              0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
              0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
              0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f],
            &[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
              0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff],
            &[0x8e, 0xa2, 0xb7, 0xca, 0x51, 0x67, 0x45, 0xbf,
              0xea, 0xfc, 0x49, 0x90, 0x4b, 0x49, 0x60, 0x89],
        ),
        // More test vectors...
    ];
    
    #[test]
    fn test_aes_against_vectors() {
        for (key, plaintext, expected) in AES_TEST_VECTORS {
            let ctx = AesContext::new(key).unwrap();
            let mut output = [0u8; 16];
            
            let plaintext_block: [u8; 16] = plaintext.try_into().unwrap();
            ctx.encrypt_block(&plaintext_block, &mut output);
            
            assert_eq!(&output, expected, "AES test vector failed");
        }
    }
    
    // Cross-validation with C implementation
    #[test]
    fn test_compatibility_with_c_implementation() {
        let key = [0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
                   0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
                   0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
                   0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
        let plaintext = [0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d,
                        0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37, 0x07, 0x34];
        
        // Rust implementation
        let rust_ctx = AesContext::new(&key).unwrap();
        let mut rust_output = [0u8; 16];
        rust_ctx.encrypt_block(&plaintext, &mut rust_output);
        
        // C implementation via FFI
        let c_ctx = unsafe { aes_context_new(key.as_ptr(), key.len()) };
        assert!(!c_ctx.is_null());
        
        let mut c_output = [0u8; 16];
        let result = unsafe {
            aes_encrypt_block_c(c_ctx, plaintext.as_ptr(), c_output.as_mut_ptr())
        };
        assert_eq!(result, 0);
        
        // Results should match
        assert_eq!(rust_output, c_output, "Rust and C implementations differ");
        
        // Cleanup
        unsafe {
            Box::from_raw(c_ctx);
        }
    }
}
```

### 6.4 Debugging and Tooling {#debugging-and-tooling}

Essential debugging techniques and tools for embedded Rust crypto development.

#### RTT Debugging for Crypto

```rust
use rtt_target::{rprintln, rtt_init_print};

fn debug_crypto_operations() {
    rtt_init_print!();
    
    let key = [0u8; 32];
    let plaintext = b"debug message";
    
    rprintln!("Starting encryption with key: {:02x?}", &key[..8]);
    
    let start_time = get_cycle_count();
    let ciphertext = encrypt_aes(&key, plaintext).unwrap();
    let end_time = get_cycle_count();
    
    rprintln!("Encryption completed in {} cycles", end_time - start_time);
    rprintln!("Ciphertext: {:02x?}", &ciphertext[..16]);
    
    // Verify decryption
    let decrypted = decrypt_aes(&key, &ciphertext).unwrap();
    rprintln!("Decryption successful: {}", decrypted == plaintext);
}

// Timing analysis for side-channel detection
fn analyze_crypto_timing() {
    const NUM_SAMPLES: usize = 1000;
    let mut timings = [0u32; NUM_SAMPLES];
    
    for i in 0..NUM_SAMPLES {
        let key = [i as u8; 32]; // Different keys
        let plaintext = [0u8; 16];
        
        let start = get_cycle_count();
        let _ = encrypt_aes(&key, &plaintext);
        let end = get_cycle_count();
        
        timings[i] = end - start;
    }
    
    // Basic statistical analysis
    let min_time = timings.iter().min().unwrap();
    let max_time = timings.iter().max().unwrap();
    let avg_time = timings.iter().sum::<u32>() / NUM_SAMPLES as u32;
    
    rprintln!("Timing analysis:");
    rprintln!("  Min: {} cycles", min_time);
    rprintln!("  Max: {} cycles", max_time);
    rprintln!("  Avg: {} cycles", avg_time);
    rprintln!("  Variation: {} cycles", max_time - min_time);
    
    if max_time - min_time > avg_time / 10 {
        rprintln!("WARNING: High timing variation detected!");
    }
}
```

### 6.5 Performance Considerations {#performance-considerations}

Optimizing cryptographic performance in embedded Rust applications.

#### Benchmarking and Profiling

```rust
// Cycle-accurate benchmarking
fn benchmark_crypto_operations() {
    const ITERATIONS: usize = 1000;
    
    // Benchmark AES encryption
    let key = [0u8; 32];
    let mut plaintext = [0u8; 16];
    
    let start = get_cycle_count();
    for i in 0..ITERATIONS {
        plaintext[0] = i as u8; // Vary input slightly
        let _ = encrypt_aes_block(&key, &plaintext);
    }
    let end = get_cycle_count();
    
    let cycles_per_block = (end - start) / ITERATIONS as u32;
    let throughput_mbps = (16 * 8 * SYSTEM_CLOCK_HZ) / (cycles_per_block * 1_000_000);
    
    rprintln!("AES-256 Performance:");
    rprintln!("  {} cycles per block", cycles_per_block);
    rprintln!("  {} Mbps throughput", throughput_mbps);
}

// Memory usage analysis
fn analyze_memory_usage() {
    extern "C" {
        static mut _heap_start: u8;
        static mut _heap_end: u8;
        static mut _stack_start: u8;
    }
    
    let heap_size = unsafe {
        &_heap_end as *const u8 as usize - &_heap_start as *const u8 as usize
    };
    
    let stack_ptr = cortex_m::register::msp::read() as usize;
    let stack_usage = unsafe {
        &_stack_start as *const u8 as usize - stack_ptr
    };
    
    rprintln!("Memory Usage:");
    rprintln!("  Heap size: {} bytes", heap_size);
    rprintln!("  Stack usage: {} bytes", stack_usage);
    
    // Crypto-specific memory analysis
    let crypto_ctx_size = core::mem::size_of::<CryptoContext>();
    let session_keys_size = core::mem::size_of::<SessionKeys>();
    
    rprintln!("Crypto Memory:");
    rprintln!("  CryptoContext: {} bytes", crypto_ctx_size);
    rprintln!("  SessionKeys: {} bytes", session_keys_size);
}
```

---

## Conclusion

This restructured tutorial provides a comprehensive, streamlined guide for experienced embedded cryptography C programmers transitioning to Rust. The document eliminates redundancy while maintaining all essential information, organized for both linear learning and quick reference use.

### Key Benefits of This Structure

- **Quick Reference First** - Immediate productivity through comprehensive lookup tables
- **Consolidated Setup** - Single authoritative environment configuration guide  
- **Focused Core Concepts** - Essential Rust concepts explained once, thoroughly
- **Embedded-Specific Patterns** - Dedicated coverage of no-std and hardware integration
- **Crypto Implementation Focus** - Security-first patterns and side-channel mitigations
- **Practical Migration Guidance** - Real-world strategies for transitioning existing codebases

### Navigation Features

- **Deep Linking** - Every section accessible via anchor links
- **Cross-References** - Related concepts linked throughout
- **Progressive Disclosure** - Detailed information available without overwhelming basics
- **Dual-Purpose Design** - Works as both tutorial and reference manual

### Next Steps

1. **Start with Quick Reference** - Use Section 1 for immediate C-to-Rust translation
2. **Follow Setup Guide** - Complete environment configuration using Section 2
3. **Master Core Concepts** - Work through Section 3 for fundamental understanding
4. **Apply Embedded Patterns** - Implement Section 4 patterns in your projects
5. **Secure Implementation** - Use Section 5 for production crypto code
6. **Plan Migration** - Follow Section 6 for systematic codebase transition

The document structure and navigation framework is now complete, providing a solid foundation for embedded Rust cryptography development.