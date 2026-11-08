use std::sync::Arc;
use std::time::Duration;

use orbsmith_core::{AppSummary, Catalog, Os, SimulationInfo};
use serde::{Serialize, Serializer};
use tauri::State;

use crate::state::AppState;

/// Errors reach the UI as plain messages.
#[derive(Debug)]
pub struct CommandError(String);

impl<E: std::fmt::Display> From<E> for CommandError {
    fn from(err: E) -> Self {
        Self(err.to_string())
    }
}

impl Serialize for CommandError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

type CommandResult<T> = Result<T, CommandError>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppList {
    apps: Vec<AppSummary>,
    /// Milliseconds since the Unix epoch.
    fetched_at: u64,
}

/// Apps that can be simulated on this platform. Uses the cache unless
/// `refresh` is set or there is no usable cache yet.
#[tauri::command]
pub async fn get_apps(state: State<'_, AppState>, refresh: bool) -> CommandResult<AppList> {
    let catalog = catalog(&state, refresh).await?;
    Ok(AppList {
        apps: catalog.summaries(Os::CURRENT),
        fetched_at: catalog.fetched_at_ms(),
    })
}

#[tauri::command]
pub fn list_simulations(state: State<'_, AppState>) -> Vec<SimulationInfo> {
    state.simulations().list()
}

/// Starts simulating an app. With `duration_secs`, it stops automatically
/// after that long.
#[tauri::command]
pub async fn start_simulation(
    state: State<'_, AppState>,
    app_id: String,
    duration_secs: Option<u64>,
) -> CommandResult<SimulationInfo> {
    let catalog = catalog(&state, false).await?;
    let app = catalog
        .get(&app_id)
        .ok_or_else(|| orbsmith_core::Error::UnknownApp(app_id.clone()))?;
    let info =
        state
            .simulations()
            .start(app, Os::CURRENT, duration_secs.map(Duration::from_secs))?;
    Ok(info)
}

#[tauri::command]
pub fn stop_simulation(state: State<'_, AppState>, app_id: String) -> CommandResult<()> {
    state.simulations().stop(&app_id)?;
    Ok(())
}

/// Replaces a running simulation's timer: stop `duration_secs` from now, or
/// never if `None`.
#[tauri::command]
pub fn set_simulation_timer(
    state: State<'_, AppState>,
    app_id: String,
    duration_secs: Option<u64>,
) -> CommandResult<SimulationInfo> {
    let info = state
        .simulations()
        .set_timer(&app_id, duration_secs.map(Duration::from_secs))?;
    Ok(info)
}

async fn catalog(state: &AppState, refresh: bool) -> CommandResult<Arc<Catalog>> {
    let mut slot = state.catalog.lock().await;
    if !refresh && let Some(catalog) = slot.as_ref() {
        return Ok(catalog.clone());
    }

    let cache_file = state.layout.catalog_cache();
    let cached = if refresh {
        None
    } else {
        let file = cache_file.clone();
        tauri::async_runtime::spawn_blocking(move || Catalog::load(&file))
            .await?
            .inspect_err(|err| log::warn!("ignoring unreadable app list cache: {err}"))
            .ok()
            .flatten()
    };
    let catalog = match cached {
        Some(catalog) => catalog,
        None => Catalog::fetch(&cache_file).await?,
    };

    let catalog = Arc::new(catalog);
    *slot = Some(catalog.clone());
    Ok(catalog)
}
