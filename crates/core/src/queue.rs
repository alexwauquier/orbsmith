//! The queue: games to play for a given time, started one after another
//! or all at once.
//!
//! The queue only decides what to start. The caller owns the simulations: it
//! starts what [`Queue::next`] returns and reports failures back.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueItem {
    pub app_id: String,
    pub app_name: String,
    /// How long the game runs.
    pub duration: Duration,
    /// Time left after a pause interrupted the game; it runs for this long
    /// when resumed.
    pub left: Option<Duration>,
    /// The executable to run, if not the app's default.
    pub executable: Option<String>,
}

impl QueueItem {
    pub fn new(app_id: impl Into<String>, app_name: impl Into<String>, duration: Duration) -> Self {
        Self {
            app_id: app_id.into(),
            app_name: app_name.into(),
            duration,
            left: None,
            executable: None,
        }
    }

    /// Runs as `executable` instead of the app's default one.
    pub fn with_executable(mut self, executable: Option<String>) -> Self {
        self.executable = executable;
        self
    }

    /// How long the game should run when started: the time left after a
    /// pause, or the whole duration.
    pub fn run_time(&self) -> Duration {
        self.left.unwrap_or(self.duration)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItemInfo {
    pub app_id: String,
    pub app_name: String,
    pub duration_secs: u64,
    /// Time left after a pause, if the game was interrupted.
    pub left_secs: Option<u64>,
}

impl From<&QueueItem> for QueueItemInfo {
    fn from(item: &QueueItem) -> Self {
        Self {
            app_id: item.app_id.clone(),
            app_name: item.app_name.clone(),
            duration_secs: item.duration.as_secs(),
            left_secs: item.left.map(|left| left.as_secs()),
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

    /// Empties the queue and resets it, as if never started. Returns the items
    /// that were active, so the caller can stop their games.
    pub fn clear(&mut self) -> Vec<QueueItem> {
        self.queued.clear();
        self.running = false;
        self.paused = false;
        std::mem::take(&mut self.active)
    }

    /// Moves a waiting item one place towards the front (`up`) or the back.
    /// Returns whether it moved; it can't move past either end.
    pub fn move_queued(&mut self, app_id: &str, up: bool) -> bool {
        let Some(index) = self.queued.iter().position(|item| item.app_id == app_id) else {
            return false;
        };
        let Some(target) = (if up {
            index.checked_sub(1)
        } else {
            Some(index + 1).filter(|&next| next < self.queued.len())
        }) else {
            return false;
        };
        self.queued.swap(index, target);
        true
    }

    /// The queue saved in `file`. One that was running or paused comes back
    /// paused, so the user decides when it resumes. Active items come back first, with the time they had left if the
    /// queue was paused when saved, or their whole duration after a crash.
    pub fn load(file: &Path) -> Result<Self, Error> {
        let saved: SavedQueue = match fs::read(file) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(Error::InvalidQueueFile)?,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Self::new()),
            Err(err) => return Err(err.into()),
        };
        let mut queue = Self::new();
        for item in saved.items {
            // Skip duplicates rather than failing on a hand-edited file.
            let _ = queue.add(item.into());
        }
        queue.paused = saved.paused && !queue.queued.is_empty();
        Ok(queue)
    }

    /// Writes every unfinished item, active ones first, through a temporary
    /// file.
    pub fn save(&self, file: &Path) -> Result<(), Error> {
        let saved = SavedQueue {
            paused: self.running || self.paused,
            items: self
                .active
                .iter()
                .chain(&self.queued)
                .map(SavedItem::from)
                .collect(),
        };
        if let Some(dir) = file.parent() {
            fs::create_dir_all(dir)?;
        }
        let json = serde_json::to_vec_pretty(&saved).map_err(Error::InvalidQueueFile)?;
        let tmp = file.with_extension("json.tmp");
        fs::write(&tmp, json)?;
        fs::rename(&tmp, file)?;
        Ok(())
    }

    /// Starts the queue, or resumes it after a pause.
    pub fn start(&mut self) {
        self.running = self.remaining() > 0;
        self.paused = false;
    }

    /// Pauses the queue. Active items go back to the front, in order, keeping
    /// the time `left` for their game so they resume where they stopped; an
    /// item with no time left is done. Returns the items that were active, so
    /// the caller can stop their games. [`Queue::start`] resumes.
    pub fn pause(&mut self, left: &HashMap<String, Duration>) -> Vec<QueueItem> {
        let was_active = self.running || !self.active.is_empty();
        let active = std::mem::take(&mut self.active);
        let interrupted = active.iter().filter_map(|item| {
            let time = left.get(&item.app_id).filter(|time| !time.is_zero())?;
            Some(QueueItem {
                left: Some(*time),
                ..item.clone()
            })
        });
        for item in interrupted.collect::<Vec<_>>().into_iter().rev() {
            self.queued.push_front(item);
        }
        self.running = false;
        self.paused = was_active && !self.queued.is_empty();
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

/// On-disk form of the queue, in `queue.json`.
#[derive(Debug, Serialize, Deserialize)]
struct SavedQueue {
    /// Whether it was running or paused, rather than never started.
    #[serde(default)]
    paused: bool,
    items: Vec<SavedItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedItem {
    app_id: String,
    app_name: String,
    duration_secs: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    left_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    executable: Option<String>,
}

impl From<&QueueItem> for SavedItem {
    fn from(item: &QueueItem) -> Self {
        Self {
            app_id: item.app_id.clone(),
            app_name: item.app_name.clone(),
            duration_secs: item.duration.as_secs(),
            left_ms: item
                .left
                .map(|left| u64::try_from(left.as_millis()).unwrap_or(u64::MAX)),
            executable: item.executable.clone(),
        }
    }
}

impl From<SavedItem> for QueueItem {
    fn from(saved: SavedItem) -> Self {
        Self {
            left: saved.left_ms.map(Duration::from_millis),
            ..Self::new(
                saved.app_id,
                saved.app_name,
                Duration::from_secs(saved.duration_secs),
            )
            .with_executable(saved.executable)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str) -> QueueItem {
        QueueItem::new(id, format!("Game {id}"), Duration::from_secs(60))
    }

    fn left(entries: &[(&str, u64)]) -> HashMap<String, Duration> {
        entries
            .iter()
            .map(|(id, secs)| ((*id).to_owned(), Duration::from_secs(*secs)))
            .collect()
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
    fn pausing_keeps_the_time_left_and_resumes_with_it() {
        let mut queue = queue(&["a", "b"]);
        queue.start();
        assert_eq!(started(&queue.next(&running(&[]), false)), ["a"]);

        let stopped = queue.pause(&left(&[("a", 42)]));
        assert_eq!(stopped.len(), 1, "a's game must be stopped");
        assert!(queue.is_paused() && !queue.is_running());
        let state = queue.state();
        assert!(state.active.is_empty());
        assert_eq!(state.queued[0].app_id, "a");
        assert_eq!(state.queued[0].left_secs, Some(42));
        assert!(
            queue.next(&running(&[]), false).start.is_empty(),
            "nothing starts"
        );

        queue.start();
        assert!(!queue.is_paused());
        let step = queue.next(&running(&[]), false);
        assert_eq!(started(&step), ["a"]);
        assert_eq!(step.start[0].run_time(), Duration::from_secs(42));
    }

    #[test]
    fn pausing_in_parallel_keeps_every_game_in_order() {
        let mut queue = queue(&["a", "b", "c"]);
        queue.start();
        queue.next(&running(&[]), true);
        queue.add(item("d")).unwrap();

        let stopped = queue.pause(&left(&[("a", 10), ("b", 20), ("c", 30)]));
        assert_eq!(stopped.len(), 3);
        let state = queue.state();
        let order: Vec<_> = state.queued.iter().map(|q| q.app_id.as_str()).collect();
        assert_eq!(order, ["a", "b", "c", "d"]);
        assert_eq!(state.queued[3].left_secs, None);
    }

    #[test]
    fn an_item_with_no_time_left_is_done_when_paused() {
        let mut queue = queue(&["a", "b"]);
        queue.start();
        queue.next(&running(&[]), false);
        queue.pause(&left(&[("a", 0)]));
        let order: Vec<_> = queue.state().queued.into_iter().map(|q| q.app_id).collect();
        assert_eq!(order, ["b"]);
    }

    #[test]
    fn a_pause_with_nothing_left_clears_itself() {
        let mut queue = queue(&["a", "b"]);
        queue.start();
        queue.next(&running(&[]), false);
        queue.pause(&left(&[("a", 5)]));
        queue.remove("a");
        queue.remove("b");
        let step = queue.next(&running(&[]), false);
        assert!(step.changed && !step.finished);
        assert!(!queue.is_paused());
    }

    #[test]
    fn pausing_an_idle_queue_does_nothing() {
        let mut queue = queue(&["a"]);
        assert!(queue.pause(&left(&[])).is_empty());
        assert!(!queue.is_paused());
        assert_eq!(queue.remaining(), 1);
    }

    #[test]
    fn saves_and_restores_paused_with_the_time_left() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("queue.json");
        let mut queue = queue(&["a", "b"]);
        queue.start();
        queue.next(&running(&[]), false);
        queue.pause(&left(&[("a", 42)]));
        queue.save(&file).unwrap();
        assert!(!file.with_extension("json.tmp").exists());

        let mut restored = Queue::load(&file).unwrap();
        assert!(restored.is_paused() && !restored.is_running());
        assert_eq!(restored.state().queued, queue.state().queued);
        restored.start();
        let step = restored.next(&running(&[]), false);
        assert_eq!(step.start[0].run_time(), Duration::from_secs(42));
    }

    #[test]
    fn restores_active_items_first_after_a_crash() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("queue.json");
        let mut queue = queue(&["a", "b"]);
        queue.start();
        queue.next(&running(&[]), false);
        // Saved while "a" was running, then Orbsmith died.
        queue.save(&file).unwrap();

        let restored = Queue::load(&file).unwrap();
        assert!(restored.is_paused(), "it was running");
        let state = restored.state();
        let order: Vec<_> = state.queued.iter().map(|q| q.app_id.as_str()).collect();
        assert_eq!(order, ["a", "b"]);
        assert_eq!(
            state.queued[0].left_secs, None,
            "a restarts from the beginning"
        );
    }

    #[test]
    fn keeps_the_chosen_executable_across_restarts() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("queue.json");
        let mut queue = Queue::new();
        queue
            .add(item("a").with_executable(Some("league of legends.exe".into())))
            .unwrap();
        queue.add(item("b")).unwrap();
        queue.save(&file).unwrap();

        let mut restored = Queue::load(&file).unwrap();
        restored.start();
        let step = restored.next(&running(&[]), true);
        assert_eq!(
            step.start[0].executable.as_deref(),
            Some("league of legends.exe")
        );
        assert_eq!(step.start[1].executable, None);
    }

    #[test]
    fn a_queue_never_started_is_not_restored_as_paused() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("queue.json");
        queue(&["a"]).save(&file).unwrap();
        let restored = Queue::load(&file).unwrap();
        assert!(!restored.is_paused());
        assert_eq!(restored.remaining(), 1);
    }

    #[test]
    fn loads_an_empty_queue_without_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let queue = Queue::load(&dir.path().join("queue.json")).unwrap();
        assert_eq!(queue.remaining(), 0);
        assert!(!queue.is_paused());
    }

    #[test]
    fn reports_a_corrupt_file_and_skips_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("queue.json");
        fs::write(&file, "not json").unwrap();
        assert!(matches!(
            Queue::load(&file),
            Err(Error::InvalidQueueFile(_))
        ));

        fs::write(
            &file,
            r#"{ "items": [
                { "appId": "a", "appName": "A", "durationSecs": 60 },
                { "appId": "a", "appName": "A", "durationSecs": 60 }
            ] }"#,
        )
        .unwrap();
        assert_eq!(Queue::load(&file).unwrap().remaining(), 1);
    }

    #[test]
    fn moves_waiting_items_up_and_down() {
        let mut queue = queue(&["a", "b", "c"]);
        let order = |queue: &Queue| -> Vec<String> {
            queue.state().queued.into_iter().map(|q| q.app_id).collect()
        };
        assert!(queue.move_queued("c", true));
        assert_eq!(order(&queue), ["a", "c", "b"]);
        assert!(queue.move_queued("a", false));
        assert_eq!(order(&queue), ["c", "a", "b"]);
        assert!(!queue.move_queued("c", true), "already first");
        assert!(!queue.move_queued("b", false), "already last");
        assert!(!queue.move_queued("x", true), "unknown");
    }

    #[test]
    fn active_items_do_not_move() {
        let mut queue = queue(&["a", "b"]);
        queue.start();
        queue.next(&running(&[]), false);
        assert!(!queue.move_queued("a", false));
    }

    #[test]
    fn clearing_empties_and_resets_the_queue() {
        let mut queue = queue(&["a", "b", "c"]);
        queue.start();
        queue.next(&running(&[]), false);
        let stopped = queue.clear();
        assert_eq!(stopped.len(), 1, "a's game must be stopped");
        assert_eq!(queue.remaining(), 0);
        assert!(!queue.is_running() && !queue.is_paused());
        assert!(queue.next(&running(&[]), false).start.is_empty());
        queue.add(item("a")).unwrap();
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
                    left_secs: None,
                }],
            }
        );
    }
}
