# 1.4 Error Handling Patterns

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `if (result < 0) return -1;` | `result.map_err(\|e\| MyError::from(e))?` | Use `Result<T, E>` type |
| `errno` | `Result<T, Error>` | Errors are values, not global state |
| `goto cleanup;` | Early return with `?` | Automatic error propagation |
| `assert(condition)` | `assert!(condition)` | Debug assertions |
| `abort()` | `panic!("message")` | Controlled program termination |

#### Result Type Usage

> **Dependencies needed**: `cortex-r-rt = "0.1"`, `panic-halt = "0.2"`

```rust
#![no_std]
#![no_main]

use panic_halt as _;

// Rust error handling with Result type
fn divide(a: i32, b: i32) -> Result<i32, &'static str> {
    if b == 0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}

// Usage with ? operator for error propagation
fn calculate() -> Result<i32, &'static str> {
    let result = divide(10, 2)?;  // Automatically propagates error
    Ok(result * 2)
}

#[no_mangle]
pub unsafe extern "C" fn Reset() -> ! {
    match calculate() {
        Ok(value) => {
            // In embedded: store result, send via UART, etc.
            let _ = value; // Result would be 20
        },
        Err(_e) => {
            // In embedded: set error flag, halt, etc.
        }
    }
    
    loop {
        // Main embedded loop
        // In real code: cortex_r::asm::wfi();
    }
}
```