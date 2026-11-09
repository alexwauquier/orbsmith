/// Operating systems as named by the detectable-apps API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    Linux,
    MacOs,
}

impl Os {
    #[cfg(windows)]
    pub const CURRENT: Os = Os::Windows;
    #[cfg(target_os = "linux")]
    pub const CURRENT: Os = Os::Linux;
    #[cfg(target_os = "macos")]
    pub const CURRENT: Os = Os::MacOs;

    /// The value of an executable's `os` field in the API.
    pub fn api_name(self) -> &'static str {
        match self {
            Os::Windows => "win32",
            Os::Linux => "linux",
            Os::MacOs => "darwin",
        }
    }

    /// The `os` values whose executables the detector watches on this OS, in
    /// order of preference.
    ///
    /// Discord's Linux client matches running processes against the names of
    /// every OS, so a `win32` name works there too: most games only list one.
    /// Verified with a renamed native binary on Ubuntu. Native names are tried
    /// first.
    pub fn watched_api_names(self) -> &'static [&'static str] {
        match self {
            Os::Windows => &["win32"],
            Os::Linux => &["linux", "win32"],
            Os::MacOs => &["darwin"],
        }
    }

    /// Whether file paths compare case-insensitively by default.
    pub fn case_insensitive_paths(self) -> bool {
        matches!(self, Os::Windows | Os::MacOs)
    }

    /// Name of Orbsmith's folder in the per-user data dir, following each
    /// platform's habits: `Orbsmith` on Windows and macOS, lowercase on Linux.
    pub fn data_dir_name(self) -> &'static str {
        match self {
            Os::Windows | Os::MacOs => "Orbsmith",
            Os::Linux => "orbsmith",
        }
    }
}
