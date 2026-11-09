//! Runs the quest queue: starts the games it picks, reports failures, and
//! announces when every quest is done.

use std::collections::HashSet;

use orbsmith_core::Os;
use tauri::{AppHandle, Manager};

use crate::commands;
use crate::notify;
use crate::state::AppState;

/// Lets the queue move on: called every second, and right after any change
/// to the quests or the settings.
pub async fn advance(app: &AppHandle) {
    let state = app.state::<AppState>();
    let running: HashSet<String> = state
        .simulations()
        .list()
        .into_iter()
        .map(|sim| sim.app_id)
        .collect();
    let parallel = state.settings().quests_in_parallel;
    let step = state.quests().next(&running, parallel);
    if !step.changed {
        return;
    }

    if !step.start.is_empty() {
        let catalog = commands::catalog(&state, false).await;
        for quest in &step.start {
            let result = match &catalog {
                Err(err) => Err(err.message().to_owned()),
                Ok(catalog) => match catalog.get(&quest.app_id) {
                    None => Err(format!("{} is no longer in the game list", quest.app_name)),
                    Some(detectable) => state
                        .simulations()
                        .start(detectable, Os::CURRENT, Some(quest.duration))
                        .map(|_| ())
                        .map_err(|err| err.to_string()),
                },
            };
            if let Err(message) = result {
                log::warn!(
                    "failed to start the quest for {}: {message}",
                    quest.app_name
                );
                state.quests().failed(&quest.app_id);
                notify::quest_failed(app, &quest.app_name, &message);
            }
        }
        crate::simulations_changed(app);
    }
    if step.finished {
        notify::quests_done(app);
    }
    crate::quests_changed(app);
}

/// Pauses the queue and stops the games of its active quests.
pub fn stop(app: &AppHandle) {
    let state = app.state::<AppState>();
    let stopped = state.quests().stop();
    for quest in &stopped {
        // Already gone if it ended in the meantime.
        let _ = state.simulations().stop(&quest.app_id);
    }
    crate::simulations_changed(app);
    crate::quests_changed(app);
}
