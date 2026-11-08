use std::path::{Path, PathBuf};

use crate::os::Os;

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

    /// Orbsmith's folder inside `user_data_dir`, the platform's per-user data
    /// dir (`%APPDATA%` on Windows, `~/.local/share` on Linux).
    pub fn for_user(user_data_dir: &Path, os: Os) -> Self {
        Self::new(user_data_dir.join(os.data_dir_name()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_a_readable_folder_name_per_platform() {
        let base = Path::new("data");
        assert_eq!(
            Layout::for_user(base, Os::Windows).data_dir(),
            base.join("Orbsmith")
        );
        assert_eq!(
            Layout::for_user(base, Os::Linux).executables_dir(),
            base.join("orbsmith").join("executables")
        );
    }
}
