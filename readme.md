🍖 ham, a programming language made in rust 

status: **alpha**

### Goals
- Speed
- Security
- Comfort

### Ideas
- Rust interoperability
- Ability to import externals files
- std library
- Low-level (networking, file system...) APIs

### Example

```rust

fn calc(value){
    if value == 5 {
        return 0
    }
    value.mut_sum(1)
    println(format("Value is {}", value))
    return calc(value)
}

let _ = calc(0)
```

There are more examples in /examples.

### About
ham is a general purpose language. It is heavily inspired by Rust and TypeScript.

### Usage

Built-in repl:
```shell
ham repl
```

Run files:
```shell
ham run examples/demo.ham
```

### Building

Compile:
```shell
cargo build --release
```

Running directly:
```shell
cargo run -- run examples/demo.ham
```

Made by Marc Espín Sanz

MIT License