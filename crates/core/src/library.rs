//! Favorite and recent games, and how each game was last started, saved as
//! `library.json`. The frontend owns this logic (`src/lib/library.ts`); this
//! module only stores the result, trimmed to the same limits.

use std::fmt;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::Error;

/// Recent games kept.
pub const RECENT_LIMIT: usize = 8;
/// Games whose last run is remembered.
pub const LAST_RUNS_LIMIT: usize = 100;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Library {
    /// App IDs, newest first.
    pub favorites: Vec<String>,
    /// App IDs, newest first.
    pub recent: Vec<String>,
    /// Keyed by app ID, oldest first: a JSON object whose order matters.
    #[serde(with = "ordered_map")]
    pub last_runs: Vec<(String, LastRun)>,
}

/// How a game was last started, to start it the same way next time.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LastRun {
    /// The auto-stop duration, when one was set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minutes: Option<f64>,
    /// A non-default executable, when one was chosen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable: Option<String>,
}

impl Library {
    /// The saved library, or `None` if none was saved yet.
    pub fn load(file: &Path) -> Result<Option<Self>, Error> {
        match fs::read(file) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(Error::InvalidLibraryFile),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    /// Writes through a temporary file, so a crash never leaves a truncated
    /// file behind.
    pub fn save(&self, file: &Path) -> Result<(), Error> {
        if let Some(dir) = file.parent() {
            fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_vec_pretty(self).map_err(Error::InvalidLibraryFile)?;
        let tmp = file.with_extension("json.tmp");
        fs::write(&tmp, json)?;
        fs::rename(&tmp, file)?;
        Ok(())
    }

    /// Drops duplicates and invalid durations, and keeps the newest entries
    /// within the limits.
    pub fn sanitized(mut self) -> Self {
        dedup_keep_first(&mut self.favorites);
        dedup_keep_first(&mut self.recent);
        self.recent.truncate(RECENT_LIMIT);

        let mut runs: Vec<(String, LastRun)> = Vec::new();
        for (id, mut run) in self.last_runs {
            run.minutes = run
                .minutes
                .filter(|minutes| *minutes >= 1.0 && minutes.fract() == 0.0);
            // A later entry for the same game is the newer one.
            runs.retain(|(other, _)| *other != id);
            runs.push((id, run));
        }
        let excess = runs.len().saturating_sub(LAST_RUNS_LIMIT);
        runs.drain(..excess);
        self.last_runs = runs;
        self
    }
}

fn dedup_keep_first(ids: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    ids.retain(|id| seen.insert(id.clone()));
}

/// (De)serializes a list of pairs as a JSON object, keeping its order, which
/// `serde_json`'s map type would sort.
mod ordered_map {
    use super::*;

    pub fn serialize<S: Serializer>(
        entries: &[(String, LastRun)],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_map(entries.iter().map(|(id, run)| (id, run)))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<(String, LastRun)>, D::Error> {
        struct Entries;

        impl<'de> Visitor<'de> for Entries {
            type Value = Vec<(String, LastRun)>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an object keyed by app ID")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::with_capacity(map.size_hint().unwrap_or(0));
                while let Some(entry) = map.next_entry()? {
                    entries.push(entry);
                }
                Ok(entries)
            }
        }

        deserializer.deserialize_map(Entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(minutes: Option<f64>) -> LastRun {
        LastRun {
            minutes,
            executable: None,
        }
    }

    #[test]
    fn none_without_a_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            Library::load(&dir.path().join("library.json")).unwrap(),
            None
        );
    }

    #[test]
    fn round_trips_keeping_the_order_of_last_runs() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("nested").join("library.json");
        let library = Library {
            favorites: vec!["2".into(), "1".into()],
            recent: vec!["3".into()],
            last_runs: vec![
                ("9".into(), run(Some(15.0))),
                (
                    "1".into(),
                    LastRun {
                        minutes: None,
                        executable: Some("win64/game.exe".into()),
                    },
                ),
                ("5".into(), run(None)),
            ],
        };
        library.save(&file).unwrap();
        assert_eq!(Library::load(&file).unwrap(), Some(library));
        assert!(!file.with_extension("json.tmp").exists());
    }

    #[test]
    fn reads_the_format_the_frontend_writes() {
        let json = r#"{
            "favorites": ["1"],
            "recent": [],
            "lastRuns": { "7": { "minutes": 30 }, "1": {} }
        }"#;
        let library: Library = serde_json::from_str(json).unwrap();
        assert_eq!(
            library.last_runs,
            [("7".into(), run(Some(30.0))), ("1".into(), run(None))]
        );
        assert_eq!(
            serde_json::to_string(&library.last_runs[1].1).unwrap(),
            "{}"
        );
    }

    #[test]
    fn missing_fields_default_to_empty() {
        let library: Library = serde_json::from_str("{}").unwrap();
        assert_eq!(library, Library::default());
    }

    #[test]
    fn reports_a_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("library.json");
        fs::write(&file, "not json").unwrap();
        assert!(matches!(
            Library::load(&file),
            Err(Error::InvalidLibraryFile(_))
        ));
    }

    #[test]
    fn sanitizing_trims_to_the_limits() {
        let library = Library {
            favorites: vec!["1".into(), "2".into(), "1".into()],
            recent: (0..20).map(|i| (i % 10).to_string()).collect(),
            last_runs: (0..150)
                .map(|i| (i.to_string(), run(Some(10.0))))
                .chain([
                    ("149".into(), run(Some(0.0))),
                    ("148".into(), run(Some(2.5))),
                ])
                .collect(),
        }
        .sanitized();

        assert_eq!(library.favorites, ["1", "2"]);
        assert_eq!(library.recent, ["0", "1", "2", "3", "4", "5", "6", "7"]);
        assert_eq!(library.last_runs.len(), LAST_RUNS_LIMIT);
        assert_eq!(library.last_runs[0].0, "50");
        // Repeated IDs move to the end, and invalid durations are dropped.
        assert_eq!(library.last_runs[98], ("149".into(), run(None)));
        assert_eq!(library.last_runs[99], ("148".into(), run(None)));
    }
}
