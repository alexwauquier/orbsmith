use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

type BuildResult<T = ()> = Result<T, Box<dyn Error>>;

fn main() -> BuildResult {
    build_dummy()?;
    tauri_build::build();
    Ok(())
}

/// Builds `orbsmith-dummy` for the current target and puts it where Tauri
/// expects the `binaries/orbsmith-dummy` sidecar declared in `tauri.conf.json`.
///
/// It uses its own target dir: the outer cargo holds the lock on the main one.
fn build_dummy() -> BuildResult {
    let manifest_dir = PathBuf::from(env_var("CARGO_MANIFEST_DIR")?);
    let workspace = manifest_dir
        .parent()
        .ok_or("src-tauri has no parent directory")?;
    let target = env_var("TARGET")?;
    let target_dir = workspace.join("target").join("dummy");

    println!(
        "cargo:rerun-if-changed={}",
        workspace.join("crates").join("dummy").display()
    );

    let status = Command::new(env_var("CARGO")?)
        .args([
            "build",
            "--release",
            "--package",
            "orbsmith-dummy",
            "--target",
        ])
        .arg(&target)
        .arg("--target-dir")
        .arg(&target_dir)
        .current_dir(workspace)
        // Set for build scripts only; the nested build reads its own flags.
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .status()?;
    if !status.success() {
        return Err(format!("building orbsmith-dummy failed: {status}").into());
    }

    let suffix = if target.contains("windows") {
        ".exe"
    } else {
        ""
    };
    let built = target_dir
        .join(&target)
        .join("release")
        .join(format!("orbsmith-dummy{suffix}"));
    let sidecar_dir = manifest_dir.join("binaries");
    fs::create_dir_all(&sidecar_dir)?;
    copy_if_changed(
        &built,
        &sidecar_dir.join(format!("orbsmith-dummy-{target}{suffix}")),
    )
}

fn env_var(name: &str) -> BuildResult<String> {
    env::var(name).map_err(|err| format!("{name}: {err}").into())
}

/// Skips the copy when nothing changed, so the sidecar's mtime doesn't trigger
/// needless rebuilds.
fn copy_if_changed(from: &Path, to: &Path) -> BuildResult {
    let new = fs::read(from).map_err(|err| format!("{}: {err}", from.display()))?;
    if fs::read(to).ok().as_ref() != Some(&new) {
        fs::write(to, new)?;
    }
    make_executable(to)
}

/// `fs::write` creates files without the execute bit, and every later copy of
/// the sidecar, by Tauri and then by Orbsmith for each game, keeps that mode.
#[cfg(unix)]
fn make_executable(path: &Path) -> BuildResult {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> BuildResult {
    Ok(())
}
