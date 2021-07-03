# 🍖 ham

ham is a general purpose language. It is heavily inspired by Rust and TypeScript.

Wanna try it out? [Install it](./introduction/installing.md)

### Goals
- Speed
- Security
- Comfort

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

