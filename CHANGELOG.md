# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-11-09

### Added

- Search the list of detectable games by name or alias, with their icons. The list is cached and can be refreshed.
- Simulate a game by launching a stand-in process under its executable name.
- Auto-stop timer per simulation, set from the quest duration and editable while the game runs.
- Panel of running simulations with their remaining time and a stop button.
- Stand-in processes are stopped when Orbsmith exits, exit on their own if it crashes, and leftovers are cleaned up at startup.
- Windows installers (`setup.exe` and MSI). The `setup.exe` uninstaller can delete the app data.

[Unreleased]: https://github.com/alexwauquier/orbsmith/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/alexwauquier/orbsmith/releases/tag/v0.1.0
