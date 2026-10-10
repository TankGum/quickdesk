//! Which process is listening on which TCP port, and killing it.

#[cfg(unix)]
pub mod docker;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(target_os = "linux"))]
mod other;

use std::collections::BTreeMap;
use std::net::IpAddr;

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("permission denied: process {0} belongs to another user")]
    PermissionDenied(u32),
    #[error("process {0} not found")]
    NoSuchProcess(u32),
    #[error("refusing to kill process {0}")]
    Protected(u32),
    #[error("{0}")]
    Platform(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// One listening TCP port owned by one process (all its bound addresses merged).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortEntry {
    pub port: u16,
    /// Bound addresses, e.g. `["0.0.0.0", "::"]`.
    pub addrs: Vec<String>,
    /// `None` when the owning process is not visible to us (another user's).
    pub pid: Option<u32>,
    pub process: Option<String>,
    pub cmdline: Option<String>,
    /// Owner of the socket; known even when the pid is hidden (Linux).
    pub user: Option<String>,
    /// Set when the port is published by a Docker container.
    #[cfg(unix)]
    pub container: Option<docker::Container>,
    #[cfg(not(unix))]
    pub container: Option<()>,
}

/// A raw listening socket before grouping.
#[derive(Debug, Clone)]
pub(crate) struct Socket {
    pub addr: IpAddr,
    pub port: u16,
    pub pid: Option<u32>,
    pub process: Option<String>,
    pub cmdline: Option<String>,
    pub user: Option<String>,
}

/// Merge sockets of the same (port, owner) and sort by port.
pub(crate) fn group(sockets: Vec<Socket>) -> Vec<PortEntry> {
    let mut map: BTreeMap<(u16, Option<u32>, Option<String>), PortEntry> = BTreeMap::new();
    for s in sockets {
        let key = (s.port, s.pid, if s.pid.is_none() { s.user.clone() } else { None });
        let entry = map.entry(key).or_insert_with(|| PortEntry {
            port: s.port,
            addrs: vec![],
            pid: s.pid,
            process: s.process.clone(),
            cmdline: s.cmdline.clone(),
            user: s.user.clone(),
            container: None,
        });
        let addr = s.addr.to_string();
        if !entry.addrs.contains(&addr) {
            entry.addrs.push(addr);
        }
    }
    map.into_values().collect()
}

/// All listening TCP ports.
pub fn scan() -> Result<Vec<PortEntry>> {
    #[cfg(target_os = "linux")]
    let sockets = linux::listening_sockets()?;
    #[cfg(not(target_os = "linux"))]
    let sockets = other::listening_sockets()?;
    #[allow(unused_mut)]
    let mut entries = group(sockets);
    #[cfg(unix)]
    {
        let containers = docker::containers_by_port();
        for e in &mut entries {
            e.container = containers.get(&e.port).cloned();
        }
    }
    Ok(entries)
}

fn check_killable(pid: u32) -> Result<()> {
    if pid <= 1 || pid == std::process::id() {
        return Err(Error::Protected(pid));
    }
    Ok(())
}

/// Ask the process to stop (SIGTERM), or force it (SIGKILL / TerminateProcess).
pub fn kill(pid: u32, force: bool) -> Result<()> {
    check_killable(pid)?;
    imp::kill(pid, force)
}

pub fn is_alive(pid: u32) -> bool {
    imp::is_alive(pid)
}

#[cfg(unix)]
mod imp {
    use super::*;

    fn errno_error(pid: u32) -> Error {
        let err = std::io::Error::last_os_error();
        match err.raw_os_error() {
            Some(libc::EPERM) => Error::PermissionDenied(pid),
            Some(libc::ESRCH) => Error::NoSuchProcess(pid),
            _ => Error::Io(err),
        }
    }

    pub fn kill(pid: u32, force: bool) -> Result<()> {
        let sig = if force { libc::SIGKILL } else { libc::SIGTERM };
        // SAFETY: kill(2) has no memory-safety preconditions.
        if unsafe { libc::kill(pid as libc::pid_t, sig) } == 0 {
            Ok(())
        } else {
            Err(errno_error(pid))
        }
    }

    pub fn is_alive(pid: u32) -> bool {
        // Signal 0 checks existence; EPERM means it exists but is not ours.
        // SAFETY: as above.
        let exists = unsafe { libc::kill(pid as libc::pid_t, 0) } == 0;
        exists || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    }
}

#[cfg(windows)]
mod imp {
    use super::*;
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    /// Console tools started from a GUI app would flash a window otherwise.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    pub fn kill(pid: u32, force: bool) -> Result<()> {
        let mut cmd = Command::new("taskkill");
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.args(["/PID", &pid.to_string()]);
        if force {
            cmd.arg("/F");
        }
        let out = cmd.output()?;
        if out.status.success() {
            Ok(())
        } else {
            Err(Error::Platform(String::from_utf8_lossy(&out.stderr).trim().to_owned()))
        }
    }

    pub fn is_alive(pid: u32) -> bool {
        Command::new("tasklist")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr, TcpListener};

    fn sock(port: u16, addr: IpAddr, pid: Option<u32>, user: &str) -> Socket {
        Socket { addr, port, pid, process: pid.map(|_| "node".into()), cmdline: None, user: Some(user.into()) }
    }

    #[test]
    fn groups_dual_stack_and_keeps_owners_apart() {
        let v4 = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
        let v6 = IpAddr::V6(Ipv6Addr::UNSPECIFIED);
        let entries = group(vec![
            sock(8000, v4, Some(10), "me"),
            sock(8000, v6, Some(10), "me"),
            sock(3000, v4, Some(11), "me"),
            sock(5432, v4, None, "postgres"),
            sock(5432, v6, None, "postgres"),
        ]);
        let summary: Vec<(u16, Vec<String>, Option<u32>)> =
            entries.into_iter().map(|e| (e.port, e.addrs, e.pid)).collect();
        assert_eq!(
            summary,
            vec![
                (3000, vec!["0.0.0.0".into()], Some(11)),
                (5432, vec!["0.0.0.0".into(), "::".into()], None),
                (8000, vec!["0.0.0.0".into(), "::".into()], Some(10)),
            ]
        );
    }

    #[test]
    fn scan_finds_our_own_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let entries = scan().unwrap();
        let ours = entries.iter().find(|e| e.port == port).expect("own port listed");
        assert_eq!(ours.pid, Some(std::process::id()));
        assert_eq!(ours.addrs, vec!["127.0.0.1"]);
        assert!(ours.process.is_some());
    }

    #[test]
    fn refuses_to_kill_init_or_self() {
        assert!(matches!(kill(1, false), Err(Error::Protected(1))));
        assert!(matches!(kill(std::process::id(), true), Err(Error::Protected(_))));
    }

    #[cfg(unix)]
    #[test]
    fn kill_terminates_a_child_process() {
        let mut child = std::process::Command::new("sleep").arg("30").spawn().unwrap();
        let pid = child.id();
        assert!(is_alive(pid));
        kill(pid, false).unwrap();
        let status = child.wait().unwrap();
        assert!(!status.success());
        assert!(matches!(kill(pid, false), Err(Error::NoSuchProcess(_))));
    }
}
