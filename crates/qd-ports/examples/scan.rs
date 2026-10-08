//! `cargo run -p qd-ports --example scan` — print listening ports.
fn main() {
    for e in qd_ports::scan().expect("scan failed") {
        println!(
            "{:<6} {:<22} {:<8} {:<16} {:<10} {:<28} {}",
            e.port,
            e.addrs.join(","),
            e.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
            e.process.as_deref().unwrap_or("?"),
            e.user.as_deref().unwrap_or("?"),
            e.container.as_ref().map(|c| format!("🐳 {}:{}", c.name, c.private_port)).unwrap_or_default(),
            e.cmdline.as_deref().unwrap_or("").chars().take(60).collect::<String>()
        );
    }
}
