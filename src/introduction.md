# Embedded Rust Tutorial for Cryptography Engineers

*A streamlined guide for experienced embedded cryptography C programmers transitioning to Rust*

> **⚠️ Important: Code Examples and Compilation**
> 
> All code examples in this book are written for **ARM Cortex-R5** with `no_std` (no standard library) and no heap allocation. The web-based Rust Playground cannot compile these examples. To run the examples:
> 
> 1. **Copy the code** using the copy button
> 2. **Set up a local Cortex-R5 project** (see [Environment Setup](environment-setup/README.md))
> 3. **Build with**: `cargo build --target armv7r-none-eabi --release`
> 
> Each example is complete and will compile when proper dependencies are added to `Cargo.toml`.

As an experienced embedded cryptography engineer, you understand the critical importance of correctness and security in cryptographic implementations. A single buffer overflow, timing leak, or use-after-free can compromise an entire system. Traditional C development requires constant vigilance against these threats while managing the complexities of embedded systems.

Rust addresses these challenges while maintaining the performance and control essential for cryptographic operations. This tutorial is specifically designed for experienced C programmers who need to become productive in Rust quickly, without wading through basic programming concepts.

## 🚀 Quick Start Navigation

**For immediate productivity:** Jump directly to [Quick Reference](quick-reference/README.md) for C-to-Rust lookup tables.

**For systematic learning:** Follow the linear path: [Environment Setup](environment-setup/README.md) → [Core Concepts](core-concepts/README.md) → [Embedded Patterns](embedded-patterns/README.md) → [Crypto Implementation](cryptography/README.md)

**For specific needs:** Use the navigation sidebar or search functionality to find specific topics.

## 📖 How to Use This Book

### Document Usage Modes

**🔍 Reference Mode:** Use this book as a lookup reference
- Navigate to any section using the sidebar table of contents
- Use the search functionality (🔍) to find specific C patterns or Rust concepts
- Each section is self-contained with complete examples
- Quick reference tables provide immediate answers

**📚 Tutorial Mode:** Follow the book linearly for systematic learning
- Start with [Environment Setup](environment-setup/README.md) if you're new to Rust
- Progress through [Core Concepts](core-concepts/README.md) for foundational understanding
- Continue to [Embedded Patterns](embedded-patterns/README.md) for embedded-specific knowledge
- Finish with [Crypto Implementation](cryptography/README.md) for advanced topics

**🎯 Targeted Learning:** Focus on specific areas of interest
- Each major section includes cross-references to related topics
- Prerequisites are clearly marked for each advanced topic
- Examples build incrementally but can be understood independently

### Navigation Features

- **Sidebar Navigation:** Complete table of contents with expandable sections
- **Search Functionality:** Full-text search across all content
- **Cross-references:** Follow links to related concepts throughout the book
- **Previous/Next Navigation:** Move sequentially through chapters
- **Code Examples:** All examples are complete and compilable
- **Copy-to-Clipboard:** Click the copy button on code blocks

## 🎯 Learning Paths

### Path 1: Quick Productivity (30 minutes)
1. [C-to-Rust Syntax Mapping](quick-reference/syntax-mapping.md) - Essential syntax translations
2. [Memory and Pointer Patterns](quick-reference/memory-patterns.md) - Memory management equivalents
3. [Critical Differences and Gotchas](quick-reference/gotchas.md) - Avoid common pitfalls

### Path 2: Systematic Foundation (2-3 hours)
1. [Environment Setup](environment-setup/README.md) - Get your development environment ready
2. [Core Language Concepts](core-concepts/README.md) - Understand Rust's fundamental differences
3. [Embedded-Specific Patterns](embedded-patterns/README.md) - Learn embedded Rust patterns
4. [Quick Reference](quick-reference/README.md) - Build your reference toolkit

### Path 3: Crypto-Focused Deep Dive (4-6 hours)
1. Complete Path 2 above
2. [Cryptography Implementation](cryptography/README.md) - Advanced crypto-specific patterns
3. [Migration and Integration](migration/README.md) - Practical migration strategies

### Path 4: Specific Topic Focus
- **Memory Safety:** [Ownership](core-concepts/ownership.md) → [Memory Model](core-concepts/memory-model.md) → [Safety Guarantees](core-concepts/safety.md)
- **Hardware Integration:** [No-std Programming](embedded-patterns/no-std.md) → [Hardware Abstraction](embedded-patterns/hardware-abstraction.md) → [DMA Integration](embedded-patterns/dma-integration.md)
- **Secure Implementation:** [Secure Patterns](cryptography/secure-patterns.md) → [Constant-Time](cryptography/constant-time.md) → [Side-Channel Mitigations](cryptography/side-channels.md)

## 🔧 Why This Tutorial is Different

- **Assumes C expertise** - No basic programming explanations
- **Crypto-focused examples** - All examples relevant to cryptographic engineering
- **Quick reference first** - Immediate productivity through comprehensive lookup tables
- **Embedded-specific** - Covers no-std, hardware integration, and real-time constraints
- **Streamlined organization** - Eliminates redundancy, focuses on practical differences

## 🎁 What You'll Gain

- **Memory safety without performance overhead** - Zero-cost abstractions
- **Automatic key material zeroization** - Compiler-enforced security
- **Type-safe protocol state machines** - Prevent protocol implementation errors
- **Compile-time prevention of common crypto vulnerabilities** - Catch bugs before they ship
- **Clear boundaries between safe and unsafe code** - Minimize attack surface

## 📚 Book Structure

This book is organized into six main sections:

1. **[Quick Reference](quick-reference/README.md)** - Immediate lookup tables for C-to-Rust translations
2. **[Environment Setup](environment-setup/README.md)** - Get your development environment configured
3. **[Core Language Concepts](core-concepts/README.md)** - Understand Rust's fundamental differences from C
4. **[Embedded-Specific Patterns](embedded-patterns/README.md)** - Learn embedded Rust development patterns
5. **[Cryptography Implementation](cryptography/README.md)** - Advanced patterns for secure crypto code
6. **[Migration and Integration](migration/README.md)** - Practical strategies for adopting Rust

Each section builds upon previous knowledge but can also be used independently for reference.

---

**Ready to get started?** Choose your learning path above or jump directly to the [Quick Reference](quick-reference/README.md) for immediate productivity.