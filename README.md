# odirect

Minimal cross-platform direct I/O abstraction for Rust.

## Features

- Linux: `O_DIRECT`
- macOS: `F_NOCACHE`
- Windows: `FILE_FLAG_NO_BUFFERING`

## Installation

```toml
[dependencies]
odirect = "0.2.0"
```

## Usage

```rust
use odirect::{open_direct_file, AccessMode};

fn main() {
    let file = open_direct_file(
        "data.bin",
        AccessMode::ReadWrite,
    );

    match file {
        Ok(_) => println!("opened successfully"),
        Err(e) => println!("error: {}", e),
    }
}
```

## Supported Access Modes

```rust
AccessMode::Read
AccessMode::Write
AccessMode::ReadWrite
```

## Notes

Direct/unbuffered I/O behavior differs across operating systems.

- Linux uses `O_DIRECT`
- macOS uses `F_NOCACHE`
- Windows uses `FILE_FLAG_NO_BUFFERING`

Some platforms may require aligned buffers and aligned reads/writes when performing direct I/O operations.
