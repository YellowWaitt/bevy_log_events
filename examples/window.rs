mod plugins;

use bevy::{
    app::AppExit,
    prelude::*,
    window::{AppLifecycle, WindowClosed, WindowCreated, WindowResized},
};
use bevy_egui::EguiPlugin;
use bevy_log_events::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin::default(),
            LogEventsPlugin::new("assets/window.ron"),
            plugins::plugin,
        ))
        .log_message::<WindowResized>()
        .log_message::<WindowCreated>()
        .log_message::<WindowClosed>()
        .log_message::<CursorMoved>()
        .log_message::<CursorEntered>()
        .log_message::<CursorLeft>()
        .log_message::<WindowMoved>()
        .log_message::<AppLifecycle>()
        .log_message::<AppExit>()
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
    mut plugin_settings: ResMut<LogEventsPluginSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        plugin_settings.show_window = !plugin_settings.show_window;
    }
}
