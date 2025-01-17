# Changelog

## 0.4.1

### Added

- The `show_window` bool field to the `LogEventsPluginSettings` struct.

### Changed

- The `LogEventsPlugin` now directly rely on [`bevy_egui`](https://github.com/vladbat00/bevy_egui) instead of [`bevy_editor_pls`](https://github.com/jakobhellermann/bevy_editor_pls) to show the `LoggedEventSettings` editor window. The `EguiPlugin` will be added for you if it has not already been.

## 0.4.0

### Added

- The `add_and_log_state_scoped_event` functions to the `LogEvent` trait.

### Changed

- Updated `bevy` version to 0.15.
- Temporarily removed the `editor_window` feature waiting for the `bevy_editor_pls` update.
- The functions of the `LogEvent` trait now check if they were already used for the
  same events, preventing events to be logged twice.

## 0.3.0

### Added

- Triggered events can now be logged via the new `log_triggered` and `log_trigger` functions
  of the `LogEvent` trait.
- The `LogEventsPlugin` can be disabled by removing the `enabled` default feature.
- The `RegisterEventsSet` system set has been added.

### Changed

- The unwraps from the `save_settings` systems were removed and the system will
  now log an error instead of panicking.

## 0.2.0

### Changes

- Updated `bevy` version to 0.14.

## 0.1.0

Initial release.
