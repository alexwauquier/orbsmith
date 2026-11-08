use std::path::{Path, PathBuf};

/// Where Orbsmith keeps its runtime files, inside the per-user app data dir.
#[derive(Debug, Clone)]
pub struct Layout {
    data_dir: PathBuf,
}

impl Layout {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Cached response of the detectable-apps API.
    pub fn catalog_cache(&self) -> PathBuf {
        self.data_dir.join("detectable-apps.json")
    }

    /// Root of the renamed dummy copies. Every process running from here is
    /// one of Orbsmith's dummies.
    pub fn executables_dir(&self) -> PathBuf {
        self.data_dir.join("executables")
    }
}
