//! The test binary doubles as a dummy: copied under another name and run with
//! [`HELPER_ARGS`], it only executes [`helper_process`], which waits like a real
//! dummy does.

use std::time::Duration;

use sysinfo::{Pid, ProcessRefreshKind, ProcessStatus, ProcessesToUpdate, System};

use crate::simulation::TITLE_ENV;

/// Arguments that make the test binary run only the helper below.
pub const HELPER_ARGS: &str = "test_support::helper_process --exact --ignored --test-threads=1";

/// Window title (app name) that makes the helper exit at once instead of
/// waiting.
pub const EXIT_AT_ONCE: &str = "exit at once";

#[test]
#[ignore = "run as a child process by other tests"]
fn helper_process() {
    if std::env::var(TITLE_ENV).as_deref() != Ok(EXIT_AT_ONCE) {
        std::thread::sleep(Duration::from_secs(30));
    }
}

pub fn test_binary() -> std::path::PathBuf {
    std::env::current_exe().unwrap()
}

pub fn is_alive(pid: u32) -> bool {
    let pid = Pid::from_u32(pid);
    let mut system = System::new().unwrap();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing(),
    );
    system
        .process(pid)
        .is_some_and(|p| p.status() != ProcessStatus::Zombie)
}
