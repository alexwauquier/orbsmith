//! Platform-independent core of Orbsmith: the detectable-apps catalog, safe
//! executable paths, the lifecycle of dummy processes, the queue and the
//! settings.

pub mod catalog;
pub mod cleanup;
pub mod error;
pub mod exe_path;
pub mod layout;
pub mod os;
pub mod queue;
pub mod settings;
pub mod simulation;

pub use catalog::{AppSummary, Catalog, DetectableApp};
pub use error::Error;
pub use layout::Layout;
pub use os::Os;
pub use queue::{Queue, QueueItem, QueueState};
pub use settings::Settings;
pub use simulation::{EndReason, Ended, SimulationInfo, Simulations};

#[cfg(test)]
pub(crate) mod test_support;
