//! Cooperative Decksmith lock plus best-effort OpenDeck detection (not a HID lock).
use crate::DeviceError;
use std::{
    fs::{File, OpenOptions},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};
fn competing_in(root: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(root) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let numeric = entry
            .file_name()
            .to_string_lossy()
            .bytes()
            .all(|b| b.is_ascii_digit());
        numeric
            && std::fs::read_to_string(entry.path().join("comm"))
                .is_ok_and(|name| name.trim().eq_ignore_ascii_case("opendeck"))
    })
}
pub fn competing_application_running() -> bool {
    competing_in(Path::new("/proc"))
}
pub fn acquire() -> Result<File, DeviceError> {
    if competing_application_running() {
        return Err(DeviceError::DeviceBusy);
    }
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").ok_or(DeviceError::DeviceBusy)?;
    let runtime = Path::new(&runtime);
    let meta = runtime.metadata().map_err(|_| DeviceError::DeviceBusy)?;
    let own_uid = std::fs::metadata("/proc/self")
        .map_err(|_| DeviceError::DeviceBusy)?
        .uid();
    if !runtime.is_absolute() || !meta.is_dir() || meta.uid() != own_uid || meta.mode() & 0o077 != 0
    {
        return Err(DeviceError::DeviceBusy);
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(runtime.join("decksmith-device.lock"))
        .map_err(|_| DeviceError::DeviceBusy)?;
    file.try_lock().map_err(|_| DeviceError::DeviceBusy)?;
    Ok(file)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_exact_competing_process_name() {
        let root = std::env::temp_dir().join(format!("decksmith-proc-{}", std::process::id()));
        std::fs::create_dir_all(root.join("123")).unwrap();
        std::fs::write(root.join("123/comm"), "OpenDeck\n").unwrap();
        assert!(competing_in(&root));
        std::fs::write(root.join("123/comm"), "my-opendeck-docs\n").unwrap();
        assert!(!competing_in(&root));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn lock_excludes_second_handle_and_releases_on_drop() {
        let path = std::env::temp_dir().join(format!("decksmith-lock-{}", std::process::id()));
        let a = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .unwrap();
        let b = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        a.try_lock().unwrap();
        assert!(b.try_lock().is_err());
        drop(a);
        b.try_lock().unwrap();
        drop(b);
        std::fs::remove_file(path).unwrap();
    }
}
