[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/YellowWaitt/bevy_log_events#license)
[![crates.io](https://img.shields.io/crates/v/bevy_log_events)](https://crates.io/crates/bevy_log_events)
[![docs.rs](https://docs.rs/bevy_log_events/badge.svg)](https://docs.rs/bevy_log_events)
[![Following released Bevy versions](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/quick-start/plugin-development/#main-branch-tracking)


# bevy_log_events

[`bevy_log_events`](https://github.com/YellowWaitt/bevy_log_events) is a [Bevy](https://bevyengine.org/) plugin that introduce the `LogEvent` trait for Bevy's App. It will helps you log your `Event` and `Message` while allowing you to configure independently how each of them are logged during runtime.

## Features

- Easily log your events and message by adding a single line of code for each of them.
- You can configure independently how each of them will be logged using the `LoggedEventSettings<E>` resources.
- Your settings are saved when you exit your application and reloaded the next time you launch it.
- You can use a window to edit the settings for all your logged events :

![](assets/editor_window.png)

## Usage

Add the crate to your Cargo.toml :
```
cargo add bevy_log_events
```

Then  you just have to add the `EguiPlugin` and `LogEventsPlugin` plugin and use the functions from the `LogEvent` trait to log your events :

```rust
use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_egui::EguiPlugin;
use bevy_log_events::prelude::*;

// You must implement Debug for the messages and events you want to log
#[derive(Message, Debug)]
struct MyMessage {
    value: usize,
}

#[derive(Event, Debug)]
struct MyEvent {
    value: usize,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // You need to add the EguiPlugin before the LogEventsPlugin
            EguiPlugin::default(),
            // Add the logging plugin
            LogEventsPlugin::new("assets/simple.ron"),
        ))
        // You can now use add_and_log_message instead of add_message to add and log your messages
        .add_and_log_message::<MyMessage>()
        // Events can be log too with the use of observers
        .log_event::<MyEvent>()
        // Using log_message you can also log external messages you did not add yourself
        .log_message::<CursorEntered>()
        .add_systems(
            Update,
            (
                toggle_window,
                (send_my_event, trigger_my_event).run_if(on_timer(Duration::from_secs(1))),
            ),
        )
        .run();
}

// MyMessage will be sent and logged every second at the end of each frame
fn send_my_event(mut events: MessageWriter<MyMessage>) {
    events.write(MyMessage { value: 28 });
}

// MyEvent will be triggered and logged every second during Update
fn trigger_my_event(mut commands: Commands) {
    commands.trigger(MyEvent { value: 496 });
}

// Toggle the editor window
fn toggle_window(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut plugin_settings: ResMut<LogEventsPluginSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        plugin_settings.show_window = !plugin_settings.show_window;
    }
}
```

## Examples

To run the minimal example from above :
```
cargo run --example simple
```

To run an example with more events and to see how to use the settings editor window :
```
cargo run --example window
```

To see a more complete example with triggered events check the copy of the official bevy example of observers :
```
cargo run --example observers
```

You can also try these examples by adding the `bevy/track_location` feature to see that the locations of send and trigger events are also logged.

## Cargo Features

### enabled

This feature if removed will entirely disable the `LogEventsPlugin` and the functions from the `LogEvent` trait to make them do nothing. No systems will be added, no resources will be inserted and no logging will occur.

To remove it you can setup your `Cargo.toml` as follow :
```toml
[features]
# Create a feature to state that the crate is enabled
dev = ["bevy_log_events/enabled"]
# You may want to set that feature as default
default = ["dev"]

[dependencies]
# Declare that you do not want default-features in your dependencies
bevy_log_events = { version = "0.7.0", default-features = false }
```

Then you can run your program as follow :
```
// If you do not set the feature dev as default do
cargo run --features dev
// Otherwise run your program without default features
cargo run --no-default-features
```

## Note

Messages registered with the use of `log_message` or `add_and_log_message` are all logged in the `Last` schedule inside the `LogMessagesSystems` at the end of each frame. So keep in mind that these messages will be logged with a delay and if many messages of different types are sent in the same frame they may not be logged in the same order they were sent.

## Bevy Versions Table

| bevy_log_events | bevy | bevy_egui |
| --------------- | ---- | --------- |
| 0.7             | 0.18 | 0.39      |
| 0.6             | 0.17 | 0.37      |
| 0.5             | 0.16 | 0.34      |
| 0.4.2           | 0.15 | 0.32      |

| bevy_log_events | bevy | bevy_editor_pls |
| --------------- | ---- | --------------- |
| 0.3             | 0.14 | 0.9 - 0.10      |
| 0.2             | 0.14 | 0.9 - 0.10      |
| 0.1             | 0.13 | 0.8             |
