//! Node.js through nvm (github.com/nvm-sh/nvm). nvm is a shell function, so
//! commands run as `bash -c '. "$NVM_DIR/nvm.sh" --no-use && nvm <verb> "$1"'`
//! with the version as `$1`, never spliced into the script.

use std::path::{Path, PathBuf};

use crate::run::{Cmd, Runner};
use crate::{dir_size, home_relative, version_key, Action, Available, Installed, Manager, Probe, Result};

pub struct Nvm {
    dir: PathBuf,
    home_dir: String,
}

impl Nvm {
    pub fn detect(probe: &Probe) -> Option<Self> {
        let dir = probe.dir("NVM_DIR", ".nvm");
        dir.join("nvm.sh").is_file().then(|| Nvm { home_dir: home_relative(probe, &dir), dir })
    }

    fn versions_dir(&self) -> PathBuf {
        self.dir.join("versions").join("node")
    }

    fn script(&self, body: &str) -> Cmd {
        let script = format!(". \"$NVM_DIR/nvm.sh\" --no-use && {body}");
        Cmd::new("bash", &["-c", &script, "quickdesk"]).env("NVM_DIR", self.dir.display().to_string())
    }

    /// Follow nvm aliases (`default` → `lts/*` → `lts/krypton` → `v24.21.0`)
    /// to an installed version.
    fn resolve(&self, alias: &str, installed: &[Installed], depth: u8) -> Option<String> {
        let alias = alias.trim();
        if depth > 4 || alias.is_empty() {
            return None;
        }
        if matches!(alias, "node" | "stable") {
            return installed.first().map(|i| i.id.clone());
        }
        if alias.starts_with("lts/") {
            let next = std::fs::read_to_string(self.dir.join("alias").join(alias)).ok()?;
            return self.resolve(&next, installed, depth + 1);
        }
        let want = version_key(alias);
        if want.is_empty() {
            let next = std::fs::read_to_string(self.dir.join("alias").join(alias)).ok()?;
            return self.resolve(&next, installed, depth + 1);
        }
        // `22` matches the newest v22.x.y; `installed` is newest first.
        installed.iter().find(|i| version_key(&i.id).starts_with(&want)).map(|i| i.id.clone())
    }
}

impl Manager for Nvm {
    fn id(&self) -> &'static str {
        "nvm"
    }
    fn lang(&self) -> &'static str {
        "node"
    }

    fn version(&self) -> Option<String> {
        let pkg = std::fs::read_to_string(self.dir.join("package.json")).ok()?;
        let v: serde_json::Value = serde_json::from_str(&pkg).ok()?;
        v.get("version")?.as_str().map(str::to_owned)
    }

    fn installed(&self) -> Vec<Installed> {
        let Ok(entries) = std::fs::read_dir(self.versions_dir()) else { return Vec::new() };
        let mut list: Vec<Installed> = entries
            .flatten()
            .filter(|e| e.path().join("bin").join("node").exists())
            .filter_map(|e| {
                let id = e.file_name().to_str()?.to_owned();
                Some(Installed {
                    version: id.trim_start_matches('v').to_owned(),
                    path: e.path().join("bin").join("node").display().to_string(),
                    bytes: dir_size(&e.path()),
                    is_default: false,
                    id,
                })
            })
            .collect();
        list.sort_by_key(|i| std::cmp::Reverse(version_key(&i.id)));
        list
    }

    fn default_id(&self, installed: &[Installed]) -> Option<String> {
        let alias = std::fs::read_to_string(self.dir.join("alias").join("default")).ok()?;
        self.resolve(&alias, installed, 0)
    }

    fn available(&self, run: &dyn Runner) -> Result<Vec<Available>> {
        let out = run.run(&self.script("nvm ls-remote"), &mut |_| {})?;
        Ok(parse_ls_remote(&out))
    }

    fn command(&self, action: Action, version: &str) -> Cmd {
        let verb = match action {
            Action::Install => "nvm install \"$1\"",
            Action::Uninstall => "nvm uninstall \"$1\"",
            Action::SetDefault => "nvm alias default \"$1\"",
            Action::InstallManager => unreachable!("handled by System::command"),
        };
        let mut cmd = self.script(verb);
        cmd.args.push(version.to_owned());
        cmd
    }

    fn project_file(&self) -> &'static str {
        ".nvmrc"
    }
    fn project_content(&self, version: &str) -> String {
        format!("{version}\n")
    }

    fn manages(&self, path: &str) -> bool {
        Path::new(path).starts_with(self.versions_dir())
    }

    fn loaded(&self, probe: &Probe) -> bool {
        let versions = self.versions_dir();
        probe.path.iter().any(|p| Path::new(p).starts_with(&versions))
    }

    fn init_lines(&self) -> Vec<String> {
        vec![
            format!("export NVM_DIR=\"{}\"", self.home_dir),
            "[ -s \"$NVM_DIR/nvm.sh\" ] && . \"$NVM_DIR/nvm.sh\"".into(),
        ]
    }
}

/// `nvm ls-remote`: one version per line, newest last, `(LTS: Krypton)` tags.
fn parse_ls_remote(out: &str) -> Vec<Available> {
    let mut list: Vec<Available> = out
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_start_matches("->").trim();
            let id = line.split_whitespace().next()?;
            if !id.starts_with('v') || version_key(id).len() != 3 {
                return None;
            }
            let tag = line
                .split_once('(')
                .map(|(_, t)| t.trim_end_matches(')').trim().replace("Latest LTS", "LTS").to_owned())
                .filter(|t| t.starts_with("LTS"));
            Some(Available {
                id: id.to_owned(),
                version: id.trim_start_matches('v').to_owned(),
                tag,
                line: String::new(),
                installed: false,
            })
        })
        .collect();
    list.reverse();
    if let Some(first) = list.first_mut() {
        first.tag.get_or_insert_with(|| "latest".into());
    }
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{FakeRunner, TempDir};

    fn nvm_home(t: &TempDir) -> Nvm {
        t.file(".nvm/nvm.sh", "");
        t.file(".nvm/package.json", r#"{"name":"nvm","version":"0.40.8"}"#);
        for v in ["v22.9.1", "v22.10.0", "v20.18.0"] {
            t.file(&format!(".nvm/versions/node/{v}/bin/node"), "#!");
        }
        t.file(".nvm/alias/default", "lts/*\n");
        t.file(".nvm/alias/lts/*", "lts/jod\n");
        t.file(".nvm/alias/lts/jod", "v22.10.0\n");
        let probe = Probe { home: t.path().to_path_buf(), ..Default::default() };
        Nvm::detect(&probe).unwrap()
    }

    #[test]
    fn lists_installed_and_follows_default_aliases() {
        let t = TempDir::new("nvm");
        let nvm = nvm_home(&t);
        assert_eq!(nvm.version().as_deref(), Some("0.40.8"));
        let installed = nvm.installed();
        let ids: Vec<&str> = installed.iter().map(|i| i.id.as_str()).collect();
        assert_eq!(ids, ["v22.10.0", "v22.9.1", "v20.18.0"]);
        assert_eq!(nvm.default_id(&installed).as_deref(), Some("v22.10.0"));
        assert_eq!(nvm.resolve("20", &installed, 0).as_deref(), Some("v20.18.0"));
        assert_eq!(nvm.resolve("22", &installed, 0).as_deref(), Some("v22.10.0"));
        assert_eq!(nvm.resolve("18", &installed, 0), None);
        assert_eq!(nvm.resolve("node", &installed, 0).as_deref(), Some("v22.10.0"));
        assert_eq!(nvm.init_lines()[0], "export NVM_DIR=\"$HOME/.nvm\"");
    }

    #[test]
    fn passes_the_version_as_an_argument_not_in_the_script() {
        let t = TempDir::new("nvm-cmd");
        let nvm = nvm_home(&t);
        let cmd = nvm.command(Action::Install, "v18.20.4");
        assert_eq!(cmd.program, PathBuf::from("bash"));
        assert_eq!(cmd.args[1], ". \"$NVM_DIR/nvm.sh\" --no-use && nvm install \"$1\"");
        assert_eq!(cmd.args[2..], ["quickdesk", "v18.20.4"]);
        assert_eq!(
            nvm.command(Action::SetDefault, "22").args[1].rsplit("&& ").next(),
            Some("nvm alias default \"$1\"")
        );
    }

    #[test]
    fn parses_ls_remote_newest_first_with_lts_tags() {
        let t = TempDir::new("nvm-ls");
        let nvm = nvm_home(&t);
        let run = FakeRunner {
            output: "        v0.1.14\n       v22.10.0   (LTS: Jod)\n->     v24.21.0   (Latest LTS: Krypton)\n       v26.11.1\n       N/A\n".into(),
            ..Default::default()
        };
        let list = nvm.available(&run).unwrap();
        let got: Vec<(&str, Option<&str>)> = list.iter().map(|a| (a.version.as_str(), a.tag.as_deref())).collect();
        assert_eq!(
            got,
            [
                ("26.11.1", Some("latest")),
                ("24.21.0", Some("LTS: Krypton")),
                ("22.10.0", Some("LTS: Jod")),
                ("0.1.14", None)
            ]
        );
    }

    #[test]
    fn knows_what_it_manages() {
        let t = TempDir::new("nvm-path");
        let nvm = nvm_home(&t);
        let node = t.path().join(".nvm/versions/node/v22.10.0/bin/node");
        assert!(nvm.manages(&node.display().to_string()));
        assert!(!nvm.manages("/usr/bin/node"));
        let probe = Probe { path: vec!["/usr/bin".into()], ..Default::default() };
        assert!(!nvm.loaded(&probe));
        let probe = Probe { path: vec![node.parent().unwrap().display().to_string()], ..Default::default() };
        assert!(nvm.loaded(&probe));
    }
}
