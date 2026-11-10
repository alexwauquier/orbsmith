<p align="center">
  <img src="assets/logo.svg" alt="" width="128" height="128">
</p>

<h1 align="center">Orbsmith</h1>

Desktop app that launches stand-in processes named after game executables, so that software detecting running games sees them as launched.

> [!IMPORTANT]
> Orbsmith is not affiliated with any software or game publisher.

> [!WARNING]
> Using Orbsmith may go against the terms of service of the software that detects running games. Use it at your own risk.

![Orbsmith with two games running](docs/screenshots/main.png)

<sub>Orbsmith is not affiliated with any of the games shown.</sub>

## Installation

Orbsmith runs on Windows 10 and 11 and on Linux (x64).

### Windows

1. Download the latest installer from the [Releases](https://github.com/alexwauquier/orbsmith/releases) page:
   - `Orbsmith_<version>_x64-setup.exe` (recommended), or
   - `Orbsmith_<version>_x64_en-US.msi`, for deployment with Windows Installer.
2. Run it. The installers are not code-signed, so Windows SmartScreen warns about an unrecognized app: click **More info**, then **Run anyway**.

The setup installs Microsoft Edge WebView2 if it is missing. It is preinstalled on Windows 11 and on up-to-date Windows 10.

### Linux

Download the package for your distribution from the [Releases](https://github.com/alexwauquier/orbsmith/releases) page:

- Debian, Ubuntu and derivatives: `Orbsmith_<version>_amd64.deb`, installed with `sudo apt install ./Orbsmith_<version>_amd64.deb`.
- Fedora, openSUSE and derivatives: `Orbsmith-<version>-1.x86_64.rpm`, installed with `sudo dnf install ./Orbsmith-<version>-1.x86_64.rpm`.
- Any other distribution: `Orbsmith_<version>_amd64.AppImage`. Make it executable (`chmod +x`), then run it. It may need FUSE 2 (`libfuse2` on Ubuntu).

On GNOME, the tray icon needs the AppIndicator extension, which Ubuntu ships by default. Without it, start Orbsmith again to bring back a window closed while games run.

### Verifying a download

Releases are not code-signed, but every file on the Releases page comes with a build provenance attestation: a signed record, kept by GitHub, of the workflow run and the commit that built it. With the [GitHub CLI](https://cli.github.com) 2.49 or later, check a downloaded file with:

```sh
gh attestation verify Orbsmith_<version>_x64-setup.exe -R alexwauquier/orbsmith
```

The Releases page also lists the SHA-256 checksum of each file.

## Usage

1. Search for a game by name. Star the ones you use often: favorites and recently started games show up before you search. **Clear** empties the recent games.
2. Click **Start**, enter how long the game should run (for example, a quest duration), then **Start now**: the game stops automatically once that time has elapsed. Untick **Stop automatically** to keep it running until you stop it. If a game lists several executables and isn't detected, pick another one in **Executable**. Orbsmith remembers the duration and executable you last used for each game.
3. The **Running** panel lists the active games with their remaining time. You can change the timer or stop a game at any time.

![The start dialog](docs/screenshots/start-dialog.png)

### Queue

To run several games in a row, choose **Add to queue** instead of **Start now**: the game joins the **Queue** tab with its duration. Reorder it with the arrows, or empty it with the bin button. **Start queue** runs them one after another, each for its duration, and works through the list on its own, even when the window is hidden in the tray. **Pause** stops the current game and keeps its remaining time: **Resume** restarts it for just that long, then carries on with the queue. **Stop all** in the tray menu pauses the queue the same way. Stopping a queued game from the **Running** tab skips to the next one. The queue is saved: after closing Orbsmith, it comes back paused, ready to resume.

![The queue, running its first game](docs/screenshots/queue.png)

To run every game in the queue at the same time instead, turn on **Run the queue in parallel** in the settings (gear icon in the header).

Simulated games run in the background: they don't appear in the taskbar. A notification tells you when one stops on its own.

Closing the window while games are running keeps Orbsmith in the system tray, so they keep running. Click the tray icon to reopen it; its menu also stops every game or quits. With nothing running, closing the window quits Orbsmith. If Orbsmith crashes, its stand-in processes exit on their own, and any leftover is cleaned up at the next start.

The list of games is cached and downloaded again once it is a day old; the refresh button in the header updates it right away. Orbsmith only downloads that public list, the game icons and, to look for updates, the latest release from GitHub: no account, no login.

### Updates

At startup, Orbsmith checks GitHub for a new version. When there is one, a banner shows what's new; **Update and restart** stops the running games, pauses the queue with its time left, installs the update and restarts. Nothing is installed unless you choose to. Turn off the check in the settings, or use **Check now** there.

Updates are signed, and Orbsmith refuses any download whose signature does not match. On Linux, updating a `.deb` or `.rpm` install asks for your password.

Versions before 1.0.0 have no updater: install the new version over the old one, as the first time. Your data is kept.

## Troubleshooting

**A game is missing from the list.** New games are added to the list over time: refresh it with the button in the header. Some games are not in it at all, because they are detected through their store (a Steam or other launcher install) rather than by their executable. Those cannot be simulated.

**A game runs in Orbsmith but is not detected.** Click **Start** again and pick another executable in **Executable**: some are only recognized for a given store or region. Game detection only works in the official desktop app, not in browsers or third-party clients. On Linux, Orbsmith must run as the same user as that app.

**My antivirus warns about Orbsmith.** Releases are not code-signed, which some antivirus software flags. Check that your download is the file built from this repository: see [Verifying a download](#verifying-a-download).

**Something else is wrong.** [Open an issue](https://github.com/alexwauquier/orbsmith/issues) and attach the log: `logs/orbsmith.log` in the data folder (see below).

## Data and uninstalling

Orbsmith keeps its data in `%APPDATA%\Orbsmith` on Windows and `~/.local/share/orbsmith` on Linux: the cached list of games, your settings, the queue, the stand-in executables, the log, and the WebView data, which includes your favorites, recent games and how you last started each one.

- The `setup.exe` uninstaller deletes that folder if you tick **Delete the application data**.
- The MSI uninstaller leaves it in place, as Windows Installer does for user data. Delete the folder by hand to remove it.
- On Linux, uninstall with your package manager (`sudo apt remove orbsmith` or `sudo dnf remove orbsmith`), or delete the AppImage. The data folder is never removed: delete it by hand.

## Tech stack

- [Tauri 2](https://tauri.app) (Rust backend, system WebView)
- [Svelte 5](https://svelte.dev) and [Vite](https://vite.dev)
- [Tailwind CSS 4](https://tailwindcss.com) and [shadcn-svelte](https://shadcn-svelte.com)
- [Oxlint](https://oxc.rs/docs/guide/usage/linter) and [Oxfmt](https://oxc.rs/docs/guide/usage/formatter)

## Development

### Prerequisites

- Node.js 24+ and pnpm
- Rust stable (1.90+)
- [Tauri system prerequisites](https://tauri.app/start/prerequisites/) for your platform

### Project layout

- `src/`: the Svelte frontend
- `src-tauri/`: the Tauri app, which exposes the commands to the frontend
- `crates/core/`: the game list, executable path validation and process management
- `crates/dummy/`: the stand-in process launched under each game's executable name

### Commands

```sh
pnpm install       # install dependencies
pnpm tauri dev     # run the app in development mode
pnpm tauri build   # build the installers
pnpm check         # type-check
pnpm lint          # lint the code
pnpm format        # format the code
pnpm format:check  # check formatting
pnpm test          # run the frontend tests
cargo fmt --all    # format the Rust code
cargo clippy --workspace --all-targets
cargo test --workspace
```

On Linux, the Tauri app needs the [system prerequisites](https://tauri.app/start/prerequisites/) to build; `cargo test -p orbsmith-core` works without them.

`pnpm tauri build` also signs the installers for the updater, which needs the private key in `TAURI_SIGNING_PRIVATE_KEY` (and its password in `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`). To build without it, skip the updater files: `pnpm tauri build -c '{"bundle":{"createUpdaterArtifacts":false}}'`.

### Releases

Pushing a `v*` tag builds the Windows installers and the Linux packages and attaches them to a draft GitHub release. The tag must match the version in `package.json`, `Cargo.toml` and `src-tauri/tauri.conf.json`, and be the newest release in `src-tauri/linux/io.github.alexwauquier.orbsmith.metainfo.xml`. The workflow signs the installers with the `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets and uploads the `latest.json` manifest that the updater reads. Publish releases as full releases, not pre-releases: the updater only sees the latest full release.

## License

Released under the [MIT License](LICENSE).
