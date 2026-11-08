use std::fmt;
use std::path::{Path, PathBuf};

use crate::os::Os;

/// An executable name from the API, validated so that it can be safely joined
/// under a base directory.
///
/// Names are relative paths using `/` as separator, such as `game/bin/game.exe`.
/// Some carry a leading `>`, which is a matching hint for the detector and not
/// part of the file name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExePath {
    dirs: Vec<String>,
    file_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid executable name {name:?}: {reason}")]
pub struct InvalidExeName {
    pub name: String,
    pub reason: &'static str,
}

const WINDOWS_FORBIDDEN: &[char] = &['<', '>', ':', '"', '|', '?', '*', '\\'];

const WINDOWS_RESERVED: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

impl ExePath {
    pub fn parse(raw: &str, os: Os) -> Result<Self, InvalidExeName> {
        let invalid = |reason| InvalidExeName {
            name: raw.to_owned(),
            reason,
        };

        let name = raw.strip_prefix('>').unwrap_or(raw);
        if name.is_empty() {
            return Err(invalid("empty name"));
        }

        let mut dirs = name
            .split('/')
            .map(|component| check_component(component, os).map_err(invalid))
            .collect::<Result<Vec<_>, _>>()?;
        let file_name = dirs.pop().ok_or_else(|| invalid("empty name"))?;

        Ok(Self { dirs, file_name })
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// The directory holding this executable under `base`. Lexically never
    /// escapes `base`.
    pub fn dir_under(&self, base: &Path) -> PathBuf {
        let mut path = base.to_path_buf();
        path.extend(&self.dirs);
        path
    }

    /// The path of this executable under `base`. Lexically never escapes
    /// `base`.
    pub fn under(&self, base: &Path) -> PathBuf {
        self.dir_under(base).join(&self.file_name)
    }
}

impl fmt::Display for ExePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for dir in &self.dirs {
            write!(f, "{dir}/")?;
        }
        f.write_str(&self.file_name)
    }
}

fn check_component(component: &str, os: Os) -> Result<String, &'static str> {
    if component.is_empty() {
        return Err("empty path component or absolute path");
    }
    if component == "." || component == ".." {
        return Err("relative path component");
    }
    if component.chars().any(char::is_control) {
        return Err("control character");
    }
    if os == Os::Windows {
        if component.contains(WINDOWS_FORBIDDEN) {
            return Err("character not allowed in Windows file names");
        }
        // Windows silently strips these, so the process path would not match.
        if component.ends_with(['.', ' ']) {
            return Err("trailing dot or space");
        }
        let stem = component.split('.').next().unwrap_or(component);
        if WINDOWS_RESERVED.contains(&stem.to_ascii_lowercase().as_str()) {
            return Err("reserved Windows device name");
        }
    }
    Ok(component.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win(raw: &str) -> Result<ExePath, InvalidExeName> {
        ExePath::parse(raw, Os::Windows)
    }

    #[test]
    fn accepts_plain_and_nested_names() {
        assert_eq!(win("overwatch.exe").unwrap().file_name(), "overwatch.exe");

        let nested = win("_retail_/wow-64.exe").unwrap();
        assert_eq!(nested.file_name(), "wow-64.exe");
        assert_eq!(nested.to_string(), "_retail_/wow-64.exe");
        assert_eq!(
            nested.dir_under(Path::new("base")),
            Path::new("base").join("_retail_")
        );
        assert_eq!(
            nested.under(Path::new("base")),
            Path::new("base").join("_retail_").join("wow-64.exe")
        );
    }

    #[test]
    fn accepts_dots_inside_components() {
        assert!(win("oh...sir! the insult simulator/ohsir.exe").is_ok());
        assert!(win("missedmessageswindows/missed messages..exe").is_ok());
    }

    #[test]
    fn strips_leading_match_hint() {
        assert_eq!(win(">hl2.exe").unwrap().to_string(), "hl2.exe");
        assert!(win(">").is_err());
        assert!(win(">>hl2.exe").is_err());
    }

    #[test]
    fn rejects_traversal_and_absolute_paths() {
        for raw in [
            "",
            "../evil.exe",
            "game/../../evil.exe",
            "./game.exe",
            "/etc/evil",
            "game//game.exe",
            "game/",
            "c:/windows/evil.exe",
            "c:evil.exe",
            "\\\\server\\share\\evil.exe",
            "game\\..\\evil.exe",
        ] {
            assert!(win(raw).is_err(), "{raw:?} should be rejected");
        }
    }

    #[test]
    fn rejects_names_windows_cannot_represent() {
        for raw in [
            "con",
            "nul.exe",
            "game/aux.txt",
            "game./game.exe",
            "game.exe ",
            "a\u{0}b",
        ] {
            assert!(win(raw).is_err(), "{raw:?} should be rejected");
        }
    }

    #[test]
    fn linux_rules_allow_what_unix_file_names_allow() {
        assert!(ExePath::parse("va-11 hall-a/runner\"", Os::Linux).is_ok());
        assert!(ExePath::parse("con", Os::Linux).is_ok());
        assert!(ExePath::parse("../evil", Os::Linux).is_err());
        assert!(ExePath::parse("/usr/bin/evil", Os::Linux).is_err());
    }
}
