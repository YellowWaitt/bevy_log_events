#![warn(missing_docs)]

//! [`bevy_log_events`](https://github.com/YellowWaitt/bevy_log_events) is a
//! [Bevy](https://bevyengine.org/) plugin that introduce
//! the [LogEvent] trait for Bevy's App.
//! It will helps you log your [Event] while allowing you to configure independently
//! how each events are logged even during program execution.

#[cfg(feature = "enabled")]
#[cfg(feature = "editor_window")]
compile_error!(
    "The \"editor_window\" feature is not yet available for Bevy 0.15.
It will be made available again when the \"bevy_editor_pls\" will be updated to Bevy 0.15."
);
// mod editor_window;
#[cfg(feature = "enabled")]
mod settings_window;
#[cfg(feature = "enabled")]
mod systems;
#[cfg(feature = "enabled")]
mod utils;

#[cfg(feature = "enabled")]
use std::{any::type_name, collections::BTreeMap};
use std::{marker::PhantomData, path::PathBuf};

use bevy::{log::Level, prelude::*, state::state::FreelyMutableState};

#[cfg(feature = "enabled")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "enabled")]
use systems::{log_component, log_event, log_triggered, register_component, register_event};
#[cfg(feature = "enabled")]
use utils::{deserialize_level, serialize_level, trigger_name};

/// Re-export of everything you need.
pub mod prelude {
    pub use super::{
        EventSettings, LogEvent, LogEventsPlugin, LogEventsPluginSettings, LogEventsSet,
        LoggedEventSettings, RegisterEventsSet,
    };
}

/// The [Plugin] to add to enable the logging of [Event].
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

/// The [SystemSet] were the [Event] are registred.
///
/// This [SystemSet] is configured to run in the [Startup] schedule. This is were
/// the saved [LoggedEventSettings] resources from the previous run of the program
/// will be restored. After this set you can access these resources to read and write
/// on them.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegisterEventsSet;

/// The [SystemSet] were the [Event] registred with [log_event](LogEvent::log_event)
/// and [add_and_log_event](LogEvent::add_and_log_event) will be log.
///
/// This [SystemSet] is configured to run in the [Last] schedule at the end of each
/// frame and the events will be log one [Event] type at a time.
/// So keep in mind that the events logged this way will be with a delay and not
/// necessarily in the same order they were sent.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogEventsSet;

/// Common structure used to describe how the [Event] will be logged.
///
/// To modify how a particular [Event] will be logged you will need to access his
/// [LoggedEventSettings] associated [Resource].
#[derive(Clone, Copy)]
#[cfg_attr(feature = "enabled", derive(Deserialize, Serialize))]
pub struct EventSettings {
    /// Whether the [Event] will be logged or not.
    pub enabled: bool,
    /// If true use the pretty-printing debug flag `{:#?}` to log the [Event].
    /// Otherwise use the compact-printing debug flag `{:?}`.
    pub pretty: bool,
    #[cfg_attr(
        feature = "enabled",
        serde(
            serialize_with = "serialize_level",
            deserialize_with = "deserialize_level"
        )
    )]
    /// The [Level] at which the [Event] will be logged.
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
    /// If false no [Event] will be logged.
    pub enabled: bool,
    /// Whether to show or not the window to configure the [LoggedEventSettings].
    pub show_window: bool,
    #[cfg(feature = "enabled")]
    saved_settings: PathBuf,
    #[cfg(feature = "enabled")]
    previous_settings: BTreeMap<String, EventSettings>,
}

/// The [Resource] that contains the settings used to log a particular [Event].
#[derive(Resource, Deref, DerefMut)]
pub struct LoggedEventSettings<E, C = ()> {
    /// The settings describing how the [Event] will be logged. See [EventSettings].
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

/// The Trait implemented on [App] that helps you log [Event].
///
/// In Bevy you can interact with events in two ways :
/// 1. using [EventWriter] to send events and [EventReader] to read them.
/// 2. using [Commands] to trigger events and [Observer] to react to the [Trigger].
///
/// This trait offers two complementary ways of interacting with events, depending on how they were emitted :
/// 1. [log_event](LogEvent::log_event), [add_and_log_event](LogEvent::add_and_log_event) and
///    [add_and_log_state_scoped_event](LogEvent::add_and_log_state_scoped_event) will
///    log [Event] sent with an [EventWriter].<br>
///    These functions will not interact with triggered events.<br>
///    These events will be logged with a delay at the end of each frame inside the [LogEventsSet].
/// 2. [log_triggered](LogEvent::log_triggered) and [log_trigger](LogEvent::log_trigger)
///    will log triggered [Event].<br>
///    These functions will not interact with sent events.<br>
///    These events will logged without a delay as soon as they are triggered.
///
/// As each one of these functions log events in independant situations you can use
/// several of them at the same time for the same [Event] type, you will not get the same
/// event log multiple times by doing that.
///
/// If an event `E` is registred with [log_event](LogEvent::log_event) and [log_triggered](LogEvent::log_triggered),
/// it will share the same [LoggedEventSettings] resource for logging in both context.<br>
/// In case of [log_trigger](LogEvent::log_trigger), you will get one [LoggedEventSettings] resource
/// for each pair of event and component (`E`, `C`) you register.
pub trait LogEvent {
    /// This function add a system in the [Last] schedule inside the [LogEventsSet]
    /// in charge of logging all the [Event] `E` sent with the corresponding [EventWriter].
    fn log_event<E>(&mut self) -> &mut Self
    where
        E: Event + std::fmt::Debug;

    /// Add and log an [Event] in one go. This is equivalent to :
    /// ```
    /// app.add_event::<E>()
    ///    .log_event::<E>()
    /// ```
    fn add_and_log_event<E>(&mut self) -> &mut Self
    where
        E: Event + std::fmt::Debug;

    /// Add and log a state scoped [Event] in one go. This is equivalent to :
    /// ```
    /// app.add_state_scoped_event::<E>(state)
    ///    .log_event::<E>()
    /// ```
    /// See [add_state_scoped_event](StateScopedEventsAppExt::add_state_scoped_event) for details.
    fn add_and_log_state_scoped_event<E>(&mut self, state: impl FreelyMutableState) -> &mut Self
    where
        E: Event + std::fmt::Debug;

    /// This function spawn an [Observer] that will log all triggered [Event] `E`.
    /// If in addition the [Trigger] targets an [Entity], it will also log the entity
    /// id and its [Name] if any.
    ///
    /// As an example:
    /// ```
    /// // If you log triggered events MyEvent
    /// app.log_triggered::<MyEvent>();
    ///
    /// // This will log MyEvent
    /// commands.trigger(MyEvent);
    /// // This will log MyEvent and the entity id
    /// commands.trigger_targets(MyEvent, entity);
    /// ```
    fn log_triggered<E>(&mut self) -> &mut Self
    where
        E: Event + std::fmt::Debug;

    /// This function spawn an [Observer] that react when an event [Event] `E` is triggered.
    /// If the [Trigger] targets an [Entity] `e`, it will fetch the [Component] `C` associated
    /// to `e` and log it with the entity id and its [Name] if any.
    ///
    /// This will not log the content of the triggered event. If you want to log the event use
    /// [log_triggered](LogEvent::log_triggered).
    ///
    /// This was designed with [OnAdd], [OnInsert], [OnRemove] and [OnReplace] in mind but you can use
    /// it with your own events too.
    ///
    /// As an example :
    /// ```
    /// // If you log MyComponent when MyEvent is triggered
    /// app.log_trigger::<MyEvent, MyComponent>();
    ///
    /// // This will log nothing
    /// commands.trigger(MyEvent);
    /// // If the entity has a MyComponent component it will
    /// // log the entity id and its associated MyComponent
    /// commands.trigger_targets(MyEvent, entity);
    ///
    /// // With this everytime MyComponent is added to an entity it
    /// // will log MyComponent and the entity id it was added to
    /// app.log_trigger::<OnAdd, MyComponent>();
    /// ```
    fn log_trigger<E, C>(&mut self) -> &mut Self
    where
        E: Event,
        C: Component + std::fmt::Debug;

    /// Log all the [ComponentHooks](bevy::ecs::component::ComponentHooks)
    /// for the given component. This is equivalent to :
    /// ```
    /// app.log_trigger::<OnAdd, C>()
    ///    .log_trigger::<OnInsert, C>()
    ///    .log_trigger::<OnReplace, C>()
    ///    .log_trigger::<OnRemove, C>()
    /// ```
    fn log_component_hooks<C>(&mut self) -> &mut Self
    where
        C: Component + std::fmt::Debug,
    {
        self.log_trigger::<OnAdd, C>()
            .log_trigger::<OnInsert, C>()
            .log_trigger::<OnReplace, C>()
            .log_trigger::<OnRemove, C>()
    }
}

impl LogEvent for App {
    fn log_event<E>(&mut self) -> &mut Self
    where
        E: Event + std::fmt::Debug,
    {
        #[cfg(feature = "enabled")]
        {
            if !self.world().contains_resource::<LoggedEventSettings<E>>() {
                self.insert_resource(LoggedEventSettings::<E>::default())
                    .add_systems(Startup, register_event::<E>.in_set(RegisterEventsSet))
                    .add_systems(Last, log_event::<E>.in_set(LogEventsSet));
            } else {
                warn!(
                    "You tried to use log_event twice for the event \"{}\"",
                    type_name::<E>()
                );
            }
        }
        self
    }

    fn add_and_log_event<E>(&mut self) -> &mut Self
    where
        E: Event + std::fmt::Debug,
    {
        self.add_event::<E>().log_event::<E>()
    }

    fn add_and_log_state_scoped_event<E>(&mut self, state: impl FreelyMutableState) -> &mut Self
    where
        E: Event + std::fmt::Debug,
    {
        self.add_state_scoped_event::<E>(state).log_event::<E>()
    }

    fn log_triggered<E>(&mut self) -> &mut Self
    where
        E: Event + std::fmt::Debug,
    {
        #[cfg(feature = "enabled")]
        {
            if !self.world().contains_resource::<LoggedEventSettings<E>>() {
                let observer = Observer::new(log_triggered::<E>);
                self.world_mut().spawn((
                    observer,
                    Name::new(format!("LogTrigger<{}>", type_name::<E>())),
                ));
                self.insert_resource(LoggedEventSettings::<E>::default())
                    .add_systems(Startup, register_event::<E>.in_set(RegisterEventsSet));
            } else {
                warn!(
                    "You tried to use log_triggered twice for the event \"{}\"",
                    type_name::<E>()
                );
            }
        }
        self
    }

    fn log_trigger<E, C>(&mut self) -> &mut Self
    where
        E: Event,
        C: Component + std::fmt::Debug,
    {
        #[cfg(feature = "enabled")]
        {
            if !self
                .world()
                .contains_resource::<LoggedEventSettings<E, C>>()
            {
                let observer = Observer::new(log_component::<E, C>);
                self.world_mut().spawn((
                    observer,
                    Name::new(format!("Log{}", trigger_name::<E, C>())),
                ));
                self.insert_resource(LoggedEventSettings::<E, C>::default())
                    .add_systems(
                        Startup,
                        register_component::<E, C>.in_set(RegisterEventsSet),
                    );
            } else {
                warn!(
                    "You tried to use log_trigger twice for the trigger \"{}\"",
                    trigger_name::<E, C>()
                );
            }
        }
        self
    }
}
