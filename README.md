
# odirect

Minimal cross-platform direct I/O abstraction for Rust.

## Features

- Linux: `O_DIRECT` with optional `O_DSYNC` / `O_SYNC`
- macOS: `F_NOCACHE`
- Windows: `FILE_FLAG_NO_BUFFERING` with optional write-through

## Installation

```toml
[dependencies]
odirect = "0.4.0"
```

## Usage

```rust
use odirect::{open_direct_file, AccessMode, Integrity};

fn main() {
    let file = open_direct_file(
        "data.bin",
        AccessMode::ReadWrite,
        Integrity::Data,
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

## Supported Integrity Levels

Controls durability guarantees when writing:

```rust
Integrity::Null   // No sync (default buffering behavior)
Integrity::Data   // Data integrity (Linux: O_DSYNC)
Integrity::File   // Full file integrity (Linux: O_SYNC)
```

## Platform Notes

Direct/unbuffered I/O behavior differs across operating systems:

- **Linux**: Uses `O_DIRECT`. Integrity levels map to `O_DSYNC` and `O_SYNC`.
- **macOS**: Uses `F_NOCACHE`. Integrity levels are not applied at open time—handle durability separately with `fsync` / `F_FULLFSYNC` after writes.
- **Windows**: Uses `FILE_FLAG_NO_BUFFERING`. Integrity levels other than `Null` add write-through behavior.

Some platforms may require aligned buffers and aligned reads/writes when performing direct I/O operations.


The key changes:
- Added `Integrity` to the usage example since it's now a required parameter
- Added an "Integrity Levels" section
- Updated platform notes to accurately reflect how each OS handles integrity
- Kept the overall structure and simplicity intact
