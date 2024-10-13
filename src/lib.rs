#![warn(missing_docs)]

//! [`bevy_log_events`](https://github.com/YellowWaitt/bevy_log_events) is a
//! [Bevy](https://bevyengine.org/) plugin that introduce
//! the [add_and_log_event](LogEvent::add_and_log_event) function for Bevy's App.
//! This plugin lets you log your Event while allowing you to configure independently
//! how each Event are logged during program execution.

#[cfg(feature = "editor_window")]
mod editor_window;
mod utils;

use std::{
    any::type_name,
    collections::BTreeMap,
    error::Error,
    fs::File,
    io::Write,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use bevy::{ecs::component::ComponentId, log::Level, prelude::*};

use ron::{de::from_reader, ser::PrettyConfig};
use serde::{Deserialize, Serialize};

use utils::{deserialize_level, get_log_settings_by_id, serialize_level, LoggedEventsSettings};

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

/// The [SystemSet] were the [Event] will be logged.
///
/// All the [Event] are logged inside the [Last] schedule at the end of each frame,
/// one [Event] type at a time. So keep in mind that if many [Event] of differents
/// type are sent in the same frame they will not be logged in the same order they are sent.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogEventsSet;

#[derive(Resource, Default, Deref, DerefMut)]
struct LogSettingsIds(BTreeMap<String, ComponentId>);

/// Common structure used to describe how the [Event] will be logged.
///
/// To modify how a particular [Event] will be logged you will need to access his
/// [LoggedEventSettings] associated [Resource].
#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct EventSettings {
    /// Whether the [Event] will be logged or not.
    pub enabled: bool,
    /// If true use the pretty-printing debug flag `{:#?}` to log the [Event].
    /// Otherwise use the compact printing debug flag `{:?}`.
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
pub struct LoggedEventSettings<T: Event> {
    /// The settings describing how the [Event] will be logged. See [EventSettings].
    #[deref]
    pub settings: EventSettings,
    _phantom: PhantomData<T>,
}

impl<T: Event> Default for LoggedEventSettings<T> {
    fn default() -> Self {
        Self {
            settings: EventSettings::default(),
            _phantom: PhantomData,
        }
    }
}

/// The Trait implemented on [App] that lets you log [Event].
pub trait LogEvent {
    /// Enable the logging for the [Event] `T`. This function add one system in charge of
    /// logging the [Event] inside the [LogEventsSet] and one system in [Startup]
    /// that will restore to the corresponding [LoggedEventSettings] the previous
    /// saved settings.
    fn log_event<T>(&mut self) -> &mut Self
    where
        T: Event + std::fmt::Debug;

    /// Lets you add and log an [Event] in one go. This is equivalent to :
    /// ```
    /// app.add_event::<T>()
    ///     .log_event::<T>()
    /// ```
    fn add_and_log_event<T>(&mut self) -> &mut Self
    where
        T: Event + std::fmt::Debug;
}

impl LogEvent for App {
    fn log_event<T>(&mut self) -> &mut Self
    where
        T: Event + std::fmt::Debug,
    {
        self.insert_resource(LoggedEventSettings::<T>::default())
            .add_systems(Startup, register_event::<T>)
            .add_systems(Last, log_event::<T>.in_set(LogEventsSet))
    }

    fn add_and_log_event<T>(&mut self) -> &mut Self
    where
        T: Event + std::fmt::Debug,
    {
        self.add_event::<T>().log_event::<T>()
    }
}

fn register_event<T: Event>(world: &mut World) {
    let name = type_name::<T>().to_string();
    world.resource_scope(|world, plugin_settings: Mut<LogEventsPluginSettings>| {
        if let Some(previous) = plugin_settings.previous_settings.get(&name) {
            let mut event_settings = world.resource_mut::<LoggedEventSettings<T>>();
            **event_settings = *previous;
        }
    });
    world.resource_scope(|world, mut log_settings_ids: Mut<LogSettingsIds>| {
        let id = world
            .components()
            .resource_id::<LoggedEventSettings<T>>()
            .unwrap();
        log_settings_ids.insert(name, id);
    });
}

fn log_event<T>(settings: Res<LoggedEventSettings<T>>, mut events: EventReader<T>)
where
    T: Event + std::fmt::Debug,
{
    if !settings.enabled {
        return;
    }
    for event in events.read() {
        let to_log = if settings.pretty {
            format!("{}: {:#?}", type_name::<T>(), event)
        } else {
            format!("{}: {:?}", type_name::<T>(), event)
        };
        match settings.level {
            Level::ERROR => error!("{}", to_log),
            Level::WARN => warn!("{}", to_log),
            Level::INFO => info!("{}", to_log),
            Level::DEBUG => debug!("{}", to_log),
            Level::TRACE => trace!("{}", to_log),
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
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut file = File::create(path).unwrap();
    let serialized = ron::ser::to_string_pretty(
        &to_serialize,
        PrettyConfig::default()
            .struct_names(true)
            .separate_tuple_members(true),
    )
    .unwrap();
    file.write_all(serialized.as_bytes()).unwrap();
}
