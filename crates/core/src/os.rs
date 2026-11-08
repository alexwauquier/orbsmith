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
}
