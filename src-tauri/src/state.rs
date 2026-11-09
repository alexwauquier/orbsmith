use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use orbsmith_core::{Catalog, Layout, Queue, Settings, Simulations};

pub struct AppState {
    pub layout: Layout,
    /// Loaded lazily. An async mutex, so that concurrent requests wait for one
    /// download instead of each starting their own.
    pub catalog: tokio::sync::Mutex<Option<Arc<Catalog>>>,
    simulations: Mutex<Simulations>,
    queue: Mutex<Queue>,
    settings: Mutex<Settings>,
}

impl AppState {
    pub fn new(layout: Layout, dummy: PathBuf, settings: Settings, queue: Queue) -> Self {
        let simulations = Simulations::new(dummy, layout.executables_dir());
        Self {
            layout,
            catalog: tokio::sync::Mutex::new(None),
            simulations: Mutex::new(simulations),
            queue: Mutex::new(queue),
            settings: Mutex::new(settings),
        }
    }

    // The accessors recover from poisoning: keeping control of running
    // dummies, and being able to kill them on exit, matters more than the
    // panic. Never hold two of these guards at once.

    pub fn simulations(&self) -> MutexGuard<'_, Simulations> {
        self.simulations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn queue(&self) -> MutexGuard<'_, Queue> {
        self.queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn settings(&self) -> MutexGuard<'_, Settings> {
        self.settings
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
