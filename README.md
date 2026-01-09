# els

Enhanced directory listing utility for Unix systems.

## Dependencies

Requires `less` for paged output. Falls back to stdout if unavailable.

## Usage

```shell
els [OPTIONS] [PATH]
```

**Options:**

- `-f, --full` - Full output mode (includes ACLs, owner, file type, preview)
- `-g, --filter <pattern>` - Filter results by substring (case-insensitive)
- `-h, --help` - Show help

**Examples:**

```shell
els                  # List current directory
els /home/user       # List specific directory
els -f .             # Full output mode
els -g test          # Filter files containing "test"
```

## Building

```shell
cargo build --release
```

Binary is output to `target/release/els`.

## Testing

Run the test suite:

```shell
cargo test
```

Run tests with output visible:

```shell
cargo test -- --nocapture
```

Run tests for a specific module:

```shell
cargo test utils::tests
cargo test colors::tests
```

## Cross-Compilation Release Builds

Build Linux release binaries via [cross](https://github.com/cross-rs/cross):

- `x86_64-unknown-linux-musl` (Linux x86_64)
- `aarch64-unknown-linux-musl` (Linux ARM64)

### Prerequisites

```shell
cargo install cross --git https://github.com/cross-rs/cross
rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
```

Docker must be installed and running.

### Build

```shell
make release
```

Binaries are output to `dist/`.

### macOS

macOS binaries must be built natively on a Mac:

```shell
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

## License

Licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.
