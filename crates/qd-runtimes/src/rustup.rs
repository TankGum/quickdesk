//! Rust through rustup. Toolchains are channels (`stable`, `nightly`) or
//! versions (`1.82.0`); rustup cannot list what exists, so versions are typed.

use std::path::{Path, PathBuf};

use crate::run::{Cmd, Runner};
use crate::{dir_size, home_relative, Action, Available, Installed, Manager, Probe, Result, Update};

pub struct Rustup {
    bin: PathBuf,
    rustup_home: PathBuf,
    cargo_bin: PathBuf,
    cargo_env: String,
    version: Option<String>,
    env: Vec<(String, String)>,
}

impl Rustup {
    pub fn detect(probe: &Probe) -> Option<Self> {
        let cargo_home = probe.dir("CARGO_HOME", ".cargo");
        let cargo_bin = cargo_home.join("bin");
        let bin = probe
            .found
            .get("rustup")
            .map(|f| PathBuf::from(&f.path))
            .or_else(|| Some(cargo_bin.join(crate::exe("rustup"))).filter(|p| p.is_file()))?;
        let version = probe.found.get("rustup").and_then(|f| crate::probe::version_in(&f.version_line));
        Some(Rustup {
            bin,
            rustup_home: probe.dir("RUSTUP_HOME", ".rustup"),
            cargo_env: home_relative(probe, &cargo_home.join("env")),
            cargo_bin,
            version,
            env: crate::shell_env(probe, &["RUSTUP_HOME", "CARGO_HOME"]),
        })
    }
}

/// `stable-x86_64-unknown-linux-gnu` → `stable`; `nightly-2026-01-01-x86_64-…` → `nightly-2026-01-01`.
fn short_name(toolchain: &str) -> &str {
    for arch in ["-x86_64-", "-aarch64-", "-i686-", "-armv7-", "-riscv64gc-", "-powerpc64le-", "-s390x-"] {
        if let Some(i) = toolchain.find(arch) {
            return &toolchain[..i];
        }
    }
    toolchain
}

/// `rustup check`: `stable-x86_64-unknown-linux-gnu - update available: 1.98.0 (abc 2026-08-01) -> 1.99.0 (def 2026-09-28)`
/// (rustup before 1.29 capitalises it: `Update available : …`).
fn parse_check(output: &str, installed: &[Installed]) -> Vec<Update> {
    // `1.99.0 (def 2026-09-28)` → `1.99.0`, and its date for nightlies, whose version can stay the same.
    let parts = |s: &str| {
        let s = s.trim();
        let version = s.split_whitespace().next().unwrap_or("").to_owned();
        let date = s.trim_end_matches(')').rsplit(' ').next().unwrap_or("").to_owned();
        (version, date)
    };
    output
        .lines()
        .filter_map(|line| {
            let (name, rest) = line.split_once(" - ")?;
            let rest = rest.trim();
            if !rest.to_lowercase().starts_with("update available") {
                return None;
            }
            let i = installed.iter().find(|i| i.id == name.trim())?;
            let (from, to) = rest.split_once(':')?.1.split_once("->")?;
            let ((fv, fd), (tv, td)) = (parts(from), parts(to));
            if tv.is_empty() {
                return None;
            }
            let (from_version, to_version) =
                if fv == tv { (format!("{fv} {fd}"), format!("{tv} {td}")) } else { (fv, tv) };
            Some(Update { from: i.id.clone(), from_version, to: i.id.clone(), to_version })
        })
        .collect()
}

impl Manager for Rustup {
    fn id(&self) -> &'static str {
        "rustup"
    }
    fn lang(&self) -> &'static str {
        "rust"
    }
    fn version(&self) -> Option<String> {
        self.version.clone()
    }

    fn installed(&self) -> Vec<Installed> {
        let Ok(entries) = std::fs::read_dir(self.rustup_home.join("toolchains")) else { return Vec::new() };
        let mut list: Vec<Installed> = entries
            .flatten()
            .filter(|e| e.path().join("bin").join(crate::exe("rustc")).exists())
            .filter_map(|e| {
                let id = e.file_name().to_str()?.to_owned();
                Some(Installed {
                    version: short_name(&id).to_owned(),
                    path: e.path().join("bin").join(crate::exe("rustc")).display().to_string(),
                    bytes: dir_size(&e.path()),
                    is_default: false,
                    id,
                })
            })
            .collect();
        // Channels first in their usual order, then versions newest first.
        let rank = |v: &str| match v {
            "stable" => 0,
            "beta" => 1,
            v if v.starts_with("nightly") => 2,
            _ => 3,
        };
        list.sort_by(|a, b| {
            rank(&a.version)
                .cmp(&rank(&b.version))
                .then_with(|| crate::version_key(&b.version).cmp(&crate::version_key(&a.version)))
        });
        list
    }

    fn default_id(&self, installed: &[Installed]) -> Option<String> {
        let settings = std::fs::read_to_string(self.rustup_home.join("settings.toml")).ok()?;
        let name = settings.lines().find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == "default_toolchain").then(|| v.trim().trim_matches('"').to_owned())
        })?;
        installed.iter().find(|i| i.id == name || i.version == name).map(|i| i.id.clone())
    }

    fn available(&self, _run: &dyn Runner) -> Result<Vec<Available>> {
        Ok(["stable", "beta", "nightly"]
            .into_iter()
            .map(|c| Available { id: c.into(), version: c.into(), tag: None, line: String::new(), installed: false })
            .collect())
    }

    fn command(&self, action: Action, version: &str) -> Cmd {
        let args: &[&str] = match action {
            Action::Install => &["toolchain", "install", version],
            Action::Uninstall => &["toolchain", "uninstall", version],
            Action::SetDefault => &["default", version],
            Action::InstallManager => unreachable!("handled by System::command"),
        };
        let mut cmd = Cmd::new(&self.bin, args);
        cmd.env = self.env.clone();
        cmd
    }

    fn project_file(&self) -> &'static str {
        "rust-toolchain.toml"
    }
    fn project_content(&self, version: &str) -> String {
        format!("[toolchain]\nchannel = \"{}\"\n", short_name(version))
    }

    fn manages(&self, path: &str) -> bool {
        crate::probe::under(Path::new(path), &self.cargo_bin) || crate::probe::under(Path::new(path), &self.rustup_home)
    }

    fn loaded(&self, probe: &Probe) -> bool {
        probe.on_path(&self.cargo_bin)
    }

    fn init_lines(&self) -> Vec<String> {
        vec![format!(". \"{}\"", self.cargo_env)]
    }

    fn path_dirs(&self) -> Vec<PathBuf> {
        vec![self.cargo_bin.clone()]
    }

    fn free_input(&self) -> bool {
        true
    }

    fn channel_updates(&self, run: &dyn Runner) -> Option<Result<Vec<Update>>> {
        let mut cmd = Cmd::new(&self.bin, &["check"]);
        cmd.env = self.env.clone();
        // Newer rustup exits with 100 when there are updates: keep the lines either way.
        let mut out = String::new();
        let result = run.run(&cmd, &mut |l| {
            out.push_str(l);
            out.push('\n');
        });
        let installed = self.installed();
        let found = parse_check(&out, &installed);
        Some(match result {
            Err(e) if found.is_empty() && !out.contains(" - ") => Err(e),
            _ => Ok(found),
        })
    }

    fn update_in_place(&self, id: &str) -> Option<Cmd> {
        let mut cmd = Cmd::new(&self.bin, &["update", "--no-self-update", id]);
        cmd.env = self.env.clone();
        Some(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    #[test]
    fn lists_toolchains_and_reads_the_default() {
        let t = TempDir::new("rustup");
        t.file(&format!(".cargo/bin/{}", crate::exe("rustup")), "#!");
        for tc in [
            "stable-x86_64-unknown-linux-gnu",
            "1.82.0-x86_64-unknown-linux-gnu",
            "nightly-2026-01-01-x86_64-unknown-linux-gnu",
        ] {
            t.file(&format!(".rustup/toolchains/{tc}/bin/{}", crate::exe("rustc")), "#!");
        }
        t.file(
            ".rustup/settings.toml",
            "version = \"12\"\ndefault_toolchain = \"stable-x86_64-unknown-linux-gnu\"\nprofile = \"minimal\"\n",
        );
        let probe = Probe { home: t.path().to_path_buf(), ..Default::default() };
        let r = Rustup::detect(&probe).unwrap();
        let installed = r.installed();
        let names: Vec<&str> = installed.iter().map(|i| i.version.as_str()).collect();
        assert_eq!(names, ["stable", "nightly-2026-01-01", "1.82.0"]);
        assert_eq!(r.default_id(&installed).as_deref(), Some("stable-x86_64-unknown-linux-gnu"));
        assert_eq!(r.command(Action::SetDefault, "1.82.0").args, ["default", "1.82.0"]);
        assert_eq!(
            r.project_content("nightly-2026-01-01-x86_64-unknown-linux-gnu"),
            "[toolchain]\nchannel = \"nightly-2026-01-01\"\n"
        );
        assert_eq!(r.init_lines(), [". \"$HOME/.cargo/env\""]);
        assert!(r.manages(&t.path().join(".cargo/bin/rustc").display().to_string()));
    }

    #[test]
    fn reads_rustup_check() {
        let inst = |id: &str| Installed {
            id: id.into(),
            version: short_name(id).into(),
            path: String::new(),
            bytes: 0,
            is_default: false,
        };
        let installed = [
            inst("stable-x86_64-unknown-linux-gnu"),
            inst("nightly-x86_64-unknown-linux-gnu"),
            inst("1.82.0-x86_64-unknown-linux-gnu"),
        ];
        let out = "stable-x86_64-unknown-linux-gnu - update available: 1.98.0 (aaa 2026-08-01) -> 1.99.0 (bbb 2026-09-28)\n\
                   nightly-x86_64-unknown-linux-gnu - Update available : 1.100.0-nightly (ccc 2026-10-01) -> 1.100.0-nightly (ddd 2026-10-10)\n\
                   1.82.0-x86_64-unknown-linux-gnu - up to date: 1.82.0 (eee 2024-10-15)\n\
                   rustup - update available : 1.29.1 -> 1.30.0\n";
        let ups = parse_check(out, &installed);
        assert_eq!(ups.len(), 2, "{ups:?}");
        assert_eq!(
            (ups[0].from.as_str(), ups[0].to.as_str()),
            ("stable-x86_64-unknown-linux-gnu", "stable-x86_64-unknown-linux-gnu")
        );
        assert_eq!((ups[0].from_version.as_str(), ups[0].to_version.as_str()), ("1.98.0", "1.99.0"));
        assert_eq!(ups[1].to_version, "1.100.0-nightly 2026-10-10");
        assert!(
            parse_check("stable-x86_64-unknown-linux-gnu - up to date: 1.99.0 (b 2026-09-28)\n", &installed).is_empty()
        );
    }
}
