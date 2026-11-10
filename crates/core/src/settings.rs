use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::queue::{MAX_GAP, Pacing};

/// User preferences, stored as JSON in the app data dir. Missing fields take
/// their default, so older files keep loading as settings are added.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Start every queued game at once instead of one after another.
    pub parallel_queue: bool,
    /// Look for a new release at startup.
    pub check_for_updates: bool,
    /// Wait between two games of the queue when they run one after another,
    /// in seconds.
    pub queue_gap_secs: u64,
    /// Make queued games run a little longer than their duration, and the
    /// waits a little shorter or longer.
    pub vary_queue_times: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            parallel_queue: false,
            check_for_updates: true,
            queue_gap_secs: 0,
            vary_queue_times: false,
        }
    }
}

impl Settings {
    /// How the queue paces its games.
    pub fn pacing(&self) -> Pacing {
        Pacing {
            parallel: self.parallel_queue,
            gap: Duration::from_secs(self.queue_gap_secs),
            vary: self.vary_queue_times,
        }
    }

    /// Rejects values the UI does not offer.
    pub fn validate(&self) -> Result<(), Error> {
        if Duration::from_secs(self.queue_gap_secs) > MAX_GAP {
            return Err(Error::GapTooLong);
        }
        Ok(())
    }

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
        assert!(settings.check_for_updates);
    }

    #[test]
    fn round_trips_through_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("nested").join("settings.json");
        let settings = Settings {
            parallel_queue: true,
            check_for_updates: false,
            queue_gap_secs: 300,
            vary_queue_times: true,
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

        // A file saved before a setting existed gets that setting's default.
        fs::write(&file, r#"{ "parallelQueue": true }"#).unwrap();
        let settings = Settings::load(&file).unwrap();
        assert!(settings.parallel_queue);
        assert!(settings.check_for_updates);
    }

    #[test]
    fn caps_the_wait_between_games() {
        let mut settings = Settings {
            queue_gap_secs: 24 * 60 * 60,
            ..Settings::default()
        };
        assert!(settings.validate().is_ok());
        assert_eq!(settings.pacing().gap, MAX_GAP);
        settings.queue_gap_secs += 1;
        assert!(matches!(settings.validate(), Err(Error::GapTooLong)));
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
