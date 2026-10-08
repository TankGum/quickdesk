//! Windows / macOS: delegate socket enumeration to the `listeners` crate.

use listeners::{Protocol, SocketState};

use crate::{Error, Result, Socket};

pub(crate) fn listening_sockets() -> Result<Vec<Socket>> {
    let all = listeners::get_all().map_err(|e| Error::Platform(e.to_string()))?;
    Ok(all
        .into_iter()
        .filter(|l| l.protocol == Protocol::TCP && l.state == SocketState::Listen)
        .map(|l| Socket {
            addr: l.socket.ip(),
            port: l.socket.port(),
            pid: Some(l.process.pid),
            process: Some(l.process.name),
            cmdline: (!l.process.path.is_empty()).then_some(l.process.path),
            user: None,
        })
        .collect())
}
