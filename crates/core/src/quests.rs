//! The quest queue: games to play for a given time, started one after another
//! or all at once.
//!
//! The queue only decides what to start. The caller owns the simulations: it
//! starts what [`QuestQueue::next`] returns and reports failures back.

use std::collections::{HashSet, VecDeque};
use std::time::Duration;

use serde::Serialize;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quest {
    pub app_id: String,
    pub app_name: String,
    /// How long the game runs, auto-stop margin included.
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestInfo {
    pub app_id: String,
    pub app_name: String,
    pub duration_secs: u64,
}

impl From<&Quest> for QuestInfo {
    fn from(quest: &Quest) -> Self {
        Self {
            app_id: quest.app_id.clone(),
            app_name: quest.app_name.clone(),
            duration_secs: quest.duration.as_secs(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestsState {
    /// Whether the queue is being worked through.
    pub running: bool,
    /// Quests whose game was started and is still running.
    pub active: Vec<QuestInfo>,
    /// Quests waiting their turn, in order.
    pub queued: Vec<QuestInfo>,
}

/// What [`QuestQueue::next`] decided.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Step {
    /// Quests to start now. They already count as active; report the ones
    /// that fail with [`QuestQueue::failed`].
    pub start: Vec<Quest>,
    /// The last quest just ended: the queue is done.
    pub finished: bool,
    /// Whether the state changed, so listeners need updating.
    pub changed: bool,
}

#[derive(Debug, Default)]
pub struct QuestQueue {
    queued: VecDeque<Quest>,
    active: Vec<Quest>,
    running: bool,
}

impl QuestQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a quest. A game can only be queued or active once.
    pub fn add(&mut self, quest: Quest) -> Result<(), Error> {
        if self.contains(&quest.app_id) {
            return Err(Error::QuestAlreadyAdded(quest.app_name));
        }
        self.queued.push_back(quest);
        Ok(())
    }

    /// Removes a quest that has not started yet.
    pub fn remove(&mut self, app_id: &str) -> Option<Quest> {
        let index = self
            .queued
            .iter()
            .position(|quest| quest.app_id == app_id)?;
        self.queued.remove(index)
    }

    pub fn start(&mut self) {
        self.running = !self.queued.is_empty() || !self.active.is_empty();
    }

    /// Pauses the queue. Active quests go back to the front, in order, and are
    /// returned so the caller can stop their games.
    pub fn stop(&mut self) -> Vec<Quest> {
        self.running = false;
        let active = std::mem::take(&mut self.active);
        for quest in active.iter().rev() {
            self.queued.push_front(quest.clone());
        }
        active
    }

    /// Forgets active quests whose game no longer runs, since they are done or
    /// were stopped, then picks the quests to start. `running_apps` holds the
    /// IDs of every running simulation. A quest whose game already runs waits.
    pub fn next(&mut self, running_apps: &HashSet<String>, parallel: bool) -> Step {
        let before = self.active.len();
        self.active
            .retain(|quest| running_apps.contains(&quest.app_id));
        let mut step = Step {
            changed: self.active.len() != before,
            ..Step::default()
        };
        if !self.running {
            return step;
        }

        if parallel {
            let mut waiting = VecDeque::new();
            while let Some(quest) = self.queued.pop_front() {
                if running_apps.contains(&quest.app_id) {
                    waiting.push_back(quest);
                } else {
                    step.start.push(quest);
                }
            }
            self.queued = waiting;
        } else if self.active.is_empty()
            && self
                .queued
                .front()
                .is_some_and(|quest| !running_apps.contains(&quest.app_id))
            && let Some(quest) = self.queued.pop_front()
        {
            step.start.push(quest);
        }

        self.active.extend(step.start.iter().cloned());
        if self.active.is_empty() && self.queued.is_empty() {
            self.running = false;
            step.finished = true;
        }
        step.changed |= !step.start.is_empty() || step.finished;
        step
    }

    /// Drops an active quest whose game could not be started.
    pub fn failed(&mut self, app_id: &str) {
        self.active.retain(|quest| quest.app_id != app_id);
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Quests not finished yet: active and queued.
    pub fn remaining(&self) -> usize {
        self.active.len() + self.queued.len()
    }

    pub fn state(&self) -> QuestsState {
        QuestsState {
            running: self.running,
            active: self.active.iter().map(QuestInfo::from).collect(),
            queued: self.queued.iter().map(QuestInfo::from).collect(),
        }
    }

    fn contains(&self, app_id: &str) -> bool {
        self.queued
            .iter()
            .chain(&self.active)
            .any(|quest| quest.app_id == app_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quest(id: &str) -> Quest {
        Quest {
            app_id: id.to_owned(),
            app_name: format!("Game {id}"),
            duration: Duration::from_secs(60),
        }
    }

    fn queue(ids: &[&str]) -> QuestQueue {
        let mut queue = QuestQueue::new();
        for id in ids {
            queue.add(quest(id)).unwrap();
        }
        queue
    }

    fn running(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(|id| (*id).to_owned()).collect()
    }

    fn started(step: &Step) -> Vec<&str> {
        step.start
            .iter()
            .map(|quest| quest.app_id.as_str())
            .collect()
    }

    #[test]
    fn rejects_the_same_game_twice() {
        let mut queue = queue(&["a"]);
        assert!(matches!(
            queue.add(quest("a")),
            Err(Error::QuestAlreadyAdded(_))
        ));
        queue.start();
        queue.next(&running(&[]), false);
        assert!(queue.add(quest("a")).is_err(), "active quests count too");
    }

    #[test]
    fn does_nothing_until_started() {
        let mut queue = queue(&["a"]);
        assert_eq!(queue.next(&running(&[]), false), Step::default());
        assert!(!queue.is_running());
    }

    #[test]
    fn runs_quests_one_after_another() {
        let mut queue = queue(&["a", "b"]);
        queue.start();

        let step = queue.next(&running(&[]), false);
        assert_eq!(started(&step), ["a"]);
        assert!(step.changed);
        assert_eq!(queue.next(&running(&["a"]), false), Step::default());

        // "a" ended: "b" starts.
        assert_eq!(started(&queue.next(&running(&[]), false)), ["b"]);

        let step = queue.next(&running(&[]), false);
        assert!(step.start.is_empty());
        assert!(step.finished && step.changed);
        assert!(!queue.is_running());
        assert_eq!(queue.remaining(), 0);
    }

    #[test]
    fn runs_quests_in_parallel() {
        let mut queue = queue(&["a", "b", "c"]);
        queue.start();
        assert_eq!(started(&queue.next(&running(&[]), true)), ["a", "b", "c"]);
        assert_eq!(queue.remaining(), 3);

        let step = queue.next(&running(&["b"]), true);
        assert!(step.changed && !step.finished);
        assert_eq!(queue.remaining(), 1);
        assert!(queue.next(&running(&[]), true).finished);
    }

    #[test]
    fn waits_while_the_game_already_runs() {
        let mut queue = queue(&["a", "b"]);
        queue.start();
        // Sequential: "a" waits, and so does everything behind it.
        assert!(queue.next(&running(&["a"]), false).start.is_empty());
        // Parallel: only "a" waits.
        assert_eq!(started(&queue.next(&running(&["a"]), true)), ["b"]);
        assert_eq!(queue.state().queued[0].app_id, "a");
    }

    #[test]
    fn a_failed_start_moves_on_to_the_next_quest() {
        let mut queue = queue(&["a", "b"]);
        queue.start();
        assert_eq!(started(&queue.next(&running(&[]), false)), ["a"]);
        queue.failed("a");
        assert_eq!(started(&queue.next(&running(&[]), false)), ["b"]);
    }

    #[test]
    fn stopping_puts_active_quests_back_in_front() {
        let mut queue = queue(&["a", "b", "c"]);
        queue.start();
        queue.next(&running(&[]), true);
        queue.add(quest("d")).unwrap();

        let stopped = queue.stop();
        assert_eq!(stopped.len(), 3);
        assert!(!queue.is_running());
        let order: Vec<_> = queue.state().queued.into_iter().map(|q| q.app_id).collect();
        assert_eq!(order, ["a", "b", "c", "d"]);
        assert!(queue.next(&running(&[]), false).start.is_empty());
    }

    #[test]
    fn removes_only_queued_quests() {
        let mut queue = queue(&["a", "b"]);
        queue.start();
        queue.next(&running(&[]), false);
        assert!(queue.remove("a").is_none(), "a is active");
        assert_eq!(queue.remove("b").map(|q| q.app_id), Some("b".to_owned()));
        assert!(queue.remove("b").is_none());
    }

    #[test]
    fn starting_an_empty_queue_does_nothing() {
        let mut queue = QuestQueue::new();
        queue.start();
        assert!(!queue.is_running());
    }

    #[test]
    fn exposes_durations_in_seconds() {
        let queue = queue(&["a"]);
        assert_eq!(
            queue.state(),
            QuestsState {
                running: false,
                active: vec![],
                queued: vec![QuestInfo {
                    app_id: "a".into(),
                    app_name: "Game a".into(),
                    duration_secs: 60,
                }],
            }
        );
    }
}
