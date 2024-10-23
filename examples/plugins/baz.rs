use bevy::prelude::*;

use super::add_event;

pub(super) fn plugin(app: &mut App) {
    add_event::<A>(app);
    add_event::<B>(app);
    add_event::<C>(app);
    add_event::<D>(app);
    add_event::<E>(app);
    add_event::<F>(app);
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

#[derive(Event, Debug, Default)]
struct F;
