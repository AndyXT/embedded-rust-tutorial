# 1.3 Control Flow and Functions

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `int func(void)` | `fn func() -> i32` | Explicit return type |
| `void func(void)` | `fn func()` | Unit type `()` implied |
| `int* func()` | `fn func() -> Option<i32>` | Use `Option` instead of nullable pointers |
| `if (condition) { ... }` | `if condition { ... }` | No parentheses needed around condition |
| `for (int i = 0; i < n; i++)` | `for i in 0..n` | Iterator-based loops |
| `while (condition)` | `while condition` | Same syntax, no parentheses |
| `switch (value)` | `match value` | More powerful pattern matching |
| `do { ... } while (cond);` | `loop { ... if !cond { break; } }` | No direct equivalent, use loop + break |
| `goto label;` | *Not available* | Use structured control flow instead |