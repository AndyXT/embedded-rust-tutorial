# 5.3 Key Management and Zeroization {key-management-and-zeroization}

Proper key management is critical for cryptographic security, especially in embedded systems where key material may persist in memory longer than expected. Rust's ownership system and the `zeroize` crate provide automatic, guaranteed cleanup of sensitive material.

**Enhanced by Functional Programming:**
- → [Functional Programming and Data Processing](../core-concepts/functional.md) - Use secure iterators for key processing operations

## Automatic Key Zeroization Patterns

Rust's `Drop` trait ensures that sensitive data is automatically cleared when it goes out of scope, preventing key material from lingering in memory:

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

// Secure key wrapper with automatic cleanup
#[derive(ZeroizeOnDrop)]
struct SecureKey<const N: usize> {
    key_material: [u8; N],
    key_id: u32,
    created_at: u64,
}

impl<const N: usize> SecureKey<N> {
    fn new(key_data: [u8; N], id: u32) -> Self {
        Self {
            key_material: key_data,
            key_id: id,
            created_at: get_timestamp(),
        }
    }
    
    fn as_bytes(&self) -> &[u8; N] {
        &self.key_material
    }
    
    fn key_id(&self) -> u32 {
        self.key_id
    }
    
    fn age(&self) -> u64 {
        get_timestamp() - self.created_at
    }
    
    // Manual zeroization if needed before drop
    fn zeroize_now(&mut self) {
        self.key_material.zeroize();
        self.key_id = 0;
        self.created_at = 0;
    }
}

// Usage example - automatic cleanup guaranteed
fn process_encrypted_message(encrypted_data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    // Key automatically zeroized when function exits
    let session_key = SecureKey::<32>::new(derive_session_key()?, 1);
    
    // Use key for decryption
    let plaintext = decrypt_aes_gcm(session_key.as_bytes(), encrypted_data)?;
    
    // Verify message integrity
    if !verify_message_integrity(&plaintext) {
        return Err(CryptoError::IntegrityCheckFailed);
    }
    
    Ok(plaintext)
    // session_key automatically zeroized here, even if error occurred
}
```

#### Hierarchical Key Derivation with Automatic Cleanup

```rust
use hkdf::Hkdf;
use sha2::Sha256;
use heapless::FnvIndexMap;

#[derive(ZeroizeOnDrop)]
struct KeyHierarchy {
    master_key: SecureKey<32>,
    derived_keys: FnvIndexMap<&'static str, SecureKey<32>, 16>,
    derivation_counter: u32,
}

impl KeyHierarchy {
    fn new(master_key_data: [u8; 32]) -> Self {
        Self {
            master_key: SecureKey::new(master_key_data, 0),
            derived_keys: FnvIndexMap::new(),
            derivation_counter: 0,
        }
    }
    
    fn derive_key(&mut self, purpose: &'static str) -> Result<&SecureKey<32>, CryptoError> {
        // Check if key already exists
        if self.derived_keys.contains_key(purpose) {
            return Ok(&self.derived_keys[purpose]);
        }
        
        // Derive new key using HKDF
        let hk = Hkdf::<Sha256>::new(None, self.master_key.as_bytes());
        let mut derived = [0u8; 32];
        
        // Include counter to ensure unique derivation
        let info = format_args!("{}:{}", purpose, self.derivation_counter);
        let info_bytes = format!("{}", info);
        
        hk.expand(info_bytes.as_bytes(), &mut derived)
            .map_err(|_| CryptoError::KeyDerivationFailed)?;
        
        // Store derived key with automatic cleanup
        self.derivation_counter += 1;
        let secure_key = SecureKey::new(derived, self.derivation_counter);
        
        self.derived_keys.insert(purpose, secure_key)
            .map_err(|_| CryptoError::TooManyKeys)?;
        
        Ok(&self.derived_keys[purpose])
    }
    
    fn get_encryption_key(&mut self) -> Result<&SecureKey<32>, CryptoError> {
        self.derive_key("encryption")
    }
    
    fn get_mac_key(&mut self) -> Result<&SecureKey<32>, CryptoError> {
        self.derive_key("authentication")
    }
    
    fn get_key_wrapping_key(&mut self) -> Result<&SecureKey<32>, CryptoError> {
        self.derive_key("key_wrapping")
    }
    
    fn rotate_master_key(&mut self, new_master: [u8; 32]) {
        // Clear all derived keys (automatically zeroized)
        self.derived_keys.clear();
        
        // Update master key (old one automatically zeroized)
        self.master_key = SecureKey::new(new_master, 0);
        self.derivation_counter = 0;
    }
    
    fn clear_derived_keys(&mut self) {
        // Explicitly clear derived keys while keeping master
        self.derived_keys.clear();
    }
    
    fn key_count(&self) -> usize {
        self.derived_keys.len()
    }
}

// Usage example with automatic cleanup
fn secure_session_example() -> Result<(), CryptoError> {
    // Master key automatically zeroized when hierarchy is dropped
    let master_key_data = generate_master_key()?;
    let mut key_hierarchy = KeyHierarchy::new(master_key_data);
    
    // Derive session keys
    let enc_key = key_hierarchy.get_encryption_key()?;
    let mac_key = key_hierarchy.get_mac_key()?;
    
    // Use keys for secure communication
    let message = b"secure message";
    let encrypted = encrypt_and_mac(enc_key.as_bytes(), mac_key.as_bytes(), message)?;
    
    // Process multiple messages with same keys
    for i in 0..10 {
        let msg = format!("message {}", i);
        let encrypted = encrypt_and_mac(enc_key.as_bytes(), mac_key.as_bytes(), msg.as_bytes())?;
        transmit_encrypted_message(&encrypted)?;
    }
    
    // All keys automatically zeroized when key_hierarchy is dropped
    Ok(())
}
```

#### Secure Random Number Generation with Hardware Integration

```rust
use rand_core::{RngCore, CryptoRng, Error as RngError};

// Hardware RNG wrapper with error handling and health checks
struct HardwareRng {
    rng_peripheral: RngPeripheral,
    health_check_counter: u32,
    last_health_check: u64,
}

impl HardwareRng {
    fn new(rng_peripheral: RngPeripheral) -> Result<Self, CryptoError> {
        let mut rng = Self {
            rng_peripheral,
            health_check_counter: 0,
            last_health_check: 0,
        };
        
        // Initial health check
        rng.perform_health_check()?;
        Ok(rng)
    }
    
    fn perform_health_check(&mut self) -> Result<(), CryptoError> {
        // Check if hardware RNG is functioning properly
        if !self.rng_peripheral.is_available() {
            return Err(CryptoError::RngNotAvailable);
        }
        
        // Perform statistical tests on random output
        let mut test_data = [0u8; 256];
        self.fill_bytes_internal(&mut test_data)?;
        
        // Simple entropy test - check for obvious patterns
        let mut zero_count = 0;
        let mut one_count = 0;
        
        for &byte in &test_data {
            zero_count += (byte == 0) as u32;
            one_count += byte.count_ones();
        }
        
        // Rough entropy check - should be roughly balanced
        if zero_count > 20 || one_count < 900 || one_count > 1100 {
            return Err(CryptoError::RngHealthCheckFailed);
        }
        
        self.last_health_check = get_timestamp();
        Ok(())
    }
    
    fn fill_bytes_internal(&mut self, dest: &mut [u8]) -> Result<(), CryptoError> {
        // Periodic health checks
        self.health_check_counter += 1;
        if self.health_check_counter % 1000 == 0 {
            self.perform_health_check()?;
        }
        
        for chunk in dest.chunks_mut(4) {
            // Wait for hardware RNG to be ready with timeout
            let mut timeout = 10000;
            while !self.rng_peripheral.is_ready() && timeout > 0 {
                cortex_m::asm::nop();
                timeout -= 1;
            }
            
            if timeout == 0 {
                return Err(CryptoError::RngTimeout);
            }
            
            let random = self.rng_peripheral.read_random();
            let bytes = random.to_le_bytes();
            
            for (i, &byte) in bytes.iter().enumerate() {
                if i < chunk.len() {
                    chunk[i] = byte;
                }
            }
        }
        
        Ok(())
    }
}

impl RngCore for HardwareRng {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        self.fill_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }
    
    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        self.fill_bytes(&mut bytes);
        u64::from_le_bytes(bytes)
    }
    
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.try_fill_bytes(dest).expect("Hardware RNG failure")
    }
    
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), RngError> {
        self.fill_bytes_internal(dest)
            .map_err(|_| RngError::new("Hardware RNG error"))
    }
}

impl CryptoRng for HardwareRng {}

// Secure key generation with hardware RNG
fn generate_session_keys(rng: &mut impl CryptoRng) -> Result<SessionKeys, CryptoError> {
    let mut master_key = [0u8; 32];
    rng.try_fill_bytes(&mut master_key)
        .map_err(|_| CryptoError::RngFailed)?;
    
    // Derive session keys from master key
    let session_keys = SessionKeys::derive_from_master(&master_key, b"session")?;
    
    // Clear master key from stack
    master_key.zeroize();
    
    Ok(session_keys)
}

// Key generation with entropy mixing
fn generate_mixed_entropy_key(hw_rng: &mut HardwareRng) -> Result<SecureKey<32>, CryptoError> {
    // Collect entropy from multiple sources
    let mut hw_entropy = [0u8; 32];
    let mut timing_entropy = [0u8; 32];
    
    // Hardware entropy
    hw_rng.try_fill_bytes(&mut hw_entropy)
        .map_err(|_| CryptoError::RngFailed)?;
    
    // Timing-based entropy (less reliable but adds diversity)
    for i in 0..32 {
        let start = get_cycle_count();
        cortex_m::asm::nop();
        let end = get_cycle_count();
        timing_entropy[i] = (end.wrapping_sub(start) & 0xFF) as u8;
    }
    
    // Mix entropy sources using HKDF
    let hk = Hkdf::<Sha256>::new(Some(&timing_entropy), &hw_entropy);
    let mut mixed_key = [0u8; 32];
    hk.expand(b"mixed_entropy_key", &mut mixed_key)
        .map_err(|_| CryptoError::KeyDerivationFailed)?;
    
    // Clear intermediate entropy
    hw_entropy.zeroize();
    timing_entropy.zeroize();
    
    Ok(SecureKey::new(mixed_key, 1))
}
```

#### Key Lifecycle Management

```rust
use core::time::Duration;

#[derive(ZeroizeOnDrop)]
struct ManagedKey<const N: usize> {
    key: SecureKey<N>,
    max_age: Duration,
    max_uses: u32,
    current_uses: u32,
    created_at: u64,
}

impl<const N: usize> ManagedKey<N> {
    fn new(key_data: [u8; N], max_age: Duration, max_uses: u32) -> Self {
        Self {
            key: SecureKey::new(key_data, 1),
            max_age,
            max_uses,
            current_uses: 0,
            created_at: get_timestamp(),
        }
    }
    
    fn is_valid(&self) -> bool {
        let age = Duration::from_millis(get_timestamp() - self.created_at);
        age < self.max_age && self.current_uses < self.max_uses
    }
    
    fn use_key(&mut self) -> Result<&[u8; N], CryptoError> {
        if !self.is_valid() {
            return Err(CryptoError::KeyExpired);
        }
        
        self.current_uses += 1;
        Ok(self.key.as_bytes())
    }
    
    fn remaining_uses(&self) -> u32 {
        self.max_uses.saturating_sub(self.current_uses)
    }
    
    fn time_until_expiry(&self) -> Duration {
        let age = Duration::from_millis(get_timestamp() - self.created_at);
        self.max_age.saturating_sub(age)
    }
}

// Key rotation manager
#[derive(ZeroizeOnDrop)]
struct KeyRotationManager {
    current_key: Option<ManagedKey<32>>,
    next_key: Option<ManagedKey<32>>,
    rng: HardwareRng,
}

impl KeyRotationManager {
    fn new(mut rng: HardwareRng) -> Result<Self, CryptoError> {
        let initial_key = Self::generate_new_key(&mut rng)?;
        
        Ok(Self {
            current_key: Some(initial_key),
            next_key: None,
            rng,
        })
    }
    
    fn generate_new_key(rng: &mut HardwareRng) -> Result<ManagedKey<32>, CryptoError> {
        let mut key_data = [0u8; 32];
        rng.try_fill_bytes(&mut key_data)
            .map_err(|_| CryptoError::RngFailed)?;
        
        // Keys valid for 1 hour or 10000 uses
        let max_age = Duration::from_secs(3600);
        let max_uses = 10000;
        
        Ok(ManagedKey::new(key_data, max_age, max_uses))
    }
    
    fn get_current_key(&mut self) -> Result<&[u8; 32], CryptoError> {
        // Check if current key needs rotation
        if let Some(ref current) = self.current_key {
            if !current.is_valid() {
                self.rotate_keys()?;
            }
        } else {
            return Err(CryptoError::NoValidKey);
        }
        
        // Prepare next key if needed
        if self.next_key.is_none() {
            self.next_key = Some(Self::generate_new_key(&mut self.rng)?);
        }
        
        self.current_key.as_mut().unwrap().use_key()
    }
    
    fn rotate_keys(&mut self) -> Result<(), CryptoError> {
        // Move next key to current
        if let Some(next_key) = self.next_key.take() {
            self.current_key = Some(next_key);
        } else {
            // Generate new key if no next key available
            self.current_key = Some(Self::generate_new_key(&mut self.rng)?);
        }
        
        // Generate new next key
        self.next_key = Some(Self::generate_new_key(&mut self.rng)?);
        
        Ok(())
    }
    
    fn force_rotation(&mut self) -> Result<(), CryptoError> {
        self.current_key = None; // Force rotation on next access
        self.get_current_key().map(|_| ())
    }
}

// Usage example with automatic key management
fn managed_encryption_example() -> Result<(), CryptoError> {
    let hw_rng = HardwareRng::new(get_rng_peripheral())?;
    let mut key_manager = KeyRotationManager::new(hw_rng)?;
    
    // Process messages with automatic key rotation
    for i in 0..15000 {
        let message = format!("message {}", i);
        
        // Key automatically rotated when needed
        let key = key_manager.get_current_key()?;
        let encrypted = encrypt_message(key, message.as_bytes())?;
        
        transmit_encrypted_message(&encrypted)?;
        
        // Force rotation every 5000 messages for testing
        if i % 5000 == 0 {
            key_manager.force_rotation()?;
        }
    }
    
    // All keys automatically zeroized when key_manager is dropped
    Ok(())
}
```