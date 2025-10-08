use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_egui::EguiPlugin;
use bevy_log_events::prelude::*;

// You must implement Debug for the messages and events you want to log
#[derive(Message, Debug)]
struct MyMessage {
    #[allow(dead_code)]
    value: usize,
}

#[derive(Event, Debug)]
struct MyEvent {
    #[allow(dead_code)]
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
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_window,
                (send_my_event, trigger_my_event).run_if(on_timer(Duration::from_secs(1))),
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite {
        image: asset_server.load("bevy_icon.png"),
        ..default()
    });
    commands.spawn(Node::default()).with_children(|builder| {
        builder.spawn((
            Text::new("Press Space to toggle the settings window"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
        ));
    });
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
