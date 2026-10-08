//! Fast-path single-instance IPC over a unix socket.
//!
//! A hotkey runs `quickdesk toggle notes` as a new process. Forwarding argv
//! here happens *before* Tauri/GTK/WebKit initialize, so that process exits in
//! a few milliseconds. `tauri-plugin-single-instance` remains as a fallback
//! (and is the only mechanism on Windows).

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// One forwarded invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Forwarded {
    pub args: Vec<String>,
    /// Wall-clock ms when the client process sent it, for latency diagnostics.
    pub sent_at_ms: u64,
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn encode(f: &Forwarded) -> String {
    // Args are joined with \x1f (unit separator), which cannot appear in our verbs.
    format!("{}\x1e{}\n", f.sent_at_ms, f.args.join("\x1f"))
}

fn decode(line: &str) -> Option<Forwarded> {
    let (ts, args) = line.trim_end_matches('\n').split_once('\x1e')?;
    let args = if args.is_empty() { vec![] } else { args.split('\x1f').map(str::to_owned).collect() };
    Some(Forwarded { args, sent_at_ms: ts.parse().ok()? })
}

pub fn socket_path() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let suffix = if cfg!(debug_assertions) { "-dev" } else { "" };
    dir.join(format!("quickdesk{suffix}.sock"))
}

#[cfg(unix)]
mod imp {
    use super::*;
    use std::os::unix::net::{UnixListener, UnixStream};

    /// Returns true if a running instance accepted the args (caller should exit).
    pub fn try_forward(args: &[String]) -> bool {
        let Ok(mut stream) = UnixStream::connect(socket_path()) else { return false };
        let msg = Forwarded { args: args.to_vec(), sent_at_ms: now_ms() };
        stream.write_all(encode(&msg).as_bytes()).is_ok()
    }

    /// Bind the socket and handle each message on a background thread.
    pub fn serve(on_message: impl Fn(Forwarded) + Send + 'static) -> std::io::Result<()> {
        let path = socket_path();
        // Only reached when connect() failed, so any existing file is stale.
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path)?;
        std::thread::Builder::new().name("qd-ipc".into()).spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut line = String::new();
                if BufReader::new(stream).read_line(&mut line).is_ok() {
                    match decode(&line) {
                        Some(msg) => on_message(msg),
                        None => tracing::warn!(?line, "malformed ipc message"),
                    }
                }
            }
        })?;
        Ok(())
    }

    pub fn cleanup() {
        let _ = std::fs::remove_file(socket_path());
    }
}

#[cfg(not(unix))]
mod imp {
    use super::*;

    pub fn try_forward(_args: &[String]) -> bool {
        false
    }
    pub fn serve(_on_message: impl Fn(Forwarded) + Send + 'static) -> std::io::Result<()> {
        Ok(())
    }
    pub fn cleanup() {}
}

pub use imp::{cleanup, serve, try_forward};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        for args in [vec![], vec!["toggle".to_owned(), "notes".to_owned()]] {
            let f = Forwarded { args, sent_at_ms: 42 };
            assert_eq!(decode(&encode(&f)), Some(f));
        }
        assert_eq!(decode("garbage\n"), None);
    }
}
