use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use orbsmith_core::{Catalog, Layout, Simulations};

pub struct AppState {
    pub layout: Layout,
    /// Loaded lazily. An async mutex, so that concurrent requests wait for one
    /// download instead of each starting their own.
    pub catalog: tokio::sync::Mutex<Option<Arc<Catalog>>>,
    simulations: Mutex<Simulations>,
}

impl AppState {
    pub fn new(layout: Layout, dummy: PathBuf) -> Self {
        let simulations = Simulations::new(dummy, layout.executables_dir());
        Self {
            layout,
            catalog: tokio::sync::Mutex::new(None),
            simulations: Mutex::new(simulations),
        }
    }

    pub fn simulations(&self) -> MutexGuard<'_, Simulations> {
        // Recover from poisoning: keeping control of running dummies, and
        // being able to kill them on exit, matters more than the panic.
        self.simulations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
