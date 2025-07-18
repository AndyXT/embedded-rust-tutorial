# 1.6 Embedded-Specific Quick Reference

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
use cortex_r5_helpers::*;
```

```rust
#![no_std]
#![no_main]
use core::mem;
use core::fmt;

use core::result::Result;


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