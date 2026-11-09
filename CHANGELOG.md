# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Queue: add games with a duration, then let Orbsmith run them one after another, even while the window is hidden in the tray.
- Settings window, with an option to run the queue in parallel instead.
- Notifications when the queue is finished, or when one of its games cannot start.

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

[Unreleased]: https://github.com/alexwauquier/orbsmith/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/alexwauquier/orbsmith/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/alexwauquier/orbsmith/releases/tag/v0.1.0
