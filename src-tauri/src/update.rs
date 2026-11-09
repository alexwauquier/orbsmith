//! In-app updates from the GitHub releases, through `tauri-plugin-updater`.
//!
//! The release workflow uploads a `latest.json` manifest listing each
//! package with its signature. An update is only installed if its signature
//! matches the public key in `tauri.conf.json`.

use std::sync::{Mutex, MutexGuard};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::commands::CommandError;
use crate::state::AppState;

/// The update found by the last check, kept for [`install_update`].
#[derive(Default)]
pub struct PendingUpdate(Mutex<Option<Update>>);

impl PendingUpdate {
    fn lock(&self) -> MutexGuard<'_, Option<Update>> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    version: String,
    current_version: String,
    /// The release notes, as Markdown.
    notes: Option<String>,
}

/// Looks for a newer release. Resolves to `None` when Orbsmith is up to date.
#[tauri::command]
pub async fn check_for_update(
    app: AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<Option<UpdateInfo>, CommandError> {
    let handle = app.clone();
    let update = app
        .updater_builder()
        // On Windows the plugin starts the installer, then ends the process
        // with `std::process::exit`, which skips Tauri's cleanup. Release the
        // single-instance lock first, or the installer's relaunch of
        // Orbsmith would take itself for a second instance and quit.
        .on_before_exit(move || tauri_plugin_single_instance::destroy(&handle))
        .build()?
        .check()
        .await?;
    let info = update.as_ref().map(|update| UpdateInfo {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
        notes: update.body.clone(),
    });
    *pending.lock() = update;
    Ok(info)
}

/// Downloads the update found by [`check_for_update`], then stops the games
/// the way quitting does and installs it. Orbsmith restarts on success.
#[tauri::command]
pub async fn install_update(
    app: AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<(), CommandError> {
    let update = pending
        .lock()
        .take()
        .ok_or_else(|| CommandError::from("No update to install. Check for updates again."))?;
    // Games keep running during the download, which may take a while or fail.
    let bytes = update.download(|_, _| {}, || {}).await?;

    // Keeps the queue's time left, as on exit.
    crate::queue::pause(&app);
    app.state::<AppState>().simulations().stop_all();
    crate::simulations_changed(&app);

    update.install(bytes)?;
    // Windows never gets here: the installer replaces the running app and
    // starts the new version itself.
    app.restart()
}
