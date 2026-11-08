//! Platform-independent core of Orbsmith: the detectable-apps catalog, safe
//! executable paths and the lifecycle of dummy processes.

pub mod exe_path;
pub mod os;

pub use os::Os;
