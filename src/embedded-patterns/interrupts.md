# 4.3 Interrupt Handling

This section consolidates all interrupt handling patterns for embedded crypto applications, covering both basic interrupt handlers and advanced real-time frameworks.

## Safe Interrupt Handling Fundamentals

Interrupt handlers in embedded systems must be carefully designed to avoid data races and ensure real-time performance.

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use cortex_r_rt::{entry, interrupt};
use critical_section::Mutex;
use core::cell::RefCell;
use heapless::spsc::{Queue, Producer, Consumer};

#[derive(Debug)]
struct CryptoError(&'static str);

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

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

## RTIC Framework for Real-Time Crypto

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};


use zeroize::{Zeroize, ZeroizeOnDrop};
use core::mem;
use core::fmt;

use core::result::Result;

// Advanced real-time crypto using RTIC framework
// #[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1])] // Hardware-specific code - adapt for your target
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

## Interrupt Priority and Timing Considerations

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
//             stm32f4xx_pac::Interrupt::CRYP,  // Hardware-specific code - adapt for your target
            CRYPTO_HW_PRIORITY
        );
        
        // Set DMA interrupt to high priority
        cortex_m::peripheral::NVIC::set_priority(
//             stm32f4xx_pac::Interrupt::DMA1_STREAM0,  // Hardware-specific code - adapt for your target
            DMA_PRIORITY
        );
        
        // Set timer interrupt for key management
        cortex_m::peripheral::NVIC::set_priority(
//             stm32f4xx_pac::Interrupt::TIM2,  // Hardware-specific code - adapt for your target
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

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```