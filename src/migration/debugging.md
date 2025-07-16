# 6.4 Debugging and Tooling

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