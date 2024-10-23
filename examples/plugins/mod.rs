mod bar;
mod baz;
mod foo;
mod qux;

use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_log_events::prelude::*;

fn add_event<T: Event + std::fmt::Debug + Default>(app: &mut App) {
    app.add_and_log_event::<T>().add_systems(
        Update,
        fire_event::<T>.run_if(on_timer(Duration::from_secs(1))),
    );
}

pub(super) fn plugin(app: &mut App) {
    add_event::<A>(app);
    add_event::<B>(app);
    add_event::<C>(app);
    add_event::<D>(app);
    app.add_plugins((bar::plugin, foo::plugin, baz::plugin, qux::plugin))
        .add_and_log_event::<TriggeredAndSent>()
        .log_triggered::<TriggeredAndSent>()
        .log_triggered::<Triggered>()
        .log_trigger::<OnAdd, MyComponent>()
        .log_trigger::<OnInsert, MyComponent>()
        .log_trigger::<OnRemove, MyComponent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (trigger, trigger_and_send, modify_entity).run_if(on_timer(Duration::from_secs(1))),
        );
}

fn fire_event<T: Event + Default>(mut events: EventWriter<T>) {
    events.send(T::default());
}

fn trigger(mut commands: Commands) {
    commands.trigger(Triggered);
}

fn trigger_and_send(mut commands: Commands, mut events: EventWriter<TriggeredAndSent>) {
    commands.trigger(TriggeredAndSent { triggered: true });
    events.send(TriggeredAndSent { triggered: false });
}

fn setup(mut commands: Commands) {
    commands.spawn((MyComponent { index: 0 }, MyEntity));
}

fn modify_entity(
    mut commands: Commands,
    query: Query<(Entity, Option<&MyComponent>), With<MyEntity>>,
    mut index: Local<usize>,
) {
    let (entity, my_component) = query.single();
    if my_component.is_some() && *index % 5 == 0 {
        commands.entity(entity).remove::<MyComponent>();
    } else {
        commands
            .entity(entity)
            .insert(MyComponent { index: *index });
    }
    *index += 1;
}

#[derive(Event, Debug, Default)]
struct A;

#[derive(Event, Debug, Default)]
struct B;

#[derive(Event, Debug, Default)]
struct C;

#[derive(Event, Debug, Default)]
struct D;

#[derive(Event, Debug)]
struct Triggered;

#[derive(Event, Debug)]
struct TriggeredAndSent {
    #[allow(dead_code)]
    triggered: bool,
}

#[derive(Component)]
struct MyEntity;

#[derive(Component, Debug)]
struct MyComponent {
    #[allow(dead_code)]
    index: usize,
}
