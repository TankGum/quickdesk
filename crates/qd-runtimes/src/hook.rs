//! "Apply right away in open terminals": a prompt hook QuickDesk keeps in
//! `~/.config/quickdesk/shell/hook.sh`, loaded by one line in QuickDesk's
//! block of the rc file. After every version change QuickDesk rewrites the
//! `changed` file next to it; the hook notices at the next prompt.

use std::path::{Path, PathBuf};

use crate::{home_relative, Probe, Result};

pub const SCRIPT: &str = include_str!("hook.sh");

fn shell_dir(probe: &Probe) -> PathBuf {
    probe.dir("XDG_CONFIG_HOME", ".config").join("quickdesk").join("shell")
}

pub fn script_path(probe: &Probe) -> PathBuf {
    shell_dir(probe).join("hook.sh")
}

/// The rc line that loads the hook.
pub fn line(probe: &Probe) -> String {
    let p = home_relative(probe, &script_path(probe));
    format!("[ -s \"{p}\" ] && . \"{p}\"")
}

/// Tell open terminals (at their next prompt) that versions changed.
pub fn mark_changed(probe: &Probe, stamp: &str) -> Result<()> {
    let dir = shell_dir(probe);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("changed"), stamp)?;
    Ok(())
}

pub(crate) fn install(probe: &Probe) -> Result<()> {
    let path = script_path(probe);
    std::fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))?;
    std::fs::write(path, SCRIPT)?;
    Ok(())
}

pub(crate) fn remove(probe: &Probe) {
    let _ = std::fs::remove_file(script_path(probe));
    let _ = std::fs::remove_file(shell_dir(probe).join("changed"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    /// Run `body` in `shell` after loading the hook (twice), with `nvm`
    /// replaced by a function that logs. `None` when the shell is missing.
    fn in_shell(shell: &str, t: &TempDir, body: &str) -> Option<String> {
        if std::process::Command::new(shell).arg("-c").arg("true").status().is_err() {
            return None;
        }
        let script = t.file("cfg/quickdesk/shell/hook.sh", SCRIPT);
        let log = t.path().join("log");
        let full = format!(
            "nvm() {{ echo \"$*\" >> \"{log}\"; }}\n. \"{s}\"; . \"{s}\"\n{body}",
            log = log.display(),
            s = script.display()
        );
        let out = std::process::Command::new(shell)
            .arg("-c")
            .arg(&full)
            .env("NVM_DIR", t.path().join("nvm"))
            .env("XDG_CONFIG_HOME", t.path().join("cfg"))
            .env("PATH", format!("{}:{}:/usr/bin:/bin", t.path().join("a").display(), t.path().join("b").display()))
            .env_remove("PROMPT_COMMAND")
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        Some(std::fs::read_to_string(log).unwrap_or_default())
    }

    #[cfg(unix)]
    #[test]
    fn nvm_follows_nvmrc_then_the_default_and_registers_once() {
        let t = TempDir::new("hook-nvm");
        t.file("nvm/alias/default", "22\n");
        t.file("proj/.nvmrc", "20\n");
        std::fs::create_dir_all(t.path().join("proj/sub")).unwrap();
        let body = format!(
            "cd \"{p}/sub\"; __quickdesk_prompt; __quickdesk_prompt\ncd /; __quickdesk_prompt\n\
             echo 18 > \"$NVM_DIR/alias/default\"; __quickdesk_prompt\n\
             echo \"hooks: ${{PROMPT_COMMAND-}}\" >> \"{log}\"",
            p = t.path().join("proj").display(),
            log = t.path().join("log").display()
        );
        let log = in_shell("bash", &t, &body).expect("bash");
        assert_eq!(log, "use --silent 20\nuse --silent 22\nuse --silent 18\nhooks: __quickdesk_prompt\n", "{log}");
    }

    #[cfg(unix)]
    #[test]
    fn a_change_makes_the_shell_find_new_commands() {
        use std::os::unix::fs::PermissionsExt;
        let t = TempDir::new("hook-hash");
        let tool = |dir: &str, says: &str| {
            let p = t.file(&format!("{dir}/qdtool"), &format!("#!/bin/sh\necho {says}\n"));
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        };
        tool("b", "old");
        let log = t.path().join("log").display().to_string();
        let changed = t.path().join("cfg/quickdesk/shell/changed").display().to_string();
        // `a` comes first in PATH, but bash remembers `b/qdtool` once used.
        let body = format!(
            "__quickdesk_prompt; qdtool >> \"{log}\"\n\
             printf '#!/bin/sh\\necho new\\n' > \"{a}/qdtool\"; chmod +x \"{a}/qdtool\"\n\
             __quickdesk_prompt; qdtool >> \"{log}\"\n\
             echo 1 > \"{changed}\"; __quickdesk_prompt; qdtool >> \"{log}\"",
            a = t.path().join("a").display(),
        );
        std::fs::create_dir_all(t.path().join("a")).unwrap();
        let out = in_shell("bash", &t, &body).expect("bash");
        assert_eq!(out, "old\nold\nnew\n", "{out}");
    }

    #[test]
    fn paths_follow_xdg_config_home() {
        let mut probe = Probe { home: "/home/u".into(), ..Default::default() };
        assert_eq!(
            line(&probe),
            "[ -s \"$HOME/.config/quickdesk/shell/hook.sh\" ] && . \"$HOME/.config/quickdesk/shell/hook.sh\""
        );
        probe.vars.insert("XDG_CONFIG_HOME".into(), "/data/cfg".into());
        assert_eq!(script_path(&probe), PathBuf::from("/data/cfg/quickdesk/shell/hook.sh"));
    }
}
