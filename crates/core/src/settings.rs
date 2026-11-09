use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Error;

/// User preferences, stored as JSON in the app data dir. Missing fields take
/// their default, so older files keep loading as settings are added.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Start every queued game at once instead of one after another.
    pub parallel_queue: bool,
}

impl Settings {
    /// The saved settings, or the defaults if none were saved yet.
    pub fn load(file: &Path) -> Result<Self, Error> {
        match fs::read(file) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(Error::InvalidSettings),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(err) => Err(err.into()),
        }
    }

    /// Writes through a temporary file, so a crash never leaves a truncated
    /// file behind.
    pub fn save(&self, file: &Path) -> Result<(), Error> {
        if let Some(dir) = file.parent() {
            fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_vec_pretty(self).map_err(Error::InvalidSettings)?;
        let tmp = file.with_extension("json.tmp");
        fs::write(&tmp, json)?;
        fs::rename(&tmp, file)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_without_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let settings = Settings::load(&dir.path().join("settings.json")).unwrap();
        assert_eq!(settings, Settings::default());
        assert!(!settings.parallel_queue);
    }

    #[test]
    fn round_trips_through_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("nested").join("settings.json");
        let settings = Settings {
            parallel_queue: true,
        };
        settings.save(&file).unwrap();
        assert_eq!(Settings::load(&file).unwrap(), settings);
        assert!(!file.with_extension("json.tmp").exists());
    }

    #[test]
    fn ignores_unknown_and_missing_fields() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        fs::write(&file, r#"{ "removedSetting": 1 }"#).unwrap();
        assert_eq!(Settings::load(&file).unwrap(), Settings::default());
    }

    #[test]
    fn reports_a_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.json");
        fs::write(&file, "not json").unwrap();
        assert!(matches!(
            Settings::load(&file),
            Err(Error::InvalidSettings(_))
        ));
    }
}
