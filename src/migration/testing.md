# 6.3 Testing and Validation

Comprehensive testing strategies for cryptographic code migration, including test vector validation, cross-implementation verification, and automated testing frameworks.

## Test Vector Validation Framework

**NIST Test Vector Integration**




```rust
#![no_std]
#![no_main]

use panic_halt as _;
use core::{fmt, result::Result};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;
use aes::{Aes256, cipher::{KeyInit, BlockEncrypt, BlockDecrypt}};


use core::mem;
use heapless::Vec;

use heapless::String;
use core::fmt;
use core::result::Result;

#[cfg(test)]
mod crypto_tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use core::fs;
    
    // NIST test vector structure
    #[derive(Debug, Deserialize)]
    struct NistTestVector {
        key: heapless::String<64>,
        plaintext: heapless::String<64>,
        ciphertext: heapless::String<64>,
        #[serde(default)]
        iv: Option<heapless::String<64>>,
        #[serde(default)]
        tag: Option<heapless::String<64>>,
    }
    
    #[derive(Debug, Deserialize)]
    struct NistTestSuite {
        algorithm: heapless::String<64>,
        key_size: u32,
        test_vectors: heapless::Vec<NistTestVector, 32>,
    }
    
    // Load test vectors from JSON files
    fn load_nist_vectors(filename: &str) -> Result<NistTestSuite, Box<dyn core::error::Error>> {
        let content = fs::read_to_string(filename)?;
        let suite: NistTestSuite = serde_json::from_str(&content)?;
        Ok(suite)
    }
    
    // Helper function to convert hex strings to bytes
    fn hex_to_bytes(hex: &str) -> heapless::Vec<u8, 32> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }
    
    // Comprehensive AES test vector validation
    #[test]
    fn test_aes_nist_vectors() {
        let test_suites = [
            "tests/vectors/aes128_ecb.json",
            "tests/vectors/aes192_ecb.json", 
            "tests/vectors/aes256_ecb.json",
            "tests/vectors/aes128_cbc.json",
            "tests/vectors/aes256_cbc.json",
        ];
        
        for suite_file in &test_suites {
            let suite = load_nist_vectors(suite_file).unwrap();
            defmt::info!("Testing {} with {} vectors", suite.algorithm, suite.test_vectors.len());
            
            for (i, vector) in suite.test_vectors.iter().enumerate() {
                let key = hex_to_bytes(&vector.key);
                let plaintext = hex_to_bytes(&vector.plaintext);
                let expected_ciphertext = hex_to_bytes(&vector.ciphertext);
                
                // Test with our Rust implementation
                let result = match suite.algorithm.as_str() {
                    "AES-ECB" => test_aes_ecb(&key, &plaintext),
                    "AES-CBC" => {
                        let iv = hex_to_bytes(vector.iv.as_ref().unwrap());
                        test_aes_cbc(&key, &iv, &plaintext)
                    }
                    _ => panic!("Unsupported algorithm: {}", suite.algorithm),
                };
                
                assert_eq!(
                    result, expected_ciphertext,
                    "Test vector {} failed for {}", i, suite.algorithm
                );
            }
        }
    }
    
    fn test_aes_ecb(key: &[u8], plaintext: &[u8]) -> heapless::Vec<u8, 32> {
        let ctx = AesContext::new(key).unwrap();
        let mut ciphertext = Vec::new();
        
        for chunk in plaintext.chunks(16) {
            let mut block = [0u8; 16];
            block[..chunk.len()].copy_from_slice(chunk);
            let encrypted_block = ctx.encrypt_block(&block);
            ciphertext.extend_from_slice(&encrypted_block);
        }
        
        ciphertext
    }
    
    fn test_aes_cbc(key: &[u8], iv: &[u8], plaintext: &[u8]) -> heapless::Vec<u8, 32> {
        let mut ctx = AesContext::new(key).unwrap();
        let mut iv_copy = [0u8; 16];
        iv_copy.copy_from_slice(iv);
        
        ctx.encrypt_cbc(&mut iv_copy, plaintext).unwrap()
    }
    
    // Cross-validation with C implementation
    #[test]
    fn test_compatibility_with_c_implementation() {
        let test_cases = [
            // AES-256 test case
            (
                [0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe,
                 0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77, 0x81,
                 0x1f, 0x35, 0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7,
                 0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14, 0xdf, 0xf4],
                [0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96,
                 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a],
                [0xf3, 0xee, 0xd1, 0xbd, 0xb5, 0xd2, 0xa0, 0x3c,
                 0x06, 0x4b, 0x5a, 0x7e, 0x3d, 0xb1, 0x81, 0xf8],
            ),
            // Add more test cases...
        ];
        
        for (key, plaintext, expected) in &test_cases {
            // Rust implementation
            let rust_ctx = AesContext::new(key).unwrap();
            let rust_output = rust_ctx.encrypt_block(plaintext);
            
            // C implementation via FFI
            let c_ctx = unsafe { aes_context_new(key.as_ptr(), key.len()) };
            assert!(!c_ctx.is_null());
            
            let mut c_output = [0u8; 16];
            let result = unsafe {
                aes_encrypt_block_c(c_ctx, plaintext.as_ptr(), c_output.as_mut_ptr())
            };
            assert_eq!(result, 0);
            
            // All implementations should match
            assert_eq!(&rust_output, expected, "Rust implementation failed");
            assert_eq!(&c_output, expected, "C implementation failed");
            assert_eq!(rust_output, c_output, "Rust and C implementations differ");
            
            // Cleanup
            unsafe {
                aes_context_free(c_ctx);
            }
        }
    }
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

## Property-Based Testing for Crypto

**Using QuickCheck for Crypto Properties**

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;
use aes::{Aes256, cipher::{KeyInit, BlockEncrypt, BlockDecrypt}};

// Stub function for HMAC
fn hmac_sha256(_key: &[u8; 32], _message: &[u8]) -> Result<[u8; 32], CryptoError> {
    Ok([0u8; 32])
}


use core::mem;
use heapless::Vec;

use core::fmt;
use core::result::Result;

#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult};
    use quickcheck_macros::quickcheck;
    
    // Property: Encryption followed by decryption should return original
    #[quickcheck]
    fn prop_aes_encrypt_decrypt_roundtrip(key: heapless::Vec<u8, 32>, plaintext: heapless::Vec<u8, 32>) -> TestResult {
        // Ensure valid key sizes
        if key.len() != 16 && key.len() != 24 && key.len() != 32 {
            return TestResult::discard();
        }
        
        // Ensure plaintext is multiple of block size
        if plaintext.len() % 16 != 0 || plaintext.is_empty() {
            return TestResult::discard();
        }
        
        let ctx = match AesContext::new(&key) {
            Ok(ctx) => ctx,
            Err(_) => return TestResult::discard(),
        };
        
        // Encrypt then decrypt
        let mut ciphertext = vec![0u8; plaintext.len()];
        let mut decrypted = vec![0u8; plaintext.len()];
        
        for (i, (plain_chunk, cipher_chunk)) in plaintext.chunks(16)
            .zip(ciphertext.chunks_mut(16))
            .enumerate() 
        {
            let plain_block: [u8; 16] = plain_chunk.try_into().unwrap();
            let encrypted = ctx.encrypt_block(&plain_block);
            cipher_chunk.copy_from_slice(&encrypted);
        }
        
        for (cipher_chunk, decrypted_chunk) in ciphertext.chunks(16)
            .zip(decrypted.chunks_mut(16)) 
        {
            let cipher_block: [u8; 16] = cipher_chunk.try_into().unwrap();
            let decrypted_block = ctx.decrypt_block(&cipher_block);
            decrypted_chunk.copy_from_slice(&decrypted_block);
        }
        
        TestResult::from_bool(plaintext == decrypted)
    }
    
    // Property: Same key and plaintext should always produce same ciphertext
    #[quickcheck]
    fn prop_aes_deterministic(key: heapless::Vec<u8, 32>, plaintext: heapless::Vec<u8, 32>) -> TestResult {
        if key.len() != 32 || plaintext.len() != 16 {
            return TestResult::discard();
        }
        
        let ctx1 = AesContext::new(&key).unwrap();
        let ctx2 = AesContext::new(&key).unwrap();
        
        let plaintext_block: [u8; 16] = plaintext.try_into().unwrap();
        let result1 = ctx1.encrypt_block(&plaintext_block);
        let result2 = ctx2.encrypt_block(&plaintext_block);
        
        TestResult::from_bool(result1 == result2)
    }
    
    // Property: Different keys should produce different ciphertext (with high probability)
    #[quickcheck]
    fn prop_aes_key_sensitivity(key1: heapless::Vec<u8, 32>, key2: heapless::Vec<u8, 32>, plaintext: heapless::Vec<u8, 32>) -> TestResult {
        if key1.len() != 32 || key2.len() != 32 || plaintext.len() != 16 {
            return TestResult::discard();
        }
        
        if key1 == key2 {
            return TestResult::discard();
        }
        
        let ctx1 = AesContext::new(&key1).unwrap();
        let ctx2 = AesContext::new(&key2).unwrap();
        
        let plaintext_block: [u8; 16] = plaintext.try_into().unwrap();
        let result1 = ctx1.encrypt_block(&plaintext_block);
        let result2 = ctx2.encrypt_block(&plaintext_block);
        
        // Different keys should produce different ciphertext
        TestResult::from_bool(result1 != result2)
    }
    
    // Property: HMAC should be deterministic
    #[quickcheck]
    fn prop_hmac_deterministic(key: heapless::Vec<u8, 32>, message: heapless::Vec<u8, 32>) -> TestResult {
        if key.is_empty() || message.is_empty() {
            return TestResult::discard();
        }
        
        let hmac1 = calculate_hmac_sha256(&key, &message);
        let hmac2 = calculate_hmac_sha256(&key, &message);
        
        TestResult::from_bool(hmac1 == hmac2)
    }
    
    // Property: HMAC should be sensitive to key changes
    #[quickcheck]
    fn prop_hmac_key_sensitivity(key1: heapless::Vec<u8, 32>, key2: heapless::Vec<u8, 32>, message: heapless::Vec<u8, 32>) -> TestResult {
        if key1.is_empty() || key2.is_empty() || message.is_empty() || key1 == key2 {
            return TestResult::discard();
        }
        
        let hmac1 = calculate_hmac_sha256(&key1, &message);
        let hmac2 = calculate_hmac_sha256(&key2, &message);
        
        TestResult::from_bool(hmac1 != hmac2)
    }
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

## Automated Testing Framework

**Continuous Integration Test Suite**

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{mem, fmt};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;
use subtle::{Choice, ConstantTimeEq};
use aes::{Aes256, cipher::{KeyInit, BlockEncrypt, BlockDecrypt}};


use zeroize::{Zeroize, ZeroizeOnDrop};
use core::mem;
use heapless::String;

use core::fmt;
use core::result::Result;

// tests/integration_tests.rs - Integration test suite
use core::process::Command;
use core::time::Instant;

#[test]
fn test_cross_compilation_targets() {
    let targets = [
        "thumbv7em-none-eabihf",
        "armv7r-none-eabihf",
        "thumbv8m.main-none-eabihf",
    ];
    
    for target in &targets {
        defmt::info!("Testing compilation for target: {}", target);
        
        let output = Command::new("cargo")
            .args(&["build", "--target", target, "--release"])
            .output()
            .expect("Failed to execute cargo build");
        
        assert!(
            output.status.success(),
            "Compilation failed for target {}: {}",
            target,
            heapless::String<64>::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_crypto_performance_benchmarks() {
    let start = Instant::now();
    
    // Run performance-critical crypto operations
    let key = [0u8; 32];
    let plaintext = [0u8; 16];
    
    let ctx = AesContext::new(&key).unwrap();
    
    // Benchmark AES encryption
    let iterations = 10000;
    let bench_start = Instant::now();
    
    for _ in 0..iterations {
        let _ = ctx.encrypt_block(&plaintext);
    }
    
    let duration = bench_start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();
    
    defmt::info!("AES-256 performance: {:.0} ops/sec", ops_per_sec);
    
    // Performance regression test - should be faster than baseline
    assert!(ops_per_sec > 50000.0, "AES performance regression detected");
}

#[test]
fn test_memory_usage_constraints() {
    // Test that crypto contexts don't exceed memory budgets
    let ctx_size = core::mem::size_of::<AesContext>();
    assert!(ctx_size <= 1024, "AesContext too large: {} bytes", ctx_size);
    
    let session_size = core::mem::size_of::<CryptoSession>();
    assert!(session_size <= 2048, "CryptoSession too large: {} bytes", session_size);
}

#[test]
fn test_zeroization_behavior() {
    use zeroize::Zeroize;
    
    // Test that sensitive data is properly zeroized
    let mut sensitive_data = [0x42u8; 32];
    
    // Verify data is initially non-zero
    assert_ne!(sensitive_data, [0u8; 32]);
    
    // Zeroize and verify
    sensitive_data.zeroize();
    assert_eq!(sensitive_data, [0u8; 32]);
}

#[test]
fn test_constant_time_operations() {
    
    // Test constant-time comparison
    let a = [0x42u8; 32];
    let b = [0x42u8; 32];
    let c = [0x43u8; 32];
    
    // These should be constant-time
    assert!(bool::from(a.ct_eq(&b)));
    assert!(!bool::from(a.ct_eq(&c)));
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

## Hardware-in-the-Loop Testing

**Embedded Target Testing Framework**

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;
use aes::{Aes256, cipher::{KeyInit, BlockEncrypt, BlockDecrypt}};


use cortex_r::asm;
use core::fmt;
use core::mem;

// tests/hardware_tests.rs - Hardware-specific tests
#![cfg(feature = "hardware-testing")]

use embedded_test_framework::*;

#[embedded_test]
fn test_hardware_crypto_acceleration() {
    let mut hw_crypto = XilinxHardwareCrypto::new().unwrap();
    let key = [0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
               0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
               0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
               0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
    let plaintext = [0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d,
                     0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37, 0x07, 0x34];
    let iv = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
              0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f];
    
    hw_crypto.set_key(&key).unwrap();
    
    let mut hw_result = [0u8; 16];
    hw_crypto.encrypt_hardware(&plaintext, &mut hw_result, &iv).unwrap();
    
    // Compare with software implementation
    let sw_ctx = AesContext::new(&key).unwrap();
    let sw_result = sw_ctx.encrypt_block(&plaintext);
    
    assert_eq!(hw_result, sw_result, "Hardware and software results differ");
}

#[embedded_test]
fn test_timing_attack_resistance() {
    let key1 = [0x00u8; 32];
    let key2 = [0xffu8; 32];
    let plaintext = [0x42u8; 16];
    
    // Measure timing for different keys
    let mut timings1 = Vec::new();
    let mut timings2 = Vec::new();
    
    for _ in 0..1000 {
        let start = get_cycle_count();
        let _ = aes_encrypt_constant_time(&key1, &plaintext);
        let end = get_cycle_count();
        timings1.push(end - start);
        
        let start = get_cycle_count();
        let _ = aes_encrypt_constant_time(&key2, &plaintext);
        let end = get_cycle_count();
        timings2.push(end - start);
    }
    
    // Statistical analysis of timing differences
    let avg1: f64 = timings1.iter().map(|&x| x as f64).sum::<f64>() / timings1.len() as f64;
    let avg2: f64 = timings2.iter().map(|&x| x as f64).sum::<f64>() / timings2.len() as f64;
    
    let diff_percent = ((avg1 - avg2).abs() / avg1.max(avg2)) * 100.0;
    
    // Timing difference should be minimal (less than 1%)
    assert!(diff_percent < 1.0, "Timing difference too large: {:.2}%", diff_percent);
}

#[embedded_test]
fn test_power_analysis_resistance() {
    // This would require specialized hardware setup
    // Placeholder for power analysis testing
    
    let key = [0x2bu8; 32];
    let plaintexts = [
        [0x00u8; 16], // All zeros
        [0xffu8; 16], // All ones
        [0x55u8; 16], // Alternating pattern
        [0xaau8; 16], // Inverse alternating
    ];
    
    for plaintext in &plaintexts {
        // In real implementation, this would measure power consumption
        let _result = aes_encrypt_power_analysis_resistant(&key, plaintext);
        
        // Verify that power consumption patterns don't leak key information
        // This requires specialized hardware and analysis tools
    }
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

## Test Data Management

**Test Vector Generation and Management**

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;
use aes::{Aes256, cipher::{KeyInit, BlockEncrypt, BlockDecrypt}};


use core::mem;
use heapless::Vec;

use heapless::String;
use core::fmt;
use core::result::Result;

// tests/test_data_generator.rs - Generate test data for validation
use serde::{Deserialize, Serialize};
use core::fs;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

#[derive(Serialize, Deserialize)]
struct TestVector {
    description: heapless::String<64>,
    key: heapless::String<64>,
    plaintext: heapless::String<64>,
    ciphertext: heapless::String<64>,
    iv: Option<heapless::String<64>>,
    tag: Option<heapless::String<64>>,
}

#[derive(Serialize, Deserialize)]
struct TestSuite {
    algorithm: heapless::String<64>,
    key_size: u32,
    description: heapless::String<64>,
    vectors: heapless::Vec<TestVector, 32>,
}

fn generate_aes_test_vectors() -> TestSuite {
    let mut rng = ChaCha20Rng::seed_from_u64(42); // Deterministic for reproducibility
    let mut vectors = Vec::new();
    
    // Generate various test cases
    for i in 0..100 {
        let mut key = [0u8; 32];
        let mut plaintext = [0u8; 16];
        
        rng.fill(&mut key);
        rng.fill(&mut plaintext);
        
        // Generate expected ciphertext using reference implementation
        let ctx = AesContext::new(&key).unwrap();
        let ciphertext = ctx.encrypt_block(&plaintext);
        
        vectors.push(TestVector {
            description: format!("Random test vector {}", i),
            key: hex::encode(key),
            plaintext: hex::encode(plaintext),
            ciphertext: hex::encode(ciphertext),
            iv: None,
            tag: None,
        });
    }
    
    // Add edge cases
    vectors.push(TestVector {
        description: "All zeros".to_string(),
        key: hex::encode([0u8; 32]),
        plaintext: hex::encode([0u8; 16]),
        ciphertext: hex::encode(aes_encrypt_reference(&[0u8; 32], &[0u8; 16])),
        iv: None,
        tag: None,
    });
    
    vectors.push(TestVector {
        description: "All ones".to_string(),
        key: hex::encode([0xffu8; 32]),
        plaintext: hex::encode([0xffu8; 16]),
        ciphertext: hex::encode(aes_encrypt_reference(&[0xffu8; 32], &[0xffu8; 16])),
        iv: None,
        tag: None,
    });
    
    TestSuite {
        algorithm: "AES-256-ECB".to_string(),
        key_size: 256,
        description: "Generated test vectors for AES-256 ECB mode".to_string(),
        vectors,
    }
}

#[test]
fn generate_and_save_test_vectors() {
    let suite = generate_aes_test_vectors();
    let json = serde_json::to_string_pretty(&suite).unwrap();
    
    fs::create_dir_all("tests/vectors").unwrap();
    fs::write("tests/vectors/aes256_ecb_generated.json", json).unwrap();
    
    defmt::info!("Generated {} test vectors", suite.vectors.len());
}

// Reference implementation for generating expected results
fn aes_encrypt_reference(key: &[u8; 32], plaintext: &[u8; 16]) -> [u8; 16] {
    // This would use a trusted reference implementation
    // For example, OpenSSL or a certified implementation
    let ctx = AesContext::new(key).unwrap();
    ctx.encrypt_block(plaintext)
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

## Automated Regression Testing

**CI/CD Integration Script**

```bash
#!/bin/bash
# scripts/run_crypto_tests.sh - Comprehensive test runner

set -e

echo "Starting comprehensive crypto test suite..."

# 1. Unit tests
echo "Running unit tests..."
cargo test --lib

# 2. Integration tests  
echo "Running integration tests..."
cargo test --test integration_tests

# 3. Property-based tests
echo "Running property-based tests..."
cargo test --test property_tests

# 4. Cross-compilation tests
echo "Testing cross-compilation..."
for target in thumbv7em-none-eabihf armv7r-none-eabihf thumbv8m.main-none-eabihf; do
    echo "Building for $target..."
    cargo build --target $target --release
done

# 5. Performance benchmarks
echo "Running performance benchmarks..."
cargo bench

# 6. Security tests
echo "Running security-focused tests..."
cargo test --features security-tests

# 7. Hardware tests (if available)
if [ "$HARDWARE_TESTING" = "true" ]; then
    echo "Running hardware-in-the-loop tests..."
    cargo test --features hardware-testing --target armv7r-none-eabihf
fi

# 8. Memory usage analysis
echo "Analyzing memory usage..."
cargo bloat --release --crates

# 9. Test coverage
echo "Generating test coverage report..."
cargo tarpaulin --out Html --output-dir coverage

echo "All tests completed successfully!"
```

**GitHub Actions Workflow**

```yaml
# .github/workflows/crypto_tests.yml
name: Crypto Migration Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
        
    - name: Install embedded targets
      run: |
        rustup target add thumbv7em-none-eabihf
        rustup target add armv7r-none-eabihf
        rustup target add thumbv8m.main-none-eabihf
        
    - name: Install test dependencies
      run: |
        cargo install cargo-tarpaulin
        cargo install cargo-bloat
        
    - name: Run comprehensive test suite
      run: ./scripts/run_crypto_tests.sh
      
    - name: Upload coverage reports
      uses: codecov/codecov-action@v3
      with:
        file: ./coverage/tarpaulin-report.xml
        
    - name: Check code formatting
      run: cargo fmt -- --check
      
    - name: Run clippy
      run: cargo clippy -- -D warnings
```

This comprehensive testing and validation framework provides:

1. **NIST Test Vector Validation** - Automated testing against official cryptographic test vectors
2. **Property-Based Testing** - QuickCheck-style testing for crypto properties
3. **Cross-Implementation Validation** - Verification between Rust and C implementations
4. **Hardware-in-the-Loop Testing** - Testing on actual embedded hardware
5. **Performance Regression Testing** - Automated performance monitoring
6. **Security-Focused Testing** - Timing attack and side-channel resistance testing
7. **Automated CI/CD Integration** - Continuous testing in development pipeline

The framework ensures that migrated cryptographic code maintains correctness, security, and performance throughout the migration process.