🍖 ham, a programming language made in Rust 

**NOTE: I'm learning rust, I am still a noob**

status: **alpha**

You can **download** it from [here](https://github.com/marc2332/ham/releases).

### Goals
- Speed
- Security
- Comfort

### Language Ideas
- Rust interoperability
- Ability to import externals files
- std library
- Low-level (networking, file system...) APIs

### Project ideas
- CD integration to release a new version in each commit
- More unit tests

### Example

```rust

fn calc(value){
    // If the value is 5 end the function
    if value == 5 {
        return 0
    }
    
    // Add 1 
    value.mut_sum(1)
        
    // Print it's value
    println(format("Value is {}", value))
        
    // Call the function again with the latest value    
    return calc(value)
}

// This will print from `Value is 1` to `Value is 5`
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

Run a project (This will run `1_project/src/main.ham` automatically):
```shell
ham run examples/1_project
```

### Contribuding

Compiling:
```shell
cargo build --release
```

Linting:
```shell
cargo clippy
```

Formatting:
```shell
cargo fmt
```

Testing:
```shell
cargo test
```

Running directly:
```shell
cargo run -- run examples/demo.ham
```

Made by Marc Espín Sanz

MIT License