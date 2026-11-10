# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- When the computer went to sleep, game timers and the wait between queued games kept counting, so a game could stop on wake without having run for its whole duration. Time spent asleep is now added back to them.

## [1.0.0] - 2026-11-10

### Added

- In-app updates. Orbsmith checks GitHub for a new version at startup, shows what's new, and installs it when you choose to. Updates are signed, and a download whose signature does not match is refused. The check can be turned off in the settings, which also have a **Check now** button.
- Release files come with a build provenance attestation, to check with `gh attestation verify` that they were built from this repository.
- A log file, `logs/orbsmith.log` in the data folder, to attach to bug reports.
- A wait between queued games, set in minutes, with a countdown in the **Queue** tab.
- **Vary times**: queued games run up to 10% longer than their duration, never shorter, and each wait between games is up to 50% shorter or longer.

### Changed

- The game list is downloaded again once it is a day old, so new games show up without a manual refresh.
- Search ignores punctuation, accents and trademark signs: "plants vs zombies" finds "Plants vs. Zombies™: Replanted".
- Searching for a game that is not in the list explains that games detected through their store cannot be simulated.

### Fixed

- Starting a game whose executable another running game already uses failed with an unclear error. The message now names that game.

## [0.5.0] - 2026-11-09

### Added

- Linux support, with `.deb`, `.rpm` and AppImage packages. Games without a Linux executable use their Windows executable name, which is detected on Linux too.

### Fixed

- Starting Orbsmith while it is already running stopped every game it was running. The running instance now shows its window instead.

## [0.4.0] - 2026-11-09

### Added

- The queue is kept across restarts, and comes back paused with the time its games had left.
- Reorder the queue with up and down buttons.
- Choose which executable to simulate when a game has several, for another store, region or version.
- The start dialog remembers the duration and executable last used for each game.
- Games waiting in the queue show an "In queue" badge in the lists.
- Buttons to clear the queue and the recent games.

## [0.3.0] - 2026-11-09

### Added

- Queue: add games with a duration, then let Orbsmith run them one after another, even while the window is hidden in the tray. Pausing stops the current game and keeps its time left for when the queue resumes.
- Settings window, with an option to run the queue in parallel instead.
- Notifications when the queue is finished, or when one of its games cannot start.

### Changed

- The auto-stop timer runs for exactly the duration entered, without an extra second.

## [0.2.0] - 2026-11-09

### Added

- Tray icon to reopen Orbsmith, see the running games, stop them all or quit.
- Closing the window while games are running keeps them running in the tray.
- Desktop notification when a game stops on its own, because its timer ran out or it exited.
- Favorite and recently started games, shown before searching.

### Changed

- Simulated games no longer appear in the taskbar.

## [0.1.0] - 2026-11-09

### Added

- Search the list of detectable games by name or alias, with their icons. The list is cached and can be refreshed.
- Simulate a game by launching a stand-in process under its executable name.
- Auto-stop timer per simulation, set from the quest duration and editable while the game runs.
- Panel of running simulations with their remaining time and a stop button.
- Stand-in processes are stopped when Orbsmith exits, exit on their own if it crashes, and leftovers are cleaned up at startup.
- Windows installers (`setup.exe` and MSI). The `setup.exe` uninstaller can delete the app data.

[Unreleased]: https://github.com/alexwauquier/orbsmith/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/alexwauquier/orbsmith/compare/v0.5.0...v1.0.0
[0.5.0]: https://github.com/alexwauquier/orbsmith/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/alexwauquier/orbsmith/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/alexwauquier/orbsmith/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/alexwauquier/orbsmith/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/alexwauquier/orbsmith/releases/tag/v0.1.0
