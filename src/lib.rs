use std::fs::File;
use std::fs::OpenOptions;

use std::os::unix::fs::OpenOptionsExt;

#[allow(dead_code)]
pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}

#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub fn open_direct_file(path: &str, mode: AccessMode) -> std::io::Result<File> {
    const O_DIRECT: i32 = 0o0040000;

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

#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn open_direct_file(path: &str, mode: AccessMode) -> std::io::Result<File> {
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
pub fn open_direct_file(path: &str, mode: AccessMode) -> std::io::Result<File> {
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

    #[test]
    fn open_read_mode() {
        let path = unique_file("read_mode");

        create_test_file(&path);

        let file = open_direct_file(&path, AccessMode::Read);

        assert!(file.is_ok());

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn open_write_mode() {
        let path = unique_file("write_mode");

        create_test_file(&path);

        let file = open_direct_file(&path, AccessMode::Write);

        assert!(file.is_ok());

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn open_readwrite_mode() {
        let path = unique_file("readwrite_mode");

        create_test_file(&path);

        let file = open_direct_file(&path, AccessMode::ReadWrite);

        assert!(file.is_ok());

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn open_nonexistent_file_fails() {
        let path = unique_file("missing_file");

        let file = open_direct_file(&path, AccessMode::Read);

        assert!(file.is_err());
    }

    #[test]
    fn multiple_open_calls() {
        let path = unique_file("multiple");

        create_test_file(&path);

        for _ in 0..100 {
            let file = open_direct_file(&path, AccessMode::ReadWrite);

            assert!(file.is_ok());
        }

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn sequential_modes() {
        let path = unique_file("sequential");

        create_test_file(&path);

        let r = open_direct_file(&path, AccessMode::Read);
        assert!(r.is_ok());

        let w = open_direct_file(&path, AccessMode::Write);
        assert!(w.is_ok());

        let rw = open_direct_file(&path, AccessMode::ReadWrite);
        assert!(rw.is_ok());

        std::fs::remove_file(path).unwrap();
    }
}
