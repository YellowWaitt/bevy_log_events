use std::{any::type_name, collections::BTreeMap};

use bevy::{ecs::component::ComponentId, log::Level, prelude::*};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

use crate::EventSettings;

#[derive(Serialize, Deserialize)]
pub(crate) struct LoggedEventsSettings {
    pub plugin_enabled: bool,
    pub events_settings: BTreeMap<String, EventSettings>,
}

pub(crate) fn serialize_level<S>(level: &Level, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(level.as_str())
}

pub(crate) fn deserialize_level<'de, D>(d: D) -> Result<Level, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(d)?;
    match s {
        "ERROR" => Ok(Level::ERROR),
        "WARN" => Ok(Level::WARN),
        "INFO" => Ok(Level::INFO),
        "DEBUG" => Ok(Level::DEBUG),
        "TRACE" => Ok(Level::TRACE),
        _ => Err(D::Error::custom(format!(
            "\"{s}\" does not represent a valid log Level"
        ))),
    }
}

fn type_stem<'a, T>() -> &'a str {
    type_name::<T>().split("::").last().unwrap()
}

pub(crate) fn trigger_name<E, C>() -> String {
    format!("{}<{}>", type_stem::<E>(), type_name::<C>())
}

pub(crate) fn get_log_settings_by_id<'a>(world: &'a World, id: &ComponentId) -> &'a EventSettings {
    let ptr = world.get_resource_by_id(*id).unwrap();
    unsafe { ptr.deref::<EventSettings>() }
}

pub(crate) fn get_log_settings_mut_by_id<'a>(
    world: &'a mut World,
    id: &ComponentId,
) -> &'a mut EventSettings {
    let mut_ptr = world.get_resource_mut_by_id(*id).unwrap();
    unsafe { mut_ptr.into_inner().deref_mut::<EventSettings>() }
}
