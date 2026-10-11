//! Windows' counterpart of `shellrc`: the fix for "new terminals run another
//! copy" is the user's own PATH (HKCU\Environment\Path, what System →
//! Environment Variables edits), not a shell file. QuickDesk only prepends
//! directories, after the user agreed, keeps a copy of the old value, and
//! remembers what it added so undo removes exactly that. The string handling
//! is plain code, tested everywhere; the registry part is Windows-only.

use std::path::{Path, PathBuf};

use crate::Result;

/// Entries compare like Windows does: no case, no trailing separator.
fn key(entry: &str) -> String {
    entry.trim().trim_end_matches(['\\', '/']).replace('/', "\\").to_lowercase()
}

/// `current` with `dirs` in front (those not already in it), in order.
pub fn with_dirs(current: &str, dirs: &[String]) -> String {
    let present: Vec<String> = current.split(';').map(key).collect();
    let mut front: Vec<&str> = Vec::new();
    for d in dirs {
        if !present.contains(&key(d)) && !front.iter().any(|f| key(f) == key(d)) {
            front.push(d);
        }
    }
    if front.is_empty() {
        return current.to_owned();
    }
    let rest = current.trim_matches(';');
    if rest.is_empty() {
        front.join(";")
    } else {
        format!("{};{rest}", front.join(";"))
    }
}

/// `current` without any of `dirs`.
pub fn without_dirs(current: &str, dirs: &[String]) -> String {
    let drop: Vec<String> = dirs.iter().map(|d| key(d)).collect();
    current.split(';').filter(|e| !e.trim().is_empty() && !drop.contains(&key(e))).collect::<Vec<_>>().join(";")
}

/// Where QuickDesk keeps its PATH notes: the old values and what it added.
pub fn state_dir(home: &Path) -> PathBuf {
    home.join(".config").join("quickdesk").join("path")
}

fn added_file(dir: &Path) -> PathBuf {
    dir.join("added.txt")
}

/// The directories QuickDesk put in the user PATH.
pub fn added(dir: &Path) -> Vec<String> {
    std::fs::read_to_string(added_file(dir))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect()
}

fn remember(dir: &Path, dirs: &[String]) -> Result<()> {
    let mut all = added(dir);
    for d in dirs {
        if !all.iter().any(|a| key(a) == key(d)) {
            all.push(d.clone());
        }
    }
    std::fs::create_dir_all(dir)?;
    std::fs::write(added_file(dir), all.join("\n") + "\n")?;
    Ok(())
}

/// Prepend `dirs` to the user PATH. Returns the copy of the old value when
/// something changed. `stamp` names that copy.
pub fn apply(home: &Path, dirs: &[String], stamp: &str) -> Result<Option<PathBuf>> {
    let (current, kind) = registry::read()?;
    let next = with_dirs(&current, dirs);
    if next == current {
        return Ok(None);
    }
    let dir = state_dir(home);
    std::fs::create_dir_all(&dir)?;
    let backup = dir.join(format!("path-before-{stamp}.txt"));
    std::fs::write(&backup, &current)?;
    registry::write(&next, kind)?;
    remember(&dir, dirs)?;
    registry::broadcast();
    Ok(Some(backup))
}

/// Take out what QuickDesk added. `false` when it had added nothing.
pub fn undo(home: &Path, stamp: &str) -> Result<bool> {
    let dir = state_dir(home);
    let ours = added(&dir);
    if ours.is_empty() {
        return Ok(false);
    }
    let (current, kind) = registry::read()?;
    std::fs::write(dir.join(format!("path-before-{stamp}.txt")), &current)?;
    registry::write(&without_dirs(&current, &ours), kind)?;
    let _ = std::fs::remove_file(added_file(&dir));
    registry::broadcast();
    Ok(true)
}

#[cfg(windows)]
mod registry {
    use windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND;
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE,
        KEY_SET_VALUE, REG_EXPAND_SZ, REG_SZ, REG_VALUE_TYPE,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };

    use crate::{Error, Result};

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    struct Key(HKEY);

    impl Key {
        fn open(access: u32) -> Result<Self> {
            let mut h: HKEY = std::ptr::null_mut();
            let sub = wide("Environment");
            // SAFETY: valid NUL-terminated name and out pointer.
            let rc = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, access, &mut h) };
            if rc != 0 {
                return Err(Error::Io(format!("cannot open HKCU\\Environment (error {rc})")));
            }
            Ok(Key(h))
        }
    }

    impl Drop for Key {
        fn drop(&mut self) {
            // SAFETY: we opened it.
            unsafe { RegCloseKey(self.0) };
        }
    }

    /// The user PATH and its type (REG_EXPAND_SZ keeps `%USERPROFILE%`-style entries working).
    pub fn read() -> Result<(String, REG_VALUE_TYPE)> {
        let key = Key::open(KEY_QUERY_VALUE)?;
        let name = wide("Path");
        let (mut kind, mut size) = (0u32, 0u32);
        // SAFETY: size query, then a buffer of that size.
        unsafe {
            let rc =
                RegQueryValueExW(key.0, name.as_ptr(), std::ptr::null(), &mut kind, std::ptr::null_mut(), &mut size);
            if rc == ERROR_FILE_NOT_FOUND {
                return Ok((String::new(), REG_EXPAND_SZ));
            }
            if rc != 0 {
                return Err(Error::Io(format!("cannot read the user PATH (error {rc})")));
            }
            let mut buf = vec![0u16; size as usize / 2 + 1];
            let rc =
                RegQueryValueExW(key.0, name.as_ptr(), std::ptr::null(), &mut kind, buf.as_mut_ptr().cast(), &mut size);
            if rc != 0 {
                return Err(Error::Io(format!("cannot read the user PATH (error {rc})")));
            }
            let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
            let kind = if kind == REG_SZ { REG_SZ } else { REG_EXPAND_SZ };
            Ok((String::from_utf16_lossy(&buf[..len]), kind))
        }
    }

    pub fn write(value: &str, kind: REG_VALUE_TYPE) -> Result<()> {
        let key = Key::open(KEY_SET_VALUE)?;
        let (name, data) = (wide("Path"), wide(value));
        // SAFETY: NUL-terminated UTF-16 data with its byte length.
        let rc =
            unsafe { RegSetValueExW(key.0, name.as_ptr(), 0, kind, data.as_ptr().cast(), (data.len() * 2) as u32) };
        if rc != 0 {
            return Err(Error::Io(format!("cannot write the user PATH (error {rc})")));
        }
        Ok(())
    }

    /// Tell Explorer (and so every terminal started from now on) that the
    /// environment changed.
    pub fn broadcast() {
        let env = wide("Environment");
        let mut result = 0usize;
        // SAFETY: broadcast with a timeout so a hung window cannot block us.
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                env.as_ptr() as isize,
                SMTO_ABORTIFHUNG,
                2000,
                &mut result,
            )
        };
    }
}

#[cfg(not(windows))]
mod registry {
    use crate::{Error, Result};

    pub fn read() -> Result<(String, u32)> {
        Err(Error::Unsupported("the user PATH in the registry is a Windows thing".into()))
    }
    pub fn write(_value: &str, _kind: u32) -> Result<()> {
        Err(Error::Unsupported("the user PATH in the registry is a Windows thing".into()))
    }
    pub fn broadcast() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn prepends_only_what_is_missing() {
        let path = r"C:\Users\Me\AppData\Local\Microsoft\WindowsApps;%USERPROFILE%\.cargo\bin\";
        let next = with_dirs(path, &v(&[r"C:\Users\Me\AppData\Local\mise\shims", r"%userprofile%\.cargo\bin"]));
        assert_eq!(next, format!(r"C:\Users\Me\AppData\Local\mise\shims;{path}"));
        assert_eq!(with_dirs(&next, &v(&[r"c:\users\me\appdata\local\mise\shims\"])), next, "already there");
        assert_eq!(with_dirs("", &v(&["A", "B", "a"])), "A;B");
    }

    #[test]
    fn removes_exactly_those_entries() {
        let path = r"C:\Mise\shims;C:\Windows;;c:\mise\SHIMS\;D:\tools";
        assert_eq!(without_dirs(path, &v(&[r"C:\mise\shims"])), r"C:\Windows;D:\tools");
    }

    #[test]
    fn remembers_what_it_added() {
        let t = crate::testutil::TempDir::new("winpath");
        let dir = state_dir(t.path());
        remember(&dir, &v(&[r"C:\A", r"C:\B"])).unwrap();
        remember(&dir, &v(&[r"c:\a\", r"C:\C"])).unwrap();
        assert_eq!(added(&dir), v(&[r"C:\A", r"C:\B", r"C:\C"]));
    }
}
