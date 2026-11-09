//! Desktop notifications, sent from Rust so they show while the window is
//! hidden. The webview has no notification permission.

use orbsmith_core::{EndReason, Ended};
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub fn simulation_ended(app: &AppHandle, ended: &Ended) {
    let body = match ended.reason {
        EndReason::Timer => "The auto-stop timer ran out.",
        EndReason::Exited => "The simulated game exited unexpectedly.",
    };
    show(app, &format!("{} stopped", ended.app_name), body);
}

pub fn hidden_to_tray(app: &AppHandle, running: usize) {
    let games = if running == 1 {
        "1 game keeps".to_owned()
    } else {
        format!("{running} games keep")
    };
    show(
        app,
        "Orbsmith is still running",
        &format!("{games} running. Open Orbsmith from the tray icon to stop them."),
    );
}

fn show(app: &AppHandle, title: &str, body: &str) {
    if let Err(err) = app.notification().builder().title(title).body(body).show() {
        log::warn!("failed to show a notification: {err}");
    }
}
