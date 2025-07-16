# 4.2 Hardware Abstraction Patterns

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