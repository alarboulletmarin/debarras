//! The outside world: the configuration folder and the disk itself. No gpui.

pub mod remove;
pub mod scan;

use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

/// The NVIDIA Vulkan driver (package `nvidia-utils`) takes about 2 s to give up when no NVIDIA
/// card is plugged in: the loader is asked to ignore it. Measured: the window shows in 35 ms
/// instead of 1.9 s on a machine with an Intel GPU only.
#[cfg(target_os = "linux")]
pub fn skip_absent_nvidia_driver() {
    let has_nvidia = fs::read_dir("/sys/bus/pci/devices")
        .is_ok_and(|devices| devices.flatten().any(|d| fs::read_to_string(d.path().join("vendor")).is_ok_and(|v| v.trim() == "0x10de")));
    if !has_nvidia && env::var_os("VK_LOADER_DRIVERS_DISABLE").is_none() {
        // Safe: called at the very start of `run`, before any thread is created.
        unsafe { env::set_var("VK_LOADER_DRIVERS_DISABLE", "*nvidia*") };
    }
}

/// The user's configuration folder, per platform.
pub fn config_dir() -> PathBuf {
    let var = |key: &str| env::var_os(key).filter(|v| !v.is_empty()).map(PathBuf::from);
    let dir = if cfg!(windows) {
        var("APPDATA")
    } else if cfg!(target_os = "macos") {
        var("HOME").map(|home| home.join("Library/Application Support"))
    } else {
        var("XDG_CONFIG_HOME").or_else(|| var("HOME").map(|home| home.join(".config")))
    };
    dir.unwrap_or_default().join("debarras")
}

/// The user's home folder: what a first scan looks at.
pub fn home_dir() -> Option<PathBuf> {
    let key = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    env::var_os(key).filter(|v| !v.is_empty()).map(PathBuf::from)
}

/// A file of the configuration folder; empty when it is missing or unreadable.
pub fn load_file(name: &str) -> String {
    fs::read_to_string(config_dir().join(name)).unwrap_or_default()
}

/// Writes a file of the configuration folder, atomically.
pub fn save_file(name: &str, text: &str) -> io::Result<()> {
    let path = config_dir().join(name);
    fs::create_dir_all(config_dir())?;
    write(&path, text)
}

/// Atomic write: a crash never leaves a truncated file.
pub fn write(path: &Path, content: &str) -> io::Result<()> {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let tmp = path.with_file_name(format!(".{name}.tmp"));
    let mut file = fs::File::create(&tmp)?;
    file.write_all(content.as_bytes())?;
    // Without `fsync`, a power cut can leave the rename without the data.
    file.sync_all()?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_atomically_and_replaces() {
        let dir = std::env::temp_dir().join(format!("debarras-write-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("réglages é.txt");
        write(&path, "un").unwrap();
        write(&path, "deux").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "deux");
        // No temporary file left behind.
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        // A folder that does not exist: an error, not a panic.
        assert!(write(&dir.join("absent/x"), "x").is_err());
        fs::remove_dir_all(&dir).unwrap();
    }
}
