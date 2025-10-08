mod bar;
mod baz;
mod foo;
mod qux;

use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_log_events::prelude::*;

fn add_message<M: Message + std::fmt::Debug + Default>(app: &mut App) {
    app.add_and_log_message::<M>().add_systems(
        Update,
        write_message::<M>.run_if(on_timer(Duration::from_secs(1))),
    );
}

pub(super) fn plugin(app: &mut App) {
    add_message::<A>(app);
    add_message::<B>(app);
    add_message::<C>(app);
    add_message::<D>(app);
    add_message::<E>(app);
    add_message::<F>(app);
    app.add_plugins((bar::plugin, foo::plugin, baz::plugin, qux::plugin))
        .log_event::<MyEvent>()
        .log_event::<MyEntityEvent>()
        .log_component_lifecycle::<MyComponent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (trigger_event, trigger_entity_event, modify_entity)
                .run_if(on_timer(Duration::from_secs(1))),
        );
}

fn write_message<M: Message + Default>(mut events: MessageWriter<M>) {
    events.write(M::default());
}

fn trigger_event(mut commands: Commands) {
    commands.trigger(MyEvent);
}

fn trigger_entity_event(mut commands: Commands, my_entity: Single<Entity, With<MyEntity>>) {
    commands.trigger(MyEntityEvent {
        entity: my_entity.into_inner(),
    });
}

fn setup(mut commands: Commands) {
    commands.spawn((Name::new("MyEntity"), MyComponent { index: 0 }, MyEntity));
}

fn modify_entity(
    mut commands: Commands,
    my_entity: Single<(Entity, Option<&MyComponent>), With<MyEntity>>,
    mut index: Local<usize>,
) -> Result {
    let (entity, my_component) = my_entity.into_inner();
    if my_component.is_some() && index.is_multiple_of(5) {
        commands.entity(entity).remove::<MyComponent>();
    } else {
        commands
            .entity(entity)
            .insert(MyComponent { index: *index });
    }
    *index += 1;
    Ok(())
}

#[derive(Message, Debug, Default)]
struct A;

#[derive(Message, Debug, Default)]
struct B;

#[derive(Message, Debug, Default)]
struct C;

#[derive(Message, Debug, Default)]
struct D;

#[derive(Message, Debug, Default)]
struct E;

#[derive(Message, Debug, Default)]
struct F;

#[derive(Event, Debug)]
struct MyEvent;

#[derive(EntityEvent, Debug)]
struct MyEntityEvent {
    entity: Entity,
}

#[derive(Component)]
struct MyEntity;

#[derive(Component, Debug)]
struct MyComponent {
    #[allow(dead_code)]
    index: usize,
}
