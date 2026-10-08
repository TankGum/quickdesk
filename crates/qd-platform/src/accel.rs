use std::fmt;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("invalid accelerator {0:?}: expected e.g. \"Super+Shift+N\"")]
pub struct InvalidAccelerator(pub String);

/// A keyboard shortcut in the app's canonical form, e.g. `Super+Shift+N`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Accelerator {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub sup: bool,
    /// Upper-cased key name: `N`, `F5`, `SPACE`.
    pub key: String,
}

impl FromStr for Accelerator {
    type Err = InvalidAccelerator;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || InvalidAccelerator(s.to_owned());
        let mut acc = Accelerator { ctrl: false, alt: false, shift: false, sup: false, key: String::new() };
        let parts: Vec<&str> = s.split('+').map(str::trim).collect();
        let (key, mods) = parts.split_last().ok_or_else(invalid)?;
        for m in mods {
            match m.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => acc.ctrl = true,
                "alt" => acc.alt = true,
                "shift" => acc.shift = true,
                "super" | "meta" | "cmd" | "win" => acc.sup = true,
                _ => return Err(invalid()),
            }
        }
        if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(invalid());
        }
        acc.key = key.to_ascii_uppercase();
        Ok(acc)
    }
}

impl fmt::Display for Accelerator {
    /// Canonical form, also accepted by `tauri-plugin-global-shortcut`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (on, name) in [(self.ctrl, "Ctrl+"), (self.alt, "Alt+"), (self.shift, "Shift+"), (self.sup, "Super+")] {
            if on {
                f.write_str(name)?;
            }
        }
        f.write_str(&self.key)
    }
}

impl Accelerator {
    /// GTK/GNOME accelerator syntax, e.g. `<Super><Shift>n`.
    pub fn to_gtk(&self) -> String {
        let mut out = String::new();
        for (on, name) in [(self.ctrl, "<Control>"), (self.alt, "<Alt>"), (self.shift, "<Shift>"), (self.sup, "<Super>")] {
            if on {
                out.push_str(name);
            }
        }
        let key = match self.key.as_str() {
            "SPACE" => "space".to_owned(),
            k if k.len() == 1 => k.to_ascii_lowercase(),
            k => k.to_owned(), // F1..F12 keep their case
        };
        out.push_str(&key);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_normalizes() {
        let a: Accelerator = "super + shift + n".parse().unwrap();
        assert!(a.sup && a.shift && !a.ctrl && !a.alt);
        assert_eq!(a.to_string(), "Shift+Super+N");
        assert_eq!(a.to_gtk(), "<Shift><Super>n");
    }

    #[test]
    fn function_and_space_keys() {
        assert_eq!("Ctrl+Alt+F5".parse::<Accelerator>().unwrap().to_gtk(), "<Control><Alt>F5");
        assert_eq!("Alt+Space".parse::<Accelerator>().unwrap().to_gtk(), "<Alt>space");
    }

    #[test]
    fn rejects_garbage() {
        for bad in ["", "Super+", "Hyper+N", "Super+Shift+?"] {
            assert!(bad.parse::<Accelerator>().is_err(), "{bad:?}");
        }
    }
}
