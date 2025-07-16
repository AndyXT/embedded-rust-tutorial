# 1.2 Memory and Pointer Patterns

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `uint8_t buffer[256];` | `let mut buffer = [0u8; 256];` | Stack array, bounds checked |
| `uint8_t* ptr = malloc(size);` | `let mut vec = vec![0u8; size];` | Heap allocation (needs `alloc` feature) |
| `if (ptr != NULL)` | `if let Some(ref_val) = option` | No null pointers, use `Option<T>` |
| `ptr[i]` | `slice[i]` | Bounds checked at runtime |
| `memcpy(dst, src, len)` | `dst.copy_from_slice(src)` | Safe, bounds checked copy |
| `memset(ptr, 0, len)` | `slice.fill(0)` | Safe initialization |
| `free(ptr)` | *automatic* | Memory freed when owner goes out of scope |
| `ptr++` | `ptr = ptr.offset(1)` | Pointer arithmetic requires `unsafe` |