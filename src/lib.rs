#![warn(missing_docs)]

//! [`bevy_log_events`](https://github.com/YellowWaitt/bevy_log_events) is a
//! [Bevy](https://bevyengine.org/) plugin that introduce the [LogEvent] trait for Bevy's App.
//! It will helps you log your [Event] and [Message] while allowing you to configure
//! independently how each of them are logged during runtime.

#[cfg(feature = "enabled")]
pub mod settings_window;
#[cfg(feature = "enabled")]
mod systems;
#[cfg(feature = "enabled")]
mod utils;

#[cfg(feature = "enabled")]
use std::{any::type_name, collections::BTreeMap};
use std::{marker::PhantomData, path::PathBuf};

use bevy::{log::Level, prelude::*};

#[cfg(feature = "enabled")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "enabled")]
use systems::{log_component, log_event, log_message, register_event};
#[cfg(feature = "enabled")]
use utils::{deserialize_level, serialize_level, trigger_name};

/// Re-export of everything you need.
pub mod prelude {
    pub use super::{
        EventSettings, LogEvent, LogEventsPlugin, LogEventsPluginSettings, LogMessagesSystems,
        LoggedEventSettings,
    };
}

/// The [Plugin] to add to enable the logging of [Event] and [Message].
pub struct LogEventsPlugin {
    /// Path were the settings will be stored and loaded. If the specified file
    /// can not be found a new one will be created.
    pub settings_path: PathBuf,
}

impl LogEventsPlugin {
    /// Lets you specify the location were the settings will be stored.
    pub fn new(settings_path: impl Into<PathBuf>) -> Self {
        Self {
            settings_path: settings_path.into(),
        }
    }
}

impl Default for LogEventsPlugin {
    fn default() -> Self {
        Self {
            settings_path: "assets/log_settings.ron".into(),
        }
    }
}

#[cfg(not(feature = "enabled"))]
impl Plugin for LogEventsPlugin {
    fn build(&self, _app: &mut App) {}
}

/// The [SystemSet] were the [Message] registered with [log_message](LogEvent::log_message)
/// and [add_and_log_message](LogEvent::add_and_log_message) will be logged.
///
/// This [SystemSet] is configured to run in the [Last] schedule at the end of each
/// frame and the events will be log one [Message] type at a time.
/// So keep in mind that the messages logged this way will be with a delay and not
/// necessarily in the same order they were sent.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogMessagesSystems;

/// Common structure used to describe how the [Event] and [Message] will be logged.
///
/// To modify how a particular [Event] or [Message] will be logged you will need
/// to access his [LoggedEventSettings] associated [Resource].
#[derive(Clone, Copy)]
#[cfg_attr(feature = "enabled", derive(Deserialize, Serialize))]
pub struct EventSettings {
    /// Whether the [Event] or [Message] will be logged or not.
    pub enabled: bool,
    /// If true use the pretty-printing debug flag `{:#?}`.
    /// Otherwise use the compact-printing debug flag `{:?}`.
    pub pretty: bool,
    #[cfg_attr(
        feature = "enabled",
        serde(
            serialize_with = "serialize_level",
            deserialize_with = "deserialize_level"
        )
    )]
    /// The [Level] used for logging.
    pub level: Level,
}

impl Default for EventSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            pretty: true,
            level: Level::INFO,
        }
    }
}

/// The settings used to configure the [LogEventsPlugin].
#[derive(Resource)]
pub struct LogEventsPluginSettings {
    /// If false nothing will be logged.
    pub enabled: bool,
    /// Whether to show or not the window to configure all the [LoggedEventSettings].
    pub show_window: bool,
    #[cfg(feature = "enabled")]
    saved_settings: PathBuf,
    #[cfg(feature = "enabled")]
    previous_settings: BTreeMap<String, EventSettings>,
}

/// The [Resource] that contains the settings used to log a particular [Event] or [Message].
#[derive(Resource, Deref, DerefMut)]
pub struct LoggedEventSettings<E, C = ()> {
    /// The settings used for logging. See [EventSettings].
    #[deref]
    pub settings: EventSettings,
    _phantom: PhantomData<(E, C)>,
}

impl<E, C> Default for LoggedEventSettings<E, C> {
    fn default() -> Self {
        Self {
            settings: EventSettings::default(),
            _phantom: PhantomData,
        }
    }
}

/// The Trait implemented on [App] that helps you log [Event] and [Message].
pub trait LogEvent {
    /// This function add a system in the [Last] schedule inside the [LogMessagesSystems]
    /// in charge of logging all the [Message] `M` sent with the corresponding [MessageWriter].
    fn log_message<M>(&mut self) -> &mut Self
    where
        M: Message + std::fmt::Debug;

    /// Add and log a [Message] in one go. This is equivalent to :
    /// ```
    /// app.add_message::<M>()
    ///    .log_message::<M>()
    /// ```
    fn add_and_log_message<M>(&mut self) -> &mut Self
    where
        M: Message + std::fmt::Debug;

    /// This function spawn an [Observer] that will log all [Event] `E`.
    ///
    /// Since [EntityEvent] are [Event] you can log them using this function.
    fn log_event<E>(&mut self) -> &mut Self
    where
        E: Event + std::fmt::Debug;

    /// Using this function you can spawn an observer that can log [lifecycle] events
    /// for component `C`.
    ///
    /// [lifecycle]: bevy::ecs::lifecycle
    fn log_trigger<E, C>(&mut self) -> &mut Self
    where
        E: EntityEvent,
        C: Component + std::fmt::Debug;

    /// Log all the [lifecycle] events for the given component. This is equivalent to :
    ///
    /// [lifecycle]: bevy::ecs::lifecycle
    /// ```
    /// app.log_trigger::<Add, C>()
    ///    .log_trigger::<Insert, C>()
    ///    .log_trigger::<Replace, C>()
    ///    .log_trigger::<Remove, C>()
    ///    .log_trigger::<Despawn, C>()
    /// ```
    fn log_component_lifecycle<C>(&mut self) -> &mut Self
    where
        C: Component + std::fmt::Debug,
    {
        self.log_trigger::<Add, C>()
            .log_trigger::<Insert, C>()
            .log_trigger::<Replace, C>()
            .log_trigger::<Remove, C>()
            .log_trigger::<Despawn, C>()
    }
}

impl LogEvent for App {
    fn log_message<M>(&mut self) -> &mut Self
    where
        M: Message + std::fmt::Debug,
    {
        #[cfg(feature = "enabled")]
        {
            let name = type_name::<M>();
            if register_event::<LoggedEventSettings<M>>(self.world_mut(), name.to_string()) {
                self.add_systems(Last, log_message::<M>.in_set(LogMessagesSystems));
            } else {
                warn!("You tried to use log_message twice for the message \"{name}\"");
            }
        }
        self
    }

    fn add_and_log_message<M>(&mut self) -> &mut Self
    where
        M: Message + std::fmt::Debug,
    {
        self.add_message::<M>().log_message::<M>()
    }

    fn log_event<E>(&mut self) -> &mut Self
    where
        E: Event + std::fmt::Debug,
    {
        #[cfg(feature = "enabled")]
        {
            let name = type_name::<E>();
            if register_event::<LoggedEventSettings<E>>(self.world_mut(), name.to_string()) {
                let observer = Observer::new(log_event::<E>);
                self.world_mut()
                    .spawn((observer, Name::new(format!("LogEvent<{name}>"))));
            } else {
                warn!("You tried to use log_event twice for the event \"{name}\"");
            }
        }
        self
    }

    fn log_trigger<E, C>(&mut self) -> &mut Self
    where
        E: EntityEvent,
        C: Component + std::fmt::Debug,
    {
        #[cfg(feature = "enabled")]
        {
            let name = trigger_name::<E, C>();
            if register_event::<LoggedEventSettings<E, C>>(self.world_mut(), name.to_string()) {
                let observer = Observer::new(log_component::<E, C>);
                self.world_mut()
                    .spawn((observer, Name::new(format!("Log{name}"))));
            } else {
                warn!("You tried to use log_trigger twice for the trigger \"{name}\"");
            }
        }
        self
    }
}
