# Contributing

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

Install mdbook:
```shell
cargo install mdbook
```

Build the book:
```shell
mdbook build
```

Watch for changes on the book:
```shell
mdbook watch
```

Running directly:
```shell
cargo run -- run examples/demo.ham
```