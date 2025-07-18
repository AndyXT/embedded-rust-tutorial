# 2.1 Rust Installation and Toolchain

```bash
# Install Rust and essential embedded tools
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Core embedded development tools
cargo install cargo-binutils probe-rs --features cli cargo-embed

# Install targets for your hardware (choose what you need)
rustup target add thumbv7em-none-eabihf  # Cortex-M4F/M7F (most common)
rustup target add armv7r-none-eabihf     # Xilinx Cortex-R5F
rustup target add thumbv8m.main-none-eabihf # Cortex-M33F (with TrustZone)
```