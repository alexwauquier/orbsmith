use std::sync::Arc;
use std::time::{Duration, SystemTime};

use orbsmith_core::simulation::MAX_DURATION;
use orbsmith_core::{
    AppSummary, Catalog, Library, Os, QueueItem, QueueState, Settings, SimulationInfo,
};
use serde::{Serialize, Serializer};
use tauri::{AppHandle, State};

use crate::state::AppState;

/// Errors reach the UI as plain messages.
#[derive(Debug)]
pub struct CommandError(String);

impl<E: std::fmt::Display> From<E> for CommandError {
    fn from(err: E) -> Self {
        Self(err.to_string())
    }
}

impl CommandError {
    pub fn message(&self) -> &str {
        &self.0
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
/// `refresh` is set, there is no usable cache yet, or it is more than a day
/// old and a new list can be downloaded.
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

/// Starts simulating an app, as `executable` (one of its summary's
/// `executables`) or its default one. With `duration_secs`, it stops
/// automatically after that long.
#[tauri::command]
pub async fn start_simulation(
    handle: AppHandle,
    state: State<'_, AppState>,
    app_id: String,
    duration_secs: Option<u64>,
    executable: Option<String>,
) -> CommandResult<SimulationInfo> {
    let catalog = catalog(&state, false).await?;
    let app = catalog
        .get(&app_id)
        .ok_or_else(|| orbsmith_core::Error::UnknownApp(app_id.clone()))?;
    let info = state.simulations().start(
        app,
        Os::CURRENT,
        executable.as_deref(),
        duration_secs.map(Duration::from_secs),
    )?;
    crate::simulations_changed(&handle);
    Ok(info)
}

#[tauri::command]
pub fn stop_simulation(
    handle: AppHandle,
    state: State<'_, AppState>,
    app_id: String,
) -> CommandResult<()> {
    state.simulations().stop(&app_id)?;
    crate::simulations_changed(&handle);
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

/// The saved library, or `None` before the first save, when the frontend
/// moves over what it kept in local storage.
#[tauri::command]
pub fn get_library(state: State<'_, AppState>) -> CommandResult<Option<Library>> {
    Ok(Library::load(&state.layout.library_file())?)
}

// Synchronous, so saves run one at a time, in order, on the main thread.
#[tauri::command]
pub fn set_library(state: State<'_, AppState>, library: Library) -> CommandResult<()> {
    library.sanitized().save(&state.layout.library_file())?;
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings().clone()
}

/// Saves the settings. Switching the queue to parallel starts the waiting
/// games, and a new wait between games applies from the next one.
#[tauri::command]
pub async fn set_settings(
    handle: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> CommandResult<Settings> {
    settings.validate()?;
    settings.save(&state.layout.settings_file())?;
    *state.settings() = settings.clone();
    crate::queue::advance(&handle).await;
    Ok(settings)
}

#[tauri::command]
pub fn list_queue(state: State<'_, AppState>) -> QueueState {
    state.queue().state()
}

/// Queues `app_id` to run for `duration_secs`, as `executable` or its
/// default one.
#[tauri::command]
pub async fn add_to_queue(
    handle: AppHandle,
    state: State<'_, AppState>,
    app_id: String,
    duration_secs: u64,
    executable: Option<String>,
) -> CommandResult<QueueState> {
    let duration = Duration::from_secs(duration_secs);
    if duration > MAX_DURATION {
        return Err(orbsmith_core::Error::TimerTooLong.into());
    }
    let catalog = catalog(&state, false).await?;
    let app = catalog
        .get(&app_id)
        .ok_or_else(|| orbsmith_core::Error::UnknownApp(app_id.clone()))?;
    match executable.as_deref() {
        Some(name) if app.target_named(Os::CURRENT, name).is_none() => {
            return Err(orbsmith_core::Error::UnknownExecutable {
                app: app.name.clone(),
                executable: name.to_owned(),
            }
            .into());
        }
        None if app.target(Os::CURRENT).is_none() => {
            return Err(orbsmith_core::Error::NoExecutable(app.name.clone()).into());
        }
        _ => {}
    }
    state.queue().add(
        QueueItem::new(app.id.clone(), app.name.clone(), duration).with_executable(executable),
    )?;
    crate::queue::advance(&handle).await;
    crate::queue_changed(&handle);
    Ok(state.queue().state())
}

#[tauri::command]
pub fn remove_from_queue(
    handle: AppHandle,
    state: State<'_, AppState>,
    app_id: String,
) -> QueueState {
    state.queue().remove(&app_id);
    crate::queue_changed(&handle);
    state.queue().state()
}

/// Empties the queue and stops its running games.
#[tauri::command]
pub fn clear_queue(handle: AppHandle, state: State<'_, AppState>) -> QueueState {
    crate::queue::clear(&handle);
    state.queue().state()
}

/// Moves a waiting game one place up or down.
#[tauri::command]
pub fn move_in_queue(
    handle: AppHandle,
    state: State<'_, AppState>,
    app_id: String,
    up: bool,
) -> QueueState {
    if state.queue().move_queued(&app_id, up) {
        crate::queue_changed(&handle);
    }
    state.queue().state()
}

#[tauri::command]
pub async fn start_queue(
    handle: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<QueueState> {
    state.queue().start();
    crate::queue::advance(&handle).await;
    crate::queue_changed(&handle);
    Ok(state.queue().state())
}

/// Pauses the queue: running games keep their timer, and nothing new starts
/// until `start_queue` resumes it.
#[tauri::command]
pub fn pause_queue(handle: AppHandle, state: State<'_, AppState>) -> QueueState {
    crate::queue::pause(&handle);
    state.queue().state()
}

pub(crate) async fn catalog(state: &AppState, refresh: bool) -> CommandResult<Arc<Catalog>> {
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
        Some(cached) if cached.is_stale(SystemTime::now()) => {
            // An outdated list beats none, e.g. when offline.
            match Catalog::fetch(&cache_file).await {
                Ok(fresh) => fresh,
                Err(err) => {
                    log::warn!("keeping the outdated app list: {err}");
                    cached
                }
            }
        }
        Some(cached) => cached,
        None => Catalog::fetch(&cache_file).await?,
    };

    let catalog = Arc::new(catalog);
    *slot = Some(catalog.clone());
    Ok(catalog)
}
