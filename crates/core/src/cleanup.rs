use std::path::Path;

use crate::os::Os;

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Kills every process whose executable lives under `executables_dir`. Only
/// Orbsmith's dummies run from there, so anything found is an orphan left by a
/// crash. Returns how many processes were killed.
pub fn kill_orphans(executables_dir: &Path) -> usize {
    let mut system = match System::new() {
        Ok(system) => system,
        Err(err) => {
            log::warn!("cannot list processes, skipping orphan cleanup: {err}");
            return 0;
        }
    };
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );

    let mut killed = 0;
    for process in system.processes().values() {
        // On Linux, threads are listed as processes too; kill each process once.
        if process.thread_kind().is_some() {
            continue;
        }
        let Some(exe) = process.exe() else { continue };
        if is_within(exe, executables_dir, Os::CURRENT) && process.kill() {
            log::info!(
                "killed orphan dummy {} (pid {})",
                exe.display(),
                process.pid()
            );
            killed += 1;
        }
    }
    killed
}

fn is_within(path: &Path, dir: &Path, os: Os) -> bool {
    if os.case_insensitive_paths() {
        let path = path.to_string_lossy().to_lowercase();
        let dir = dir.to_string_lossy().to_lowercase();
        Path::new(&path).starts_with(Path::new(&dir))
    } else {
        path.starts_with(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_only_paths_inside_the_dir() {
        let dir = Path::new("/data/executables");
        for os in [Os::Windows, Os::Linux] {
            assert!(is_within(Path::new("/data/executables/game.exe"), dir, os));
            assert!(is_within(
                Path::new("/data/executables/a/b/game.exe"),
                dir,
                os
            ));
            assert!(!is_within(
                Path::new("/data/executables-old/game.exe"),
                dir,
                os
            ));
            assert!(!is_within(Path::new("/data/game.exe"), dir, os));
        }
    }

    #[test]
    fn ignores_case_only_where_the_os_does() {
        let dir = Path::new("/Data/Executables");
        let exe = Path::new("/data/executables/game.exe");
        assert!(is_within(exe, dir, Os::Windows));
        assert!(!is_within(exe, dir, Os::Linux));
    }

    #[test]
    fn kills_processes_running_from_the_dir() {
        use crate::test_support::{HELPER_ARGS, is_alive, test_binary};
        use std::process::Command;

        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join(if cfg!(windows) {
            "orphan.exe"
        } else {
            "orphan"
        });
        std::fs::copy(test_binary(), &exe).unwrap();
        let mut child = Command::new(&exe)
            .args(HELPER_ARGS.split_whitespace())
            .spawn()
            .unwrap();
        assert!(is_alive(child.id()));

        assert_eq!(kill_orphans(dir.path()), 1);
        child.wait().unwrap();
        assert!(!is_alive(child.id()));
    }
}
