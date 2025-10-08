use bevy::prelude::*;

use super::add_message;

pub(super) fn plugin(app: &mut App) {
    add_message::<A>(app);
    add_message::<B>(app);
    add_message::<C>(app);
    add_message::<D>(app);
    add_message::<E>(app);
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
