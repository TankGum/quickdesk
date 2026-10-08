//! Map published ports to Docker containers via the Engine API on the unix
//! socket. Docker's port proxies run as root, so without this those ports
//! would show up with no process at all.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    /// Port inside the container that the host port forwards to.
    pub private_port: u16,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ApiContainer {
    id: String,
    names: Vec<String>,
    image: String,
    ports: Vec<ApiPort>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ApiPort {
    private_port: u16,
    public_port: Option<u16>,
    #[serde(rename = "Type")]
    proto: String,
}

fn socket_path() -> PathBuf {
    std::env::var("DOCKER_HOST")
        .ok()
        .and_then(|h| h.strip_prefix("unix://").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("/var/run/docker.sock"))
}

/// Minimal HTTP/1.0 request (no chunked encoding to deal with).
fn request(method: &str, path: &str) -> std::io::Result<(u16, String)> {
    let mut stream = UnixStream::connect(socket_path())?;
    stream.set_read_timeout(Some(Duration::from_secs(15)))?;
    write!(stream, "{method} {path} HTTP/1.0\r\nHost: docker\r\nContent-Length: 0\r\n\r\n")?;
    let mut raw = String::new();
    stream.read_to_string(&mut raw)?;
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
    let status = head.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    Ok((status, body.to_owned()))
}

pub(crate) fn parse_containers(json: &str) -> Option<HashMap<u16, Container>> {
    let list: Vec<ApiContainer> = serde_json::from_str(json).ok()?;
    let mut map = HashMap::new();
    for c in list {
        for p in c.ports.iter().filter(|p| p.proto == "tcp") {
            if let Some(public) = p.public_port {
                map.entry(public).or_insert_with(|| Container {
                    id: c.id.chars().take(12).collect(),
                    name: c.names.first().map(|n| n.trim_start_matches('/').to_owned()).unwrap_or_default(),
                    image: c.image.clone(),
                    private_port: p.private_port,
                });
            }
        }
    }
    Some(map)
}

/// Host port → container. Empty when Docker is absent or not accessible.
pub fn containers_by_port() -> HashMap<u16, Container> {
    match request("GET", "/containers/json") {
        Ok((200, body)) => parse_containers(&body).unwrap_or_default(),
        _ => HashMap::new(),
    }
}

/// `docker stop <id>` (SIGTERM, then SIGKILL after Docker's grace period).
pub fn stop(id: &str) -> Result<()> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(Error::Platform(format!("invalid container id {id:?}")));
    }
    match request("POST", &format!("/containers/{id}/stop"))? {
        (204 | 304, _) => Ok(()),
        (status, body) => Err(Error::Platform(format!("docker stop failed ({status}): {}", body.trim()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_public_tcp_ports() {
        let json = r#"[
          {"Id":"abcdef0123456789","Names":["/api"],"Image":"svc","Ports":[
            {"IP":"0.0.0.0","PrivatePort":8000,"PublicPort":3003,"Type":"tcp"},
            {"IP":"::","PrivatePort":8000,"PublicPort":3003,"Type":"tcp"},
            {"PrivatePort":9000,"Type":"tcp"},
            {"IP":"0.0.0.0","PrivatePort":53,"PublicPort":5353,"Type":"udp"}]},
          {"Id":"ffff","Names":["/db"],"Image":"postgres:16","Ports":[]}
        ]"#;
        let map = parse_containers(json).unwrap();
        assert_eq!(map.len(), 1);
        let c = &map[&3003];
        assert_eq!((c.id.as_str(), c.name.as_str(), c.private_port), ("abcdef012345", "api", 8000));
    }

    #[test]
    fn rejects_suspicious_ids() {
        assert!(stop("../../x").is_err());
        assert!(stop("").is_err());
    }
}
