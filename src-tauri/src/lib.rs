mod commands;
mod state;
mod tray;

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use orbsmith_core::{Layout, Os, cleanup};
use tauri::utils::config::{AppDirectoriesOverride, AppDirectoryOverrides};
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};

use crate::state::AppState;

/// Emitted, without payload, whenever simulations start or stop, including
/// from the tray or on their own. Listeners reload the list.
const SIMULATIONS_CHANGED_EVENT: &str = "simulations-changed";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> tauri::Result<()> {
    let mut context = tauri::generate_context!();
    context.config_mut().app.app_directories_override = Some(app_directories());

    tauri::Builder::default()
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

            app.manage(AppState::new(layout, dummy_path()?));
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
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_apps,
            commands::list_simulations,
            commands::start_simulation,
            commands::stop_simulation,
            commands::set_simulation_timer,
        ])
        .build(context)?
        .run(|app, event| {
            if let RunEvent::Exit = event {
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

/// Applies auto-stop timers and notices dummies that exited, once per second.
async fn watch_simulations(app: AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let ended = app
            .state::<AppState>()
            .simulations()
            .tick(SystemTime::now());
        if ended.is_empty() {
            continue;
        }
        simulations_changed(&app);
    }
}
