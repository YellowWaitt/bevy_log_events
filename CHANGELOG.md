# Changelog

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

Initial release