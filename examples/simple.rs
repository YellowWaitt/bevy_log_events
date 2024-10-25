use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_log_events::prelude::*;

// You must implement Debug for the events you want to log
#[derive(Event, Debug)]
struct MyEvent {
    #[allow(dead_code)]
    source: String,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // Add the logging plugin
            LogEventsPlugin::new("assets/simple.ron"),
        ))
        // You can now use add_and_log_event instead of add_event to add and log your events
        .add_and_log_event::<MyEvent>()
        // Triggered events can be log too with the use of observers
        .log_triggered::<MyEvent>()
        // Using log_event you can also log external events you did not add yourself
        .log_event::<CursorEntered>()
        .add_systems(
            Update,
            (send_my_event, trigger_my_event).run_if(on_timer(Duration::from_secs(1))),
        )
        .run();
}

// MyEvent will be sent and logged every second at the end of each frame
fn send_my_event(mut events: EventWriter<MyEvent>) {
    events.send(MyEvent {
        source: "sent".into(),
    });
}

// MyEvent will be triggered and logged every second during Update
fn trigger_my_event(mut commands: Commands) {
    commands.trigger(MyEvent {
        source: "triggered".into(),
    });
}
