//! Running a manager's CLI: one command, its output streamed line by line, and
//! a way to stop it (the whole process group, so downloads stop too).

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::time::{Duration, Instant};

use crate::{Error, Result};

/// A command to run: never a shell string built from user input. When a shell
/// is unavoidable (nvm is a shell function), the script is fixed and the
/// version goes in as a positional argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cmd {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

impl Cmd {
    pub fn new(program: impl Into<PathBuf>, args: &[&str]) -> Self {
        Cmd { program: program.into(), args: args.iter().map(|s| (*s).to_owned()).collect(), env: Vec::new() }
    }

    pub fn env(mut self, key: &str, value: impl Into<String>) -> Self {
        self.env.push((key.to_owned(), value.into()));
        self
    }
}

pub trait Runner: Send + Sync {
    /// Run to completion, calling `on_line` for every line (or `\r`-separated
    /// progress update) of stdout and stderr. Returns everything printed.
    fn run(&self, cmd: &Cmd, on_line: &mut dyn FnMut(&str)) -> Result<String>;
}

/// Runs real processes. `cancel()` stops the one currently running.
#[derive(Default)]
pub struct SystemRunner {
    pgid: AtomicI32,
    cancelled: AtomicBool,
    /// `None`: no limit.
    pub timeout: Option<Duration>,
}

impl SystemRunner {
    pub fn with_timeout(timeout: Duration) -> Self {
        SystemRunner { timeout: Some(timeout), ..Default::default() }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        let pgid = self.pgid.load(Ordering::SeqCst);
        if pgid > 0 {
            kill_group(pgid, false);
        }
    }
}

/// Ask the group to stop (`hard`: make it). SIGCONT too: a stopped process
/// only acts on SIGTERM once it runs again.
#[cfg(unix)]
fn kill_group(pgid: i32, hard: bool) {
    // SAFETY: plain syscalls; a negative pid addresses the process group.
    unsafe {
        libc::kill(-pgid, if hard { libc::SIGKILL } else { libc::SIGTERM });
        libc::kill(-pgid, libc::SIGCONT);
    }
}

#[cfg(not(unix))]
fn kill_group(_pgid: i32, _hard: bool) {}

/// After a polite stop, how long before the group is killed outright.
const KILL_GRACE: Duration = Duration::from_secs(5);

impl Runner for SystemRunner {
    fn run(&self, cmd: &Cmd, on_line: &mut dyn FnMut(&str)) -> Result<String> {
        self.cancelled.store(false, Ordering::SeqCst);
        let mut command = Command::new(&cmd.program);
        command.args(&cmd.args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        for (k, v) in &cmd.env {
            command.env(k, v);
        }
        // Tools print progress bars only for terminals; ask for plain output.
        command.env("NO_COLOR", "1").env("TERM", "dumb");
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            // A session of its own: its own process group (to stop downloads
            // with it), and no controlling terminal. Started from a terminal
            // (`npm run tauri dev`), an interactive `bash -i` would otherwise
            // try to take that terminal over and be stopped by SIGTTOU.
            // SAFETY: setsid is async-signal-safe and touches nothing else.
            unsafe {
                command.pre_exec(|| {
                    if libc::setsid() == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
        }
        let mut child = command.spawn().map_err(|e| Error::Io(format!("{}: {e}", cmd.program.display())))?;
        self.pgid.store(child.id() as i32, Ordering::SeqCst);
        if self.cancelled.load(Ordering::SeqCst) {
            kill_group(child.id() as i32, false);
        }

        // Both pipes feed one channel so lines keep their order roughly.
        let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
        let mut readers = Vec::new();
        for pipe in [
            child.stdout.take().map(|p| Box::new(p) as Box<dyn Read + Send>),
            child.stderr.take().map(|p| Box::new(p) as Box<dyn Read + Send>),
        ]
        .into_iter()
        .flatten()
        {
            let tx = tx.clone();
            readers.push(std::thread::spawn(move || {
                let mut pipe = pipe;
                let mut buf = [0u8; 4096];
                while let Ok(n) = pipe.read(&mut buf) {
                    if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
            }));
        }
        drop(tx);

        let started = Instant::now();
        let mut all = String::new();
        let mut pending = String::new();
        let mut timed_out = false;
        let mut cancel_seen: Option<Instant> = None;
        loop {
            match rx.recv_timeout(Duration::from_millis(200)) {
                Ok(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);
                    all.push_str(&text);
                    pending.push_str(&text);
                    while let Some(i) = pending.find(['\n', '\r']) {
                        let line: String = pending.drain(..=i).collect();
                        let line = line.trim_end_matches(['\n', '\r']);
                        if !line.trim().is_empty() {
                            on_line(line);
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if self.timeout.is_some_and(|t| started.elapsed() > t) {
                timed_out = true;
                kill_group(child.id() as i32, true);
                break;
            }
            if self.cancelled.load(Ordering::SeqCst) {
                let seen = *cancel_seen.get_or_insert_with(Instant::now);
                if seen.elapsed() > KILL_GRACE {
                    kill_group(child.id() as i32, true);
                    break;
                }
            }
        }
        if !pending.trim().is_empty() {
            on_line(pending.trim_end());
        }
        let status = child.wait().map_err(|e| Error::Io(e.to_string()))?;
        self.pgid.store(0, Ordering::SeqCst);
        for r in readers {
            let _ = r.join();
        }
        if self.cancelled.swap(false, Ordering::SeqCst) {
            return Err(Error::Cancelled);
        }
        if timed_out {
            return Err(Error::Failed(format!("{} took too long", cmd.program.display())));
        }
        if !status.success() {
            return Err(Error::Failed(last_lines(&all, 6)));
        }
        Ok(all)
    }
}

/// The end of a command's output, for error messages.
pub fn last_lines(output: &str, n: usize) -> String {
    let lines: Vec<&str> = output.split(['\n', '\r']).map(str::trim_end).filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

/// A percentage in a progress line (`######## 45.3%`, `( 18 %)`), if any.
pub fn percent(line: &str) -> Option<f32> {
    let b = line.as_bytes();
    let pos = line.rfind('%')?;
    let mut end = pos;
    while end > 0 && b[end - 1] == b' ' {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && (b[start - 1].is_ascii_digit() || b[start - 1] == b'.') {
        start -= 1;
    }
    let p: f32 = line[start..end].parse().ok()?;
    (0.0..=100.0).contains(&p).then_some(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_percentages_from_progress_lines() {
        assert_eq!(percent("######################                   45.3%"), Some(45.3));
        assert_eq!(percent("  12.3 MiB /  70.1 MiB ( 18 %)"), Some(18.0));
        assert_eq!(percent("Downloading https://nodejs.org/dist/v22"), None);
        assert_eq!(percent("100%"), Some(100.0));
        assert_eq!(percent("250%"), None);
    }

    #[cfg(unix)]
    #[test]
    fn streams_lines_and_reports_failures() {
        let r = SystemRunner::default();
        let mut lines = Vec::new();
        let out = r
            .run(&Cmd::new("/bin/sh", &["-c", "printf 'a\\rb\\n'; echo c >&2"]), &mut |l| lines.push(l.to_owned()))
            .unwrap();
        assert!(out.contains('c'));
        lines.sort();
        assert_eq!(lines, ["a", "b", "c"]);
        let err = r.run(&Cmd::new("/bin/sh", &["-c", "echo boom; exit 3"]), &mut |_| {}).unwrap_err();
        assert!(matches!(err, Error::Failed(ref m) if m == "boom"), "{err}");
    }

    #[cfg(unix)]
    #[test]
    fn a_stopped_process_still_times_out() {
        // What an interactive shell does when it cannot own the terminal.
        let r = SystemRunner::with_timeout(Duration::from_secs(1));
        let started = Instant::now();
        let err = r.run(&Cmd::new("/bin/sh", &["-c", "kill -STOP $$; sleep 30"]), &mut |_| {}).unwrap_err();
        assert!(matches!(err, Error::Failed(_)), "{err}");
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[cfg(unix)]
    #[test]
    fn runs_without_a_controlling_terminal() {
        let out = SystemRunner::default().run(&Cmd::new("/bin/sh", &["-c", "tty || true"]), &mut |_| {}).unwrap();
        assert!(out.contains("not a tty"), "{out}");
    }

    #[cfg(unix)]
    #[test]
    fn cancel_stops_the_whole_group() {
        let r = std::sync::Arc::new(SystemRunner::default());
        let r2 = r.clone();
        let t = std::thread::spawn(move || r2.run(&Cmd::new("/bin/sh", &["-c", "sleep 30 & wait"]), &mut |_| {}));
        std::thread::sleep(Duration::from_millis(300));
        let started = Instant::now();
        r.cancel();
        assert!(matches!(t.join().unwrap(), Err(Error::Cancelled)));
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}
