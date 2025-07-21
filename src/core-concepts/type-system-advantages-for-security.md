# 3.3 Type System Advantages for Security

Rust's type system encodes security invariants at compile time, preventing entire classes of cryptographic vulnerabilities that are common in C implementations.

## Type-Safe Protocol State Machines




```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;


use core::mem;
use heapless::Vec;

use core::fmt;
use core::result::Result;

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
    
    fn receive_encrypted(&mut self) -> CryptoResult<heapless::Vec<u8, 32>> {
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

## Const Generics for Compile-Time Crypto Parameters

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

#[derive(Debug)]
pub struct CryptoError(&'static str);


use zeroize::{Zeroize, ZeroizeOnDrop};
use core::mem;
use heapless::Vec;

use core::fmt;
use core::result::Result;

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
fn encrypt_with_aes256(key: &Aes256Key, plaintext: &[u8]) -> CryptoResult<heapless::Vec<u8, 32>> {
    // key is guaranteed to be exactly 32 bytes at compile time
    let cipher = Aes256::new(key.as_bytes());
    cipher.encrypt(plaintext)
}

fn encrypt_with_chacha20(key: &ChaCha20Key, nonce: &[u8; 12], plaintext: &[u8]) -> CryptoResult<heapless::Vec<u8, 32>> {
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

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

## Newtype Pattern for Domain-Specific Security

```rust
#![no_std]
#![no_main]

use panic_halt as _;

use core::{fmt, result::Result};
use heapless::{Vec, String, consts::*};
type Vec32<T> = Vec<T, U32>;
type Vec256<T> = Vec<T, U256>;
type String256 = String<U256>;


use zeroize::{Zeroize, ZeroizeOnDrop};
use core::mem;
use heapless::Vec;

use core::fmt;
use core::result::Result;

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
) -> CryptoResult<heapless::Vec<u8, 32>> {
    // Function signature prevents mixing up parameters
    let cipher = ChaCha20Poly1305::new(key.0);
    cipher.encrypt(nonce, plaintext.as_bytes(), aad.as_bytes())
}

fn aead_decrypt(
    key: KeyMaterial,
    nonce: &[u8; 12],
    ciphertext: CiphertextData,
    aad: AuthenticatedData,
) -> CryptoResult<heapless::Vec<u8, 32>> {
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

#[cortex_r_rt::entry]
fn main() -> ! {
    // Example code execution
    loop {}
}
```

**→ Next:** [Advanced Type System Features](../core-concepts/advanced-types.md) - Enums, traits, and methods for crypto development
```