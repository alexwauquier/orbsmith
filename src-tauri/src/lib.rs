mod commands;
mod notify;
mod queue;
mod state;
mod tray;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use orbsmith_core::{Layout, Os, Queue, Settings, cleanup};
use tauri::utils::config::{AppDirectoriesOverride, AppDirectoryOverrides};
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};

use crate::state::AppState;

/// Emitted, without payload, whenever simulations start or stop, including
/// from the tray or on their own. Listeners reload the list.
const SIMULATIONS_CHANGED_EVENT: &str = "simulations-changed";

/// Emitted, without payload, whenever the queue changes.
const QUEUE_CHANGED_EVENT: &str = "queue-changed";

/// Whether the "still running in the tray" notification was already shown, so
/// that it appears once per run.
static TOLD_HIDDEN: AtomicBool = AtomicBool::new(false);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> tauri::Result<()> {
    let mut context = tauri::generate_context!();
    context.config_mut().app.app_directories_override = Some(app_directories());

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let layout = Layout::new(app.path().app_data_dir()?);
            let killed = cleanup::kill_orphans(&layout.executables_dir());
            if killed > 0 {
                log::info!("cleaned up {killed} orphan dummies");
            }

            let settings = Settings::load(&layout.settings_file()).unwrap_or_else(|err| {
                log::warn!("using default settings: {err}");
                Settings::default()
            });
            let queue = Queue::load(&layout.queue_file()).unwrap_or_else(|err| {
                log::warn!("starting with an empty queue: {err}");
                Queue::new()
            });
            app.manage(AppState::new(layout, dummy_path()?, settings, queue));
            tray::create(app.handle())?;
            tauri::async_runtime::spawn(watch_simulations(app.handle().clone()));
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the window while games run keeps them running in the tray;
            // with nothing running, the app quits as usual.
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let running = app
                    .try_state::<AppState>()
                    .map_or(0, |state| state.simulations().list().len());
                if running > 0 {
                    api.prevent_close();
                    if let Err(err) = window.hide() {
                        log::warn!("failed to hide the window: {err}");
                    }
                    if !TOLD_HIDDEN.swap(true, Ordering::Relaxed) {
                        notify::hidden_to_tray(app, running);
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_apps,
            commands::list_simulations,
            commands::start_simulation,
            commands::stop_simulation,
            commands::set_simulation_timer,
            commands::get_settings,
            commands::set_settings,
            commands::list_queue,
            commands::add_to_queue,
            commands::remove_from_queue,
            commands::move_in_queue,
            commands::start_queue,
            commands::pause_queue,
        ])
        .build(context)?
        .run(|app, event| {
            if let RunEvent::Exit = event {
                // Saves the queue with the time its games had left.
                queue::pause(app);
                app.state::<AppState>().simulations().stop_all();
            }
        });
    Ok(())
}

/// Puts every app directory under one readable folder in the per-user data
/// dir (`%APPDATA%\Orbsmith`, `~/.local/share/orbsmith`) instead of Tauri's
/// defaults, which are named after the bundle identifier. That includes the
/// webview data, which Tauri keeps in the local data dir. `%LOCALAPPDATA%\Orbsmith`
/// is not an option: the NSIS installer puts the program there.
fn app_directories() -> AppDirectoriesOverride {
    let root = PathBuf::from("$DATA").join(Os::CURRENT.data_dir_name());
    AppDirectoriesOverride::Directories(AppDirectoryOverrides {
        config: Some(root.clone()),
        data: Some(root.clone()),
        local_data: Some(root.join("webview")),
        cache: Some(root.join("cache")),
        log: Some(root.join("logs")),
    })
}

/// The dummy ships as a sidecar, next to the main executable.
fn dummy_path() -> std::io::Result<std::path::PathBuf> {
    Ok(std::env::current_exe()?
        .with_file_name(format!("orbsmith-dummy{}", std::env::consts::EXE_SUFFIX)))
}

/// Updates the tray and tells the frontend to reload the simulations. Must be
/// called without holding the simulations lock.
pub(crate) fn simulations_changed(app: &AppHandle) {
    tray::refresh(app);
    if let Err(err) = app.emit(SIMULATIONS_CHANGED_EVENT, ()) {
        log::warn!("failed to emit {SIMULATIONS_CHANGED_EVENT}: {err}");
    }
}

/// Saves the queue, updates the tray and tells the frontend to reload it.
pub(crate) fn queue_changed(app: &AppHandle) {
    let state = app.state::<AppState>();
    if let Err(err) = state.queue().save(&state.layout.queue_file()) {
        log::warn!("failed to save the queue: {err}");
    }
    tray::refresh(app);
    if let Err(err) = app.emit(QUEUE_CHANGED_EVENT, ()) {
        log::warn!("failed to emit {QUEUE_CHANGED_EVENT}: {err}");
    }
}

/// Applies auto-stop timers, notices dummies that exited and moves the
/// queue on, once per second.
async fn watch_simulations(app: AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let ended = app
            .state::<AppState>()
            .simulations()
            .tick(SystemTime::now());
        if !ended.is_empty() {
            for end in &ended {
                notify::simulation_ended(&app, end);
            }
            simulations_changed(&app);
        }
        queue::advance(&app).await;
    }
}
