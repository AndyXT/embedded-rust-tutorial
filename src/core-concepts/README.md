# 3. Core Language Concepts

<details>
<summary><strong>📚 Core Concepts Overview</strong> - Essential Rust concepts for C programmers</summary>

### Learning Path
1. **Start here:** [Ownership and Memory Management](./ownership.md) - The foundation of Rust
2. **Then:** [Error Handling](./error-handling.md) - Replace errno with Result types
3. **Next:** [Type System Advantages for Security](./type-system-advantages-for-security.md) - Leverage compile-time safety
4. **Finally:** [Advanced Type System Features](./advanced-types.md) - Apply to crypto development

### Key Differences from C
- **Memory safety without garbage collection** - Zero-cost abstractions
- **No null pointers** - Use Option<T> instead
- **No data races** - Ownership prevents concurrent access
- **Explicit error handling** - No hidden failure modes

### Quick Reference Links
- **Need immediate help?** → [Critical Differences and Gotchas](../quick-reference/gotchas.md)
- **Working with hardware?** → [Embedded-Specific Patterns](../embedded-patterns/README.md)
- **Implementing crypto?** → [Cryptography Implementation](../cryptography/README.md)

</details>

This section consolidates the essential Rust concepts that differ significantly from C, with focus on how they benefit cryptographic development. Each concept is explained once, thoroughly, with embedded crypto-specific examples.

## Sections

- [Learning Path](./learning-path.md)
- [Key Differences from C](./key-differences-from-c.md)
- [Quick Reference Links](./quick-reference-links.md)
- [3.1 Ownership and Memory Management](./ownership.md)
- [3.2 Error Handling Without Exceptions](./error-handling.md)
- [3.3 Type System Advantages for Security](./type-system-advantages-for-security.md)
- [3.4 Advanced Type System Features](./advanced-types.md)
- [3.5 Functional Programming and Data Processing](./functional.md)
