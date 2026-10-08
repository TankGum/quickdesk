//! Command-line verbs. The same parser handles argv of a fresh launch and
//! messages forwarded from a second instance.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Notes,
    Clipboard,
    Ports,
    Main,
}

impl Target {
    pub const HOTKEY_TARGETS: [Target; 3] = [Target::Notes, Target::Clipboard, Target::Ports];

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "notes" | "note" => Some(Target::Notes),
            "clipboard" | "clip" => Some(Target::Clipboard),
            "ports" | "port" => Some(Target::Ports),
            "main" => Some(Target::Main),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Target::Notes => "notes",
            Target::Clipboard => "clipboard",
            Target::Ports => "ports",
            Target::Main => "main",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliCommand {
    /// Plain launch, or a relaunch while running: bring up the main window.
    Launch,
    Toggle(Target),
    Show(Target),
    /// Start hidden in the tray (used by autostart).
    Background,
    Quit,
}

impl CliCommand {
    /// `args` excludes the program name.
    pub fn parse<S: AsRef<str>>(args: &[S]) -> Result<Self, String> {
        let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
        match args.as_slice() {
            [] => Ok(CliCommand::Launch),
            ["toggle", t] => Target::parse(t).map(CliCommand::Toggle).ok_or_else(|| usage(t)),
            ["show", t] => Target::parse(t).map(CliCommand::Show).ok_or_else(|| usage(t)),
            ["--background"] => Ok(CliCommand::Background),
            ["quit"] => Ok(CliCommand::Quit),
            other => Err(usage(&other.join(" "))),
        }
    }
}

fn usage(bad: &str) -> String {
    format!(
        "unrecognized arguments: {bad:?}\n\
         usage: quickdesk [toggle|show] <notes|clipboard|ports|main> | --background | quit"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_verbs() {
        assert_eq!(CliCommand::parse::<&str>(&[]), Ok(CliCommand::Launch));
        assert_eq!(CliCommand::parse(&["toggle", "notes"]), Ok(CliCommand::Toggle(Target::Notes)));
        assert_eq!(CliCommand::parse(&["toggle", "clip"]), Ok(CliCommand::Toggle(Target::Clipboard)));
        assert_eq!(CliCommand::parse(&["show", "main"]), Ok(CliCommand::Show(Target::Main)));
        assert_eq!(CliCommand::parse(&["--background"]), Ok(CliCommand::Background));
        assert_eq!(CliCommand::parse(&["quit"]), Ok(CliCommand::Quit));
    }

    #[test]
    fn rejects_unknown() {
        assert!(CliCommand::parse(&["toggle"]).is_err());
        assert!(CliCommand::parse(&["toggle", "nope"]).is_err());
        assert!(CliCommand::parse(&["frobnicate"]).is_err());
    }
}
