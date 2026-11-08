mod commands;
mod state;

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use orbsmith_core::{Layout, Os, cleanup};
use tauri::utils::config::{AppDirectoriesOverride, AppDirectoryOverrides};
use tauri::{AppHandle, Emitter, Manager, RunEvent};

use crate::state::AppState;

/// Emitted with an [`orbsmith_core::Ended`] payload when a simulation stops on
/// its own (timer or dummy exit).
const SIMULATION_ENDED_EVENT: &str = "simulation-ended";

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
            tauri::async_runtime::spawn(watch_simulations(app.handle().clone()));
            Ok(())
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

/// Applies auto-stop timers and notices dummies that exited, once per second.
async fn watch_simulations(app: AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let ended = app
            .state::<AppState>()
            .simulations()
            .tick(SystemTime::now());
        for end in ended {
            if let Err(err) = app.emit(SIMULATION_ENDED_EVENT, &end) {
                log::warn!("failed to emit {SIMULATION_ENDED_EVENT}: {err}");
            }
        }
    }
}
