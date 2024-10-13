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
