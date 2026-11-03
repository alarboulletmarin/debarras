//! Freeing space: an item moved to the system's trash, or deleted for good.
//!
//! The trash goes through the system's own tool, which knows its rules (one trash per disk, the
//! record that lets the file manager put an item back): `gio` on Linux, the Finder on macOS, the
//! recycle bin through PowerShell on Windows.

use std::{fs, io, path::Path, process::Command};

use crate::lang::tr;

/// Deletes `path` for good: a folder with all it holds. A link is removed, never what it points to.
pub fn delete(path: &Path) -> io::Result<()> {
    if fs::symlink_metadata(path)?.is_dir() { fs::remove_dir_all(path) } else { fs::remove_file(path) }
}

/// Moves `path` to the system's trash, where it still takes its room until the trash is emptied.
pub fn trash(path: &Path) -> io::Result<()> {
    fs::symlink_metadata(path)?;
    let out = trash_command(path).output().map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            io::Error::other(tr("the system's trash tool is missing", "l'outil de corbeille du système est absent"))
        } else {
            e
        }
    })?;
    if out.status.success() && fs::symlink_metadata(path).is_err() {
        return Ok(());
    }
    // `gio: file:///…/item: Trashing on system internal mounts is not supported`: the reason only.
    let said = String::from_utf8_lossy(&out.stderr);
    let said = said.trim().lines().last().unwrap_or_default();
    let said = said.rsplit(": ").next().unwrap_or(said).trim();
    Err(io::Error::other(if said.is_empty() { tr("the trash refused it", "la corbeille l'a refusé").to_string() } else { said.to_string() }))
}

// ponytail: `gio` comes with GLib, installed on nearly every desktop; a KDE-only system without it
// gets an error that says so, and can still delete for good.
#[cfg(not(any(target_os = "macos", windows)))]
fn trash_command(path: &Path) -> Command {
    let mut command = Command::new("gio");
    command.args(["trash", "--"]).arg(path);
    command
}

/// The path is passed as an argument of the script, never written into it.
#[cfg(target_os = "macos")]
fn trash_command(path: &Path) -> Command {
    let mut command = Command::new("osascript");
    command
        .args(["-e", "on run argv", "-e", "tell application \"Finder\" to delete (POSIX file (item 1 of argv) as alias)", "-e", "end run"])
        .arg(path);
    command
}

/// The path goes through the environment, never written into the script.
#[cfg(windows)]
fn trash_command(path: &Path) -> Command {
    use std::os::windows::process::CommandExt;
    const SCRIPT: &str = "Add-Type -AssemblyName Microsoft.VisualBasic; $p = $env:DEBARRAS_TRASH; \
        if (Test-Path -LiteralPath $p -PathType Container) { [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteDirectory($p, 'OnlyErrorDialogs', 'SendToRecycleBin') } \
        else { [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile($p, 'OnlyErrorDialogs', 'SendToRecycleBin') }";
    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT]).env("DEBARRAS_TRASH", path);
    command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: no console flashing
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletes_a_folder_with_what_it_holds() {
        let base = std::env::temp_dir().join(format!("debarras-suppr-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("cache/profond/plus")).unwrap();
        fs::write(base.join("cache/profond/plus/f"), b"x").unwrap();
        fs::write(base.join("fichier été 🌞"), b"x").unwrap();
        fs::create_dir(base.join("garder")).unwrap();
        fs::write(base.join("garder/précieux"), b"x").unwrap();

        delete(&base.join("cache")).unwrap();
        delete(&base.join("fichier été 🌞")).unwrap();
        assert!(!base.join("cache").exists() && !base.join("fichier été 🌞").exists());
        assert_eq!(delete(&base.join("cache")).unwrap_err().kind(), io::ErrorKind::NotFound, "already gone");
        assert_eq!(trash(&base.join("absent")).unwrap_err().kind(), io::ErrorKind::NotFound);

        // A link to a folder: the link goes, the folder stays with what it holds.
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(base.join("garder"), base.join("lien")).unwrap();
            delete(&base.join("lien")).unwrap();
            assert!(!base.join("lien").exists());
        }
        assert!(base.join("garder/précieux").exists());
        fs::remove_dir_all(&base).unwrap();
    }
}
