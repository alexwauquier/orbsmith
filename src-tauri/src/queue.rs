//! Runs the queue: starts the games it picks, reports failures, and announces
//! when every game in it is done.

use std::collections::HashSet;

use orbsmith_core::Os;
use tauri::{AppHandle, Manager};

use crate::commands;
use crate::notify;
use crate::state::AppState;

/// Lets the queue move on: called every second, and right after any change
/// to the queue or the settings.
pub async fn advance(app: &AppHandle) {
    let state = app.state::<AppState>();
    let running: HashSet<String> = state
        .simulations()
        .list()
        .into_iter()
        .map(|sim| sim.app_id)
        .collect();
    let parallel = state.settings().parallel_queue;
    let step = state.queue().next(&running, parallel);
    if !step.changed {
        return;
    }

    if !step.start.is_empty() {
        let catalog = commands::catalog(&state, false).await;
        for item in &step.start {
            let result = match &catalog {
                Err(err) => Err(err.message().to_owned()),
                Ok(catalog) => match catalog.get(&item.app_id) {
                    None => Err(format!("{} is no longer in the game list", item.app_name)),
                    Some(detectable) => state
                        .simulations()
                        .start(detectable, Os::CURRENT, Some(item.duration))
                        .map(|_| ())
                        .map_err(|err| err.to_string()),
                },
            };
            if let Err(message) = result {
                log::warn!(
                    "failed to start {} from the queue: {message}",
                    item.app_name
                );
                state.queue().failed(&item.app_id);
                notify::queue_item_failed(app, &item.app_name, &message);
            }
        }
        crate::simulations_changed(app);
    }
    if step.finished {
        notify::queue_finished(app);
    }
    crate::queue_changed(app);
}

/// Interrupts the queue and stops its running games, which go back to the
/// front. Used by Stop all; the queue can then be resumed.
pub fn stop(app: &AppHandle) {
    let state = app.state::<AppState>();
    let stopped = state.queue().stop();
    for item in &stopped {
        // Already gone if it ended in the meantime.
        let _ = state.simulations().stop(&item.app_id);
    }
    crate::simulations_changed(app);
    crate::queue_changed(app);
}
