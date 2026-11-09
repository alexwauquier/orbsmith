//! The queue: games to play for a given time, started one after another
//! or all at once.
//!
//! The queue only decides what to start. The caller owns the simulations: it
//! starts what [`Queue::next`] returns and reports failures back.

use std::collections::{HashSet, VecDeque};
use std::time::Duration;

use serde::Serialize;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueItem {
    pub app_id: String,
    pub app_name: String,
    /// How long the game runs.
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItemInfo {
    pub app_id: String,
    pub app_name: String,
    pub duration_secs: u64,
}

impl From<&QueueItem> for QueueItemInfo {
    fn from(item: &QueueItem) -> Self {
        Self {
            app_id: item.app_id.clone(),
            app_name: item.app_name.clone(),
            duration_secs: item.duration.as_secs(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueState {
    /// Whether the queue is being worked through.
    pub running: bool,
    /// Whether it was paused with items left: active games keep running, and
    /// nothing new starts until it is resumed.
    pub paused: bool,
    /// Items whose game was started and is still running.
    pub active: Vec<QueueItemInfo>,
    /// Items waiting their turn, in order.
    pub queued: Vec<QueueItemInfo>,
}

/// What [`Queue::next`] decided.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Step {
    /// Items to start now. They already count as active; report the ones
    /// that fail with [`Queue::failed`].
    pub start: Vec<QueueItem>,
    /// The last item just ended: the queue is done.
    pub finished: bool,
    /// Whether the state changed, so listeners need updating.
    pub changed: bool,
}

#[derive(Debug, Default)]
pub struct Queue {
    queued: VecDeque<QueueItem>,
    active: Vec<QueueItem>,
    running: bool,
    paused: bool,
}

impl Queue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an item. A game can only be queued or active once.
    pub fn add(&mut self, item: QueueItem) -> Result<(), Error> {
        if self.contains(&item.app_id) {
            return Err(Error::AlreadyQueued(item.app_name));
        }
        self.queued.push_back(item);
        Ok(())
    }

    /// Removes an item that has not started yet.
    pub fn remove(&mut self, app_id: &str) -> Option<QueueItem> {
        let index = self.queued.iter().position(|item| item.app_id == app_id)?;
        self.queued.remove(index)
    }

    /// Starts the queue, or resumes it after a pause.
    pub fn start(&mut self) {
        self.running = self.remaining() > 0;
        self.paused = false;
    }

    /// Stops starting new items. Active items keep their game and timer; when
    /// they end, the queue waits for [`Queue::start`].
    pub fn pause(&mut self) {
        if self.running {
            self.running = false;
            self.paused = true;
        }
    }

    /// Interrupts the queue. Active items go back to the front, in order, and
    /// are returned so the caller can stop their games.
    pub fn stop(&mut self) -> Vec<QueueItem> {
        self.running = false;
        self.paused = self.remaining() > 0;
        let active = std::mem::take(&mut self.active);
        for item in active.iter().rev() {
            self.queued.push_front(item.clone());
        }
        active
    }

    /// Forgets active items whose game no longer runs, since they are done or
    /// were stopped, then picks the items to start. `running_apps` holds the
    /// IDs of every running simulation. An item whose game already runs waits.
    pub fn next(&mut self, running_apps: &HashSet<String>, parallel: bool) -> Step {
        let before = self.active.len();
        self.active
            .retain(|item| running_apps.contains(&item.app_id));
        let mut step = Step {
            changed: self.active.len() != before,
            ..Step::default()
        };
        if !self.running {
            // Paused with nothing left: there is nothing to resume.
            if self.paused && self.remaining() == 0 {
                self.paused = false;
                step.changed = true;
            }
            return step;
        }

        if parallel {
            let mut waiting = VecDeque::new();
            while let Some(item) = self.queued.pop_front() {
                if running_apps.contains(&item.app_id) {
                    waiting.push_back(item);
                } else {
                    step.start.push(item);
                }
            }
            self.queued = waiting;
        } else if self.active.is_empty()
            && self
                .queued
                .front()
                .is_some_and(|item| !running_apps.contains(&item.app_id))
            && let Some(item) = self.queued.pop_front()
        {
            step.start.push(item);
        }

        self.active.extend(step.start.iter().cloned());
        if self.active.is_empty() && self.queued.is_empty() {
            self.running = false;
            step.finished = true;
        }
        step.changed |= !step.start.is_empty() || step.finished;
        step
    }

    /// Drops an active item whose game could not be started.
    pub fn failed(&mut self, app_id: &str) {
        self.active.retain(|item| item.app_id != app_id);
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Items not finished yet: active and queued.
    pub fn remaining(&self) -> usize {
        self.active.len() + self.queued.len()
    }

    pub fn state(&self) -> QueueState {
        QueueState {
            running: self.running,
            paused: self.paused,
            active: self.active.iter().map(QueueItemInfo::from).collect(),
            queued: self.queued.iter().map(QueueItemInfo::from).collect(),
        }
    }

    fn contains(&self, app_id: &str) -> bool {
        self.queued
            .iter()
            .chain(&self.active)
            .any(|item| item.app_id == app_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str) -> QueueItem {
        QueueItem {
            app_id: id.to_owned(),
            app_name: format!("Game {id}"),
            duration: Duration::from_secs(60),
        }
    }

    fn queue(ids: &[&str]) -> Queue {
        let mut queue = Queue::new();
        for id in ids {
            queue.add(item(id)).unwrap();
        }
        queue
    }

    fn running(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(|id| (*id).to_owned()).collect()
    }

    fn started(step: &Step) -> Vec<&str> {
        step.start.iter().map(|item| item.app_id.as_str()).collect()
    }

    #[test]
    fn rejects_the_same_game_twice() {
        let mut queue = queue(&["a"]);
        assert!(matches!(queue.add(item("a")), Err(Error::AlreadyQueued(_))));
        queue.start();
        queue.next(&running(&[]), false);
        assert!(queue.add(item("a")).is_err(), "active items count too");
    }

    #[test]
    fn does_nothing_until_started() {
        let mut queue = queue(&["a"]);
        assert_eq!(queue.next(&running(&[]), false), Step::default());
        assert!(!queue.is_running());
    }

    #[test]
    fn runs_items_one_after_another() {
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
    fn runs_items_in_parallel() {
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
    fn a_failed_start_moves_on_to_the_next_item() {
        let mut queue = queue(&["a", "b"]);
        queue.start();
        assert_eq!(started(&queue.next(&running(&[]), false)), ["a"]);
        queue.failed("a");
        assert_eq!(started(&queue.next(&running(&[]), false)), ["b"]);
    }

    #[test]
    fn stopping_puts_active_items_back_in_front() {
        let mut queue = queue(&["a", "b", "c"]);
        queue.start();
        queue.next(&running(&[]), true);
        queue.add(item("d")).unwrap();

        let stopped = queue.stop();
        assert_eq!(stopped.len(), 3);
        assert!(!queue.is_running());
        let order: Vec<_> = queue.state().queued.into_iter().map(|q| q.app_id).collect();
        assert_eq!(order, ["a", "b", "c", "d"]);
        assert!(queue.next(&running(&[]), false).start.is_empty());
        assert!(queue.is_paused(), "stopped items can be resumed");
    }

    #[test]
    fn pausing_keeps_the_running_game_and_waits() {
        let mut queue = queue(&["a", "b"]);
        queue.start();
        assert_eq!(started(&queue.next(&running(&[]), false)), ["a"]);

        queue.pause();
        assert!(queue.is_paused() && !queue.is_running());
        assert_eq!(queue.state().active[0].app_id, "a", "a keeps running");
        assert_eq!(queue.next(&running(&["a"]), false), Step::default());

        // "a" ends while paused: it is done, and "b" waits.
        let step = queue.next(&running(&[]), false);
        assert!(step.start.is_empty() && step.changed && !step.finished);
        assert_eq!(queue.remaining(), 1);
        assert!(queue.is_paused());

        queue.start();
        assert!(!queue.is_paused());
        assert_eq!(started(&queue.next(&running(&[]), false)), ["b"]);
    }

    #[test]
    fn resuming_while_the_game_still_runs_keeps_its_place() {
        let mut queue = queue(&["a", "b"]);
        queue.start();
        queue.next(&running(&[]), false);
        queue.pause();
        queue.start();
        // "a" is still active, so "b" keeps waiting for it.
        assert!(queue.next(&running(&["a"]), false).start.is_empty());
        assert_eq!(started(&queue.next(&running(&[]), false)), ["b"]);
    }

    #[test]
    fn a_pause_with_nothing_left_clears_itself() {
        let mut queue = queue(&["a"]);
        queue.start();
        queue.next(&running(&[]), false);
        queue.pause();
        let step = queue.next(&running(&[]), false);
        assert!(step.changed && !step.finished);
        assert!(!queue.is_paused() && queue.remaining() == 0);
    }

    #[test]
    fn pausing_an_idle_queue_does_nothing() {
        let mut queue = queue(&["a"]);
        queue.pause();
        assert!(!queue.is_paused());
    }

    #[test]
    fn removes_only_queued_items() {
        let mut queue = queue(&["a", "b"]);
        queue.start();
        queue.next(&running(&[]), false);
        assert!(queue.remove("a").is_none(), "a is active");
        assert_eq!(queue.remove("b").map(|q| q.app_id), Some("b".to_owned()));
        assert!(queue.remove("b").is_none());
    }

    #[test]
    fn starting_an_empty_queue_does_nothing() {
        let mut queue = Queue::new();
        queue.start();
        assert!(!queue.is_running());
    }

    #[test]
    fn exposes_durations_in_seconds() {
        let queue = queue(&["a"]);
        assert_eq!(
            queue.state(),
            QueueState {
                running: false,
                paused: false,
                active: vec![],
                queued: vec![QueueItemInfo {
                    app_id: "a".into(),
                    app_name: "Game a".into(),
                    duration_secs: 60,
                }],
            }
        );
    }
}
