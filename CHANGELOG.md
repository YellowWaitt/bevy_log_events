# Changelog

## 0.5.0

### Added

- Send and trigger event locations are now displayed if the `bevy/track_location` feature is enabled.

### Changed

- Updated `bevy` version to 0.16.
- To match `bevy_egui` [recommandations](https://docs.rs/bevy_egui/latest/bevy_egui/struct.EguiPlugin.html#note-to-developers-of-public-plugins) you will now have to add yourself the `EguiPlugin` before the `LogEventsPlugin`.

### Removed

- The deprecated `RegisterEventsSet`.
- The no longer used `editor_window` feature.

## 0.4.2

### Added

- The `log_component_hooks` function to the `LogEvent` trait.
- The `log_events_window_ui` function has been made public.

### Deprecated

- The `RegisterEventsSet` is no longer used.

### Fixed

- The protection against double event registration does not prevent the use of `log_event` and `log_triggered` for the same event type.

## 0.4.1

### Added

- The `show_window` bool field to the `LogEventsPluginSettings` struct.

### Changed

- The `LogEventsPlugin` now directly rely on [`bevy_egui`](https://github.com/vladbat00/bevy_egui) instead of [`bevy_editor_pls`](https://github.com/jakobhellermann/bevy_editor_pls) to show the `LoggedEventSettings` editor window. The `EguiPlugin` will be added for you if it has not already been.

## 0.4.0

### Added

- The `add_and_log_state_scoped_event` function to the `LogEvent` trait.

### Changed

- Updated `bevy` version to 0.15.
- Temporarily removed the `editor_window` feature waiting for the `bevy_editor_pls` update.
- The functions of the `LogEvent` trait now check if they were already used for the same events, preventing events from being logged twice.

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

### Changed

- Updated `bevy` version to 0.14.

## 0.1.0

Initial release.
