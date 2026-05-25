
# odirect

Minimal cross-platform direct I/O abstraction for Rust.

## Features

- Linux: `O_DIRECT`
- macOS: `F_NOCACHE`
- Windows: `FILE_FLAG_NO_BUFFERING`

## Installation

```toml
[dependencies]
odirect = "0.5.0"
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

## Platform Notes

Direct/unbuffered I/O behavior differs across operating systems:

- **Linux**: Uses `O_DIRECT` to bypass the page cache.
- **macOS**: Uses `F_NOCACHE` to disable file caching.
- **Windows**: Uses `FILE_FLAG_NO_BUFFERING` for unbuffered I/O.

All platforms may require aligned buffers and aligned reads/writes (typically 
512 or 4096 bytes, depending on the underlying device) when performing direct 
I/O operations. Durability guarantees (fsync, F_FULLFSYNC, FlushFileBuffers) 
must be handled separately by the caller after writes.
