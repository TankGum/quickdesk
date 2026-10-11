//! What a new terminal sees. QuickDesk is started by the desktop, whose PATH
//! is not the one your shell builds from ~/.bashrc, so ask an interactive login
//! shell: its PATH, a few variables, and which binary each command resolves to.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::run::{Cmd, Runner, SystemRunner};

/// Commands whose location and first `--version` line we want.
pub const BINARIES: &[&str] =
    &["node", "python3", "python", "rustc", "go", "java", "ruby", "deno", "bun", "php", "uv", "rustup", "mise"];
/// Variables that move where managers keep things.
const VARS: &[&str] = &[
    "NVM_DIR",
    "RUSTUP_HOME",
    "CARGO_HOME",
    "UV_PYTHON_INSTALL_DIR",
    "UV_PYTHON_BIN_DIR",
    "XDG_DATA_HOME",
    "XDG_BIN_HOME",
    "XDG_CONFIG_HOME",
    "XDG_CACHE_HOME",
    "XDG_STATE_HOME",
    "MISE_DATA_DIR",
    "MISE_CONFIG_DIR",
    "MISE_CACHE_DIR",
    "MISE_STATE_DIR",
    "MISE_GLOBAL_CONFIG_FILE",
    "LOCALAPPDATA",
    "APPDATA",
];

const START: &str = "@@QD-PROBE@@";
const END: &str = "@@QD-PROBE-END@@";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Found {
    pub path: String,
    /// First line `--version` printed (`go version`, `java -version`).
    pub version_line: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Probe {
    pub home: PathBuf,
    /// bash or zsh: which rc file a PATH fix goes into.
    pub shell: String,
    pub path: Vec<String>,
    pub vars: BTreeMap<String, String>,
    pub found: BTreeMap<String, Found>,
}

impl Probe {
    pub fn var(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(String::as_str).filter(|v| !v.is_empty())
    }

    /// `$VAR`, else `default` under the home directory.
    pub fn dir(&self, key: &str, default: &str) -> PathBuf {
        self.var(key).map(PathBuf::from).unwrap_or_else(|| self.home.join(default))
    }

    pub fn on_path(&self, dir: &Path) -> bool {
        self.path.iter().any(|p| same_dir(Path::new(p), dir))
    }

    /// Position of `dir` in PATH (first match).
    pub fn path_index(&self, dir: &Path) -> Option<usize> {
        self.path.iter().position(|p| same_dir(Path::new(p), dir))
    }
}

/// The same directory as PATH spells it: on Windows case and a trailing
/// separator do not matter (`C:\Users\Me\.cargo\bin\` = `c:\users\me\.cargo\bin`).
pub fn same_dir(a: &Path, b: &Path) -> bool {
    if cfg!(windows) {
        let norm = |p: &Path| p.to_string_lossy().trim_end_matches(['\\', '/']).replace('/', "\\").to_lowercase();
        norm(a) == norm(b)
    } else {
        a == b
    }
}

/// `path` lies inside `dir`, the way Windows compares paths when it runs there.
pub fn under(path: &Path, dir: &Path) -> bool {
    if cfg!(windows) {
        let norm = |p: &Path| p.to_string_lossy().replace('/', "\\").to_lowercase();
        let (p, d) = (norm(path), norm(dir));
        let d = d.trim_end_matches('\\');
        p == d || p.starts_with(&format!("{d}\\"))
    } else {
        path.starts_with(dir)
    }
}

/// The script run by the shell. Fixed text: nothing from the user goes in.
fn script() -> String {
    let mut s = format!("printf '%s\\n' '{START}'\nprintf 'PATH\\t%s\\n' \"$PATH\"\n");
    for v in VARS {
        s.push_str(&format!("printf 'VAR\\t{v}\\t%s\\n' \"${v}\"\n"));
    }
    for b in BINARIES {
        let version = match *b {
            "go" => "version",
            "java" => "-version",
            _ => "--version",
        };
        s.push_str(&format!(
            "p=$(command -v {b} 2>/dev/null); case \"$p\" in /*) v=$(\"$p\" {version} 2>&1 </dev/null | head -n 1); printf 'BIN\\t{b}\\t%s\\t%s\\n' \"$p\" \"$v\";; esac\n"
        ));
    }
    s.push_str(&format!("printf '%s\\n' '{END}'\n"));
    s
}

/// Windows: what a new PowerShell sees (its profile loaded, as in a real
/// terminal). Store "app execution aliases" (WindowsApps\python.exe, which
/// only opens the Store) are skipped. Fixed text: nothing from the user goes in.
fn script_powershell() -> String {
    let mut s = String::from(
        "$ErrorActionPreference = 'SilentlyContinue'\n[Console]::OutputEncoding = [Text.Encoding]::UTF8\n",
    );
    // What a terminal opened now gets: QuickDesk's own PATH dates from login.
    s.push_str(
        "$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')\n",
    );
    s.push_str(&format!("Write-Output '{START}'\nWrite-Output (\"PATH`t\" + $env:Path)\n"));
    for v in VARS {
        s.push_str(&format!("Write-Output (\"VAR`t{v}`t\" + [Environment]::GetEnvironmentVariable('{v}'))\n"));
    }
    for b in BINARIES {
        let version = match *b {
            "go" => "version",
            "java" => "-version",
            _ => "--version",
        };
        s.push_str(&format!(
            "$c = Get-Command '{b}' -CommandType Application | Where-Object {{ $_.Source -notlike '*\\WindowsApps\\*' }} | Select-Object -First 1\n\
             if ($c) {{ $l = (& $c.Source {version} 2>&1 | Select-Object -First 1); Write-Output (\"BIN`t{b}`t\" + $c.Source + \"`t\" + $l) }}\n"
        ));
    }
    s.push_str(&format!("Write-Output '{END}'\n"));
    s
}

/// Ask the user's shell: bash or zsh (anything else is probed with bash), or
/// PowerShell on Windows.
pub fn run(home: &Path, shell_env: Option<&str>) -> Probe {
    if cfg!(windows) {
        let runner = SystemRunner::with_timeout(Duration::from_secs(15));
        let cmd = Cmd::new("powershell.exe", &["-NoLogo", "-NonInteractive", "-Command", &script_powershell()]);
        let out = runner.run(&cmd, &mut |_| {}).unwrap_or_default();
        let mut probe = parse_with(&out, ';');
        probe.home = home.to_path_buf();
        probe.shell = "powershell".into();
        return probe;
    }
    let shell = match shell_env.and_then(|s| Path::new(s).file_name()?.to_str().map(str::to_owned)) {
        Some(s) if s == "zsh" => "zsh".to_owned(),
        _ => "bash".to_owned(),
    };
    let program = if shell == "zsh" { shell_env.unwrap_or("zsh").to_owned() } else { "bash".to_owned() };
    let runner = SystemRunner::with_timeout(Duration::from_secs(12));
    let out = runner.run(&Cmd::new(program, &["-lic", &script()]), &mut |_| {}).unwrap_or_default();
    let mut probe = parse(&out);
    probe.home = home.to_path_buf();
    probe.shell = shell;
    probe
}

pub fn parse(out: &str) -> Probe {
    parse_with(out, ':')
}

/// `sep`: how PATH is joined (`:`, or `;` on Windows).
pub fn parse_with(out: &str, sep: char) -> Probe {
    let mut probe = Probe::default();
    let body = out.split_once(START).map(|(_, b)| b).unwrap_or("");
    let body = body.split_once(END).map(|(b, _)| b).unwrap_or(body);
    for line in body.lines() {
        let mut parts = line.splitn(4, '\t');
        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some("PATH"), Some(p), _, _) => {
                probe.path = p.split(sep).filter(|s| !s.is_empty()).map(str::to_owned).collect()
            }
            (Some("VAR"), Some(k), v, _) => {
                probe.vars.insert(k.to_owned(), v.unwrap_or("").to_owned());
            }
            (Some("BIN"), Some(name), Some(path), v) => {
                probe.found.insert(
                    name.to_owned(),
                    Found { path: path.to_owned(), version_line: v.unwrap_or("").trim().to_owned() },
                );
            }
            _ => {}
        }
    }
    probe
}

/// The first version-looking token: `v22.1.0` → `22.1.0`, `go1.23.2` → `1.23.2`,
/// `openjdk version "21.0.4"` → `21.0.4`.
pub fn version_in(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit()
            && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric() || matches!(bytes[i - 1], b'v' | b'o'))
        {
            let mut j = i;
            let mut dots = 0;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || matches!(bytes[j], b'.' | b'-' | b'+')) {
                if bytes[j] == b'.' {
                    dots += 1;
                }
                j += 1;
            }
            if dots >= 1 {
                return Some(line[i..j].trim_end_matches(['.', '-', '+']).to_owned());
            }
            i = j;
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_between_markers_and_ignores_rc_noise() {
        let out = "bash: no job control in this shell\nWelcome!\n@@QD-PROBE@@\nPATH\t/home/u/.nvm/versions/node/v22.1.0/bin:/usr/bin:\nVAR\tNVM_DIR\t/home/u/.nvm\nVAR\tRUSTUP_HOME\t\nBIN\tnode\t/home/u/.nvm/versions/node/v22.1.0/bin/node\tv22.1.0\nBIN\tgo\t/usr/bin/go\tgo version go1.22.2 linux/amd64\n@@QD-PROBE-END@@\nbye\n";
        let p = parse(out);
        assert_eq!(p.path, ["/home/u/.nvm/versions/node/v22.1.0/bin", "/usr/bin"]);
        assert_eq!(p.var("NVM_DIR"), Some("/home/u/.nvm"));
        assert_eq!(p.var("RUSTUP_HOME"), None);
        assert_eq!(p.found["node"].path, "/home/u/.nvm/versions/node/v22.1.0/bin/node");
        assert_eq!(p.found["go"].version_line, "go version go1.22.2 linux/amd64");
        assert!(parse("garbage").found.is_empty());
    }

    #[test]
    fn parses_powershell_output_with_windows_paths() {
        let out = "@@QD-PROBE@@\r\nPATH\tC:\\Users\\Me\\.cargo\\bin;C:\\WINDOWS\\system32;\r\nVAR\tLOCALAPPDATA\tC:\\Users\\Me\\AppData\\Local\r\nBIN\tgo\tC:\\Program Files\\Go\\bin\\go.exe\tgo version go1.23.4 windows/amd64\r\n@@QD-PROBE-END@@\r\n";
        let p = parse_with(out, ';');
        assert_eq!(p.path, ["C:\\Users\\Me\\.cargo\\bin", "C:\\WINDOWS\\system32"]);
        assert_eq!(p.var("LOCALAPPDATA"), Some("C:\\Users\\Me\\AppData\\Local"));
        assert_eq!(p.found["go"].path, "C:\\Program Files\\Go\\bin\\go.exe");
        assert_eq!(version_in(&p.found["go"].version_line).as_deref(), Some("1.23.4"));
        assert!(script_powershell().contains("-notlike '*\\WindowsApps\\*'"));
    }

    #[test]
    fn finds_versions_in_tool_output() {
        assert_eq!(version_in("v22.23.3").as_deref(), Some("22.23.3"));
        assert_eq!(version_in("Python 3.12.3").as_deref(), Some("3.12.3"));
        assert_eq!(version_in("rustc 1.90.0 (1159e78c4 2025-09-14)").as_deref(), Some("1.90.0"));
        assert_eq!(version_in("go version go1.22.2 linux/amd64").as_deref(), Some("1.22.2"));
        assert_eq!(version_in("openjdk version \"21.0.4\" 2024-07-16").as_deref(), Some("21.0.4"));
        assert_eq!(version_in("PHP 8.3.6 (cli) (built: Jul 14 2025)").as_deref(), Some("8.3.6"));
        assert_eq!(version_in("deno 2.0.0 (stable, release, x86_64)").as_deref(), Some("2.0.0"));
        assert_eq!(version_in("1.1.30").as_deref(), Some("1.1.30"));
        assert_eq!(version_in("x86_64 only"), None);
    }

    #[test]
    fn script_only_names_fixed_binaries() {
        let s = script();
        assert!(s.contains("command -v node"));
        assert!(s.contains("java -version") || s.contains("\"$p\" -version"));
    }
}
