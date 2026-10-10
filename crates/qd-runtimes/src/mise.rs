//! Everything else through mise (mise.jdx.dev): Go, Java, Ruby, Deno, Bun,
//! PHP, and Node/Python/Rust when mise is what the terminal uses. One mise
//! serves many languages, so each language gets a `MiseTool` sharing a `Mise`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::run::{Cmd, Runner};
use crate::{dir_size, home_relative, version_key, Action, Available, Installed, Manager, Probe, Result};

/// Languages mise can manage here (mise's tool name = our language id).
pub const TOOLS: &[&str] = &["node", "python", "rust", "go", "java", "ruby", "deno", "bun", "php"];

const ENV_KEYS: &[&str] = &[
    "MISE_DATA_DIR",
    "MISE_CONFIG_DIR",
    "MISE_CACHE_DIR",
    "MISE_STATE_DIR",
    "MISE_GLOBAL_CONFIG_FILE",
    "XDG_DATA_HOME",
    "XDG_CONFIG_HOME",
    "XDG_CACHE_HOME",
    "XDG_STATE_HOME",
];

pub struct Mise {
    bin: PathBuf,
    bin_rc: String,
    data_dir: PathBuf,
    global_config: PathBuf,
    shell: String,
    version: Option<String>,
    env: Vec<(String, String)>,
}

impl Mise {
    pub fn detect(probe: &Probe) -> Option<Arc<Self>> {
        let bin = probe.found.get("mise").map(|f| PathBuf::from(&f.path)).or_else(|| {
            [".local/bin/mise", ".local/share/mise/bin/mise", ".cargo/bin/mise"]
                .into_iter()
                .map(|p| probe.home.join(p))
                .find(|p| p.is_file())
        })?;
        let xdg =
            |key: &str, default: &str| probe.var(key).map(PathBuf::from).unwrap_or_else(|| probe.home.join(default));
        let data_dir = probe
            .var("MISE_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| xdg("XDG_DATA_HOME", ".local/share").join("mise"));
        let config_dir = probe
            .var("MISE_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| xdg("XDG_CONFIG_HOME", ".config").join("mise"));
        let global_config =
            probe.var("MISE_GLOBAL_CONFIG_FILE").map(PathBuf::from).unwrap_or_else(|| config_dir.join("config.toml"));
        let version = probe.found.get("mise").and_then(|f| crate::probe::version_in(&f.version_line));
        let mut env = crate::shell_env(probe, ENV_KEYS);
        // Never wait for a "trust this config?" answer nobody can give.
        env.push(("MISE_YES".into(), "1".into()));
        Some(Arc::new(Mise {
            bin_rc: home_relative(probe, &bin),
            bin,
            data_dir,
            global_config,
            shell: probe.shell.clone(),
            version,
            env,
        }))
    }

    fn cmd(&self, args: &[&str]) -> Cmd {
        let mut cmd = Cmd::new(&self.bin, args);
        cmd.env = self.env.clone();
        cmd
    }

    fn shims(&self) -> PathBuf {
        self.data_dir.join("shims")
    }
}

pub struct MiseTool {
    pub mise: Arc<Mise>,
    pub lang: &'static str,
}

impl MiseTool {
    fn installs(&self) -> PathBuf {
        self.mise.data_dir.join("installs").join(self.lang)
    }

    fn spec(&self, version: &str) -> String {
        format!("{}@{}", self.lang, version)
    }

    /// `go = "1.23"` (or `["1.23", …]`, or `{ version = "1.23" }`) in `[tools]`.
    fn configured(&self) -> Option<String> {
        let text = std::fs::read_to_string(&self.mise.global_config).ok()?;
        let mut in_tools = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_tools = line == "[tools]";
                continue;
            }
            if !in_tools {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else { continue };
            if key.trim().trim_matches('"') != self.lang {
                continue;
            }
            let value = value.trim();
            let value = value.split_once("version").map(|(_, v)| v.trim_start_matches([' ', '='])).unwrap_or(value);
            return value
                .trim_start_matches(['[', '{', ' '])
                .split(['"', '\''])
                .find(|s| !s.trim().is_empty() && !s.contains([',', ']', '}']))
                .map(|s| s.trim().to_owned());
        }
        None
    }
}

impl Manager for MiseTool {
    fn id(&self) -> &'static str {
        "mise"
    }
    fn lang(&self) -> &'static str {
        self.lang
    }
    fn version(&self) -> Option<String> {
        self.mise.version.clone()
    }

    fn installed(&self) -> Vec<Installed> {
        let Ok(entries) = std::fs::read_dir(self.installs()) else { return Vec::new() };
        let mut list: Vec<Installed> = entries
            .flatten()
            // Real directories: mise also links `1`, `1.23` and `latest` to them.
            .filter(|e| e.path().symlink_metadata().is_ok_and(|m| m.is_dir()))
            .filter_map(|e| {
                let id = e.file_name().to_str()?.to_owned();
                if id.starts_with('.') {
                    return None;
                }
                Some(Installed {
                    version: id.clone(),
                    path: e.path().display().to_string(),
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
        let want = self.configured()?;
        if want == "latest" {
            return installed.first().map(|i| i.id.clone());
        }
        if let Some(exact) = installed.iter().find(|i| i.id == want) {
            return Some(exact.id.clone());
        }
        let key = version_key(&want);
        if key.is_empty() {
            return None;
        }
        installed.iter().find(|i| version_key(&i.id).starts_with(&key)).map(|i| i.id.clone())
    }

    fn available(&self, run: &dyn Runner) -> Result<Vec<Available>> {
        let out = run.run(&self.mise.cmd(&["ls-remote", self.lang]), &mut |_| {})?;
        let mut list: Vec<Available> = out
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && crate::validate_version(l).is_ok())
            .map(|v| {
                let lower = v.to_ascii_lowercase();
                let pre = ["rc", "beta", "alpha", "-ea", "dev", "preview", "nightly"].iter().any(|p| lower.contains(p));
                Available {
                    id: v.to_owned(),
                    version: v.to_owned(),
                    tag: pre.then(|| "pre-release".to_owned()),
                    line: String::new(),
                    installed: false,
                }
            })
            .collect();
        list.reverse();
        if let Some(first) = list.iter_mut().find(|a| a.tag.is_none()) {
            first.tag = Some("latest".into());
        }
        Ok(list)
    }

    fn command(&self, action: Action, version: &str) -> Cmd {
        let spec = self.spec(version);
        match action {
            Action::Install => self.mise.cmd(&["install", &spec]),
            Action::Uninstall => self.mise.cmd(&["uninstall", &spec]),
            Action::SetDefault => self.mise.cmd(&["use", "--global", &spec]),
            Action::InstallManager => unreachable!("handled by System::command"),
        }
    }

    fn project_file(&self) -> &'static str {
        "mise.toml"
    }
    fn project_content(&self, version: &str) -> String {
        format!("[tools]\n{} = \"{version}\"\n", self.lang)
    }
    /// mise edits its own file, keeping the other tools already listed there.
    fn project_command(&self, dir: &Path, version: &str) -> Option<Cmd> {
        Some(self.mise.cmd(&["use", "--path", &dir.join("mise.toml").display().to_string(), &self.spec(version)]))
    }

    fn manages(&self, path: &str) -> bool {
        let p = Path::new(path);
        p.starts_with(self.mise.shims()) || p.starts_with(self.mise.data_dir.join("installs"))
    }

    fn loaded(&self, probe: &Probe) -> bool {
        let installs = self.mise.data_dir.join("installs");
        probe.on_path(&self.mise.shims()) || probe.path.iter().any(|p| Path::new(p).starts_with(&installs))
    }

    fn init_lines(&self) -> Vec<String> {
        let shell = if self.mise.shell == "zsh" { "zsh" } else { "bash" };
        vec![format!("eval \"$(\"{}\" activate {shell})\"", self.mise.bin_rc)]
    }
}

/// Where QuickDesk puts mise when it installs it.
pub fn install_dir(probe: &Probe) -> PathBuf {
    probe.home.join(".local/bin")
}

/// Download the latest mise for this CPU from its GitHub releases, check it
/// against the release's SHA256SUMS and put it in `dir`. Fixed script; `dir`
/// goes in as `$1`.
pub fn installer(dir: &Path) -> Cmd {
    const SCRIPT: &str = r#"set -eu
case "$(uname -m)" in
  x86_64) arch=x64 ;;
  aarch64|arm64) arch=arm64 ;;
  *) echo "mise has no build for $(uname -m)" >&2; exit 1 ;;
esac
tag=$(curl -fsSLo /dev/null -w '%{url_effective}' https://github.com/jdx/mise/releases/latest)
v=${tag##*/}
case "$v" in v[0-9]*) ;; *) echo "could not find the latest mise release" >&2; exit 1 ;; esac
name="mise-$v-linux-$arch"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
echo "Downloading $name"
curl -fL --progress-bar -o "$tmp/mise" "https://github.com/jdx/mise/releases/download/$v/$name"
curl -fsSL -o "$tmp/SHASUMS256.txt" "https://github.com/jdx/mise/releases/download/$v/SHASUMS256.txt"
want=$(awk -v n="./$name" '$2 == n { print $1 }' "$tmp/SHASUMS256.txt")
got=$(sha256sum "$tmp/mise" | cut -d ' ' -f 1)
if [ -z "$want" ] || [ "$want" != "$got" ]; then echo "checksum does not match for $name" >&2; exit 1; fi
mkdir -p "$1"
install -m 0755 "$tmp/mise" "$1/mise"
echo "Installed mise $v in $1"
"#;
    Cmd::new("bash", &["-c", SCRIPT, "quickdesk", &dir.display().to_string()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{FakeRunner, TempDir};

    fn tool(t: &TempDir, lang: &'static str) -> MiseTool {
        t.file(".local/bin/mise", "#!");
        let probe = Probe { home: t.path().to_path_buf(), shell: "bash".into(), ..Default::default() };
        MiseTool { mise: Mise::detect(&probe).unwrap(), lang }
    }

    #[cfg(unix)]
    #[test]
    fn lists_real_installs_and_reads_the_global_default() {
        let t = TempDir::new("mise");
        let go = tool(&t, "go");
        t.file(".local/share/mise/installs/go/1.23.4/bin/go", "#!");
        t.file(".local/share/mise/installs/go/1.22.12/bin/go", "#!");
        t.file(".local/share/mise/installs/go/.mise.backend.toml", "");
        std::os::unix::fs::symlink("./1.23.4", t.path().join(".local/share/mise/installs/go/latest")).unwrap();
        let installed = go.installed();
        assert_eq!(installed.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(), ["1.23.4", "1.22.12"]);
        assert_eq!(go.default_id(&installed), None);
        t.file(".config/mise/config.toml", "[settings]\ngo = \"nope\"\n\n[tools]\nnode = \"22\"\ngo = \"1.22\"\n");
        assert_eq!(go.default_id(&installed).as_deref(), Some("1.22.12"));
        t.file(".config/mise/config.toml", "[tools]\ngo = [\"1.23.4\", \"1.22\"]\n");
        assert_eq!(go.default_id(&installed).as_deref(), Some("1.23.4"));
        t.file(".config/mise/config.toml", "[tools]\ngo = { version = \"latest\" }\n");
        assert_eq!(go.default_id(&installed).as_deref(), Some("1.23.4"));
    }

    #[test]
    fn builds_commands_with_tool_specs() {
        let t = TempDir::new("mise-cmd");
        let go = tool(&t, "go");
        let cmd = go.command(Action::SetDefault, "1.23.4");
        assert_eq!(cmd.args, ["use", "--global", "go@1.23.4"]);
        assert!(cmd.env.contains(&("MISE_YES".into(), "1".into())));
        assert_eq!(go.command(Action::Uninstall, "1.22.12").args, ["uninstall", "go@1.22.12"]);
        let p = go.project_command(Path::new("/work/api"), "1.23.4").unwrap();
        assert_eq!(p.args, ["use", "--path", "/work/api/mise.toml", "go@1.23.4"]);
        assert_eq!(go.init_lines(), ["eval \"$(\"$HOME/.local/bin/mise\" activate bash)\""]);
    }

    #[test]
    fn parses_ls_remote_newest_first() {
        let t = TempDir::new("mise-ls");
        let go = tool(&t, "go");
        let run = FakeRunner { output: "1.22.12\n1.23.4\n1.24rc1\nnot a version!\n".into(), ..Default::default() };
        let got: Vec<(String, Option<String>)> =
            go.available(&run).unwrap().into_iter().map(|a| (a.version, a.tag)).collect();
        assert_eq!(
            got,
            [
                ("1.24rc1".into(), Some("pre-release".into())),
                ("1.23.4".into(), Some("latest".into())),
                ("1.22.12".into(), None)
            ]
        );
        assert_eq!(run.ran.lock().unwrap()[0].args, ["ls-remote", "go"]);
    }

    #[test]
    fn knows_shims_and_installs() {
        let t = TempDir::new("mise-path");
        let go = tool(&t, "go");
        let shims = t.path().join(".local/share/mise/shims");
        assert!(go.manages(&shims.join("go").display().to_string()));
        assert!(!go.manages("/usr/bin/go"));
        let probe = Probe { path: vec![shims.display().to_string(), "/usr/bin".into()], ..Default::default() };
        assert!(go.loaded(&probe));
        assert!(!go.loaded(&Probe { path: vec!["/usr/bin".into()], ..Default::default() }));
    }

    #[test]
    fn installer_takes_the_folder_as_an_argument() {
        let cmd = installer(Path::new("/home/u/.local/bin"));
        assert_eq!(cmd.args[0], "-c");
        assert!(cmd.args[1].contains("sha256sum") && !cmd.args[1].contains("/home/u"));
        assert_eq!(cmd.args[2..], ["quickdesk", "/home/u/.local/bin"]);
    }
}
