# 2.2 Target Configuration

Choose your target configuration based on your hardware. Each includes optimized settings for crypto workloads.

#### 2.2.1 ARM Cortex-R5 (Xilinx Ultrascale+/Versal) {#arm-cortex-r5}

**⚡ Quick Start (5 minutes):**
```bash
# Install Rust + R5 target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add armv7r-none-eabi

# Essential tools for bare-metal development
cargo install cargo-binutils
rustup component add llvm-tools-preview

# Install debugging tools
# Ubuntu/Debian:
sudo apt install openocd gdb-multiarch
# macOS:
brew install openocd arm-none-eabi-gdb
# Windows: Download from ARM website
```

**Important Target Notes:**
- Use `armv7r-none-eabi` (not `armv7r-none-eabihf`) for better compatibility
- The hard-float variant (`eabihf`) has limited support in some tools
- probe-rs doesn't support Cortex-R5 yet - use OpenOCD + GDB
- Xilinx tools (XSCT) also work well for debugging

**Project Configuration:**
```toml
# .cargo/config.toml
[target.armv7r-none-eabi]
runner = "arm-none-eabi-gdb -x gdb_init.txt"
rustflags = [
  "-C", "link-arg=-Tlink.x",
  "-C", "target-cpu=cortex-r5",
  "-C", "target-feature=+dsp",  # DSP extensions
  "-C", "link-arg=--nmagic",    # No page alignment
]

[build]
target = "armv7r-none-eabi"

[profile.dev]
debug = true
opt-level = "s"
overflow-checks = true

[profile.release]
debug = false
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"
```

**Essential Dependencies for R5:**
```toml
# Cargo.toml
[dependencies]
cortex-r-rt = "0.1"        # Cortex-R runtime
panic-halt = "0.2"         # Simple panic handler
# Note: cortex-m crates don't work with Cortex-R!

# For no-std development
[dependencies]
heapless = { version = "0.8", default-features = false }
nb = "1.1"
embedded-hal = "1.0"

# Crypto libraries (no-std compatible)
sha2 = { version = "0.10", default-features = false }
aes = { version = "0.8", default-features = false }

```

**Memory Layout (memory.x):**
```text
/* memory.x - Typical Cortex-R5 memory layout */
MEMORY
{
  /* Tightly Coupled Memory (TCM) - fastest access */
  ATCM : ORIGIN = 0x00000000, LENGTH = 64K   /* Code TCM */
  BTCM : ORIGIN = 0x00020000, LENGTH = 64K   /* Data TCM */
  
  /* On-Chip Memory (OCM) */
  OCM  : ORIGIN = 0xFFFC0000, LENGTH = 256K  /* Shared memory */
  
  /* DDR Memory (adjust addresses for your platform) */
  DDR  : ORIGIN = 0x00100000, LENGTH = 1M    /* Main memory */
}

/* Stack configuration */
_stack_start = ORIGIN(BTCM) + LENGTH(BTCM);

/* Heap configuration (if using allocator) */
_heap_size = 0;  /* No heap for no_std */

SECTIONS
{
  /* Vector table at start of ATCM */
  .vector_table ORIGIN(ATCM) : {
    KEEP(*(.vector_table));
  } > ATCM
  
  /* Code in ATCM for best performance */
  .text : {
    *(.text .text.*);
  } > ATCM
  
  /* Read-only data */
  .rodata : {
    *(.rodata .rodata.*);
  } > ATCM
  
  /* Initialized data (copied from ATCM to BTCM) */
  .data : {
    *(.data .data.*);
  } > BTCM AT > ATCM
  
  /* Uninitialized data */
  .bss : {
    *(.bss .bss.*);
  } > BTCM
}
```

**OpenOCD Configuration (openocd.cfg):**
```tcl
# OpenOCD config for Cortex-R5 debugging
# Adjust for your specific hardware

# Interface configuration (common options)
# For Digilent JTAG:
source [find interface/ftdi/digilent-hs1.cfg]
# For J-Link:
# source [find interface/jlink.cfg]
# For ST-Link:
# source [find interface/stlink.cfg]

# Target configuration
# For Xilinx Zynq UltraScale+:
source [find target/xilinx_zynqmp.cfg]
# For TI Sitara:
# source [find target/am437x.cfg]

# Common R5 settings
adapter speed 4000
transport select jtag

# Initialize
init
targets
# Select R5 core (adjust number as needed)
targets 4
halt
```

**GDB Setup (.gdbinit or gdb_commands.txt):**
```gdb
# GDB initialization for Cortex-R5
target extended-remote localhost:3333

# Reset and halt
monitor reset halt

# Load program
load

# Set breakpoints
break main
break HardFault
break rust_begin_unwind

# Continue execution
continue
```

**Build and Debug Workflow:**
```bash
# 1. Build for Cortex-R5
cargo build --target armv7r-none-eabi --release

# 2. Convert to binary if needed
cargo objcopy --release -- -O binary app.bin

# 3. Start OpenOCD (terminal 1)
openocd -f openocd.cfg

# 4. Debug with GDB (terminal 2)
arm-none-eabi-gdb target/armv7r-none-eabi/release/your-app
(gdb) target remote localhost:3333
(gdb) load
(gdb) continue

# 5. View disassembly
cargo objdump --release -- -d > disasm.txt
```

**Common Issues and Solutions:**

| Issue | Solution |
|-------|----------|
| "can't find crate for `std`" | Ensure `#![no_std]` in main.rs |
| "error: requires `start` lang_item" | Add `#![no_main]` and use cortex-r-rt |
| "undefined reference to `memset`" | Add compiler-builtins or implement manually |
| Linking errors | Check memory.x addresses match your hardware |
| Hard fault on boot | Verify stack pointer and vector table |

**Performance Tips for Crypto:**
- Place hot code in ATCM (Instruction TCM) for zero-wait execution
- Use BTCM for stack and crypto working buffers
- Align crypto buffers to cache line boundaries (32/64 bytes)
- Use `core::hint::black_box()` to prevent optimization of sensitive ops
- Enable hardware crypto accelerators if available

#### 2.2.2 ARM Cortex-M Series {#arm-cortex-m-series}

**Common Cortex-M Targets:**
```bash
# Cortex-M0/M0+
rustup target add thumbv6m-none-eabi

# Cortex-M3
rustup target add thumbv7m-none-eabi

# Cortex-M4/M7 (with FPU)
rustup target add thumbv7em-none-eabihf

# Cortex-M33/M35P (with FPU and DSP)
rustup target add thumbv8m.main-none-eabihf
```

**Example Configuration (STM32F4):**
```toml
# .cargo/config.toml
[target.thumbv7em-none-eabihf]
runner = "probe-rs run --chip STM32F411RETx"
rustflags = [
  "-C", "link-arg=-Tlink.x",
]

[build]
target = "thumbv7em-none-eabihf"
```

**Key Differences from Cortex-R5:**
- Full probe-rs support for easy debugging
- Extensive HAL ecosystem (stm32f4xx-hal, nrf-hal, etc.)
- RTIC framework for real-time applications
- Different interrupt handling (NVIC vs VIC)

#### 2.2.3 RISC-V and Other Targets {#other-embedded-targets}

```bash
# RISC-V embedded targets
rustup target add riscv32imac-unknown-none-elf   # 32-bit RISC-V
rustup target add riscv32imc-unknown-none-elf    # Without atomics
rustup target add riscv64gc-unknown-none-elf     # 64-bit RISC-V

# ESP32 (Xtensa)
# Requires esp-rs toolchain
cargo install espup
espup install
```

**Cross-Platform no_std Code:**
When writing portable embedded code:
- Use `cfg` attributes for target-specific code
- Rely on embedded-hal traits for hardware abstraction
- Keep architecture-specific code in separate modules
- Test on multiple targets in CI