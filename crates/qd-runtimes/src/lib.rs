//! Programming language versions: what a terminal runs, what each version
//! manager has installed, and changing that through the manager's own CLI
//! (nvm for Node, rustup for Rust, uv for Python, mise for the rest). QuickDesk
//! never installs a language itself and never touches system packages.

pub mod hook;
pub mod mise;
mod nvm;
pub mod probe;
pub mod run;
mod rustup;
pub mod shellrc;
mod uv;

use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

pub use probe::Probe;
pub use run::{Cmd, Runner, SystemRunner};

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    Unsupported(String),
    #[error("cancelled")]
    Cancelled,
    #[error("{0}")]
    Failed(String),
    #[error("{0}")]
    Io(String),
}

/// How many leading numbers make a release line: Node 22.x, Java 21.x,
/// Python 3.12.x, Go 1.23.x. Rust channels have no lines.
fn line_len(lang: &str) -> usize {
    match lang {
        "node" | "java" | "deno" => 1,
        "rust" => 0,
        _ => 2,
    }
}

/// `22.10.0` → `22` (Node), `3.12.4` → `3.12` (Python); empty when unknown.
pub fn line_of(lang: &str, version: &str) -> String {
    let n = line_len(lang);
    let key = version_key(version);
    if n == 0 || key.len() < n {
        return String::new();
    }
    key[..n].iter().map(|k| k.to_string()).collect::<Vec<_>>().join(".")
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// One language as this computer has it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Runtime {
    /// `node`, `python`, `rust`, `go`…
    pub id: String,
    pub name: String,
    /// The manager QuickDesk changes versions through; `None`: read-only.
    pub manager: Option<String>,
    pub manager_version: Option<String>,
    /// What a new terminal runs.
    pub active: Option<Active>,
    /// The manager's default (an `Installed::id`).
    pub default: Option<String>,
    pub installed: Vec<Installed>,
    pub issue: Option<Issue>,
    /// Versions are plain text to type in (rustup has no list to choose from).
    pub free_input: bool,
    /// The file "Set for a project" writes, e.g. `.nvmrc`.
    pub project_file: Option<String>,
    /// No manager yet, but this one could be installed to manage it (`mise`).
    pub install_manager: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Active {
    pub version: String,
    pub path: String,
    /// From the manager (as opposed to the system or something else).
    pub managed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Installed {
    /// What the manager calls it (`v22.1.0`, `stable-x86_64-unknown-linux-gnu`).
    pub id: String,
    /// What people call it (`22.1.0`, `stable`).
    pub version: String,
    pub path: String,
    pub bytes: u64,
    pub is_default: bool,
}

/// The terminal does not run the manager's default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    /// `not_loaded`: the manager is not set up in the shell; `shadowed`:
    /// another copy comes first in PATH; `other`: something else (e.g. an
    /// `nvm use` in ~/.bashrc).
    pub kind: &'static str,
    /// What should run (default version).
    pub expected: String,
    /// Lines that fix it when added to the shell's rc file.
    pub fix: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Available {
    /// Passed back to install.
    pub id: String,
    pub version: String,
    /// `LTS: Krypton`, `latest`, `pre-release`…
    pub tag: Option<String>,
    /// The release line it belongs to (`22` for Node, `3.12` for Python):
    /// updates stay within a line.
    pub line: String,
    pub installed: bool,
}

/// A newer release in the same line as an installed version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Update {
    /// `Installed::id` that has a newer release.
    pub from: String,
    pub from_version: String,
    /// `Available::id` to install.
    pub to: String,
    pub to_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Install,
    Uninstall,
    SetDefault,
    /// Install mise itself (the version is ignored).
    InstallManager,
}

impl Action {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "install" => Some(Action::Install),
            "uninstall" => Some(Action::Uninstall),
            "set_default" => Some(Action::SetDefault),
            "install_manager" => Some(Action::InstallManager),
            _ => None,
        }
    }
}

/// A version manager for one language.
pub(crate) trait Manager {
    fn id(&self) -> &'static str;
    fn lang(&self) -> &'static str;
    fn version(&self) -> Option<String>;
    fn installed(&self) -> Vec<Installed>;
    /// `Installed::id` of the default, among `installed`.
    fn default_id(&self, installed: &[Installed]) -> Option<String>;
    fn available(&self, run: &dyn Runner) -> Result<Vec<Available>>;
    /// `version` is already validated.
    fn command(&self, action: Action, version: &str) -> Cmd;
    fn project_file(&self) -> &'static str;
    fn project_content(&self, version: &str) -> String;
    /// When the manager edits its project file itself (mise), the command to run.
    fn project_command(&self, _dir: &Path, _version: &str) -> Option<Cmd> {
        None
    }
    /// `path` (what the terminal runs) comes from this manager.
    fn manages(&self, path: &str) -> bool;
    /// The manager's binaries are in the shell's PATH at all.
    fn loaded(&self, probe: &Probe) -> bool;
    /// rc lines that put the manager on PATH, first.
    fn init_lines(&self) -> Vec<String>;
    fn free_input(&self) -> bool {
        false
    }
}

struct Lang {
    id: &'static str,
    name: &'static str,
    /// First one found wins.
    bins: &'static [&'static str],
}

const LANGS: &[Lang] = &[
    Lang { id: "node", name: "Node.js", bins: &["node"] },
    Lang { id: "python", name: "Python", bins: &["python3", "python"] },
    Lang { id: "rust", name: "Rust", bins: &["rustc"] },
    Lang { id: "go", name: "Go", bins: &["go"] },
    Lang { id: "java", name: "Java", bins: &["java"] },
    Lang { id: "ruby", name: "Ruby", bins: &["ruby"] },
    Lang { id: "deno", name: "Deno", bins: &["deno"] },
    Lang { id: "bun", name: "Bun", bins: &["bun"] },
    Lang { id: "php", name: "PHP", bins: &["php"] },
];

/// Everything needed to answer questions about this computer.
pub struct System {
    pub probe: Probe,
    /// Native managers first; mise tools after them.
    managers: Vec<Box<dyn Manager + Send + Sync>>,
    has_mise: bool,
}

impl System {
    /// Probe the user's shell. Takes a few hundred milliseconds.
    pub fn detect() -> Self {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"));
        let shell = std::env::var("SHELL").ok();
        Self::from_probe(probe::run(&home, shell.as_deref()))
    }

    pub fn from_probe(probe: Probe) -> Self {
        let mut managers: Vec<Box<dyn Manager + Send + Sync>> = Vec::new();
        if let Some(m) = nvm::Nvm::detect(&probe) {
            managers.push(Box::new(m));
        }
        if let Some(m) = rustup::Rustup::detect(&probe) {
            managers.push(Box::new(m));
        }
        if let Some(m) = uv::Uv::detect(&probe) {
            managers.push(Box::new(m));
        }
        let mise = mise::Mise::detect(&probe);
        let has_mise = mise.is_some();
        if let Some(mise) = mise {
            for lang in mise::TOOLS {
                managers.push(Box::new(mise::MiseTool { mise: mise.clone(), lang }));
            }
        }
        System { probe, managers, has_mise }
    }

    /// The binary a new terminal runs for `lang`.
    fn found(&self, lang: &str) -> Option<&probe::Found> {
        let l = LANGS.iter().find(|l| l.id == lang)?;
        l.bins.iter().find_map(|b| self.probe.found.get(*b))
    }

    /// The manager for `lang`: the one the terminal's copy comes from, else
    /// the language's own manager (nvm, rustup, uv), else mise.
    fn pick(&self, lang: &str) -> Option<&(dyn Manager + Send + Sync)> {
        let all: Vec<&(dyn Manager + Send + Sync)> =
            self.managers.iter().filter(|m| m.lang() == lang).map(|m| m.as_ref()).collect();
        self.found(lang).and_then(|f| all.iter().copied().find(|m| m.manages(&f.path))).or_else(|| all.first().copied())
    }

    fn manager(&self, lang: &str) -> Result<&(dyn Manager + Send + Sync)> {
        self.pick(lang)
            .ok_or_else(|| Error::Unsupported(format!("no supported version manager for {lang} on this computer")))
    }

    /// Languages present on this computer, through a manager or on PATH.
    pub fn runtimes(&self) -> Vec<Runtime> {
        LANGS.iter().filter_map(|l| self.runtime(l)).collect()
    }

    fn runtime(&self, lang: &Lang) -> Option<Runtime> {
        let manager = self.pick(lang.id);
        let found = self.found(lang.id);
        // mise could manage any language; list only the ones it has or the terminal runs.
        if found.is_none() && manager.is_none_or(|m| m.id() == "mise" && m.installed().is_empty()) {
            return None;
        }
        let active = found.map(|f| Active {
            version: probe::version_in(&f.version_line).unwrap_or_else(|| f.version_line.clone()),
            path: f.path.clone(),
            managed: manager.is_some_and(|m| m.manages(&f.path)),
        });
        let mut rt = Runtime {
            id: lang.id.into(),
            name: lang.name.into(),
            manager: None,
            manager_version: None,
            active,
            default: None,
            installed: Vec::new(),
            issue: None,
            free_input: false,
            project_file: None,
            install_manager: (manager.is_none() && !self.has_mise && mise::TOOLS.contains(&lang.id))
                .then(|| "mise".to_owned()),
        };
        if let Some(m) = manager {
            let mut installed = m.installed();
            let default = m.default_id(&installed);
            for i in &mut installed {
                i.is_default = default.as_deref() == Some(i.id.as_str());
            }
            rt.manager = Some(m.id().into());
            rt.manager_version = m.version();
            rt.issue = issue(m, &self.probe, rt.active.as_ref(), &installed, default.as_deref());
            rt.default = default;
            rt.installed = installed;
            rt.free_input = m.free_input();
            rt.project_file = Some(m.project_file().into());
        }
        Some(rt)
    }

    pub fn available(&self, lang: &str, run: &dyn Runner) -> Result<Vec<Available>> {
        let m = self.manager(lang)?;
        let installed = m.installed();
        let mut list = m.available(run)?;
        for a in &mut list {
            a.installed = installed.iter().any(|i| same_version(&i.version, &a.version) || i.id == a.id);
            a.line = line_of(lang, &a.version);
        }
        Ok(list)
    }

    /// For the newest installed version of each line: the newest stable
    /// release of that line, when it is newer and not installed.
    pub fn updates(&self, lang: &str, available: &[Available]) -> Vec<Update> {
        let Ok(m) = self.manager(lang) else { return Vec::new() };
        let installed = m.installed();
        let mut out: Vec<Update> = Vec::new();
        for i in &installed {
            let line = line_of(lang, &i.version);
            if line.is_empty() {
                continue;
            }
            let key = version_key(&i.version);
            // Only the newest installed one of a line gets the offer.
            let newer_installed = installed
                .iter()
                .any(|o| o.id != i.id && line_of(lang, &o.version) == line && version_key(&o.version) > key);
            if newer_installed {
                continue;
            }
            let best = available
                .iter()
                .filter(|a| a.tag.as_deref() != Some("pre-release") && line_of(lang, &a.version) == line)
                .filter(|a| version_key(&a.version) > key)
                .max_by(|a, b| version_key(&a.version).cmp(&version_key(&b.version)));
            if let Some(a) = best {
                out.push(Update {
                    from: i.id.clone(),
                    from_version: i.version.clone(),
                    to: a.id.clone(),
                    to_version: a.version.clone(),
                });
            }
        }
        out
    }

    /// Install `to`; when `from` is the default, make `to` the default. The
    /// old version stays installed.
    pub fn upgrade_commands(&self, lang: &str, from: &str, to: &str) -> Result<Vec<Cmd>> {
        validate_version(from)?;
        validate_version(to)?;
        let m = self.manager(lang)?;
        let installed = m.installed();
        if !installed.iter().any(|i| i.id == from) {
            return Err(Error::Invalid(format!("{from} is not installed")));
        }
        let was_default = m.default_id(&installed).as_deref() == Some(from);
        Ok(match (was_default, m.id()) {
            // uv and mise install while setting the default.
            (true, "uv" | "mise") => vec![m.command(Action::SetDefault, to)],
            (true, _) => vec![m.command(Action::Install, to), m.command(Action::SetDefault, to)],
            (false, _) => vec![m.command(Action::Install, to)],
        })
    }

    /// The command for an action, after checking the version and that the
    /// action makes sense (no removing the default).
    pub fn command(&self, lang: &str, action: Action, version: &str) -> Result<Cmd> {
        if action == Action::InstallManager {
            if self.has_mise {
                return Err(Error::Invalid("mise is already installed".into()));
            }
            return Ok(mise::installer(&mise::install_dir(&self.probe)));
        }
        validate_version(version)?;
        let m = self.manager(lang)?;
        if action == Action::Uninstall {
            let installed = m.installed();
            let default = m.default_id(&installed);
            if default.as_deref() == Some(version) {
                return Err(Error::Invalid("this is the default version; choose another default first".into()));
            }
            if !installed.iter().any(|i| i.id == version) {
                return Err(Error::Invalid(format!("{version} is not installed")));
            }
        }
        Ok(m.command(action, version))
    }

    pub fn project_file(&self, lang: &str) -> Result<&'static str> {
        Ok(self.manager(lang)?.project_file())
    }

    /// Write the project pin, e.g. `.nvmrc` with `v22.1.0`.
    pub fn set_project(&self, dir: &Path, lang: &str, version: &str) -> Result<PathBuf> {
        validate_version(version)?;
        if !dir.is_dir() {
            return Err(Error::Invalid(format!("{} is not a folder", dir.display())));
        }
        let m = self.manager(lang)?;
        let path = dir.join(m.project_file());
        match m.project_command(dir, version) {
            Some(cmd) => {
                SystemRunner::with_timeout(std::time::Duration::from_secs(120)).run(&cmd, &mut |_| {})?;
            }
            None => std::fs::write(&path, m.project_content(version))?,
        }
        Ok(path)
    }

    /// The shell file QuickDesk's block lives in.
    pub fn rc_file(&self) -> PathBuf {
        shellrc::rc_file(&self.probe.home, &self.probe.shell)
    }

    /// Whether "apply right away in open terminals" is on (the hook's line is in the rc file).
    pub fn auto_apply(&self) -> bool {
        let rc = std::fs::read_to_string(self.rc_file()).unwrap_or_default();
        shellrc::block_lines(&rc).contains(&hook::line(&self.probe))
    }

    /// Turn the prompt hook on (write the script, add its line) or off.
    /// Returns the rc file's backup when one was made.
    pub fn set_auto_apply(&self, on: bool, stamp: &str) -> Result<Option<PathBuf>> {
        let line = hook::line(&self.probe);
        if on {
            hook::install(&self.probe)?;
            shellrc::apply(&self.rc_file(), &[line], stamp)
        } else {
            shellrc::remove_lines(&self.rc_file(), &[line], stamp)?;
            hook::remove(&self.probe);
            Ok(None)
        }
    }

    /// Lines to add to the rc file so the terminal runs `lang`'s default.
    pub fn shell_fix(&self, lang: &str) -> Result<Vec<String>> {
        Ok(self.manager(lang)?.init_lines())
    }
}

fn issue(
    m: &dyn Manager,
    probe: &Probe,
    active: Option<&Active>,
    installed: &[Installed],
    default: Option<&str>,
) -> Option<Issue> {
    let default = installed.iter().find(|i| Some(i.id.as_str()) == default)?;
    let expected = default.version.clone();
    match active {
        Some(a) if a.managed => {
            // nvm and uv name exact versions; rustup channels ("stable") cannot be compared.
            if m.free_input() || runs_version(&a.version, &default.version) {
                None
            } else {
                Some(Issue { kind: "other", expected, fix: Vec::new() })
            }
        }
        _ => {
            let kind = if m.loaded(probe) { "shadowed" } else { "not_loaded" };
            Some(Issue { kind, expected, fix: m.init_lines() })
        }
    }
}

/// `v22.1.0` and `22.1.0` are the same version.
fn same_version(a: &str, b: &str) -> bool {
    a.trim_start_matches('v') == b.trim_start_matches('v')
}

/// What the binary reports (`21.0.4`) matches the installed name
/// (`21.0.4+7`, `temurin-21.0.4+7`), as far as both say. A channel
/// (`stable`, `latest`) cannot be compared, so it counts as a match.
fn runs_version(reported: &str, installed: &str) -> bool {
    let (r, i) = (version_key(reported), version_key(installed));
    if r.is_empty() || i.is_empty() {
        return true;
    }
    let n = r.len().min(i.len());
    r[..n] == i[..n]
}

/// Versions go to other programs as arguments; keep them boring.
pub fn validate_version(v: &str) -> Result<()> {
    let ok = !v.is_empty()
        && v.len() <= 80
        && v.as_bytes()[0].is_ascii_alphanumeric()
        && v.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'@' | b'+' | b'-' | b'/'))
        && !v.contains("..");
    if ok {
        Ok(())
    } else {
        Err(Error::Invalid(format!("{v:?} is not a version")))
    }
}

/// The shell's values of `keys` that are set, for commands to run with: the
/// desktop started QuickDesk without what ~/.bashrc exports.
pub(crate) fn shell_env(probe: &Probe, keys: &[&str]) -> Vec<(String, String)> {
    keys.iter().filter_map(|k| probe.var(k).map(|v| ((*k).to_owned(), v.to_owned()))).collect()
}

/// Size of a directory tree, without following symlinks.
pub(crate) fn dir_size(path: &Path) -> u64 {
    let mut total = 0;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let Ok(meta) = e.path().symlink_metadata() else { continue };
            if meta.is_dir() {
                stack.push(e.path());
            } else {
                total += meta.len();
            }
        }
    }
    total
}

/// Newest first, by numeric components (`v22.10.0` > `v22.9.1`).
pub(crate) fn version_key(v: &str) -> Vec<u64> {
    v.trim_start_matches('v')
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap_or(0))
        .collect()
}

/// `$HOME/...` in rc lines when the path is under home, so the file stays portable.
pub(crate) fn home_relative(probe: &Probe, path: &Path) -> String {
    match path.strip_prefix(&probe.home) {
        // Shell syntax: always `/`, whatever the platform joined with.
        Ok(rest) if !probe.home.as_os_str().is_empty() => {
            format!("$HOME/{}", rest.display().to_string().replace('\\', "/"))
        }
        _ => path.display().to_string(),
    }
}

#[cfg(test)]
pub(crate) mod testutil {
    use std::path::{Path, PathBuf};

    /// A fresh directory under the system temp dir, removed on drop.
    pub struct TempDir(pub PathBuf);

    impl TempDir {
        pub fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "qd-runtimes-{name}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }
        pub fn path(&self) -> &Path {
            &self.0
        }
        pub fn file(&self, rel: &str, content: &str) -> PathBuf {
            let p = self.0.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, content).unwrap();
            p
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Records commands instead of running them; answers with canned output.
    #[derive(Default)]
    pub struct FakeRunner {
        pub output: String,
        pub ran: std::sync::Mutex<Vec<crate::Cmd>>,
    }

    impl crate::Runner for FakeRunner {
        fn run(&self, cmd: &crate::Cmd, on_line: &mut dyn FnMut(&str)) -> crate::Result<String> {
            self.ran.lock().unwrap().push(cmd.clone());
            for l in self.output.lines() {
                on_line(l);
            }
            Ok(self.output.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_versions_that_could_be_shell() {
        for ok in ["22", "v22.1.0", "lts/krypton", "stable", "nightly-2026-01-01", "3.12.4+freethreaded", "go@1.23"] {
            assert!(validate_version(ok).is_ok(), "{ok}");
        }
        for bad in ["", "22; rm -rf ~", "$(id)", "`id`", "-rf", "../../etc", "22 23", "a\nb", "x|y"] {
            assert!(validate_version(bad).is_err(), "{bad:?}");
        }
    }

    fn nvm_system(path: &[&str], node: &str) -> (testutil::TempDir, System) {
        let t = testutil::TempDir::new("system");
        t.file(".nvm/nvm.sh", "");
        t.file(".nvm/versions/node/v22.10.0/bin/node", "#!");
        t.file(".nvm/versions/node/v20.18.0/bin/node", "#!");
        t.file(".nvm/alias/default", "22\n");
        let home = t.path().to_path_buf();
        let node = node.replace("~", &home.display().to_string());
        let mut probe = Probe {
            home: home.clone(),
            shell: "bash".into(),
            path: path.iter().map(|p| p.replace("~", &home.display().to_string())).collect(),
            ..Default::default()
        };
        let version =
            node.split("/node/").nth(1).map(|v| v.split('/').next().unwrap().to_owned()).unwrap_or("v18.19.1".into());
        probe.found.insert("node".into(), probe::Found { path: node, version_line: version });
        probe.found.insert(
            "go".into(),
            probe::Found { path: "/usr/bin/go".into(), version_line: "go version go1.22.2 linux/amd64".into() },
        );
        (t, System::from_probe(probe))
    }

    #[test]
    fn reports_runtimes_with_and_without_a_manager() {
        let (_t, sys) =
            nvm_system(&["~/.nvm/versions/node/v22.10.0/bin", "/usr/bin"], "~/.nvm/versions/node/v22.10.0/bin/node");
        let rts = sys.runtimes();
        let ids: Vec<&str> = rts.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, ["node", "go"]);
        let node = &rts[0];
        assert_eq!(node.manager.as_deref(), Some("nvm"));
        assert_eq!(node.default.as_deref(), Some("v22.10.0"));
        assert!(node.installed[0].is_default && !node.installed[1].is_default);
        assert!(node.active.as_ref().unwrap().managed);
        assert_eq!(node.issue, None);
        assert_eq!(node.project_file.as_deref(), Some(".nvmrc"));
        let go = &rts[1];
        assert_eq!((go.manager.as_deref(), go.active.as_ref().unwrap().version.as_str()), (None, "1.22.2"));
    }

    #[test]
    fn explains_why_the_terminal_runs_another_copy() {
        // nvm is not set up in the shell: the system node runs.
        let (_t, sys) = nvm_system(&["/usr/bin"], "/usr/bin/node");
        let issue = sys.runtimes()[0].issue.clone().unwrap();
        assert_eq!((issue.kind, issue.expected.as_str()), ("not_loaded", "22.10.0"));
        assert_eq!(issue.fix.len(), 2);
        // nvm is on PATH, but a copy earlier in PATH wins.
        let (_t, sys) = nvm_system(&["/opt/node/bin", "~/.nvm/versions/node/v22.10.0/bin"], "/opt/node/bin/node");
        assert_eq!(sys.runtimes()[0].issue.as_ref().unwrap().kind, "shadowed");
        // nvm runs, but not the default (an `nvm use 20` somewhere).
        let (_t, sys) = nvm_system(&["~/.nvm/versions/node/v20.18.0/bin"], "~/.nvm/versions/node/v20.18.0/bin/node");
        let issue = sys.runtimes()[0].issue.clone().unwrap();
        assert_eq!((issue.kind, issue.fix.len()), ("other", 0));
    }

    #[test]
    fn guards_actions_and_writes_project_pins() {
        let (t, sys) = nvm_system(&["/usr/bin"], "/usr/bin/node");
        assert!(matches!(sys.command("node", Action::Uninstall, "v22.10.0"), Err(Error::Invalid(_))), "default");
        assert!(matches!(sys.command("node", Action::Uninstall, "v16.0.0"), Err(Error::Invalid(_))), "not installed");
        assert!(sys.command("node", Action::Uninstall, "v20.18.0").is_ok());
        assert!(matches!(sys.command("node", Action::Install, "22; reboot"), Err(Error::Invalid(_))));
        assert!(matches!(sys.command("go", Action::Install, "1.23"), Err(Error::Unsupported(_))));
        let project = t.path().join("proj");
        std::fs::create_dir(&project).unwrap();
        let file = sys.set_project(&project, "node", "v20.18.0").unwrap();
        assert_eq!(std::fs::read_to_string(file).unwrap(), "v20.18.0\n");
        assert!(sys.set_project(&t.path().join("missing"), "node", "v20.18.0").is_err());
        let run = testutil::FakeRunner {
            output: "       v20.18.0   (LTS: Iron)\n       v24.0.0\n".into(),
            ..Default::default()
        };
        let list = sys.available("node", &run).unwrap();
        assert_eq!(list.iter().map(|a| a.installed).collect::<Vec<_>>(), [false, true]);
    }

    #[test]
    fn offers_mise_for_languages_without_a_manager() {
        let (_t, sys) = nvm_system(&["/usr/bin"], "/usr/bin/node");
        let rts = sys.runtimes();
        let go = rts.iter().find(|r| r.id == "go").unwrap();
        assert_eq!((go.manager.as_deref(), go.install_manager.as_deref()), (None, Some("mise")));
        assert_eq!(rts[0].install_manager, None, "node has nvm");
        let cmd = sys.command("go", Action::InstallManager, "").unwrap();
        assert_eq!(cmd.args.last().map(String::as_str), Some(sys.probe.home.join(".local/bin").to_str().unwrap()));
    }

    #[test]
    fn picks_mise_where_it_is_what_the_terminal_runs() {
        let (t, _) = nvm_system(&[], "");
        t.file(".local/bin/mise", "#!");
        t.file(".local/share/mise/installs/go/1.23.4/bin/go", "#!");
        t.file(".local/share/mise/installs/node/24.1.0/bin/node", "#!");
        t.file(".config/mise/config.toml", "[tools]\ngo = \"1.23\"\n");
        let home = t.path().display().to_string();
        let mut probe = Probe {
            home: t.path().to_path_buf(),
            shell: "bash".into(),
            path: vec![format!("{home}/.local/share/mise/shims")],
            ..Default::default()
        };
        let found = |p: &str, v: &str| probe::Found { path: p.into(), version_line: v.into() };
        probe.found.insert(
            "go".into(),
            found(&format!("{home}/.local/share/mise/shims/go"), "go version go1.23.4 linux/amd64"),
        );
        let sys = System::from_probe(probe.clone());
        let rts = sys.runtimes();
        let go = rts.iter().find(|r| r.id == "go").unwrap();
        assert_eq!(
            (go.manager.as_deref(), go.default.as_deref(), go.issue.clone()),
            (Some("mise"), Some("1.23.4"), None)
        );
        assert_eq!(go.install_manager, None);
        // node: nvm wins while the terminal runs nvm's node or nothing at all…
        assert_eq!(rts.iter().find(|r| r.id == "node").unwrap().manager.as_deref(), Some("nvm"));
        // …and mise wins when the terminal runs mise's node.
        probe.found.insert("node".into(), found(&format!("{home}/.local/share/mise/shims/node"), "v24.1.0"));
        let sys = System::from_probe(probe);
        assert_eq!(sys.runtimes().iter().find(|r| r.id == "node").unwrap().manager.as_deref(), Some("mise"));
        assert!(matches!(sys.command("go", Action::InstallManager, ""), Err(Error::Invalid(_))));
        assert_eq!(sys.command("go", Action::SetDefault, "1.23.4").unwrap().args, ["use", "--global", "go@1.23.4"]);
    }

    #[test]
    fn turns_auto_apply_on_and_off() {
        let (t, sys) = nvm_system(&["/usr/bin"], "/usr/bin/node");
        t.file(".bashrc", "export A=1\n");
        assert!(!sys.auto_apply());
        assert!(sys.set_auto_apply(true, "1").unwrap().is_some(), "backup");
        let script = t.path().join(".config/quickdesk/shell/hook.sh");
        assert!(std::fs::read_to_string(&script).unwrap().contains("__quickdesk_prompt"));
        let rc = std::fs::read_to_string(t.path().join(".bashrc")).unwrap();
        assert!(
            rc.contains(
                "[ -s \"$HOME/.config/quickdesk/shell/hook.sh\" ] && . \"$HOME/.config/quickdesk/shell/hook.sh\""
            ),
            "{rc}"
        );
        assert!(sys.auto_apply());
        sys.set_auto_apply(false, "2").unwrap();
        assert_eq!(std::fs::read_to_string(t.path().join(".bashrc")).unwrap(), "export A=1\n");
        assert!(!script.exists() && !sys.auto_apply());
    }

    #[test]
    fn compares_reported_and_installed_versions_loosely() {
        assert!(runs_version("21.0.4", "21.0.4+7"));
        assert!(runs_version("21.0.4", "temurin-21.0.4+7"));
        assert!(runs_version("1.99.0", "stable"));
        assert!(runs_version("22.23.3", "v22.23.3"));
        assert!(!runs_version("20.18.0", "22.10.0"));
        assert!(!runs_version("3.12.3", "3.13.14"));
    }

    #[test]
    fn offers_patch_updates_within_a_line() {
        let (_t, sys) = nvm_system(&["/usr/bin"], "/usr/bin/node");
        let avail = |v: &str, tag: Option<&str>| Available {
            id: format!("v{v}"),
            version: v.into(),
            tag: tag.map(str::to_owned),
            line: String::new(),
            installed: false,
        };
        let list = [
            avail("24.0.0", Some("latest")),
            avail("22.12.0", Some("LTS: Jod")),
            avail("22.11.0", None),
            avail("22.13.0-rc1", Some("pre-release")),
            avail("20.18.0", None),
        ];
        let ups = sys.updates("node", &list);
        // 22.10.0 → 22.12.0 (not 24, not the rc); 20.18.0 is current.
        assert_eq!(
            ups,
            [Update {
                from: "v22.10.0".into(),
                from_version: "22.10.0".into(),
                to: "v22.12.0".into(),
                to_version: "22.12.0".into()
            }]
        );
        assert_eq!(line_of("node", "22.10.0"), "22");
        assert_eq!(line_of("python", "3.12.4"), "3.12");
        assert_eq!(line_of("go", "1.23.4"), "1.23");
        assert_eq!(line_of("rust", "1.82.0"), "");
        // Default → install then make it the default; others → install only.
        let cmds = sys.upgrade_commands("node", "v22.10.0", "v22.12.0").unwrap();
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[1].args[1].rsplit("&& ").next(), Some("nvm alias default \"$1\""));
        assert_eq!(sys.upgrade_commands("node", "v20.18.0", "v20.19.0").unwrap().len(), 1);
        assert!(sys.upgrade_commands("node", "v18.0.0", "v18.1.0").is_err());
    }

    #[test]
    fn orders_versions_numerically() {
        let mut v = vec!["v22.9.1", "v22.10.0", "v8.0.0"];
        v.sort_by_key(|s| std::cmp::Reverse(version_key(s)));
        assert_eq!(v, ["v22.10.0", "v22.9.1", "v8.0.0"]);
    }
}
