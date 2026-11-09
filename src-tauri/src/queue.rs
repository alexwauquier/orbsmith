//! Runs the queue: starts the games it picks, reports failures, and announces
//! when every game in it is done.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
                        .start(detectable, Os::CURRENT, Some(item.run_time()))
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

/// Pauses the queue: its running games stop and go back to the front with
/// the time their timer had left, so they resume where they stopped.
pub fn pause(app: &AppHandle) {
    let state = app.state::<AppState>();
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis());
    let left: HashMap<String, Duration> = state
        .simulations()
        .list()
        .into_iter()
        .filter_map(|sim| {
            let stop_at = u128::from(sim.stop_at?);
            let left_ms = u64::try_from(stop_at.saturating_sub(now_ms)).ok()?;
            Some((sim.app_id, Duration::from_millis(left_ms)))
        })
        .collect();
    let stopped = state.queue().pause(&left);
    for item in &stopped {
        // Already gone if it ended in the meantime.
        let _ = state.simulations().stop(&item.app_id);
    }
    crate::simulations_changed(app);
    crate::queue_changed(app);
}
