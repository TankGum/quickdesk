//! Python through uv (docs.astral.sh/uv). uv keeps interpreters in
//! `~/.local/share/uv/python/<key>` and, for the default one, puts `python3`
//! and `python` in `~/.local/bin`.

use std::path::{Path, PathBuf};

use crate::run::{Cmd, Runner};
use crate::{dir_size, home_relative, version_key, Action, Available, Installed, Manager, Probe, Result};

pub struct Uv {
    bin: PathBuf,
    install_dir: PathBuf,
    bin_dir: PathBuf,
    bin_dir_rc: String,
    version: Option<String>,
    env: Vec<(String, String)>,
}

impl Uv {
    pub fn detect(probe: &Probe) -> Option<Self> {
        let bin = probe.found.get("uv").map(|f| PathBuf::from(&f.path)).or_else(|| {
            [".local/bin/uv", ".cargo/bin/uv"].into_iter().map(|p| probe.home.join(p)).find(|p| p.is_file())
        })?;
        let data = probe.var("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| probe.home.join(".local/share"));
        let install_dir =
            probe.var("UV_PYTHON_INSTALL_DIR").map(PathBuf::from).unwrap_or_else(|| data.join("uv/python"));
        let bin_dir = probe
            .var("UV_PYTHON_BIN_DIR")
            .or_else(|| probe.var("XDG_BIN_HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| probe.home.join(".local/bin"));
        let version = probe.found.get("uv").and_then(|f| crate::probe::version_in(&f.version_line));
        let env =
            crate::shell_env(probe, &["UV_PYTHON_INSTALL_DIR", "UV_PYTHON_BIN_DIR", "XDG_DATA_HOME", "XDG_BIN_HOME"]);
        Some(Uv { bin_dir_rc: home_relative(probe, &bin_dir), bin, install_dir, bin_dir, version, env })
    }

    /// The installation (directory name under `install_dir`) that `path` belongs to.
    fn installation_of(&self, path: &Path) -> Option<String> {
        let real = std::fs::canonicalize(path).ok()?;
        let root = std::fs::canonicalize(&self.install_dir).ok()?;
        let rest = real.strip_prefix(&root).ok()?;
        rest.components().next()?.as_os_str().to_str().map(str::to_owned)
    }
}

/// `cpython-3.12.4-linux-x86_64-gnu` → `3.12.4`; `cpython-3.13.1+freethreaded-…`
/// → `3.13.1+freethreaded`; `pypy-3.10.14-…` → `pypy 3.10.14`.
fn display_version(key: &str) -> String {
    let mut parts = key.splitn(3, '-');
    let implementation = parts.next().unwrap_or("");
    let version = parts.next().unwrap_or(key);
    if implementation == "cpython" {
        version.to_owned()
    } else {
        format!("{implementation} {version}")
    }
}

impl Manager for Uv {
    fn id(&self) -> &'static str {
        "uv"
    }
    fn lang(&self) -> &'static str {
        "python"
    }
    fn version(&self) -> Option<String> {
        self.version.clone()
    }

    fn installed(&self) -> Vec<Installed> {
        let Ok(entries) = std::fs::read_dir(&self.install_dir) else { return Vec::new() };
        let mut list: Vec<Installed> = entries
            .flatten()
            // Real directories only: uv also keeps `cpython-3.12-…` links to the newest patch.
            .filter(|e| e.path().symlink_metadata().is_ok_and(|m| m.is_dir()))
            .filter_map(|e| {
                let id = e.file_name().to_str()?.to_owned();
                if id.starts_with('.') || !id.contains('-') {
                    return None;
                }
                Some(Installed {
                    version: display_version(&id),
                    path: e.path().join("bin").join("python3").display().to_string(),
                    bytes: dir_size(&e.path()),
                    is_default: false,
                    id,
                })
            })
            .collect();
        list.sort_by_key(|i| std::cmp::Reverse(version_key(&i.version)));
        list
    }

    fn default_id(&self, installed: &[Installed]) -> Option<String> {
        let key = self.installation_of(&self.bin_dir.join("python3"))?;
        installed.iter().find(|i| i.id == key).map(|i| i.id.clone())
    }

    fn available(&self, run: &dyn Runner) -> Result<Vec<Available>> {
        let mut cmd = Cmd::new(&self.bin, &["python", "list", "--only-downloads", "--output-format", "json"]);
        cmd.env = self.env.clone();
        let out = run.run(&cmd, &mut |_| {})?;
        Ok(parse_list(&out))
    }

    fn command(&self, action: Action, version: &str) -> Cmd {
        let mut cmd = match action {
            Action::Install => Cmd::new(&self.bin, &["python", "install", version]),
            Action::Uninstall => Cmd::new(&self.bin, &["python", "uninstall", version]),
            // Installs when missing, then links python3/python in ~/.local/bin.
            Action::SetDefault => Cmd::new(&self.bin, &["python", "install", "--default", version]),
            Action::InstallManager => unreachable!("handled by System::command"),
        };
        cmd.env = self.env.clone();
        cmd
    }

    fn project_file(&self) -> &'static str {
        ".python-version"
    }
    fn project_content(&self, version: &str) -> String {
        format!("{}\n", display_version_or_plain(version))
    }

    fn manages(&self, path: &str) -> bool {
        self.installation_of(Path::new(path)).is_some()
    }

    fn loaded(&self, probe: &Probe) -> bool {
        // Loaded and first: before /usr/bin, where the system python3 lives.
        match (probe.path_index(&self.bin_dir), probe.path_index(Path::new("/usr/bin"))) {
            (Some(ours), Some(system)) => ours < system,
            (Some(_), None) => true,
            _ => false,
        }
    }

    fn init_lines(&self) -> Vec<String> {
        vec![format!("export PATH=\"{}:$PATH\"", self.bin_dir_rc)]
    }
}

/// A key (`cpython-3.12.4-linux-…`) or a plain version (`3.12`).
fn display_version_or_plain(v: &str) -> String {
    if v.contains('-') {
        display_version(v)
    } else {
        v.to_owned()
    }
}

/// `uv python list --only-downloads --output-format json`: CPython builds,
/// newest first; one entry per version (the default variant).
fn parse_list(out: &str) -> Vec<Available> {
    let Ok(serde_json::Value::Array(items)) = serde_json::from_str::<serde_json::Value>(out) else { return Vec::new() };
    let mut list: Vec<Available> = Vec::new();
    for item in items {
        let get = |k: &str| item.get(k).and_then(|v| v.as_str()).unwrap_or("");
        if get("implementation") != "cpython" || get("variant") != "default" {
            continue;
        }
        let version = get("version").to_owned();
        if version.is_empty() || list.iter().any(|a| a.version == version) {
            continue;
        }
        let pre = version.contains(['a', 'b']) || version.contains("rc");
        list.push(Available {
            id: version.clone(),
            tag: pre.then(|| "pre-release".to_owned()),
            version,
            line: String::new(),
            installed: false,
        });
    }
    list.sort_by_key(|a| std::cmp::Reverse(version_key(&a.version)));
    if let Some(first) = list.iter_mut().find(|a| a.tag.is_none()) {
        first.tag = Some("latest".into());
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{FakeRunner, TempDir};

    #[cfg(unix)]
    #[test]
    fn lists_installs_and_finds_the_default_through_links() {
        let t = TempDir::new("uv");
        t.file(".local/bin/uv", "#!");
        let py312 = t.file(".local/share/uv/python/cpython-3.12.4-linux-x86_64-gnu/bin/python3.12", "#!");
        t.file(".local/share/uv/python/cpython-3.13.1-linux-x86_64-gnu/bin/python3.13", "#!");
        t.file(".local/share/uv/python/.lock", "");
        // uv's minor-version link and the default executables.
        std::os::unix::fs::symlink(
            t.path().join(".local/share/uv/python/cpython-3.12.4-linux-x86_64-gnu"),
            t.path().join(".local/share/uv/python/cpython-3.12-linux-x86_64-gnu"),
        )
        .unwrap();
        std::os::unix::fs::symlink(
            t.path().join(".local/share/uv/python/cpython-3.12-linux-x86_64-gnu/bin/python3.12"),
            t.path().join(".local/bin/python3"),
        )
        .unwrap();
        let probe = Probe { home: t.path().to_path_buf(), ..Default::default() };
        let uv = Uv::detect(&probe).unwrap();
        let installed = uv.installed();
        let versions: Vec<&str> = installed.iter().map(|i| i.version.as_str()).collect();
        assert_eq!(versions, ["3.13.1", "3.12.4"]);
        assert_eq!(uv.default_id(&installed).as_deref(), Some("cpython-3.12.4-linux-x86_64-gnu"));
        assert!(uv.manages(&py312.display().to_string()));
        assert!(uv.manages(&t.path().join(".local/bin/python3").display().to_string()));
        assert!(!uv.manages("/usr/bin/python3"));
        assert_eq!(uv.command(Action::SetDefault, "3.13.1").args, ["python", "install", "--default", "3.13.1"]);
        assert_eq!(uv.project_content("cpython-3.12.4-linux-x86_64-gnu"), "3.12.4\n");
        assert_eq!(uv.init_lines(), ["export PATH=\"$HOME/.local/bin:$PATH\""]);
    }

    #[test]
    fn needs_its_bin_dir_before_usr_bin() {
        let t = TempDir::new("uv-path");
        t.file(".local/bin/uv", "#!");
        let local = t.path().join(".local/bin").display().to_string();
        let uv = Uv::detect(&Probe { home: t.path().to_path_buf(), ..Default::default() }).unwrap();
        let probe =
            |path: &[&str]| Probe { path: path.iter().map(|s| (*s).to_owned()).collect(), ..Default::default() };
        assert!(uv.loaded(&probe(&[&local, "/usr/bin"])));
        assert!(!uv.loaded(&probe(&["/usr/bin", &local])));
        assert!(!uv.loaded(&probe(&["/usr/bin"])));
    }

    #[test]
    fn parses_downloads_newest_first() {
        let t = TempDir::new("uv-list");
        t.file(".local/bin/uv", "#!");
        let uv = Uv::detect(&Probe { home: t.path().to_path_buf(), ..Default::default() }).unwrap();
        let run = FakeRunner {
            output: r#"[{"version":"3.15.0b2","implementation":"cpython","variant":"default"},
                {"version":"3.15.0b2","implementation":"cpython","variant":"freethreaded"},
                {"version":"3.14.6","implementation":"cpython","variant":"default"},
                {"version":"3.9.25","implementation":"cpython","variant":"default"},
                {"version":"3.10.16","implementation":"cpython","variant":"default"},
                {"version":"3.11.13","implementation":"pypy","variant":"default"}]"#
                .into(),
            ..Default::default()
        };
        let list = uv.available(&run).unwrap();
        let got: Vec<(&str, Option<&str>)> = list.iter().map(|a| (a.version.as_str(), a.tag.as_deref())).collect();
        assert_eq!(
            got,
            [("3.15.0b2", Some("pre-release")), ("3.14.6", Some("latest")), ("3.10.16", None), ("3.9.25", None)]
        );
        assert_eq!(run.ran.lock().unwrap()[0].args, ["python", "list", "--only-downloads", "--output-format", "json"]);
    }
}
