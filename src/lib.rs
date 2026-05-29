use std::fs::File;
use std::fs::OpenOptions;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
#[allow(dead_code)]
use std::os::unix::fs::OpenOptionsExt;

#[cfg(target_os = "macos")]
#[allow(dead_code)]
use std::os::unix::io::AsRawFd;

#[cfg(target_os = "windows")]
#[allow(dead_code)]
use std::os::windows::fs::OpenOptionsExt;

#[allow(dead_code)]
pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}

/// Opens a file with direct I/O (bypasses OS cache).
/// 
/// # Platform-specific behavior
/// - Linux/FreeBSD: Uses O_DIRECT
/// - macOS: Uses F_NOCACHE via fcntl
/// - Windows: Uses FILE_FLAG_NO_BUFFERING (0x20000000)
///
/// # User responsibilities
/// - Buffers must be aligned to block size (typically 4096 bytes)
/// - I/O sizes must be multiples of block size
/// - File offsets must be block-aligned
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
#[allow(dead_code)]
pub fn open_direct_file(
    path: &str,
    mode: AccessMode,
) -> std::io::Result<File> {
    let mut opts = OpenOptions::new();

    match mode {
        AccessMode::Read => {
            opts.read(true);
        }
        AccessMode::Write => {
            opts.write(true);
        }
        AccessMode::ReadWrite => {
            opts.read(true).write(true);
        }
    }

    opts.custom_flags(libc::O_DIRECT).open(path)
}

/// Macos doesn't provides equivalent method to directly transfer data 
/// from disk to userspace without it being cached in the OS page cache. 
/// For more info:
///     - https://github.com/axboe/fio/issues/48
///     - https://github.com/ronomon/direct-io
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn open_direct_file(
    path: &str,
    mode: AccessMode,
) -> std::io::Result<File> {
    let mut opts = OpenOptions::new();

    match mode {
        AccessMode::Read => {
            opts.read(true);
        }
        AccessMode::Write => {
            opts.write(true);
        }
        AccessMode::ReadWrite => {
            opts.read(true).write(true);
        }
    }

    let file = opts.open(path)?;

    unsafe {
        let fd = file.as_raw_fd();
        libc::fcntl(fd, libc::F_NOCACHE, 1);
    }

    Ok(file)
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn open_direct_file(
    path: &str,
    mode: AccessMode,
) -> std::io::Result<File> {
    const O_DIRECT: u32 = 0x20000000;
    let mut opts = OpenOptions::new();

    match mode {
        AccessMode::Read => {
            opts.read(true);
        }
        AccessMode::Write => {
            opts.write(true);
        }
        AccessMode::ReadWrite => {
            opts.read(true).write(true);
        }
    }

    opts.custom_flags(O_DIRECT).open(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_file(name: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        format!("{}_{}.tmp", name, nanos)
    }

    fn create_test_file(path: &str) {
        let mut file = File::create(path).unwrap();
        writeln!(file, "hello").unwrap();
    }

    fn cleanup(path: &str) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn open_read_mode() {
        let path = unique_file("read_mode");
        create_test_file(&path);

        assert!(open_direct_file(&path, AccessMode::Read).is_ok());

        cleanup(&path);
    }

    #[test]
    fn open_write_mode() {
        let path = unique_file("write_mode");
        create_test_file(&path);

        assert!(open_direct_file(&path, AccessMode::Write).is_ok());

        cleanup(&path);
    }

    #[test]
    fn open_readwrite_mode() {
        let path = unique_file("readwrite_mode");
        create_test_file(&path);

        assert!(open_direct_file(&path, AccessMode::ReadWrite).is_ok());

        cleanup(&path);
    }

    #[test]
    fn open_nonexistent_file_fails() {
        let path = unique_file("missing_file");

        assert!(open_direct_file(&path, AccessMode::Read).is_err());
    }

    #[test]
    fn multiple_open_calls() {
        let path = unique_file("multiple");
        create_test_file(&path);

        for _ in 0..50 {
            assert!(open_direct_file(&path, AccessMode::ReadWrite).is_ok());
        }

        cleanup(&path);
    }

    #[test]
    fn sequential_modes() {
        let path = unique_file("sequential");
        create_test_file(&path);

        assert!(open_direct_file(&path, AccessMode::Read).is_ok());
        assert!(open_direct_file(&path, AccessMode::Write).is_ok());
        assert!(open_direct_file(&path, AccessMode::ReadWrite).is_ok());

        cleanup(&path);
    }
}
