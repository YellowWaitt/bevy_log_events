#![warn(missing_docs)]

//! [`bevy_log_events`](https://github.com/YellowWaitt/bevy_log_events) is a
//! [Bevy](https://bevyengine.org/) plugin that introduce
//! the [LogEvent] trait for Bevy's App.
//! It will helps you log your [Event] while allowing you to configure independently
//! how each events are logged even during program execution.

#[cfg(feature = "editor_window")]
mod editor_window;
mod utils;

use std::{
    any::type_name,
    collections::BTreeMap,
    error::Error,
    fmt::Write,
    fs::{create_dir_all, File},
    marker::PhantomData,
    path::{Path, PathBuf},
};

use bevy::{ecs::component::ComponentId, log::Level, prelude::*};

use ron::{de::from_reader, ser::PrettyConfig};
use serde::{Deserialize, Serialize};

use utils::{
    deserialize_level, get_log_settings_by_id, serialize_level, trigger_name, LoggedEventsSettings,
};

/// Re-export of everything you need.
pub mod prelude {
    pub use super::{
        EventSettings, LogEvent, LogEventsPlugin, LogEventsPluginSettings, LogEventsSet,
        LoggedEventSettings,
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

impl Plugin for LogEventsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LogEventsPluginSettings::new(self))
            .insert_resource(LogSettingsIds::default())
            .configure_sets(Last, LogEventsSet.run_if(plugin_enabled))
            .add_systems(PostUpdate, save_settings.run_if(on_event::<AppExit>()));
        #[cfg(feature = "editor_window")]
        {
            app.add_plugins(editor_window::plugin);
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct LogSettingsIds(BTreeMap<String, ComponentId>);

/// The [SystemSet] were the [Event] will be logged.
///
/// All the [Event] are logged inside the [Last] schedule at the end of each frame,
/// one [Event] type at a time. So keep in mind that if many [Event] of differents
/// type are sent in the same frame they may not be logged in the same order they were sent.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogEventsSet;

/// Common structure used to describe how the [Event] will be logged.
///
/// To modify how a particular [Event] will be logged you will need to access his
/// [LoggedEventSettings] associated [Resource].
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct EventSettings {
    /// Whether the [Event] will be logged or not.
    pub enabled: bool,
    /// If true use the pretty-printing debug flag `{:#?}` to log the [Event].
    /// Otherwise use the compact-printing debug flag `{:?}`.
    pub pretty: bool,
    #[serde(
        serialize_with = "serialize_level",
        deserialize_with = "deserialize_level"
    )]
    /// The [Level] at which the [Event] will be logged.
    pub level: Level,
}

impl Default for EventSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            level: Level::INFO,
            pretty: true,
        }
    }
}

/// The settings used to configure the [LogEventsPlugin].
#[derive(Resource)]
pub struct LogEventsPluginSettings {
    /// If false no [Event] will be logged.
    pub enabled: bool,
    saved_settings: PathBuf,
    previous_settings: BTreeMap<String, EventSettings>,
}

impl LogEventsPluginSettings {
    fn new(log_plugin: &LogEventsPlugin) -> Self {
        let path = &log_plugin.settings_path;
        match Self::load_saved_settings(path) {
            Ok(new) => new,
            Err(err) => {
                warn!("Error while trying to load settings from {:?}: {}. Using default settings instead.", path, err);
                LogEventsPluginSettings::default(path)
            }
        }
    }

    fn default(path: &Path) -> Self {
        Self {
            enabled: true,
            saved_settings: path.to_path_buf(),
            previous_settings: BTreeMap::new(),
        }
    }

    fn load_saved_settings(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let saved_settings: LoggedEventsSettings = from_reader(file)?;
        let new = Self {
            enabled: saved_settings.plugin_enabled,
            saved_settings: path.to_path_buf(),
            previous_settings: saved_settings.events_settings,
        };
        Ok(new)
    }
}

fn plugin_enabled(plugin_settings: Res<LogEventsPluginSettings>) -> bool {
    plugin_settings.enabled
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
/// 1. [log_event](LogEvent::log_event) and [add_and_log_event](LogEvent::add_and_log_event)
///    will log [Event] sent with an [EventWriter].\
///    These functions will not interact with triggered events.\
///    These events will be logged with a delay at the end of each frame inside the [LogEventsSet].
/// 2. [log_triggered](LogEvent::log_triggered) and [log_trigger](LogEvent::log_trigger)
///    will log triggered [Event].\
///    These functions will not interact with sent events.\
///    These events will logged without a delay as soon as they are triggered.
///
/// As each one of these functions log events in independant situations you can use
/// several of them at the same time for the same [Event], you will not get the same
/// event log multiple times by doing that.
///
/// If an event `E` is registred with [log_event](LogEvent::log_event) and [log_triggered](LogEvent::log_triggered),
/// it will share the same [LoggedEventSettings] resource for logging in both context.\
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
    /// This was designed with [OnAdd], [OnInsert] and [OnRemove] in mind but you can use
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
}

impl LogEvent for App {
    fn log_event<E>(&mut self) -> &mut Self
    where
        E: Event + std::fmt::Debug,
    {
        self.insert_resource(LoggedEventSettings::<E>::default())
            .add_systems(Startup, register_event::<E>)
            .add_systems(Last, log_event::<E>.in_set(LogEventsSet))
    }

    fn add_and_log_event<E>(&mut self) -> &mut Self
    where
        E: Event + std::fmt::Debug,
    {
        self.add_event::<E>().log_event::<E>()
    }

    fn log_triggered<E>(&mut self) -> &mut Self
    where
        E: Event + std::fmt::Debug,
    {
        let observer = Observer::new(log_triggered::<E>);
        self.world_mut().spawn((
            observer,
            Name::new(format!("LogTrigger<{}>", type_name::<E>())),
        ));
        self.insert_resource(LoggedEventSettings::<E>::default())
            .add_systems(Startup, register_event::<E>)
    }

    fn log_trigger<E, C>(&mut self) -> &mut Self
    where
        E: Event,
        C: Component + std::fmt::Debug,
    {
        let observer = Observer::new(log_component::<E, C>);
        self.world_mut().spawn((
            observer,
            Name::new(format!("Log{}", trigger_name::<E, C>())),
        ));
        self.insert_resource(LoggedEventSettings::<E, C>::default())
            .add_systems(Startup, register_component::<E, C>)
    }
}

fn register_event<E: Event>(world: &mut World) {
    let name = type_name::<E>().to_string();
    world.resource_scope(|world, plugin_settings: Mut<LogEventsPluginSettings>| {
        if let Some(previous) = plugin_settings.previous_settings.get(&name) {
            let mut event_settings = world.resource_mut::<LoggedEventSettings<E>>();
            **event_settings = *previous;
        }
    });
    world.resource_scope(|world, mut log_settings_ids: Mut<LogSettingsIds>| {
        let id = world
            .components()
            .resource_id::<LoggedEventSettings<E>>()
            .unwrap();
        log_settings_ids.insert(name, id);
    });
}

fn register_component<E: Event, C: Component>(world: &mut World) {
    let name = trigger_name::<E, C>();
    world.resource_scope(|world, plugin_settings: Mut<LogEventsPluginSettings>| {
        if let Some(previous) = plugin_settings.previous_settings.get(&name) {
            let mut event_settings = world.resource_mut::<LoggedEventSettings<E, C>>();
            **event_settings = *previous;
        }
    });
    world.resource_scope(|world, mut log_settings_ids: Mut<LogSettingsIds>| {
        let id = world
            .components()
            .resource_id::<LoggedEventSettings<E, C>>()
            .unwrap();
        log_settings_ids.insert(name, id);
    });
}

fn log(level: Level, to_log: &str) {
    match level {
        Level::ERROR => error!("{}", to_log),
        Level::WARN => warn!("{}", to_log),
        Level::INFO => info!("{}", to_log),
        Level::DEBUG => debug!("{}", to_log),
        Level::TRACE => trace!("{}", to_log),
    }
}

fn format_and_log_event<E>(settings: &EventSettings, event: &E)
where
    E: std::fmt::Debug,
{
    let name = type_name::<E>();
    let to_log = if settings.pretty {
        format!("{}: {:#?}", name, event)
    } else {
        format!("{}: {:?}", name, event)
    };
    log(settings.level, &to_log);
}

fn format_entity_and_object<T>(
    settings: &EventSettings,
    event_name: &str,
    entity_name: &Option<&Name>,
    entity: Entity,
    object: &T,
) -> Result<String, Box<dyn Error>>
where
    T: std::fmt::Debug,
{
    let mut to_log = String::new();
    to_log.write_fmt(format_args!("{} on ", event_name))?;
    if let Some(name) = entity_name {
        to_log.write_fmt(format_args!("{}({}): ", name, entity))?;
    } else {
        to_log.write_fmt(format_args!("{}: ", entity))?;
    }
    if settings.pretty {
        to_log.write_fmt(format_args!("{:#?}", object))?;
    } else {
        to_log.write_fmt(format_args!("{:?}", object))?;
    }
    Ok(to_log)
}

fn log_event<E>(settings: Res<LoggedEventSettings<E>>, mut events: EventReader<E>)
where
    E: Event + std::fmt::Debug,
{
    if !settings.enabled {
        return;
    }
    for event in events.read() {
        format_and_log_event(&settings, event);
    }
}

fn log_triggered<E>(
    trigger: Trigger<E>,
    plugin_settings: Res<LogEventsPluginSettings>,
    settings: Res<LoggedEventSettings<E>>,
    names: Query<&Name>,
) where
    E: Event + std::fmt::Debug,
{
    if !plugin_settings.enabled || !settings.enabled {
        return;
    }
    let entity = trigger.entity();
    let event = trigger.event();
    if entity != Entity::PLACEHOLDER {
        let name = names.get(entity).ok();
        if let Ok(to_log) =
            format_entity_and_object::<E>(&settings, type_name::<E>(), &name, entity, event)
        {
            log(settings.level, &to_log);
        }
    } else {
        format_and_log_event(&settings, event);
    }
}

fn log_component<E, C>(
    trigger: Trigger<E, C>,
    plugin_settings: Res<LogEventsPluginSettings>,
    settings: Res<LoggedEventSettings<E, C>>,
    query: Query<(&C, Option<&Name>)>,
) where
    E: Event,
    C: Component + std::fmt::Debug,
{
    if !plugin_settings.enabled || !settings.enabled {
        return;
    }
    let entity = trigger.entity();
    if let Ok((component, name)) = query.get(entity) {
        if let Ok(to_log) = format_entity_and_object::<C>(
            &settings,
            &trigger_name::<E, C>(),
            &name,
            entity,
            component,
        ) {
            log(settings.level, &to_log);
        }
    }
}

fn save_settings(world: &mut World) {
    let log_settings_ids = world.resource::<LogSettingsIds>();
    let mut all_settings = BTreeMap::new();
    for (name, id) in log_settings_ids.iter() {
        let event_settings = get_log_settings_by_id(world, id);
        all_settings.insert(name.clone(), *event_settings);
    }
    let plugin_settings = world.resource::<LogEventsPluginSettings>();
    let to_serialize = LoggedEventsSettings {
        plugin_enabled: plugin_settings.enabled,
        events_settings: all_settings,
    };
    let path = plugin_settings.saved_settings.clone();
    if let Err(e) = serialize_settings(&path, to_serialize) {
        error!(
            "Could not save {} at {:?} due to {:?}",
            type_name::<LoggedEventsSettings>(),
            path,
            e
        );
    }
}

fn serialize_settings(
    path: &PathBuf,
    to_serialize: LoggedEventsSettings,
) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    let config = PrettyConfig::default().struct_names(true);
    let serialized = ron::ser::to_string_pretty(&to_serialize, config)?;
    std::io::Write::write_all(&mut file, serialized.as_bytes())?;
    Ok(())
}
