# 2.3 Project Structure and Dependencies

```toml
# Cargo.toml - Essential crypto dependencies
[package]
name = "embedded-crypto-app"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core embedded
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"

# Crypto (no-std)
chacha20poly1305 = { version = "0.10", default-features = false }
aes-gcm = { version = "0.10", default-features = false }
sha2 = { version = "0.10", default-features = false }
subtle = { version = "2.5", default-features = false }
zeroize = { version = "1.6", default-features = false }

# Collections
heapless = "0.7"

[profile.release]
opt-level = "z"
lto = true
panic = "abort"
```

project/
├── Cargo.toml
├── .cargo/config.toml
├── memory.x
├── src/
│   ├── main.rs
│   ├── crypto/          # Crypto implementations
│   ├── hardware/        # Hardware abstraction
│   └── protocol/        # Communication protocols
```