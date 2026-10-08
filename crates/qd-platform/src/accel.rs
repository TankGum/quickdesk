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
        acc.key = key.to_ascii_uppercase();
        if !valid_key(&acc.key) {
            return Err(invalid());
        }
        Ok(acc)
    }
}

/// Keys we support: a letter or digit, F1–F24, or SPACE (upper-cased).
fn valid_key(key: &str) -> bool {
    let mut chars = key.chars();
    match (chars.next(), key.len()) {
        (Some(c), 1) => c.is_ascii_alphanumeric(),
        (Some('F'), 2..=3) => key[1..].parse::<u8>().is_ok_and(|n| (1..=24).contains(&n)),
        _ => key == "SPACE",
    }
}

impl fmt::Display for Accelerator {
    /// Canonical form, also accepted by `tauri-plugin-global-shortcut`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (on, name) in [(self.sup, "Super+"), (self.ctrl, "Ctrl+"), (self.alt, "Alt+"), (self.shift, "Shift+")] {
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
        for (on, name) in
            [(self.ctrl, "<Control>"), (self.alt, "<Alt>"), (self.shift, "<Shift>"), (self.sup, "<Super>")]
        {
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

    /// Parse GTK/GNOME syntax (`<Super><Shift>n`, `<Primary>v`, `<Ctrl><Alt>Delete`).
    /// Returns `None` for keys we cannot express (e.g. `Print`, `XF86AudioMute`).
    pub fn from_gtk(s: &str) -> Option<Self> {
        let mut acc = Accelerator { ctrl: false, alt: false, shift: false, sup: false, key: String::new() };
        let mut rest = s.trim();
        while let Some(stripped) = rest.strip_prefix('<') {
            let (m, after) = stripped.split_once('>')?;
            match m.to_ascii_lowercase().as_str() {
                "control" | "ctrl" | "primary" => acc.ctrl = true,
                "alt" | "mod1" => acc.alt = true,
                "shift" => acc.shift = true,
                "super" | "mod4" | "meta" | "hyper" => acc.sup = true,
                _ => return None,
            }
            rest = after;
        }
        acc.key = rest.to_ascii_uppercase();
        valid_key(&acc.key).then_some(acc)
    }

    /// Whether this is sensible as a *global* shortcut: it needs Ctrl, Alt or
    /// Super, otherwise it would steal ordinary typing (Shift+N is just "N").
    pub fn validate_global(&self) -> Result<(), String> {
        if !(self.ctrl || self.alt || self.sup) {
            return Err(format!("{self}: add Ctrl, Alt or Super so it does not steal normal typing"));
        }
        Ok(())
    }

    /// Input methods such as IBus Unikey/Bamboo only let a key through to
    /// desktop shortcuts when Ctrl or Alt is held; Super(+Shift) alone is
    /// swallowed while a text field is focused.
    pub fn may_be_eaten_by_input_method(&self) -> bool {
        self.sup && !self.ctrl && !self.alt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_normalizes() {
        let a: Accelerator = "super + shift + n".parse().unwrap();
        assert!(a.sup && a.shift && !a.ctrl && !a.alt);
        assert_eq!(a.to_string(), "Super+Shift+N");
        assert_eq!(a.to_gtk(), "<Shift><Super>n");
    }

    #[test]
    fn function_and_space_keys() {
        assert_eq!("Ctrl+Alt+F5".parse::<Accelerator>().unwrap().to_gtk(), "<Control><Alt>F5");
        assert_eq!("Alt+Space".parse::<Accelerator>().unwrap().to_gtk(), "<Alt>space");
    }

    #[test]
    fn parses_gtk_syntax() {
        let a = Accelerator::from_gtk("<Shift><Super>n").unwrap();
        assert_eq!(a, "Super+Shift+N".parse().unwrap());
        assert_eq!(Accelerator::from_gtk("<Primary><Alt>t").unwrap().to_string(), "Ctrl+Alt+T");
        assert_eq!(Accelerator::from_gtk("<Super>space").unwrap().to_string(), "Super+SPACE");
        assert_eq!(Accelerator::from_gtk("<Alt><Super>n").unwrap().to_string(), "Super+Alt+N");
        assert!(Accelerator::from_gtk("XF86AudioMute").is_none());
        assert!(Accelerator::from_gtk("<Super>Page_Up").is_none());
        assert!(Accelerator::from_gtk("").is_none());
    }

    #[test]
    fn global_validation_and_input_method_hint() {
        assert!("Shift+N".parse::<Accelerator>().unwrap().validate_global().is_err());
        assert!("Super+Alt+N".parse::<Accelerator>().unwrap().validate_global().is_ok());
        assert!("Super+Shift+V".parse::<Accelerator>().unwrap().may_be_eaten_by_input_method());
        assert!(!"Super+Alt+V".parse::<Accelerator>().unwrap().may_be_eaten_by_input_method());
        assert!(!"Ctrl+Alt+V".parse::<Accelerator>().unwrap().may_be_eaten_by_input_method());
    }

    #[test]
    fn rejects_garbage() {
        for bad in ["", "Super+", "Hyper+N", "Super+Shift+?"] {
            assert!(bad.parse::<Accelerator>().is_err(), "{bad:?}");
        }
    }
}
