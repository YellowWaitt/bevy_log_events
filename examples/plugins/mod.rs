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
    add_event::<E>(app);
    app.add_plugins((bar::plugin, foo::plugin, baz::plugin, qux::plugin));
}

fn fire_event<T: Event + Default>(mut events: EventWriter<T>) {
    events.send(T::default());
}

#[derive(Event, Debug, Default)]
struct A;

#[derive(Event, Debug, Default)]
struct B;

#[derive(Event, Debug, Default)]
struct C;

#[derive(Event, Debug, Default)]
struct D;

#[derive(Event, Debug, Default)]
struct E;
