mod bar;
mod baz;
mod foo;
mod qux;

use bevy::{prelude::*, time::Stopwatch};
use bevy_log_events::prelude::*;

use rand::Rng;

fn add_event<T: Event + std::fmt::Debug + Default>(app: &mut App) {
    app.add_and_log_event::<T>()
        .add_systems(Update, fire_event::<T>);
}

pub(super) fn plugin(app: &mut App) {
    add_event::<A>(app);
    add_event::<B>(app);
    add_event::<C>(app);
    add_event::<D>(app);
    add_event::<E>(app);
    app.add_plugins((bar::plugin, foo::plugin, baz::plugin, qux::plugin));
}

fn fire_event<T: Event + Default>(
    time: Res<Time>,
    mut events: EventWriter<T>,
    mut stopwatch: Local<Stopwatch>,
    mut trigger: Local<f32>,
) {
    stopwatch.tick(time.delta());
    if stopwatch.elapsed_secs() > *trigger {
        stopwatch.reset();
        *trigger = rand::thread_rng().gen_range(1.0..5.0);
        events.send(T::default());
    }
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
