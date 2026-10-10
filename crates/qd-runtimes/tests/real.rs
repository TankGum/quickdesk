//! Against this computer: `cargo test -p qd-runtimes --test real -- --ignored --nocapture`.

#[test]
#[ignore = "probes the real shell and version managers"]
fn scan_this_computer() {
    let started = std::time::Instant::now();
    let sys = qd_runtimes::System::detect();
    let rts = sys.runtimes();
    println!("{}", serde_json::to_string_pretty(&rts).unwrap());
    println!("scan took {:?}", started.elapsed());
    assert!(!rts.is_empty());
}

use qd_runtimes::probe::{Found, Probe};
use qd_runtimes::{Action, Runner, System, SystemRunner};

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("qd-runtimes-live-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn home() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap())
}

fn run(sys: &System, lang: &str, action: Action, version: &str) {
    let cmd = sys.command(lang, action, version).unwrap();
    let mut percents = 0;
    SystemRunner::with_timeout(std::time::Duration::from_secs(600))
        .run(&cmd, &mut |l| {
            if qd_runtimes::run::percent(l).is_some() {
                percents += 1;
            }
        })
        .unwrap_or_else(|e| panic!("{action:?} {version}: {e}"));
    println!("{lang} {action:?} {version}: ok ({percents} progress lines)");
}

fn default_version(sys: &System, lang: &str) -> Option<String> {
    let rt = sys.runtimes().into_iter().find(|r| r.id == lang)?;
    rt.installed.iter().find(|i| i.is_default).map(|i| i.version.clone())
}

/// uv with its install and bin dirs moved to a scratch folder.
#[test]
#[ignore = "downloads two Pythons with uv (into a temp folder)"]
fn uv_install_default_uninstall() {
    let dir = scratch("uv");
    let uv = home().join(".local/bin/uv");
    let mut probe = Probe { home: home(), shell: "bash".into(), ..Default::default() };
    probe.vars.insert("UV_PYTHON_INSTALL_DIR".into(), dir.join("py").display().to_string());
    probe.vars.insert("UV_PYTHON_BIN_DIR".into(), dir.join("bin").display().to_string());
    probe.found.insert("uv".into(), Found { path: uv.display().to_string(), version_line: "uv 0.11".into() });
    let sys = || System::from_probe(probe.clone());

    run(&sys(), "python", Action::Install, "3.13");
    run(&sys(), "python", Action::Install, "3.12");
    let ids: Vec<(String, String)> = sys()
        .runtimes()
        .into_iter()
        .find(|r| r.id == "python")
        .unwrap()
        .installed
        .iter()
        .map(|i| (i.id.clone(), i.version.clone()))
        .collect();
    println!("installed: {ids:?}");
    assert_eq!(ids.len(), 2);
    let (key313, v313) = ids.iter().find(|(_, v)| v.starts_with("3.13")).cloned().unwrap();
    let (key312, v312) = ids.iter().find(|(_, v)| v.starts_with("3.12")).cloned().unwrap();
    assert_eq!(default_version(&sys(), "python"), None);

    // Set the default by the key the UI passes (an Installed::id).
    run(&sys(), "python", Action::SetDefault, &key313);
    assert_eq!(default_version(&sys(), "python").as_deref(), Some(v313.as_str()));
    run(&sys(), "python", Action::SetDefault, &key312);
    assert_eq!(default_version(&sys(), "python").as_deref(), Some(v312.as_str()));
    assert!(sys().command("python", Action::Uninstall, &key312).is_err(), "default is protected");
    run(&sys(), "python", Action::Uninstall, &key313);
    let left: Vec<String> = sys()
        .runtimes()
        .into_iter()
        .find(|r| r.id == "python")
        .unwrap()
        .installed
        .iter()
        .map(|i| i.version.clone())
        .collect();
    assert_eq!(left, std::slice::from_ref(&v312));
    let list = sys().available("python", &SystemRunner::default()).unwrap();
    assert!(list.iter().any(|a| a.installed), "installed one is marked");
    println!(
        "bin dir: {:?}",
        std::fs::read_dir(dir.join("bin")).unwrap().flatten().map(|e| e.file_name()).collect::<Vec<_>>()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// nvm from ~/.nvm, copied without its versions and aliases into a scratch folder.
#[test]
#[ignore = "downloads a Node.js with nvm (into a temp folder)"]
fn nvm_install_default_uninstall() {
    let dir = scratch("nvm");
    let nvm = dir.join("nvm");
    std::fs::create_dir_all(&nvm).unwrap();
    for f in ["nvm.sh", "nvm-exec", "package.json", "bash_completion"] {
        std::fs::copy(home().join(".nvm").join(f), nvm.join(f)).unwrap();
    }
    let mut probe = Probe { home: home(), shell: "bash".into(), ..Default::default() };
    probe.vars.insert("NVM_DIR".into(), nvm.display().to_string());
    let sys = || System::from_probe(probe.clone());

    let list = sys().available("node", &SystemRunner::default()).unwrap();
    println!(
        "nvm ls-remote: {} versions, newest {:?}, first LTS {:?}",
        list.len(),
        list[0],
        list.iter().find(|a| a.tag.as_deref().is_some_and(|t| t.starts_with("LTS")))
    );
    run(&sys(), "node", Action::Install, "v18.20.8");
    let rt = sys().runtimes().into_iter().find(|r| r.id == "node").unwrap();
    assert_eq!(rt.installed.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(), ["v18.20.8"]);
    run(&sys(), "node", Action::SetDefault, "v18.20.8");
    assert_eq!(default_version(&sys(), "node").as_deref(), Some("18.20.8"));
    assert!(sys().command("node", Action::Uninstall, "v18.20.8").is_err(), "default is protected");
    std::fs::remove_file(nvm.join("alias/default")).unwrap();
    run(&sys(), "node", Action::Uninstall, "v18.20.8");
    assert!(sys().runtimes().into_iter().find(|r| r.id == "node").unwrap().installed.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

/// The mise installer, then Go through that mise, all inside a scratch folder.
#[test]
#[ignore = "downloads mise and a Go (into a temp folder)"]
fn mise_install_then_go() {
    let dir = scratch("mise");
    let bin = dir.join("bin");
    SystemRunner::with_timeout(std::time::Duration::from_secs(300))
        .run(&qd_runtimes::mise::installer(&bin), &mut |l| println!("  installer: {l}"))
        .unwrap();
    let mise = bin.join("mise");
    assert!(mise.is_file());

    let mut probe = Probe { home: home(), shell: "bash".into(), ..Default::default() };
    probe
        .found
        .insert("mise".into(), Found { path: mise.display().to_string(), version_line: "2026.10.6 linux-x64".into() });
    for (k, v) in [
        ("MISE_DATA_DIR", "data"),
        ("MISE_CONFIG_DIR", "config"),
        ("MISE_CACHE_DIR", "cache"),
        ("MISE_STATE_DIR", "state"),
    ] {
        probe.vars.insert(k.into(), dir.join(v).display().to_string());
    }
    probe.vars.insert("MISE_GLOBAL_CONFIG_FILE".into(), dir.join("config/config.toml").display().to_string());
    // No system go in this probe, so mise is the only manager for it.
    let sys = || System::from_probe(probe.clone());
    let go = |s: &System| s.runtimes().into_iter().find(|r| r.id == "go");

    let list = sys().available("go", &SystemRunner::default()).unwrap();
    println!("go versions: {}, newest {:?}", list.len(), list.first());
    run(&sys(), "go", Action::Install, "1.22.12");
    run(&sys(), "go", Action::Install, "1.23.4");
    let rt = go(&sys()).unwrap();
    assert_eq!(rt.manager.as_deref(), Some("mise"));
    assert_eq!(rt.installed.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(), ["1.23.4", "1.22.12"]);
    run(&sys(), "go", Action::SetDefault, "1.23.4");
    assert_eq!(default_version(&sys(), "go").as_deref(), Some("1.23.4"));
    assert!(sys().command("go", Action::Uninstall, "1.23.4").is_err(), "default is protected");

    let project = dir.join("project");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("mise.toml"), "[tools]\nnode = \"22\"\n").unwrap();
    let file = sys().set_project(&project, "go", "1.22.12").unwrap();
    let text = std::fs::read_to_string(file).unwrap();
    println!("mise.toml:\n{text}");
    assert!(text.contains("node") && text.contains("go = \"1.22.12\""), "keeps other tools");

    run(&sys(), "go", Action::Uninstall, "1.22.12");
    assert_eq!(go(&sys()).unwrap().installed.len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}
