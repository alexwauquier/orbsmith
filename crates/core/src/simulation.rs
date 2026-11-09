use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime};
use std::{fs, io};

use serde::Serialize;

use crate::catalog::{DetectableApp, unix_ms};
use crate::error::Error;
use crate::os::Os;

/// Environment variable through which a dummy learns Orbsmith's PID, so it can
/// exit when Orbsmith does. Must match the dummy crate.
pub const PARENT_PID_ENV: &str = "ORBSMITH_PARENT_PID";
/// Environment variable holding the dummy's window title. Must match the dummy
/// crate.
pub const TITLE_ENV: &str = "ORBSMITH_TITLE";

/// Longest accepted auto-stop timer. Typical timers last minutes; without a timer a
/// simulation already runs until stopped.
pub const MAX_DURATION: Duration = Duration::from_secs(7 * 24 * 60 * 60);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationInfo {
    pub app_id: String,
    pub app_name: String,
    pub executable: String,
    pub pid: u32,
    /// Milliseconds since the Unix epoch.
    pub started_at: u64,
    /// Milliseconds since the Unix epoch, or `None` to run until stopped.
    pub stop_at: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EndReason {
    /// The auto-stop timer expired.
    Timer,
    /// The dummy exited by itself, e.g. its window was closed.
    Exited,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ended {
    pub app_id: String,
    pub app_name: String,
    pub reason: EndReason,
}

struct Running {
    info: SimulationInfo,
    /// Where the dummy runs from, under the executables dir.
    exe: PathBuf,
    stop_at: Option<SystemTime>,
    child: Child,
}

fn deadline(now: SystemTime, duration: Option<Duration>) -> Result<Option<SystemTime>, Error> {
    duration
        .map(|duration| {
            if duration > MAX_DURATION {
                return Err(Error::TimerTooLong);
            }
            now.checked_add(duration).ok_or(Error::TimerTooLong)
        })
        .transpose()
}

/// Creates `exe_dir` under `base` and checks that `exe` really resolves
/// inside it. Names are validated lexically, but a symlink placed under
/// `base` could still redirect the copy and the spawn elsewhere, so no
/// existing symlink is followed.
fn prepare_exe_dir(base: &Path, exe_dir: &Path, exe: &Path) -> Result<(), Error> {
    let outside = || Error::OutsideExecutablesDir(exe.display().to_string());

    fs::create_dir_all(base)?;
    let relative = exe_dir.strip_prefix(base).map_err(|_| outside())?;
    let mut current = base.to_path_buf();
    for component in relative.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => return Err(outside()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => fs::create_dir(&current)?,
            Err(err) => return Err(err.into()),
        }
    }
    if fs::symlink_metadata(exe).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return Err(outside());
    }

    if fs::canonicalize(exe_dir)?.starts_with(fs::canonicalize(base)?) {
        Ok(())
    } else {
        Err(outside())
    }
}

/// The running dummies, keyed by app ID. All of them are killed on drop.
pub struct Simulations {
    dummy: PathBuf,
    executables_dir: PathBuf,
    running: HashMap<String, Running>,
}

impl Simulations {
    /// `dummy` is the stand-in binary that gets copied under each app's
    /// executable name.
    pub fn new(dummy: impl Into<PathBuf>, executables_dir: impl Into<PathBuf>) -> Self {
        Self {
            dummy: dummy.into(),
            executables_dir: executables_dir.into(),
            running: HashMap::new(),
        }
    }

    /// Launches a dummy for `app`, as `executable` (one of
    /// [`AppSummary::executables`](crate::AppSummary::executables)) or the
    /// default one, stopped automatically after `duration` if given.
    pub fn start(
        &mut self,
        app: &DetectableApp,
        os: Os,
        executable: Option<&str>,
        duration: Option<Duration>,
    ) -> Result<SimulationInfo, Error> {
        if self.running.contains_key(&app.id) {
            return Err(Error::AlreadyRunning(app.name.clone()));
        }
        let target = match executable {
            Some(name) => app
                .target_named(os, name)
                .ok_or_else(|| Error::UnknownExecutable {
                    app: app.name.clone(),
                    executable: name.to_owned(),
                })?,
            None => app
                .target(os)
                .ok_or_else(|| Error::NoExecutable(app.name.clone()))?,
        };
        let now = SystemTime::now();
        let stop_at = deadline(now, duration)?;

        if !self.dummy.is_file() {
            return Err(Error::MissingDummy(self.dummy.display().to_string()));
        }

        let exe_dir = target.exe.dir_under(&self.executables_dir);
        let exe = exe_dir.join(target.exe.file_name());
        // Some apps list the same executable. It cannot be replaced while it
        // runs, and the detector would only see one of the two apps anyway.
        let same_exe = |other: &Path| {
            if os.case_insensitive_paths() {
                other.to_string_lossy().to_lowercase() == exe.to_string_lossy().to_lowercase()
            } else {
                other == exe
            }
        };
        if let Some(other) = self.running.values().find(|running| same_exe(&running.exe)) {
            return Err(Error::ExecutableInUse {
                app: app.name.clone(),
                executable: target.exe.to_string(),
                other: other.info.app_name.clone(),
            });
        }
        prepare_exe_dir(&self.executables_dir, &exe_dir, &exe)?;
        fs::copy(&self.dummy, &exe)?;

        let mut command = Command::new(&exe);
        command
            .args(&target.arguments)
            .current_dir(&exe_dir)
            .env(PARENT_PID_ENV, std::process::id().to_string())
            .env(TITLE_ENV, &app.name)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let child = spawn(&mut command).map_err(|source| Error::Spawn {
            path: exe.display().to_string(),
            source,
        })?;

        let info = SimulationInfo {
            app_id: app.id.clone(),
            app_name: app.name.clone(),
            executable: target.exe.to_string(),
            pid: child.id(),
            started_at: unix_ms(now),
            stop_at: stop_at.map(unix_ms),
        };
        log::info!(
            "started {} as {} (pid {})",
            app.name,
            info.executable,
            info.pid
        );

        self.running.insert(
            app.id.clone(),
            Running {
                info: info.clone(),
                exe,
                stop_at,
                child,
            },
        );
        Ok(info)
    }

    pub fn stop(&mut self, app_id: &str) -> Result<(), Error> {
        let running = self
            .running
            .remove(app_id)
            .ok_or_else(|| Error::NotRunning(app_id.to_owned()))?;
        kill(running);
        Ok(())
    }

    pub fn stop_all(&mut self) {
        for (_, running) in self.running.drain() {
            kill(running);
        }
    }

    /// Replaces the auto-stop timer: stop `duration` from now, or never.
    pub fn set_timer(
        &mut self,
        app_id: &str,
        duration: Option<Duration>,
    ) -> Result<SimulationInfo, Error> {
        let running = self
            .running
            .get_mut(app_id)
            .ok_or_else(|| Error::NotRunning(app_id.to_owned()))?;
        running.stop_at = deadline(SystemTime::now(), duration)?;
        running.info.stop_at = running.stop_at.map(unix_ms);
        Ok(running.info.clone())
    }

    /// Running simulations, oldest first.
    pub fn list(&self) -> Vec<SimulationInfo> {
        let mut list: Vec<_> = self.running.values().map(|r| r.info.clone()).collect();
        list.sort_by_key(|info| info.started_at);
        list
    }

    /// Reaps dummies that exited and stops those whose timer expired at `now`.
    /// Meant to be called periodically.
    pub fn tick(&mut self, now: SystemTime) -> Vec<Ended> {
        let mut ended = Vec::new();
        for (app_id, running) in &mut self.running {
            let reason = if !matches!(running.child.try_wait(), Ok(None)) {
                EndReason::Exited
            } else if running.stop_at.is_some_and(|stop_at| now >= stop_at) {
                EndReason::Timer
            } else {
                continue;
            };
            ended.push(Ended {
                app_id: app_id.clone(),
                app_name: running.info.app_name.clone(),
                reason,
            });
        }
        for end in &ended {
            if let Some(running) = self.running.remove(&end.app_id) {
                kill(running);
            }
        }
        ended
    }
}

impl Drop for Simulations {
    fn drop(&mut self) {
        self.stop_all();
    }
}

/// Spawns `command`, retrying briefly on "text file busy". On Linux, a process
/// forked by another thread while the dummy copy was still open for writing
/// keeps that handle until it execs, which makes exec of the copy fail.
pub(crate) fn spawn(command: &mut Command) -> std::io::Result<Child> {
    let mut attempts = 0;
    loop {
        match command.spawn() {
            Err(err) if err.kind() == std::io::ErrorKind::ExecutableFileBusy && attempts < 10 => {
                attempts += 1;
                std::thread::sleep(Duration::from_millis(20));
            }
            result => return result,
        }
    }
}

fn kill(mut running: Running) {
    // Fails only if the process already exited, which is the goal anyway.
    let _ = running.child.kill();
    let _ = running.child.wait();
    log::info!("stopped {}", running.info.app_name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::test_support::{EXIT_AT_ONCE, HELPER_ARGS, is_alive, test_binary};
    use std::time::UNIX_EPOCH;

    const EXE: &str = if cfg!(windows) {
        "game/bin/game.exe"
    } else {
        "game/bin/game"
    };

    fn setup(name: &str) -> (tempfile::TempDir, Simulations, DetectableApp) {
        let dir = tempfile::tempdir().unwrap();
        let sims = Simulations::new(test_binary(), dir.path().join("executables"));
        let json = format!(
            r#"[{{"id": "42", "name": "{name}", "executables": [
                {{"os": "{os}", "name": "{EXE}", "is_launcher": false, "arguments": "{HELPER_ARGS}"}}
            ]}}]"#,
            os = Os::CURRENT.api_name()
        );
        let app = Catalog::from_json(json.as_bytes(), UNIX_EPOCH)
            .unwrap()
            .get("42")
            .unwrap()
            .clone();
        (dir, sims, app)
    }

    #[test]
    fn starts_the_chosen_executable() {
        let other = if cfg!(windows) {
            "other/game.exe"
        } else {
            "other/game"
        };
        let dir = tempfile::tempdir().unwrap();
        let mut sims = Simulations::new(test_binary(), dir.path().join("executables"));
        let json = format!(
            r#"[{{"id": "42", "name": "Game", "executables": [
                {{"os": "{os}", "name": "{EXE}", "is_launcher": false, "arguments": "{HELPER_ARGS}"}},
                {{"os": "{os}", "name": "{other}", "is_launcher": false, "arguments": "{HELPER_ARGS}"}}
            ]}}]"#,
            os = Os::CURRENT.api_name()
        );
        let catalog = Catalog::from_json(json.as_bytes(), UNIX_EPOCH).unwrap();
        let app = catalog.get("42").unwrap();

        assert!(matches!(
            sims.start(app, Os::CURRENT, Some("missing.exe"), None),
            Err(Error::UnknownExecutable { .. })
        ));
        let info = sims.start(app, Os::CURRENT, Some(other), None).unwrap();
        assert_eq!(info.executable, other);
        assert!(dir.path().join("executables").join(other).exists());
        assert!(!dir.path().join("executables").join(EXE).exists());
        sims.stop("42").unwrap();
    }

    #[test]
    fn starts_copy_under_exe_name_and_stops_it() {
        let (dir, mut sims, app) = setup("Game");
        let info = sims.start(&app, Os::CURRENT, None, None).unwrap();

        assert!(dir.path().join("executables").join(EXE).exists());
        assert_eq!(info.executable, EXE);
        assert!(is_alive(info.pid));
        assert_eq!(sims.list().len(), 1);
        assert!(sims.tick(SystemTime::now()).is_empty());

        sims.stop("42").unwrap();
        assert!(!is_alive(info.pid));
        assert!(sims.list().is_empty());
        assert!(matches!(sims.stop("42"), Err(Error::NotRunning(_))));
    }

    #[cfg(unix)]
    #[test]
    fn refuses_symlinked_dirs_that_escape() {
        let (dir, mut sims, app) = setup("Symlinked");
        let outside = tempfile::tempdir().unwrap();
        let game = dir.path().join("executables").join("game");
        fs::create_dir_all(game.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(outside.path(), &game).unwrap();

        assert!(matches!(
            sims.start(&app, Os::CURRENT, None, None),
            Err(Error::OutsideExecutablesDir(_))
        ));
        assert!(!outside.path().join("bin").exists());
    }

    #[test]
    fn reports_missing_dummy() {
        let (dir, _, app) = setup("Game");
        let mut sims = Simulations::new(dir.path().join("missing"), dir.path().join("executables"));
        assert!(matches!(
            sims.start(&app, Os::CURRENT, None, None),
            Err(Error::MissingDummy(_))
        ));
    }

    #[test]
    fn refuses_an_executable_another_app_runs_as() {
        let dir = tempfile::tempdir().unwrap();
        let mut sims = Simulations::new(test_binary(), dir.path().join("executables"));
        let json = format!(
            r#"[
                {{"id": "1", "name": "First", "executables": [
                    {{"os": "{os}", "name": "{EXE}", "is_launcher": false, "arguments": "{HELPER_ARGS}"}}]}},
                {{"id": "2", "name": "Second", "executables": [
                    {{"os": "{os}", "name": "{EXE}", "is_launcher": false, "arguments": "{HELPER_ARGS}"}}]}}
            ]"#,
            os = Os::CURRENT.api_name()
        );
        let catalog = Catalog::from_json(json.as_bytes(), UNIX_EPOCH).unwrap();
        let (first, second) = (catalog.get("1").unwrap(), catalog.get("2").unwrap());

        sims.start(first, Os::CURRENT, None, None).unwrap();
        assert!(matches!(
            sims.start(second, Os::CURRENT, None, None),
            Err(Error::ExecutableInUse { ref other, .. }) if other == "First"
        ));
        sims.stop("1").unwrap();
        sims.start(second, Os::CURRENT, None, None).unwrap();
        sims.stop("2").unwrap();
    }

    #[test]
    fn refuses_to_start_twice() {
        let (_dir, mut sims, app) = setup("Game");
        sims.start(&app, Os::CURRENT, None, None).unwrap();
        assert!(matches!(
            sims.start(&app, Os::CURRENT, None, None),
            Err(Error::AlreadyRunning(_))
        ));
    }

    #[test]
    fn timer_stops_the_dummy() {
        let (_dir, mut sims, app) = setup("Game");
        let info = sims
            .start(&app, Os::CURRENT, None, Some(Duration::from_secs(60)))
            .unwrap();
        assert!(info.stop_at.unwrap() >= info.started_at + 60_000);

        assert!(sims.tick(SystemTime::now()).is_empty());
        let ended = sims.tick(SystemTime::now() + Duration::from_secs(61));
        assert_eq!(
            ended,
            [Ended {
                app_id: "42".into(),
                app_name: "Game".into(),
                reason: EndReason::Timer
            }]
        );
        assert!(!is_alive(info.pid));
    }

    #[test]
    fn set_timer_replaces_the_deadline() {
        let (_dir, mut sims, app) = setup("Game");
        sims.start(&app, Os::CURRENT, None, Some(Duration::from_secs(1)))
            .unwrap();
        let info = sims.set_timer("42", None).unwrap();
        assert_eq!(info.stop_at, None);
        assert!(
            sims.tick(SystemTime::now() + Duration::from_secs(3600))
                .is_empty()
        );
    }

    #[test]
    fn rejects_timers_beyond_the_maximum() {
        let (_dir, mut sims, app) = setup("Too long");
        let too_long = Some(MAX_DURATION + Duration::from_secs(1));
        assert!(matches!(
            sims.start(&app, Os::CURRENT, None, too_long),
            Err(Error::TimerTooLong)
        ));
        assert!(sims.list().is_empty());

        sims.start(&app, Os::CURRENT, None, Some(MAX_DURATION))
            .unwrap();
        assert!(matches!(
            sims.set_timer("42", Some(Duration::MAX)),
            Err(Error::TimerTooLong)
        ));
    }

    #[test]
    fn detects_dummy_that_exited() {
        let (_dir, mut sims, app) = setup(EXIT_AT_ONCE);
        let pid = sims.start(&app, Os::CURRENT, None, None).unwrap().pid;

        let deadline = SystemTime::now() + Duration::from_secs(10);
        let ended = loop {
            let ended = sims.tick(SystemTime::now());
            if !ended.is_empty() || SystemTime::now() > deadline {
                break ended;
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        assert_eq!(ended[0].reason, EndReason::Exited);
        assert!(sims.list().is_empty());
        assert!(!is_alive(pid));
    }

    #[test]
    fn drop_kills_everything() {
        let (_dir, mut sims, app) = setup("Game");
        let pid = sims.start(&app, Os::CURRENT, None, None).unwrap().pid;
        drop(sims);
        assert!(!is_alive(pid));
    }
}
