mod plugins;

use bevy::{
    app::AppExit,
    prelude::*,
    window::{AppLifecycle, WindowClosed, WindowCreated, WindowResized},
};
use bevy_log_events::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            LogEventsPlugin::new("assets/window.ron"),
            plugins::plugin,
        ))
        .log_event::<WindowResized>()
        .log_event::<WindowCreated>()
        .log_event::<WindowClosed>()
        .log_event::<CursorMoved>()
        .log_event::<CursorEntered>()
        .log_event::<CursorLeft>()
        .log_event::<WindowMoved>()
        .log_event::<AppLifecycle>()
        .log_event::<AppExit>()
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_window)
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

fn toggle_window(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut pluggin_settings: ResMut<LogEventsPluginSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        pluggin_settings.show_window = !pluggin_settings.show_window;
    }
}
