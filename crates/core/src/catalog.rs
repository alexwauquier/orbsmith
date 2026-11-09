use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::exe_path::ExePath;
use crate::os::Os;

pub const DETECTABLE_APPS_URL: &str = "https://discord.com/api/v10/applications/detectable";

const USER_AGENT: &str = concat!(
    "Orbsmith/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/alexwauquier/orbsmith)"
);

/// An application from the detectable-apps API. Only the fields Orbsmith uses
/// are kept.
#[derive(Debug, Clone, Deserialize)]
pub struct DetectableApp {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon_hash: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub executables: Vec<Executable>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Executable {
    /// Kept as a string so that an OS value added to the API later doesn't
    /// break parsing of the whole list.
    pub os: String,
    pub name: String,
    #[serde(default)]
    pub is_launcher: bool,
    /// Command-line arguments the detector also expects, such as
    /// `-game garrysmod`.
    #[serde(default)]
    pub arguments: Option<String>,
}

/// What to launch to simulate an app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub exe: ExePath,
    pub arguments: Vec<String>,
}

impl DetectableApp {
    /// Every non-launcher executable for `os` whose name is safe to use, in
    /// the API's order, without duplicates.
    pub fn targets(&self, os: Os) -> Vec<Target> {
        let mut targets: Vec<Target> = Vec::new();
        let candidates = self
            .executables
            .iter()
            .filter(|exe| exe.os == os.api_name() && !exe.is_launcher);
        for exe in candidates {
            let Ok(path) = ExePath::parse(&exe.name, os)
                .inspect_err(|err| log::debug!("skipping executable of {}: {err}", self.id))
            else {
                continue;
            };
            if targets.iter().any(|target| target.exe == path) {
                continue;
            }
            targets.push(Target {
                exe: path,
                arguments: exe
                    .arguments
                    .as_deref()
                    .unwrap_or_default()
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect(),
            });
        }
        targets
    }

    /// The default executable for `os`: the first of [`Self::targets`].
    pub fn target(&self, os: Os) -> Option<Target> {
        self.targets(os).into_iter().next()
    }

    /// The executable for `os` whose path reads `name`, as listed in
    /// [`AppSummary::executables`].
    pub fn target_named(&self, os: Os, name: &str) -> Option<Target> {
        self.targets(os)
            .into_iter()
            .find(|target| target.exe.to_string() == name)
    }
}

/// An app that can be simulated on the current platform, as sent to the UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSummary {
    pub id: String,
    pub name: String,
    pub icon_hash: Option<String>,
    pub aliases: Vec<String>,
    /// The executable simulated by default.
    pub executable: String,
    /// Every executable that can be simulated, the default first.
    pub executables: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    apps: Vec<DetectableApp>,
    fetched_at: SystemTime,
}

impl Catalog {
    pub fn from_json(json: &[u8], fetched_at: SystemTime) -> Result<Self, Error> {
        Ok(Self {
            apps: serde_json::from_slice(json)?,
            fetched_at,
        })
    }

    /// Loads the cached list, or returns `None` if there is no cache yet.
    pub fn load(cache_file: &Path) -> Result<Option<Self>, Error> {
        let json = match fs::read(cache_file) {
            Ok(json) => json,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let fetched_at = fs::metadata(cache_file)?.modified()?;
        Self::from_json(&json, fetched_at).map(Some)
    }

    /// Downloads the list and replaces the cache. The cache is only written
    /// once the response has parsed, so a bad response never overwrites a good
    /// cache.
    pub async fn fetch(cache_file: &Path) -> Result<Self, Error> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(60))
            .build()?;
        let json = client
            .get(DETECTABLE_APPS_URL)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        let catalog = Self::from_json(&json, SystemTime::now())?;

        if let Some(dir) = cache_file.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = cache_file.with_extension("json.tmp");
        fs::write(&tmp, &json)?;
        fs::rename(&tmp, cache_file)?;

        Ok(catalog)
    }

    pub fn get(&self, id: &str) -> Option<&DetectableApp> {
        self.apps.iter().find(|app| app.id == id)
    }

    /// Milliseconds since the Unix epoch.
    pub fn fetched_at_ms(&self) -> u64 {
        unix_ms(self.fetched_at)
    }

    pub fn summaries(&self, os: Os) -> Vec<AppSummary> {
        self.apps
            .iter()
            .filter_map(|app| {
                let executables: Vec<String> = app
                    .targets(os)
                    .iter()
                    .map(|target| target.exe.to_string())
                    .collect();
                Some(AppSummary {
                    id: app.id.clone(),
                    name: app.name.clone(),
                    icon_hash: app.icon_hash.clone(),
                    aliases: app.aliases.clone(),
                    executable: executables.first()?.clone(),
                    executables,
                })
            })
            .collect()
    }
}

pub(crate) fn unix_ms(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"[
        {
            "id": "1",
            "name": "Plain",
            "icon_hash": "abc",
            "aliases": ["P"],
            "executables": [
                {"os": "win32", "name": "launcher.exe", "is_launcher": true},
                {"os": "win32", "name": "../evil.exe", "is_launcher": false},
                {"os": "win32", "name": "plain/bin/plain.exe", "is_launcher": false},
                {"os": "linux", "name": "plain", "is_launcher": false}
            ],
            "unknown_field": 42
        },
        {
            "id": "2",
            "name": "With arguments",
            "executables": [
                {"os": "win32", "name": ">hl2.exe", "is_launcher": false, "arguments": "-game  garrysmod"}
            ]
        },
        {
            "id": "4",
            "name": "Several",
            "executables": [
                {"os": "win32", "name": "garena/lol.exe", "is_launcher": false},
                {"os": "win32", "name": "league of legends.exe", "is_launcher": false},
                {"os": "win32", "name": "garena/lol.exe", "is_launcher": false},
                {"os": "win32", "name": ">league of legends.exe", "is_launcher": false}
            ]
        },
        {
            "id": "3",
            "name": "Mac only",
            "icon_hash": null,
            "executables": [{"os": "darwin", "name": "mac.app", "is_launcher": false}]
        }
    ]"#;

    fn catalog() -> Catalog {
        Catalog::from_json(SAMPLE.as_bytes(), UNIX_EPOCH).unwrap()
    }

    #[test]
    fn picks_first_safe_non_launcher_for_the_os() {
        let catalog = catalog();
        let plain = catalog.get("1").unwrap();

        let target = plain.target(Os::Windows).unwrap();
        assert_eq!(target.exe.to_string(), "plain/bin/plain.exe");
        assert!(target.arguments.is_empty());

        assert_eq!(plain.target(Os::Linux).unwrap().exe.to_string(), "plain");
    }

    #[test]
    fn lists_every_executable_once_and_finds_one_by_name() {
        let catalog = catalog();
        let several = catalog.get("4").unwrap();
        let names: Vec<_> = several
            .targets(Os::Windows)
            .iter()
            .map(|target| target.exe.to_string())
            .collect();
        assert_eq!(names, ["garena/lol.exe", "league of legends.exe"]);
        assert_eq!(
            several
                .target_named(Os::Windows, "league of legends.exe")
                .map(|target| target.exe.to_string()),
            Some("league of legends.exe".to_owned())
        );
        assert!(several.target_named(Os::Windows, "other.exe").is_none());
    }

    #[test]
    fn splits_arguments_and_strips_match_hint() {
        let target = catalog().get("2").unwrap().target(Os::Windows).unwrap();
        assert_eq!(target.exe.to_string(), "hl2.exe");
        assert_eq!(target.arguments, ["-game", "garrysmod"]);
    }

    #[test]
    fn summaries_only_include_simulatable_apps() {
        let summaries = catalog().summaries(Os::Windows);
        let ids: Vec<_> = summaries.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, ["1", "2", "4"]);
        assert_eq!(summaries[0].executable, "plain/bin/plain.exe");
        assert_eq!(summaries[0].executables, ["plain/bin/plain.exe"]);
        assert_eq!(summaries[2].executable, "garena/lol.exe");
        assert_eq!(summaries[2].executables.len(), 2);
        assert_eq!(summaries[0].icon_hash.as_deref(), Some("abc"));
    }

    #[test]
    fn load_returns_none_without_cache() {
        let dir = tempfile::tempdir().unwrap();
        assert!(
            Catalog::load(&dir.path().join("missing.json"))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn load_reads_cache() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("apps.json");
        fs::write(&file, SAMPLE).unwrap();

        let catalog = Catalog::load(&file).unwrap().unwrap();
        assert!(catalog.get("3").is_some());
        assert!(catalog.fetched_at_ms() > 0);
    }

    #[test]
    fn load_rejects_corrupt_cache() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("apps.json");
        fs::write(&file, "not json").unwrap();
        assert!(matches!(Catalog::load(&file), Err(Error::Json(_))));
    }
}
