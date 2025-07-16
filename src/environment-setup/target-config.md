# 2.2 Target Configuration

<details>
<summary><strong>▶️ Hardware-Specific Configurations</strong> - Target-specific setup instructions</summary>

Choose your target configuration based on your hardware. Each includes optimized settings for crypto workloads.

#### 2.2.1 Xilinx Ultrascale+ (Cortex-R5) {#xilinx-ultrascale-cortex-r5}

**⚡ Quick Start (5 minutes):**
```bash
# Install Rust + R5 target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add armv7r-none-eabihf

# Essential tools for R5 ELF generation
cargo install cargo-binutils
rustup component add llvm-tools-preview

# Install debugging tools (OpenOCD + GDB required for R5)
# Ubuntu/Debian:
sudo apt install openocd gdb-multiarch
# Or download ARM GDB: arm-none-eabi-gdb
```

**Why OpenOCD + GDB instead of probe-rs for R5:**
- probe-rs has limited Cortex-R5 support (especially for Xilinx parts)
- OpenOCD provides mature R5 debugging with semihosting
- GDB integration works reliably with Xilinx toolchain
- Hardware breakpoints and real-time debugging work properly

**Project Configuration:**
```toml
# .cargo/config.toml - ZynqMP/Versal configuration
[target.armv7r-none-eabihf]
runner = "arm-none-eabi-gdb -x gdb_init.txt"
rustflags = [
  "-C", "link-arg=-Tlink.x",
  "-C", "target-cpu=cortex-r5",
  "-C", "target-feature=+vfp3",
  "-C", "link-arg=--nmagic",
  "-C", "link-arg=-Tdefmt.x",  # Optional: for defmt logging
]

[build]
target = "armv7r-none-eabihf"

# Build settings for proper ELF generation
[profile.dev]
debug = true
opt-level = 1

[profile.release]
debug = true      # Keep debug info for GDB
opt-level = "s"   # Optimize for size
lto = true
```

**Essential Dependencies for R5:**
```toml
# Cargo.toml - R5-specific dependencies
[dependencies]
cortex-m = "0.7"           # Core ARM support
cortex-m-rt = "0.7"        # Runtime and startup
panic-halt = "0.2"         # Panic handler
linked_list_allocator = "0.10"  # Optional: heap allocator

# For ELF file generation and debugging
[dependencies.cortex-r]
version = "0.1"
features = ["inline-asm"]

# Build dependencies for linker script processing
[build-dependencies]
cc = "1.0"
```

**Memory Layout (memory.x):**
```rust
/* memory.x - Optimized for Xilinx R5 crypto operations */
MEMORY
{
  /* Tightly Coupled Memory - fastest access */
  ATCM : ORIGIN = 0x00000000, LENGTH = 64K   /* Instructions, critical crypto code */
  BTCM : ORIGIN = 0x00020000, LENGTH = 64K   /* Stack, local variables */
  
  /* On-Chip Memory - shared between cores */
  OCM  : ORIGIN = 0xFFFC0000, LENGTH = 256K  /* Crypto workspace, buffers */
  
  /* DDR - large data structures */
  DDR  : ORIGIN = 0x00100000, LENGTH = 2G    /* Large crypto operations */
}

/* Stack in fast BTCM */
_stack_start = ORIGIN(BTCM) + LENGTH(BTCM);

/* Crypto workspace in OCM for inter-core sharing */
_crypto_workspace = ORIGIN(OCM);
_crypto_workspace_size = LENGTH(OCM);

/* Place crypto-critical code in ATCM */
SECTIONS
{
  .crypto_code : {
    *(.crypto_critical)
  } > ATCM
}
```

**OpenOCD Configuration (openocd.cfg):**
```tcl
# OpenOCD config for Xilinx ZynqMP Cortex-R5
# Note: probe-rs doesn't support R5 well, use OpenOCD + GDB

source [find interface/ftdi/digilent-hs1.cfg]  # Or your JTAG adapter
source [find target/xilinx_zynqmp.cfg]

# Configure for R5 core debugging
set _CHIPNAME zynqmp
set _TARGETNAME $_CHIPNAME.r5

# R5 specific settings
$_TARGETNAME configure -rtos auto
$_TARGETNAME configure -coreid 0

# Enable semihosting for printf debugging
$_TARGETNAME configure -semihosting-enable

# Memory map for crypto regions
$_TARGETNAME configure -work-area-phys 0xFFFC0000 -work-area-size 0x40000

init
reset init
```

**GDB Setup (gdb_init.txt):**
```gdb
# GDB initialization for Xilinx R5 debugging
target extended-remote localhost:3333

# Load symbols and set up memory regions
monitor reset halt
monitor zynqmp pmufw /path/to/pmufw.elf
monitor zynqmp fsbl /path/to/fsbl.elf

# Set up memory regions for crypto debugging
monitor mww 0xFF5E0200 0x0100    # Enable R5 debug
monitor mww 0xFF9A0000 0x80000218 # Configure R5 clocks

# Load the ELF file
load

# Set breakpoint at main
break main

# Enable semihosting for debug output
monitor arm semihosting enable

# Start execution
continue
```

**Build and Debug Workflow:**
```bash
# 1. Build ELF file for R5
cargo build --target armv7r-none-eabihf --release

# 2. Generate additional debug formats
cargo objcopy --target armv7r-none-eabihf --release -- -O binary target/armv7r-none-eabihf/release/app.bin
cargo objdump --target armv7r-none-eabihf --release -- -d > disassembly.txt

# 3. Start OpenOCD (in separate terminal)
openocd -f openocd.cfg

# 4. Debug with GDB
arm-none-eabi-gdb target/armv7r-none-eabihf/release/your-app -x gdb_init.txt

# 5. Alternative: Use Xilinx tools
# xsct -interactive
# connect
# targets -set -filter {name =~ "*R5*#0"}
# dow target/armv7r-none-eabihf/release/your-app
# con
```

**Debugging Tips for R5:**
- **Use OpenOCD + GDB instead of probe-rs** - probe-rs has limited R5 support
- **Enable semihosting** for printf-style debugging without UART
- **Use JTAG adapters** like Digilent HS1/HS2 or Platform Cable USB II
- **Memory regions matter** - place crypto code in ATCM for best performance
- **Cache coherency** - use appropriate memory barriers for crypto operations

#### 2.2.2 ARM Cortex-M Series {#arm-cortex-m-series}

```toml
# .cargo/config.toml - STM32F4 example
[target.thumbv7em-none-eabihf]
runner = "probe-rs run --chip STM32F411RETx"
rustflags = [
  "-C", "link-arg=-Tlink.x",
  "-C", "target-cpu=cortex-m4",
  "-C", "target-feature=+fp-armv8d16",
]

[build]
target = "thumbv7em-none-eabihf"
```

#### 2.2.3 Other Embedded Targets {#other-embedded-targets}

```bash
# Additional targets for specialized applications
rustup target add riscv32imac-unknown-none-elf  # RISC-V with crypto extensions
rustup target add thumbv6m-none-eabi             # Cortex-M0+ (resource constrained)
```