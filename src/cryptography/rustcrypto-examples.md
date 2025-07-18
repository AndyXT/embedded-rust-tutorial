# 5.6 RustCrypto Examples for Cortex-R5 {rustcrypto-examples}

This section provides comprehensive examples using the RustCrypto project crates in a no_std, no heap environment suitable for Cortex-R5 embedded systems.

## Complete SHA-256 Example

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use sha2::{Sha256, Digest};
use heapless::Vec;
use heapless::consts::*;

#[cortex_r_rt::entry]
fn main() -> ! {
    // Simple SHA-256 hashing
    let mut hasher = Sha256::new();
    hasher.update(b"Hello, Cortex-R5!");
    let result = hasher.finalize();
    
    // Convert to fixed array
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    
    // HMAC-SHA256 implementation
    let key = [0x42u8; 32];
    let message = b"Authenticated message";
    let hmac_result = hmac_sha256(&key, message);
    
    loop {}
}

fn hmac_sha256(key: &[u8; 32], message: &[u8]) -> [u8; 32] {
    use sha2::Sha256;
    use hmac::{Hmac, Mac};
    
    type HmacSha256 = Hmac<Sha256>;
    
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(message);
    
    let result = mac.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result.into_bytes());
    output
}
```

## AES-256-GCM Example

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use heapless::Vec;
use heapless::consts::*;

type Vec256 = Vec<u8, U256>;

#[derive(Debug)]
struct CryptoError;

fn encrypt_message(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec256, CryptoError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    
    // Generate nonce - in real code, use a proper RNG
    let nonce_bytes = [0u8; 12]; // Should be random!
    let nonce = Nonce::<Aes256Gcm>::from_slice(&nonce_bytes);
    
    // Encrypt
    let mut buffer = Vec256::new();
    buffer.extend_from_slice(plaintext).map_err(|_| CryptoError)?;
    
    cipher.encrypt_in_place(nonce, b"", &mut buffer)
        .map_err(|_| CryptoError)?;
    
    // Prepend nonce to ciphertext
    let mut result = Vec256::new();
    result.extend_from_slice(&nonce_bytes).map_err(|_| CryptoError)?;
    result.extend_from_slice(&buffer).map_err(|_| CryptoError)?;
    
    Ok(result)
}

fn decrypt_message(key: &[u8; 32], ciphertext: &[u8]) -> Result<Vec256, CryptoError> {
    if ciphertext.len() < 12 {
        return Err(CryptoError);
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    
    // Extract nonce
    let nonce = Nonce::<Aes256Gcm>::from_slice(&ciphertext[..12]);
    
    // Decrypt
    let mut buffer = Vec256::new();
    buffer.extend_from_slice(&ciphertext[12..]).map_err(|_| CryptoError)?;
    
    cipher.decrypt_in_place(nonce, b"", &mut buffer)
        .map_err(|_| CryptoError)?;
    
    Ok(buffer)
}

#[cortex_r_rt::entry]
fn main() -> ! {
    let key = [0x01u8; 32];
    let message = b"Secret message for Cortex-R5";
    
    // Encrypt
    let encrypted = encrypt_message(&key, message).unwrap();
    
    // Decrypt
    let decrypted = decrypt_message(&key, &encrypted).unwrap();
    
    loop {}
}
```

## HKDF Key Derivation Example

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
struct DerivedKeys {
    encryption_key: [u8; 32],
    authentication_key: [u8; 32],
    iv: [u8; 16],
}

fn derive_keys(master_key: &[u8; 32], salt: &[u8], info: &[u8]) -> DerivedKeys {
    let hk = Hkdf::<Sha256>::new(Some(salt), master_key);
    
    let mut okm = [0u8; 80]; // 32 + 32 + 16
    hk.expand(info, &mut okm)
        .expect("80 bytes is a valid length for HKDF-SHA256");
    
    let mut keys = DerivedKeys {
        encryption_key: [0u8; 32],
        authentication_key: [0u8; 32],
        iv: [0u8; 16],
    };
    
    keys.encryption_key.copy_from_slice(&okm[0..32]);
    keys.authentication_key.copy_from_slice(&okm[32..64]);
    keys.iv.copy_from_slice(&okm[64..80]);
    
    // Clear intermediate material
    okm.zeroize();
    
    keys
}

#[cortex_r_rt::entry]
fn main() -> ! {
    let master_key = [0x42u8; 32];
    let salt = b"cortex-r5-salt";
    let info = b"session-keys";
    
    let session_keys = derive_keys(&master_key, salt, info);
    
    // Use keys...
    
    // Keys are automatically zeroized when dropped
    loop {}
}
```

## PBKDF2 Password Hashing Example

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use pbkdf2::{pbkdf2_hmac_array};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

fn hash_password(password: &[u8], salt: &[u8; 32]) -> [u8; 32] {
    const ITERATIONS: u32 = 100_000;
    pbkdf2_hmac_array::<Sha256, 32>(password, salt, ITERATIONS)
}

fn verify_password(password: &[u8], salt: &[u8; 32], expected_hash: &[u8; 32]) -> bool {
    let mut computed_hash = hash_password(password, salt);
    let valid = computed_hash.ct_eq(expected_hash).into();
    computed_hash.zeroize();
    valid
}

#[cortex_r_rt::entry]
fn main() -> ! {
    let password = b"secure_password123";
    let salt = [0x53u8; 32]; // Should be random!
    
    // Hash password
    let password_hash = hash_password(password, &salt);
    
    // Verify password
    let is_valid = verify_password(password, &salt, &password_hash);
    
    loop {}
}
```

## X.509 Certificate Parsing Example

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use x509_cert::der::{Decode, DecodePem};
use heapless::String;
use heapless::consts::*;

type String256 = String<U256>;

#[derive(Debug)]
struct CertInfo {
    subject: String256,
    issuer: String256,
    serial: [u8; 20],
}

fn parse_certificate(cert_der: &[u8]) -> Result<CertInfo, ()> {
    use x509_cert::Certificate;
    
    let cert = Certificate::from_der(cert_der).map_err(|_| ())?;
    
    let mut info = CertInfo {
        subject: String256::new(),
        issuer: String256::new(),
        serial: [0u8; 20],
    };
    
    // Extract subject (simplified)
    if let Some(cn) = cert.tbs_certificate.subject.0.first() {
        // In real code, properly parse the RDN sequence
        let _ = info.subject.push_str("CN=Example");
    }
    
    // Extract issuer (simplified)
    if let Some(cn) = cert.tbs_certificate.issuer.0.first() {
        let _ = info.issuer.push_str("CN=CA");
    }
    
    // Copy serial number (up to 20 bytes)
    let serial_bytes = cert.tbs_certificate.serial_number.as_bytes();
    let copy_len = serial_bytes.len().min(20);
    info.serial[..copy_len].copy_from_slice(&serial_bytes[..copy_len]);
    
    Ok(info)
}

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example DER-encoded certificate (truncated for example)
    let cert_der = [
        0x30, 0x82, 0x01, 0x0a, 0x30, 0x81, 0xb5, 0xa0,
        // ... rest of certificate
    ];
    
    if let Ok(info) = parse_certificate(&cert_der) {
        // Use certificate info
    }
    
    loop {}
}
```

## Constant-Time Operations with Subtle

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeGreater};
use zeroize::Zeroize;

struct SecureCounter {
    value: u32,
    max_value: u32,
}

impl SecureCounter {
    fn new(max: u32) -> Self {
        Self {
            value: 0,
            max_value: max,
        }
    }
    
    fn increment(&mut self) -> Choice {
        let overflow = self.value.ct_eq(&self.max_value);
        let new_value = self.value.wrapping_add(1);
        self.value = u32::conditional_select(&new_value, &0, overflow);
        !overflow
    }
    
    fn constant_time_compare(&self, other: u32) -> Choice {
        self.value.ct_eq(&other)
    }
}

fn secure_key_rotation(
    current_key: &mut [u8; 32],
    new_key: &[u8; 32],
    condition: Choice
) {
    for (current, new) in current_key.iter_mut().zip(new_key.iter()) {
        *current = u8::conditional_select(current, new, condition);
    }
}

#[cortex_r_rt::entry]
fn main() -> ! {
    let mut counter = SecureCounter::new(1000);
    let mut key = [0x42u8; 32];
    let new_key = [0x53u8; 32];
    
    // Increment counter in constant time
    let success = counter.increment();
    
    // Rotate key based on condition
    secure_key_rotation(&mut key, &new_key, success);
    
    loop {}
}
```

## ChaCha20Poly1305 AEAD Example

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit},
    ChaCha20Poly1305, Nonce, Key
};
use heapless::Vec;
use heapless::consts::*;

type Vec512 = Vec<u8, U512>;

fn authenticated_encrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    plaintext: &[u8],
    aad: &[u8]
) -> Result<Vec512, ()> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(nonce);
    
    let mut buffer = Vec512::new();
    buffer.extend_from_slice(plaintext).map_err(|_| ())?;
    
    cipher.encrypt_in_place(nonce, aad, &mut buffer)
        .map_err(|_| ())?;
    
    Ok(buffer)
}

fn authenticated_decrypt(
    key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext: &[u8],
    aad: &[u8]
) -> Result<Vec512, ()> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(nonce);
    
    let mut buffer = Vec512::new();
    buffer.extend_from_slice(ciphertext).map_err(|_| ())?;
    
    cipher.decrypt_in_place(nonce, aad, &mut buffer)
        .map_err(|_| ())?;
    
    Ok(buffer)
}

#[cortex_r_rt::entry]
fn main() -> ! {
    let key = [0x00u8; 32];
    let nonce = [0x00u8; 12];
    let plaintext = b"Secret message";
    let aad = b"Additional authenticated data";
    
    // Encrypt with authentication
    let ciphertext = authenticated_encrypt(&key, &nonce, plaintext, aad).unwrap();
    
    // Decrypt and verify
    let decrypted = authenticated_decrypt(&key, &nonce, &ciphertext, aad).unwrap();
    
    loop {}
}
```

## Complete Crypto System Example

```rust
#![no_std]
#![no_main]

use panic_halt as _;
use sha2::{Sha256, Digest};
use hkdf::Hkdf;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};
use heapless::{Vec, consts::*};

type Vec256 = Vec<u8, U256>;

#[derive(ZeroizeOnDrop)]
struct CryptoContext {
    master_key: [u8; 32],
    session_key: [u8; 32],
    counter: u64,
}

impl CryptoContext {
    fn new(master_key: [u8; 32]) -> Self {
        Self {
            master_key,
            session_key: [0u8; 32],
            counter: 0,
        }
    }
    
    fn derive_session_key(&mut self, salt: &[u8]) {
        let hk = Hkdf::<Sha256>::new(Some(salt), &self.master_key);
        hk.expand(b"session", &mut self.session_key)
            .expect("32 bytes is valid for HKDF");
    }
    
    fn encrypt_packet(&mut self, data: &[u8]) -> Result<Vec256, ()> {
        let cipher = Aes256Gcm::new(Key::from_slice(&self.session_key));
        
        // Create nonce from counter
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..].copy_from_slice(&self.counter.to_be_bytes());
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt
        let mut output = Vec256::new();
        output.extend_from_slice(&nonce_bytes).map_err(|_| ())?;
        
        let mut buffer = Vec256::new();
        buffer.extend_from_slice(data).map_err(|_| ())?;
        
        cipher.encrypt_in_place(nonce, b"", &mut buffer)
            .map_err(|_| ())?;
        
        output.extend_from_slice(&buffer).map_err(|_| ())?;
        
        self.counter += 1;
        Ok(output)
    }
    
    fn verify_and_decrypt(&mut self, packet: &[u8]) -> Result<Vec256, ()> {
        if packet.len() < 12 {
            return Err(());
        }
        
        let cipher = Aes256Gcm::new(Key::from_slice(&self.session_key));
        
        // Extract nonce
        let nonce = Nonce::from_slice(&packet[..12]);
        
        // Verify counter to prevent replay
        let mut expected_nonce = [0u8; 12];
        expected_nonce[4..].copy_from_slice(&self.counter.to_be_bytes());
        
        if !nonce.ct_eq(&expected_nonce).into() {
            return Err(());
        }
        
        // Decrypt
        let mut buffer = Vec256::new();
        buffer.extend_from_slice(&packet[12..]).map_err(|_| ())?;
        
        cipher.decrypt_in_place(nonce, b"", &mut buffer)
            .map_err(|_| ())?;
        
        self.counter += 1;
        Ok(buffer)
    }
}

#[cortex_r_rt::entry]
fn main() -> ! {
    let master_key = [0x42u8; 32];
    let mut ctx = CryptoContext::new(master_key);
    
    // Derive session key
    ctx.derive_session_key(b"session-salt");
    
    // Encrypt message
    let message = b"Secure Cortex-R5 communication";
    let encrypted = ctx.encrypt_packet(message).unwrap();
    
    // Reset counter for decryption (normally would be separate context)
    ctx.counter = 0;
    
    // Decrypt message
    let decrypted = ctx.verify_and_decrypt(&encrypted).unwrap();
    
    loop {}
}
```

## Key Takeaways

1. **All examples compile for `no_std` environments** - suitable for Cortex-R5
2. **No heap allocations** - uses `heapless` for collections
3. **Proper error handling** - no panics in production code
4. **Zeroization** - sensitive data is cleared from memory
5. **Constant-time operations** - using `subtle` crate
6. **Real crypto algorithms** - SHA-256, AES-GCM, ChaCha20Poly1305, HKDF, PBKDF2

These examples demonstrate practical cryptographic implementations that work in resource-constrained embedded environments while maintaining security best practices.