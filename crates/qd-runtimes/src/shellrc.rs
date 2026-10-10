//! The one place QuickDesk edits a shell file: a marked block at the end of
//! ~/.bashrc or ~/.zshrc, written only after the user agreed, with a backup
//! next to it. Undo removes the block and nothing else.

use std::path::{Path, PathBuf};

use crate::{Error, Result};

const BEGIN: &str = "# >>> quickdesk >>>";
const END: &str = "# <<< quickdesk <<<";
const NOTE: &str =
    "# Added by QuickDesk (Runtimes) so new terminals run your default versions. Remove this block to undo.";
/// Every wording of the note starts like this (the tab was first called Versions).
const NOTE_START: &str = "# Added by QuickDesk";

/// The rc file of `shell` (`bash` or `zsh`).
pub fn rc_file(home: &Path, shell: &str) -> PathBuf {
    home.join(if shell == "zsh" { ".zshrc" } else { ".bashrc" })
}

/// Lines currently inside QuickDesk's block.
pub fn block_lines(content: &str) -> Vec<String> {
    let Some((_, rest)) = content.split_once(BEGIN) else { return Vec::new() };
    let body = rest.split_once(END).map(|(b, _)| b).unwrap_or(rest);
    body.lines().map(str::trim_end).filter(|l| !l.is_empty() && !l.starts_with(NOTE_START)).map(str::to_owned).collect()
}

/// `content` without QuickDesk's block (and the blank line before it).
pub fn without_block(content: &str) -> String {
    let Some(start) = content.find(BEGIN) else { return content.to_owned() };
    let end = content[start..].find(END).map(|i| start + i + END.len()).unwrap_or(content.len());
    let mut before = content[..start].to_owned();
    let mut after = &content[end..];
    after = after.strip_prefix('\n').unwrap_or(after);
    while before.ends_with("\n\n") {
        before.pop();
    }
    before.push_str(after);
    before
}

/// `content` with the block holding its current lines plus `add` (no duplicates).
/// The block always goes last, so what it puts on PATH comes first.
pub fn with_lines(content: &str, add: &[String]) -> String {
    let mut lines = block_lines(content);
    for l in add {
        if !lines.contains(l) {
            lines.push(l.clone());
        }
    }
    let mut out = without_block(content);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out.push_str(BEGIN);
    out.push('\n');
    out.push_str(NOTE);
    out.push('\n');
    for l in &lines {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str(END);
    out.push('\n');
    out
}

fn check_lines(lines: &[String]) -> Result<()> {
    if lines.iter().any(|l| l.contains('\n') || l.contains(BEGIN) || l.contains(END)) {
        return Err(Error::Invalid("bad shell line".into()));
    }
    Ok(())
}

fn backup(path: &Path, stamp: &str) -> Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }
    let name = format!("{}.quickdesk-bak-{stamp}", path.file_name().and_then(|n| n.to_str()).unwrap_or("rc"));
    let to = path.with_file_name(name);
    std::fs::copy(path, &to)?;
    Ok(Some(to))
}

/// Add `lines` to the block in `path`. Returns the backup, if the file existed.
/// `stamp` names the backup (`20261009-143000`).
pub fn apply(path: &Path, lines: &[String], stamp: &str) -> Result<Option<PathBuf>> {
    check_lines(lines)?;
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let next = with_lines(&content, lines);
    if next == content {
        return Ok(None);
    }
    let saved = backup(path, stamp)?;
    write_atomically(path, &next)?;
    Ok(saved)
}

/// Take `lines` out of the block (the block goes when it is left empty).
/// `false` when none of them was there.
pub fn remove_lines(path: &Path, lines: &[String], stamp: &str) -> Result<bool> {
    let Ok(content) = std::fs::read_to_string(path) else { return Ok(false) };
    let current = block_lines(&content);
    let keep: Vec<String> = current.iter().filter(|l| !lines.contains(l)).cloned().collect();
    if keep.len() == current.len() {
        return Ok(false);
    }
    let next = if keep.is_empty() { without_block(&content) } else { with_lines(&without_block(&content), &keep) };
    backup(path, stamp)?;
    write_atomically(path, &next)?;
    Ok(true)
}

/// Remove the block. `false` when there was none.
pub fn undo(path: &Path, stamp: &str) -> Result<bool> {
    let Ok(content) = std::fs::read_to_string(path) else { return Ok(false) };
    if !content.contains(BEGIN) {
        return Ok(false);
    }
    backup(path, stamp)?;
    write_atomically(path, &without_block(&content))?;
    Ok(true)
}

/// Same permissions as before; a crash leaves either the old or the new file.
fn write_atomically(path: &Path, content: &str) -> Result<()> {
    let tmp = path.with_extension("quickdesk-tmp");
    std::fs::write(&tmp, content)?;
    if let Ok(meta) = std::fs::metadata(path) {
        let _ = std::fs::set_permissions(&tmp, meta.permissions());
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    fn lines(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn appends_one_block_and_merges_on_repeat() {
        let rc = "# my bashrc\nalias ll='ls -l'"; // no trailing newline
        let once = with_lines(rc, &lines(&[". \"$HOME/.cargo/env\""]));
        assert!(once.starts_with("# my bashrc\nalias ll='ls -l'\n\n# >>> quickdesk >>>\n"), "{once}");
        assert!(once.ends_with(". \"$HOME/.cargo/env\"\n# <<< quickdesk <<<\n"));
        let twice = with_lines(&once, &lines(&[". \"$HOME/.cargo/env\"", "export PATH=\"$HOME/.local/bin:$PATH\""]));
        assert_eq!(twice.matches(BEGIN).count(), 1);
        assert_eq!(block_lines(&twice), lines(&[". \"$HOME/.cargo/env\"", "export PATH=\"$HOME/.local/bin:$PATH\""]));
        assert_eq!(with_lines(&twice, &lines(&["export PATH=\"$HOME/.local/bin:$PATH\""])), twice, "idempotent");
    }

    #[test]
    fn moves_the_block_to_the_end_and_undo_restores_the_rest() {
        let rc = "a\n\n# >>> quickdesk >>>\nold line\n# <<< quickdesk <<<\nexport PATH=/opt/x:$PATH\n";
        let next = with_lines(rc, &lines(&["new line"]));
        assert!(next.starts_with("a\nexport PATH=/opt/x:$PATH\n\n# >>> quickdesk >>>"), "{next}");
        assert_eq!(block_lines(&next), lines(&["old line", "new line"]));
        assert_eq!(without_block(&next), "a\nexport PATH=/opt/x:$PATH\n");
        assert_eq!(without_block("no block\n"), "no block\n");
    }

    #[test]
    fn writes_with_a_backup_and_undoes() {
        let t = TempDir::new("rc");
        let rc = t.file(".bashrc", "export A=1\n");
        let backup = apply(&rc, &lines(&["export B=2"]), "20261009-120000").unwrap().unwrap();
        assert_eq!(std::fs::read_to_string(&backup).unwrap(), "export A=1\n");
        assert!(backup.ends_with(".bashrc.quickdesk-bak-20261009-120000"));
        assert!(std::fs::read_to_string(&rc).unwrap().contains("export B=2"));
        assert_eq!(apply(&rc, &lines(&["export B=2"]), "x").unwrap(), None, "nothing to change, no backup");
        assert!(undo(&rc, "20261009-120001").unwrap());
        assert_eq!(std::fs::read_to_string(&rc).unwrap(), "export A=1\n");
        assert!(!undo(&rc, "y").unwrap());
        // The note written by earlier builds is still recognised as the note.
        let old = "# >>> quickdesk >>>\n# Added by QuickDesk (Versions) so new terminals run your default versions.\nx\n# <<< quickdesk <<<\n";
        assert_eq!(block_lines(old), lines(&["x"]));
        apply(&rc, &lines(&["export B=2", "export C=3"]), "r1").unwrap();
        assert!(remove_lines(&rc, &lines(&["export B=2"]), "r2").unwrap());
        assert_eq!(block_lines(&std::fs::read_to_string(&rc).unwrap()), lines(&["export C=3"]));
        assert!(!remove_lines(&rc, &lines(&["export B=2"]), "r3").unwrap());
        assert!(remove_lines(&rc, &lines(&["export C=3"]), "r4").unwrap());
        assert_eq!(std::fs::read_to_string(&rc).unwrap(), "export A=1\n", "empty block goes");
        assert!(apply(&rc, &lines(&["evil\n# <<< quickdesk <<<"]), "z").is_err());
        // A missing rc file is created, without a backup.
        let zsh = t.path().join(".zshrc");
        assert_eq!(apply(&zsh, &lines(&["export C=3"]), "z").unwrap(), None);
        assert!(std::fs::read_to_string(&zsh).unwrap().starts_with("# >>> quickdesk >>>"));
    }
}
