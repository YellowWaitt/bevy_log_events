mod plugins;

use bevy::{
    app::AppExit,
    prelude::*,
    window::{WindowClosed, WindowCreated, WindowResized},
};
use bevy_editor_pls::EditorPlugin;
use bevy_log_events::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EditorPlugin::default(),
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
        .log_event::<AppExit>()
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load("bevy_icon.png"),
        ..default()
    });
}
