# 6.4 Debugging and Tooling

Essential debugging techniques and tools for embedded Rust crypto development.

## RTT Debugging for Crypto




```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};


use cortex_r::asm;
use core::fmt;
use core::mem;

// use rtt_target::{rprintln, rtt_init_print}; // RTT not included in minimal setup

fn debug_crypto_operations() {
//     rtt_init_print!(); // No std output in bare metal
    
    let key = [0u8; 32];
    let plaintext = b"debug message";
    
    rdefmt::info!("Starting encryption with key: {:02x?}", &key[..8]);
    
    let start_time = get_cycle_count();
    let ciphertext = encrypt_aes(&key, plaintext).unwrap();
    let end_time = get_cycle_count();
    
    rdefmt::info!("Encryption completed in {} cycles", end_time - start_time);
    rdefmt::info!("Ciphertext: {:02x?}", &ciphertext[..16]);
    
    // Verify decryption
    let decrypted = decrypt_aes(&key, &ciphertext).unwrap();
    rdefmt::info!("Decryption successful: {}", decrypted == plaintext);
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
    
    rdefmt::info!("Timing analysis:");
    rdefmt::info!("  Min: {} cycles", min_time);
    rdefmt::info!("  Max: {} cycles", max_time);
    rdefmt::info!("  Avg: {} cycles", avg_time);
    rdefmt::info!("  Variation: {} cycles", max_time - min_time);
    
    if max_time - min_time > avg_time / 10 {
        rdefmt::info!("WARNING: High timing variation detected!");
    }
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```