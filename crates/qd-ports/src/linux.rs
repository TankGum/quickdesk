//! Reads `/proc/net/tcp{,6}` directly: every listening socket is visible there
//! (with its owner uid), even when `/proc/<pid>/fd` of another user's process
//! is not readable to map it to a pid.

use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::{Result, Socket};

const TCP_LISTEN: &str = "0A";

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RawSocket {
    pub addr: IpAddr,
    pub port: u16,
    pub uid: u32,
    pub inode: u64,
}

/// Parse a `/proc/net/tcp` or `/proc/net/tcp6` table, keeping LISTEN rows.
pub(crate) fn parse_table(content: &str) -> Vec<RawSocket> {
    content
        .lines()
        .skip(1)
        .filter_map(|line| {
            let cols: Vec<&str> = line.split_whitespace().collect();
            // sl local rem st queues tr retrnsmt uid timeout inode ...
            if cols.len() < 10 || cols[3] != TCP_LISTEN {
                return None;
            }
            let (addr_hex, port_hex) = cols[1].split_once(':')?;
            Some(RawSocket {
                addr: parse_addr(addr_hex)?,
                port: u16::from_str_radix(port_hex, 16).ok()?,
                uid: cols[7].parse().ok()?,
                inode: cols[9].parse().ok()?,
            })
        })
        .collect()
}

/// The kernel prints each 32-bit word of the address in host byte order.
fn parse_addr(hex: &str) -> Option<IpAddr> {
    let words: Option<Vec<[u8; 4]>> = (0..hex.len() / 8)
        .map(|i| u32::from_str_radix(hex.get(i * 8..i * 8 + 8)?, 16).ok().map(u32::to_ne_bytes))
        .collect();
    let bytes: Vec<u8> = words?.concat();
    match bytes.len() {
        4 => Some(IpAddr::V4(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]))),
        16 => {
            let b: [u8; 16] = bytes.try_into().ok()?;
            let v6 = Ipv6Addr::from(b);
            // Show v4-mapped addresses (::ffff:1.2.3.4) as plain IPv4.
            Some(v6.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(IpAddr::V6(v6)))
        }
        _ => None,
    }
}

/// `socket inode → pid` for every process whose fds we can read.
fn socket_owners() -> HashMap<u64, u32> {
    let mut owners = HashMap::new();
    let Ok(procs) = fs::read_dir("/proc") else { return owners };
    for entry in procs.flatten() {
        let Some(pid) = entry.file_name().to_str().and_then(|s| s.parse::<u32>().ok()) else { continue };
        let Ok(fds) = fs::read_dir(entry.path().join("fd")) else { continue };
        for fd in fds.flatten() {
            if let Ok(target) = fs::read_link(fd.path()) {
                if let Some(inode) = target
                    .to_str()
                    .and_then(|t| t.strip_prefix("socket:["))
                    .and_then(|t| t.strip_suffix(']'))
                    .and_then(|t| t.parse().ok())
                {
                    owners.insert(inode, pid);
                }
            }
        }
    }
    owners
}

fn users() -> HashMap<u32, String> {
    fs::read_to_string("/etc/passwd")
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let mut f = l.split(':');
            let name = f.next()?;
            let uid = f.nth(1)?.parse().ok()?;
            Some((uid, name.to_owned()))
        })
        .collect()
}

fn process_info(pid: u32) -> (Option<String>, Option<String>) {
    let comm = fs::read_to_string(format!("/proc/{pid}/comm")).ok().map(|s| s.trim().to_owned());
    let cmdline = fs::read(format!("/proc/{pid}/cmdline")).ok().and_then(|raw| {
        let parts: Vec<String> =
            raw.split(|b| *b == 0).filter(|p| !p.is_empty()).map(|p| String::from_utf8_lossy(p).into_owned()).collect();
        (!parts.is_empty()).then(|| parts.join(" "))
    });
    (comm, cmdline)
}

pub(crate) fn listening_sockets() -> Result<Vec<Socket>> {
    let mut raw = parse_table(&fs::read_to_string("/proc/net/tcp")?);
    // tcp6 is absent when IPv6 is disabled.
    raw.extend(parse_table(&fs::read_to_string("/proc/net/tcp6").unwrap_or_default()));

    let owners = socket_owners();
    let users = users();
    let mut info_cache: HashMap<u32, (Option<String>, Option<String>)> = HashMap::new();
    Ok(raw
        .into_iter()
        .map(|r| {
            let pid = owners.get(&r.inode).copied();
            let (process, cmdline) =
                pid.map(|p| info_cache.entry(p).or_insert_with(|| process_info(p)).clone()).unwrap_or_default();
            Socket {
                addr: r.addr,
                port: r.port,
                pid,
                process,
                cmdline,
                user: Some(users.get(&r.uid).cloned().unwrap_or_else(|| r.uid.to_string())),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TCP: &str = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:1F40 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 12345 1 0000000000000000 100 0 0 10 0
   1: 00000000:1538 00000000:0000 0A 00000000:00000000 00:00000000 00000000   114        0 23456 1 0000000000000000 100 0 0 10 0
   2: 0100007F:1F40 0100007F:D2A4 01 00000000:00000000 00:00000000 00000000  1000        0 34567 1 0000000000000000 20 4 30 10 -1
";

    const TCP6: &str = "  sl  local_address                         remote_address                        st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 00000000000000000000000000000000:1F40 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 45678 1 0000000000000000 100 0 0 10 0
   1: 00000000000000000000000001000000:0BB8 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 56789 1 0000000000000000 100 0 0 10 0
   2: 0000000000000000FFFF00000100007F:1A0B 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 67890 1 0000000000000000 100 0 0 10 0
";

    #[test]
    fn parses_ipv4_listeners_only() {
        let rows = parse_table(TCP);
        assert_eq!(
            rows,
            vec![
                RawSocket { addr: "127.0.0.1".parse().unwrap(), port: 8000, uid: 1000, inode: 12345 },
                RawSocket { addr: "0.0.0.0".parse().unwrap(), port: 5432, uid: 114, inode: 23456 },
            ]
        );
    }

    #[test]
    fn parses_ipv6_including_loopback_and_v4_mapped() {
        let addrs: Vec<(String, u16)> = parse_table(TCP6).into_iter().map(|r| (r.addr.to_string(), r.port)).collect();
        assert_eq!(addrs, vec![("::".into(), 8000), ("::1".into(), 3000), ("127.0.0.1".into(), 6667)]);
    }

    #[test]
    fn ignores_garbage() {
        assert!(parse_table("header\nnot a row\n  0: zz:zz 0 0A").is_empty());
    }
}
