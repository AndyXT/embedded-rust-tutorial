# 4.5 DMA and Hardware Integration

This section consolidates all DMA and hardware integration patterns for high-performance crypto operations, providing comprehensive coverage of DMA-safe memory management, hardware crypto acceleration, and performance optimization.

#### Advanced DMA-Safe Memory Management

```rust
use cortex_m::singleton;
use core::sync::atomic::{AtomicBool, Ordering};

// DMA-safe buffer allocation with multiple buffer support
static DMA_BUFFER_IN_USE: [AtomicBool; 4] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];

// Multiple DMA buffers for concurrent operations
fn allocate_dma_buffer(size: DmaBufferSize) -> Option<DmaBuffer> {
    match size {
        DmaBufferSize::Small => {
            if DMA_BUFFER_IN_USE[0].compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                singleton!(: [u8; 256] = [0; 256]).map(|buf| DmaBuffer::Small(buf))
            } else if DMA_BUFFER_IN_USE[1].compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                singleton!(: [u8; 256] = [0; 256]).map(|buf| DmaBuffer::Small(buf))
            } else {
                None
            }
        }
        DmaBufferSize::Large => {
            if DMA_BUFFER_IN_USE[2].compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                singleton!(: [u8; 4096] = [0; 4096]).map(|buf| DmaBuffer::Large(buf))
            } else if DMA_BUFFER_IN_USE[3].compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                singleton!(: [u8; 4096] = [0; 4096]).map(|buf| DmaBuffer::Large(buf))
            } else {
                None
            }
        }
    }
}

enum DmaBuffer {
    Small(&'static mut [u8; 256]),
    Large(&'static mut [u8; 4096]),
}

impl DmaBuffer {
    fn as_mut_slice(&mut self) -> &mut [u8] {
        match self {
            DmaBuffer::Small(buf) => buf.as_mut(),
            DmaBuffer::Large(buf) => buf.as_mut(),
        }
    }
    
    fn capacity(&self) -> usize {
        match self {
            DmaBuffer::Small(_) => 256,
            DmaBuffer::Large(_) => 4096,
        }
    }
    
    fn buffer_id(&self) -> usize {
        match self {
            DmaBuffer::Small(_) => 0, // Simplified - would need proper tracking
            DmaBuffer::Large(_) => 2,
        }
    }
}

impl Drop for DmaBuffer {
    fn drop(&mut self) {
        // Secure zeroization and release buffer
        self.as_mut_slice().zeroize();
        let id = self.buffer_id();
        if id < 4 {
            DMA_BUFFER_IN_USE[id].store(false, Ordering::Release);
        }
    }
}

#[derive(Clone, Copy)]
enum DmaBufferSize {
    Small,  // 256 bytes
    Large,  // 4096 bytes
}

// Comprehensive DMA crypto operations
struct AdvancedDmaCrypto {
    dma_tx_channel: DmaChannel,
    dma_rx_channel: DmaChannel,
    crypto_peripheral: CryptoPeripheral,
    active_operations: heapless::Vec<DmaOperation, 8>,
}

#[derive(Clone, Copy)]
struct DmaOperation {
    operation_id: u32,
    operation_type: DmaOperationType,
    buffer_size: usize,
    status: DmaOperationStatus,
}

#[derive(Clone, Copy)]
enum DmaOperationType {
    Encrypt,
    Decrypt,
    Hash,
    KeyDerivation,
}

#[derive(Clone, Copy)]
enum DmaOperationStatus {
    Pending,
    InProgress,
    Complete,
    Error(u32),
}

impl AdvancedDmaCrypto {
    fn new(dma_tx: DmaChannel, dma_rx: DmaChannel, crypto: CryptoPeripheral) -> Self {
        Self {
            dma_tx_channel: dma_tx,
            dma_rx_channel: dma_rx,
            crypto_peripheral: crypto,
            active_operations: heapless::Vec::new(),
        }
    }
    
    fn encrypt_async(&mut self, data: DmaBuffer, key: &[u8; 32]) -> Result<u32, DmaError> {
        // Generate unique operation ID
        static mut OPERATION_COUNTER: u32 = 0;
        let operation_id = unsafe {
            OPERATION_COUNTER += 1;
            OPERATION_COUNTER
        };
        
        // Configure crypto peripheral for encryption
        self.crypto_peripheral.configure_aes_dma(key, AesMode::GCM)?;
        
        // Configure DMA for crypto operation
        self.dma_tx_channel.configure(
            data.as_mut_slice().as_ptr() as u32,
            self.crypto_peripheral.input_data_register(),
            data.as_mut_slice().len(),
            DmaDirection::MemoryToPeripheral,
        )?;
        
        self.dma_rx_channel.configure(
            self.crypto_peripheral.output_data_register(),
            data.as_mut_slice().as_mut_ptr() as u32,
            data.as_mut_slice().len() + 16, // Add space for GCM tag
            DmaDirection::PeripheralToMemory,
        )?;
        
        // Track operation
        let operation = DmaOperation {
            operation_id,
            operation_type: DmaOperationType::Encrypt,
            buffer_size: data.as_mut_slice().len(),
            status: DmaOperationStatus::Pending,
        };
        
        self.active_operations.push(operation)
            .map_err(|_| DmaError::TooManyOperations)?;
        
        // Start DMA transfers
        self.dma_tx_channel.start();
        self.dma_rx_channel.start();
        
        Ok(operation_id)
    }
    
    fn decrypt_async(&mut self, data: DmaBuffer, key: &[u8; 32]) -> Result<u32, DmaError> {
        // Similar to encrypt_async but for decryption
        static mut OPERATION_COUNTER: u32 = 0;
        let operation_id = unsafe {
            OPERATION_COUNTER += 1;
            OPERATION_COUNTER
        };
        
        // Configure for decryption
        self.crypto_peripheral.configure_aes_dma(key, AesMode::GCM)?;
        self.crypto_peripheral.set_decrypt_mode();
        
        // Configure DMA channels
        self.dma_tx_channel.configure(
            data.as_mut_slice().as_ptr() as u32,
            self.crypto_peripheral.input_data_register(),
            data.as_mut_slice().len(),
            DmaDirection::MemoryToPeripheral,
        )?;
        
        self.dma_rx_channel.configure(
            self.crypto_peripheral.output_data_register(),
            data.as_mut_slice().as_mut_ptr() as u32,
            data.as_mut_slice().len() - 16, // Remove GCM tag space
            DmaDirection::PeripheralToMemory,
        )?;
        
        let operation = DmaOperation {
            operation_id,
            operation_type: DmaOperationType::Decrypt,
            buffer_size: data.as_mut_slice().len(),
            status: DmaOperationStatus::Pending,
        };
        
        self.active_operations.push(operation)
            .map_err(|_| DmaError::TooManyOperations)?;
        
        self.dma_tx_channel.start();
        self.dma_rx_channel.start();
        
        Ok(operation_id)
    }
    
    fn hash_async(&mut self, data: DmaBuffer) -> Result<u32, DmaError> {
        static mut OPERATION_COUNTER: u32 = 0;
        let operation_id = unsafe {
            OPERATION_COUNTER += 1;
            OPERATION_COUNTER
        };
        
        // Configure crypto peripheral for hashing
        self.crypto_peripheral.configure_hash_dma(HashAlgorithm::Sha256)?;
        
        // Configure DMA for hash operation
        self.dma_tx_channel.configure(
            data.as_mut_slice().as_ptr() as u32,
            self.crypto_peripheral.hash_input_register(),
            data.as_mut_slice().len(),
            DmaDirection::MemoryToPeripheral,
        )?;
        
        let operation = DmaOperation {
            operation_id,
            operation_type: DmaOperationType::Hash,
            buffer_size: data.as_mut_slice().len(),
            status: DmaOperationStatus::Pending,
        };
        
        self.active_operations.push(operation)
            .map_err(|_| DmaError::TooManyOperations)?;
        
        self.dma_tx_channel.start();
        
        Ok(operation_id)
    }
    
    fn check_operation_status(&mut self, operation_id: u32) -> Option<DmaOperationStatus> {
        for operation in &mut self.active_operations {
            if operation.operation_id == operation_id {
                // Update status based on DMA and crypto peripheral state
                if self.dma_tx_channel.has_error() || self.dma_rx_channel.has_error() {
                    operation.status = DmaOperationStatus::Error(0x01);
                } else if self.crypto_peripheral.has_error() {
                    operation.status = DmaOperationStatus::Error(0x02);
                } else if self.is_operation_complete(operation) {
                    operation.status = DmaOperationStatus::Complete;
                } else if self.is_operation_in_progress(operation) {
                    operation.status = DmaOperationStatus::InProgress;
                }
                
                return Some(operation.status);
            }
        }
        None
    }
    
    fn get_completed_operations(&mut self) -> heapless::Vec<u32, 8> {
        let mut completed = heapless::Vec::new();
        
        // Find completed operations
        for operation in &self.active_operations {
            if matches!(operation.status, DmaOperationStatus::Complete) {
                let _ = completed.push(operation.operation_id);
            }
        }
        
        // Remove completed operations from active list
        self.active_operations.retain(|op| !matches!(op.status, DmaOperationStatus::Complete));
        
        completed
    }
    
    fn is_operation_complete(&self, operation: &DmaOperation) -> bool {
        match operation.operation_type {
            DmaOperationType::Encrypt | DmaOperationType::Decrypt => {
                self.dma_tx_channel.is_complete() && 
                self.dma_rx_channel.is_complete() && 
                self.crypto_peripheral.is_operation_complete()
            }
            DmaOperationType::Hash => {
                self.dma_tx_channel.is_complete() && 
                self.crypto_peripheral.is_hash_complete()
            }
            DmaOperationType::KeyDerivation => {
                self.crypto_peripheral.is_key_derivation_complete()
            }
        }
    }
    
    fn is_operation_in_progress(&self, operation: &DmaOperation) -> bool {
        match operation.operation_type {
            DmaOperationType::Encrypt | DmaOperationType::Decrypt => {
                self.dma_tx_channel.is_active() || 
                self.dma_rx_channel.is_active() || 
                self.crypto_peripheral.is_busy()
            }
            DmaOperationType::Hash => {
                self.dma_tx_channel.is_active() || 
                self.crypto_peripheral.is_busy()
            }
            DmaOperationType::KeyDerivation => {
                self.crypto_peripheral.is_busy()
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DmaError {
    ChannelBusy,
    InvalidConfiguration,
    TooManyOperations,
    HardwareError(u32),
    BufferAlignment,
    BufferSize,
}

#[derive(Clone, Copy)]
enum HashAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

// Interrupt handlers for DMA completion
#[interrupt]
fn DMA1_STREAM0() {
    static mut DMA_CRYPTO: Option<AdvancedDmaCrypto> = None;
    
    if let Some(ref mut crypto) = DMA_CRYPTO {
        // Handle TX DMA completion
        if crypto.dma_tx_channel.is_complete() {
            crypto.dma_tx_channel.clear_interrupt_flags();
            
            // Check if this completes any operations
            let completed_ops = crypto.get_completed_operations();
            for op_id in completed_ops {
                // Signal completion to main thread
                signal_operation_complete(op_id);
            }
        }
        
        if crypto.dma_tx_channel.has_error() {
            crypto.dma_tx_channel.clear_error_flags();
            handle_dma_error(DmaError::HardwareError(0x01));
        }
    }
}

#[interrupt]
fn DMA1_STREAM1() {
    static mut DMA_CRYPTO: Option<AdvancedDmaCrypto> = None;
    
    if let Some(ref mut crypto) = DMA_CRYPTO {
        // Handle RX DMA completion
        if crypto.dma_rx_channel.is_complete() {
            crypto.dma_rx_channel.clear_interrupt_flags();
            
            let completed_ops = crypto.get_completed_operations();
            for op_id in completed_ops {
                signal_operation_complete(op_id);
            }
        }
        
        if crypto.dma_rx_channel.has_error() {
            crypto.dma_rx_channel.clear_error_flags();
            handle_dma_error(DmaError::HardwareError(0x02));
        }
    }
}

// High-level DMA crypto API
pub struct DmaCryptoManager {
    dma_crypto: AdvancedDmaCrypto,
    pending_operations: heapless::FnvIndexMap<u32, DmaBuffer, 8>,
}

impl DmaCryptoManager {
    pub fn new(dma_tx: DmaChannel, dma_rx: DmaChannel, crypto: CryptoPeripheral) -> Self {
        Self {
            dma_crypto: AdvancedDmaCrypto::new(dma_tx, dma_rx, crypto),
            pending_operations: heapless::FnvIndexMap::new(),
        }
    }
    
    pub fn encrypt_data(&mut self, data: &[u8], key: &[u8; 32]) -> Result<u32, DmaError> {
        // Allocate appropriate DMA buffer
        let buffer_size = if data.len() <= 256 { 
            DmaBufferSize::Small 
        } else { 
            DmaBufferSize::Large 
        };
        
        let mut buffer = allocate_dma_buffer(buffer_size)
            .ok_or(DmaError::ChannelBusy)?;
        
        // Copy data to DMA buffer
        if data.len() > buffer.capacity() {
            return Err(DmaError::BufferSize);
        }
        
        buffer.as_mut_slice()[..data.len()].copy_from_slice(data);
        
        // Start async encryption
        let operation_id = self.dma_crypto.encrypt_async(buffer, key)?;
        
        // Track buffer for this operation
        self.pending_operations.insert(operation_id, buffer)
            .map_err(|_| DmaError::TooManyOperations)?;
        
        Ok(operation_id)
    }
    
    pub fn get_encrypted_data(&mut self, operation_id: u32) -> Option<Result<heapless::Vec<u8, 4096>, DmaError>> {
        if let Some(status) = self.dma_crypto.check_operation_status(operation_id) {
            match status {
                DmaOperationStatus::Complete => {
                    if let Some(buffer) = self.pending_operations.remove(&operation_id) {
                        let mut result = heapless::Vec::new();
                        let _ = result.extend_from_slice(buffer.as_mut_slice());
                        Some(Ok(result))
                    } else {
                        Some(Err(DmaError::InvalidConfiguration))
                    }
                }
                DmaOperationStatus::Error(code) => {
                    // Clean up failed operation
                    self.pending_operations.remove(&operation_id);
                    Some(Err(DmaError::HardwareError(code)))
                }
                _ => None, // Still in progress
            }
        } else {
            Some(Err(DmaError::InvalidConfiguration))
        }
    }
}

// Helper functions for interrupt handling
fn signal_operation_complete(operation_id: u32) {
    // Implementation would signal main thread about completion
    // Could use a queue, flag, or other mechanism
}

fn handle_dma_error(error: DmaError) {
    // Implementation would handle DMA errors appropriately
    // Could log, reset, or take other recovery actions
}
```