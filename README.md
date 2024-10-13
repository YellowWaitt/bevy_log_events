[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/YellowWaitt/bevy_log_events#license)
[![crates.io](https://img.shields.io/crates/v/bevy_log_events)](https://crates.io/crates/bevy_log_events)
[![docs.rs](https://docs.rs/bevy_log_events/badge.svg)](https://docs.rs/bevy_log_events)
[![Following released Bevy versions](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/quick-start/plugin-development/#main-branch-tracking)


# bevy_log_events

[`bevy_log_events`](https://github.com/YellowWaitt/bevy_log_events) is a [Bevy](https://bevyengine.org/) plugin that introduce the `add_and_log_event` function for Bevy's App. This plugin lets you log your Event while allowing you to configure independently how each Event are logged during program execution.

## Features

- Log your events without having to place print in several places inside your code
- You can configure independently how each events will be logged using the `LoggedEventSettings<T>` resources
- Your settings are saved when you exit your application and reloaded the next time you launch it
- Using [`bevy_editor_pls`](https://github.com/jakobhellermann/bevy_editor_pls) and the `editor_window` feature, you can use a window to edit the settings for all your logged events :

![](assets/editor_window.png)

## Usage

Add the crate to your Cargo.toml :
```
cargo add bevy_log_events
```

Then  you just have to add the `LogEventsPlugin` plugin and use `add_and_log_event` to register your events instead of `add_event` :
```rust
use bevy::{prelude::*, time::Stopwatch};
use bevy_log_events::prelude::*;

// You must implement Debug for the events you want to log
#[derive(Event, Debug)]
struct MyEvent;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // Add the logging plugin
            LogEventsPlugin::new("assets/simple.ron"),
        ))
        // You can now use add_and_log_event instead of add_event to add and log your events
        .add_and_log_event::<MyEvent>()
        // Using log_event you can also log external events you did not add yourself
        .log_event::<CursorEntered>()
        .add_systems(Update, emit_my_event)
        .run();
}

// MyEvent will be sent and logged every second
fn emit_my_event(
    time: Res<Time>,
    mut events: EventWriter<MyEvent>,
    mut stopwatch: Local<Stopwatch>,
) {
    stopwatch.tick(time.delta());
    if stopwatch.elapsed_secs() > 1.0 {
        stopwatch.reset();
        events.send(MyEvent);
    }
}

```

## Examples

To run the minimal example from above :
```
cargo run --example simple
```

To run an example with more events and the [`bevy_editor_pls`](https://github.com/jakobhellermann/bevy_editor_pls) window use :
```
cargo run --example window --features editor_window
```

## Notes

Events are all logged in the `Last` schedule inside the `LogEventSet` therefore if many events are sent in the same frame they will not be logged in the same order they are sent.

## Bevy Versions Table

| Bevy | bevy_auto_log_events |
| ---- | -------------------- |
| 0.14 | 0.2                  |
| 0.13 | 0.1                  |
