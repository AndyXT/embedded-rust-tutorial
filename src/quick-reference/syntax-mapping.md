# 1.1 C-to-Rust Syntax Mapping

#### Basic Declarations and Types

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `int x = 5;` | `let x = 5i32;` | Immutable by default, explicit type optional |
| `int x; x = 5;` | `let mut x: i32; x = 5;` | Must declare mutability explicitly |
| `const int MAX = 100;` | `const MAX: i32 = 100;` | Compile-time constants, type required |
| `#define SIZE 256` | `const SIZE: usize = 256;` | Type-safe constants instead of macros |
| `static int counter = 0;` | `static mut COUNTER: i32 = 0;` | Global mutable state requires `unsafe` |
| `typedef struct {...} my_t;` | `struct MyStruct {...}` | Rust naming conventions (PascalCase) |
| `enum state { IDLE, BUSY };` | `enum State { Idle, Busy }` | More powerful enums with data |
| `enum state { IDLE, BUSY }; state s = IDLE;` | `enum State { Idle, Busy } let s = State::Idle;` | Enum variants are namespaced |
| `switch (state) { case IDLE: ... }` | `match state { State::Idle => ... }` | Pattern matching with exhaustiveness checking |
| `enum result { OK, ERROR }; int value;` | `enum Result<T> { Ok(T), Err(E) }` | Enums can carry data |
| `union data { int i; float f; };` | `union Data { i: i32, f: f32 }` | Requires `unsafe` to access |
| `char str[256];` | `let mut str = [0u8; 256];` | Use byte arrays for C-style strings |
| `char* str = "hello";` | `let str = "hello";` | String literals are `&str` |

#### Functions and Control Flow

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

#### Traits and Methods

*For detailed examples and crypto-specific applications → [Advanced Type System Features](../core-concepts/advanced-types.md)*

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `typedef int (*func_ptr)(void*);` | `trait Behavior { fn method(&self); }` | Traits provide safe alternatives to function pointers |
| `struct_ptr->method(args)` | `obj.method(args)` | Method calls use dot notation |
| `obj.field = value; obj.method();` | `obj.field = value; obj.method();` | Same syntax for field access and methods |
| `void (*callback)(int);` | `impl Fn(i32) -> ()` | Closures and trait objects for callbacks |
| `struct vtable { int (*func1)(); };` | `dyn Trait` | Dynamic dispatch via trait objects |
| `interface->method()` | `trait_obj.method()` | Polymorphism through traits |

#### Iterators and Closures

*For detailed examples and crypto-specific applications → [Functional Programming and Data Processing](../core-concepts/functional.md)*

| C Pattern | Rust Equivalent | Notes |
|-----------|----------------|-------|
| `for (int i = 0; i < len; i++) { arr[i] = f(arr[i]); }` | `arr.iter_mut().for_each(\|x\| *x = f(*x));` | Iterator-based data processing |
| `int sum = 0; for (...) sum += arr[i];` | `let sum: i32 = arr.iter().sum();` | Built-in aggregation methods |
| `filter_array(arr, predicate_func)` | `arr.iter().filter(\|&x\| predicate(x))` | Functional filtering |
| `map_array(arr, transform_func)` | `arr.iter().map(\|&x\| transform(x))` | Functional transformation |
| `int (*operation)(int) = add_one;` | `let operation = \|x\| x + 1;` | Closures capture environment |
| `qsort(arr, len, sizeof(int), compare)` | `arr.sort_by(\|a, b\| a.cmp(b));` | Closure-based sorting |