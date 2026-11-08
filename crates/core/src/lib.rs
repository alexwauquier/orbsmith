//! Platform-independent core of Orbsmith: the detectable-apps catalog, safe
//! executable paths and the lifecycle of dummy processes.

pub mod catalog;
pub mod error;
pub mod exe_path;
pub mod layout;
pub mod os;

pub use catalog::{AppSummary, Catalog, DetectableApp};
pub use error::Error;
pub use layout::Layout;
pub use os::Os;
