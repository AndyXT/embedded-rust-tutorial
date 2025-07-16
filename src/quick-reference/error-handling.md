# 1.4 Error Handling Patterns

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `if (result < 0) return -1;` | `result.map_err(\|e\| MyError::from(e))?` | Use `Result<T, E>` type |
| `errno` | `Result<T, Error>` | Errors are values, not global state |
| `goto cleanup;` | Early return with `?` | Automatic error propagation |
| `assert(condition)` | `assert!(condition)` | Debug assertions |
| `abort()` | `panic!("message")` | Controlled program termination |

#### Result Type Usage

```rust
// C style error handling
// int divide(int a, int b, int* result) {
//     if (b == 0) return -1;
//     *result = a / b;
//     return 0;
// }

// Rust equivalent
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
```