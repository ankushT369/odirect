use std::fs::File;
use std::fs::OpenOptions;

#[cfg(target_os = "windows")]
#[allow(dead_code)]
use std::os::windows::fs::OpenOptionsExt;

#[cfg(target_os = "macos")]
#[allow(dead_code)]
use std::os::unix::io::AsRawFd;

#[allow(dead_code)]
pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}

#[allow(dead_code)]
pub enum Integrity {
    Data,
    File,
    Null,
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
#[allow(dead_code)]
pub fn open_direct_file(
    path: &str,
    mode: AccessMode,
    integrity: Integrity,
) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    let mut flags = libc::O_DIRECT;
    let mut opts = OpenOptions::new();

    match integrity {
        Integrity::Data => flags |= libc::O_DSYNC,
        Integrity::File => flags |= libc::O_SYNC,
        Integrity::Null => {}
    };

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

    opts.custom_flags(flags).open(path)
}

// macOS does not provide direct open-time flags for integrity levels like Linux (O_SYNC / O_DSYNC).
// The F_NOCACHE flag only disables file caching, it does not guarantee durability or metadata sync.
//
// Actual data and metadata persistence on macOS is handled later using functions like fsync()
// or F_FULLFSYNC, depending on how strong the durability guarantee needs to be.
//
// Because of this, integrity is not applied at open time here.
// It is better to handle durability explicitly at the write/flush stage instead of forcing it into open().
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn open_direct_file(
    path: &str,
    mode: AccessMode,
    integrity: Integrity,
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

    match integrity {
        Integrity::Data => {}
        Integrity::File => {}
        Integrity::Null => {}
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
    integrity: Integrity,
) -> std::io::Result<File> {
    const O_DIRECT: u32 = 0x20000000;
    // Windows does not expose separate controls like Linux does for direct I/O and sync behavior.
    // On Linux, we can combine O_DIRECT with O_DSYNC or O_SYNC to control data and metadata durability separately.
    //
    // On Windows, this separation does not exist in the same way.
    // So we only use a simplified model where O_DIRECT is used for direct access,
    // and O_SYNC is treated as a general durability hint.
    //
    // Because of this, integrity levels cannot be mapped one-to-one like Linux,
    // and Windows ends up using a merged behavior instead of separate sync modes.
    const O_SYNC: u32 = 0x80000000;
    let mut flags = O_DIRECT;
    let mut opts = OpenOptions::new();

    match integrity {
        Integrity::Null => {}
        _ => flags |= O_SYNC,
    };

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

    opts.custom_flags(flags).open(path)
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

        assert!(open_direct_file(&path, AccessMode::Read, Integrity::Null).is_ok());

        cleanup(&path);
    }

    #[test]
    fn open_write_mode() {
        let path = unique_file("write_mode");
        create_test_file(&path);

        assert!(open_direct_file(&path, AccessMode::Write, Integrity::Null).is_ok());

        cleanup(&path);
    }

    #[test]
    fn open_readwrite_mode() {
        let path = unique_file("readwrite_mode");
        create_test_file(&path);

        assert!(open_direct_file(&path, AccessMode::ReadWrite, Integrity::Null).is_ok());

        cleanup(&path);
    }

    #[test]
    fn open_nonexistent_file_fails() {
        let path = unique_file("missing_file");

        assert!(open_direct_file(&path, AccessMode::Read, Integrity::Null).is_err());
    }

    #[test]
    fn multiple_open_calls() {
        let path = unique_file("multiple");
        create_test_file(&path);

        for _ in 0..50 {
            assert!(open_direct_file(&path, AccessMode::ReadWrite, Integrity::Null).is_ok());
        }

        cleanup(&path);
    }

    #[test]
    fn sequential_modes() {
        let path = unique_file("sequential");
        create_test_file(&path);

        assert!(open_direct_file(&path, AccessMode::Read, Integrity::Null).is_ok());

        assert!(open_direct_file(&path, AccessMode::Write, Integrity::Null).is_ok());

        assert!(open_direct_file(&path, AccessMode::ReadWrite, Integrity::Null).is_ok());

        cleanup(&path);
    }
}
