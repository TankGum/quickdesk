//! GNOME custom keybindings (Settings → Keyboard → Custom Shortcuts).
//!
//! On GNOME Wayland < 48 there is no global-shortcut API for apps, so we register
//! custom keybindings that run `quickdesk toggle <target>`. We only ever touch
//! entries under our own `quickdesk-*` paths and keep the user's other shortcuts.

use std::io;
use std::process::Command;

use crate::Accelerator;

const LIST_SCHEMA: &str = "org.gnome.settings-daemon.plugins.media-keys";
const LIST_KEY: &str = "custom-keybindings";
const ENTRY_SCHEMA: &str = "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding";
const PATH_ROOT: &str = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/";
const OUR_PREFIX: &str = "quickdesk-";

/// Minimal gsettings surface, so the merge logic can be tested without GNOME.
pub trait Gsettings {
    fn get(&self, schema: &str, key: &str) -> io::Result<String>;
    fn set(&self, schema: &str, key: &str, value: &str) -> io::Result<()>;
    fn reset_recursively(&self, schema: &str) -> io::Result<()>;
    /// `gsettings list-recursively <schema>`: one `schema key value` per line.
    fn list_recursively(&self, schema: &str) -> io::Result<String>;
}

/// Shells out to the `gsettings` CLI that ships with every GNOME install.
pub struct GsettingsCli;

impl GsettingsCli {
    fn run(args: &[&str]) -> io::Result<String> {
        let out = Command::new("gsettings").args(args).output()?;
        if !out.status.success() {
            let msg = String::from_utf8_lossy(&out.stderr).trim().to_owned();
            return Err(io::Error::other(format!("gsettings {}: {msg}", args.join(" "))));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    }
}

impl Gsettings for GsettingsCli {
    fn get(&self, schema: &str, key: &str) -> io::Result<String> {
        Self::run(&["get", schema, key])
    }
    fn set(&self, schema: &str, key: &str, value: &str) -> io::Result<()> {
        Self::run(&["set", schema, key, value]).map(drop)
    }
    fn reset_recursively(&self, schema: &str) -> io::Result<()> {
        Self::run(&["reset-recursively", schema]).map(drop)
    }
    fn list_recursively(&self, schema: &str) -> io::Result<String> {
        Self::run(&["list-recursively", schema])
    }
}

/// One shortcut we want GNOME to run.
#[derive(Debug, Clone)]
pub struct Binding {
    /// Short stable id, becomes the path suffix: `quickdesk-<id>/`.
    pub id: String,
    pub name: String,
    pub argv: Vec<String>,
    pub accel: Accelerator,
}

pub struct GnomeKeybindings<G: Gsettings = GsettingsCli> {
    gs: G,
}

impl GnomeKeybindings<GsettingsCli> {
    pub fn new() -> Self {
        GnomeKeybindings { gs: GsettingsCli }
    }
}

impl Default for GnomeKeybindings<GsettingsCli> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G: Gsettings> GnomeKeybindings<G> {
    pub fn with(gs: G) -> Self {
        GnomeKeybindings { gs }
    }

    /// Replace all of our bindings with `bindings`, leaving the user's untouched.
    pub fn apply(&self, bindings: &[Binding]) -> io::Result<()> {
        let existing = parse_strv(&self.gs.get(LIST_SCHEMA, LIST_KEY)?);
        let wanted: Vec<String> = bindings.iter().map(|b| path_for(&b.id)).collect();
        for stale in existing.iter().filter(|p| is_ours(p) && !wanted.contains(p)) {
            self.gs.reset_recursively(&entry_schema(stale))?;
        }
        // Fill entries before listing them so GNOME never sees a half-written one.
        for (b, path) in bindings.iter().zip(&wanted) {
            let schema = entry_schema(path);
            self.gs.set(&schema, "name", &gvariant_str(&b.name))?;
            self.gs.set(&schema, "command", &gvariant_str(&shell_join(&b.argv)))?;
            self.gs.set(&schema, "binding", &gvariant_str(&b.accel.to_gtk()))?;
        }
        let mut list: Vec<String> = existing.into_iter().filter(|p| !is_ours(p)).collect();
        list.extend(wanted);
        self.gs.set(LIST_SCHEMA, LIST_KEY, &format_strv(&list))
    }

    /// Remove every binding we own.
    pub fn remove_all(&self) -> io::Result<()> {
        self.apply(&[])
    }

    /// Existing GNOME shortcuts (system and the user's custom ones, not ours)
    /// that use `accel`, as human-readable descriptions.
    pub fn conflicts(&self, accel: &Accelerator) -> Vec<String> {
        let mut found = Vec::new();
        for schema in SYSTEM_SCHEMAS {
            let Ok(listing) = self.gs.list_recursively(schema) else { continue };
            for line in listing.lines() {
                let mut parts = line.splitn(3, ' ');
                let (Some(_), Some(key), Some(value)) = (parts.next(), parts.next(), parts.next()) else { continue };
                if quoted_strings(value).iter().filter_map(|v| Accelerator::from_gtk(v)).any(|a| &a == accel) {
                    found.push(format!("GNOME: {}", key.replace('-', " ")));
                }
            }
        }
        if let Ok(list) = self.gs.get(LIST_SCHEMA, LIST_KEY) {
            for path in parse_strv(&list).into_iter().filter(|p| !is_ours(p)) {
                let schema = entry_schema(&path);
                let binding = self.gs.get(&schema, "binding").unwrap_or_default();
                if quoted_strings(&binding).iter().filter_map(|v| Accelerator::from_gtk(v)).any(|a| &a == accel) {
                    let name = self.gs.get(&schema, "name").unwrap_or_default();
                    found.push(format!("Custom shortcut: {}", quoted_strings(&name).first().cloned().unwrap_or(path)));
                }
            }
        }
        found
    }
}

/// Schemas holding the desktop's built-in keybindings.
const SYSTEM_SCHEMAS: &[&str] = &[
    "org.gnome.desktop.wm.keybindings",
    "org.gnome.shell.keybindings",
    "org.gnome.mutter.keybindings",
    "org.gnome.mutter.wayland.keybindings",
    "org.gnome.settings-daemon.plugins.media-keys",
];

/// All single-quoted GVariant strings in `value` (`'a'`, `['a', 'b']`).
fn quoted_strings(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c == '\'' {
            let mut s = String::new();
            while let Some(c) = chars.next() {
                match c {
                    '\\' => s.extend(chars.next()),
                    '\'' => break,
                    c => s.push(c),
                }
            }
            out.push(s);
        }
    }
    out
}

fn path_for(id: &str) -> String {
    format!("{PATH_ROOT}{OUR_PREFIX}{id}/")
}

fn is_ours(path: &str) -> bool {
    path.strip_prefix(PATH_ROOT).is_some_and(|rest| rest.starts_with(OUR_PREFIX))
}

fn entry_schema(path: &str) -> String {
    format!("{ENTRY_SCHEMA}:{path}")
}

/// Parse gsettings' GVariant `as` output: `@as []` or `['/a/', '/b/']`.
fn parse_strv(raw: &str) -> Vec<String> {
    let inner = raw.trim().trim_start_matches("@as").trim();
    let inner = inner.strip_prefix('[').and_then(|s| s.strip_suffix(']')).unwrap_or("");
    inner
        .split(',')
        .map(|s| s.trim().trim_matches(|c| c == '\'' || c == '"').to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

fn format_strv(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|s| gvariant_str(s)).collect();
    format!("[{}]", quoted.join(", "))
}

/// GVariant string literal.
fn gvariant_str(s: &str) -> String {
    format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'"))
}

/// GNOME parses the command with `g_shell_parse_argv`, i.e. POSIX shell quoting.
fn shell_join(argv: &[String]) -> String {
    argv.iter()
        .map(|a| {
            if !a.is_empty() && a.chars().all(|c| c.is_ascii_alphanumeric() || "/._-+=:,@".contains(c)) {
                a.clone()
            } else {
                format!("'{}'", a.replace('\'', r"'\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct Fake {
        store: RefCell<BTreeMap<(String, String), String>>,
        resets: RefCell<Vec<String>>,
    }

    impl Gsettings for &Fake {
        fn get(&self, schema: &str, key: &str) -> io::Result<String> {
            Ok(self.store.borrow().get(&(schema.into(), key.into())).cloned().unwrap_or("@as []".into()))
        }
        fn set(&self, schema: &str, key: &str, value: &str) -> io::Result<()> {
            self.store.borrow_mut().insert((schema.into(), key.into()), value.into());
            Ok(())
        }
        fn reset_recursively(&self, schema: &str) -> io::Result<()> {
            self.resets.borrow_mut().push(schema.into());
            self.store.borrow_mut().retain(|(s, _), _| s != schema);
            Ok(())
        }
        fn list_recursively(&self, schema: &str) -> io::Result<String> {
            Ok(self
                .store
                .borrow()
                .iter()
                .filter(|((s, _), _)| s == schema)
                .map(|((s, k), v)| format!("{s} {k} {v}"))
                .collect::<Vec<_>>()
                .join("\n"))
        }
    }

    fn binding(id: &str, accel: &str) -> Binding {
        Binding {
            id: id.into(),
            name: format!("QuickDesk {id}"),
            argv: vec!["/opt/Quick Desk/quickdesk".into(), "toggle".into(), id.into()],
            accel: accel.parse().unwrap(),
        }
    }

    fn list(fake: &Fake) -> Vec<String> {
        parse_strv(&fake.get(LIST_SCHEMA, LIST_KEY).unwrap())
    }

    #[test]
    fn strv_roundtrip() {
        assert!(parse_strv("@as []").is_empty());
        let v = parse_strv("['/a/', '/b/']");
        assert_eq!(v, vec!["/a/", "/b/"]);
        assert_eq!(format_strv(&v), "['/a/', '/b/']");
    }

    #[test]
    fn quoting() {
        assert_eq!(gvariant_str(r"it's \ ok"), r"'it\'s \\ ok'");
        assert_eq!(
            shell_join(&["/opt/Quick Desk/qd".into(), "toggle".into(), "it's".into()]),
            r"'/opt/Quick Desk/qd' toggle 'it'\''s'"
        );
    }

    #[test]
    fn apply_keeps_user_bindings_and_writes_ours() {
        let fake = Fake::default();
        let user = format!("{PATH_ROOT}custom0/");
        (&fake).set(LIST_SCHEMA, LIST_KEY, &format_strv(std::slice::from_ref(&user))).unwrap();

        GnomeKeybindings::with(&fake).apply(&[binding("notes", "Super+Shift+N")]).unwrap();

        let ours = path_for("notes");
        assert_eq!(list(&fake), vec![user, ours.clone()]);
        let get = |k: &str| (&fake).get(&entry_schema(&ours), k).unwrap();
        assert_eq!(get("binding"), "'<Shift><Super>n'");
        assert_eq!(get("command"), r"'\'/opt/Quick Desk/quickdesk\' toggle notes'");
    }

    #[test]
    fn apply_is_idempotent_and_drops_stale_entries() {
        let fake = Fake::default();
        let kb = GnomeKeybindings::with(&fake);
        kb.apply(&[binding("notes", "Super+Shift+N"), binding("old", "Super+Shift+O")]).unwrap();
        kb.apply(&[binding("notes", "Super+Shift+N")]).unwrap();
        kb.apply(&[binding("notes", "Super+Shift+N")]).unwrap();

        assert_eq!(list(&fake), vec![path_for("notes")]);
        assert_eq!(*fake.resets.borrow(), vec![entry_schema(&path_for("old"))]);
    }

    #[test]
    fn finds_system_and_custom_conflicts_but_not_our_own() {
        let fake = Fake::default();
        (&fake).set("org.gnome.shell.keybindings", "toggle-message-tray", "['<Super>v', '<Super>m']").unwrap();
        (&fake).set("org.gnome.desktop.wm.keybindings", "minimize", "['<Super>h']").unwrap();
        let user = format!("{PATH_ROOT}custom0/");
        (&fake).set(LIST_SCHEMA, LIST_KEY, &format_strv(std::slice::from_ref(&user))).unwrap();
        (&fake).set(&entry_schema(&user), "name", "'Terminal'").unwrap();
        (&fake).set(&entry_schema(&user), "binding", "'<Primary><Alt>t'").unwrap();
        let kb = GnomeKeybindings::with(&fake);
        kb.apply(&[binding("notes", "Super+Alt+N")]).unwrap();

        let check = |a: &str| kb.conflicts(&a.parse().unwrap());
        assert_eq!(check("Super+V"), vec!["GNOME: toggle message tray"]);
        assert_eq!(check("Ctrl+Alt+T"), vec!["Custom shortcut: Terminal"]);
        assert!(check("Super+Alt+N").is_empty(), "our own binding is not a conflict");
        assert!(check("Super+Shift+V").is_empty());
    }

    #[test]
    fn quoted_string_scanner() {
        assert_eq!(quoted_strings("['<Super>v', '<Super>m']"), vec!["<Super>v", "<Super>m"]);
        assert_eq!(quoted_strings(r"'it\'s'"), vec!["it's"]);
        assert!(quoted_strings("@as []").is_empty());
    }

    #[test]
    fn remove_all_restores_user_list() {
        let fake = Fake::default();
        let user = format!("{PATH_ROOT}custom0/");
        (&fake).set(LIST_SCHEMA, LIST_KEY, &format_strv(std::slice::from_ref(&user))).unwrap();
        let kb = GnomeKeybindings::with(&fake);
        kb.apply(&[binding("notes", "Super+Shift+N")]).unwrap();
        kb.remove_all().unwrap();
        assert_eq!(list(&fake), vec![user]);
    }
}
