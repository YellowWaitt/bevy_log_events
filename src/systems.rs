use std::{
    any::type_name,
    collections::BTreeMap,
    error::Error,
    fmt::Write,
    fs::{File, create_dir_all},
    ops::DerefMut,
    path::{Path, PathBuf},
};

use bevy::{
    ecs::{change_detection::MaybeLocation, component::ComponentId},
    log::Level,
    prelude::*,
};

use ron::{de::from_reader, ser::PrettyConfig};

use crate::{
    EventSettings, LogEventsPlugin, LogEventsPluginSettings, LogMessagesSystems,
    LoggedEventSettings,
    utils::{LoggedEventsSettings, get_log_settings_by_id, trigger_name},
};

const CRATE: &str = "bevy_log_events";

impl Plugin for LogEventsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LogEventsPluginSettings::new(self))
            .insert_resource(LogSettingsIds::default())
            .configure_sets(Last, LogMessagesSystems.run_if(plugin_enabled))
            .add_systems(PostUpdate, save_settings.run_if(on_message::<AppExit>))
            .add_plugins(crate::settings_window::plugin);
    }
}

impl LogEventsPluginSettings {
    fn new(log_plugin: &LogEventsPlugin) -> Self {
        let path = &log_plugin.settings_path;
        match Self::load_saved_settings(path) {
            Ok(new) => new,
            Err(err) => {
                warn!(target: CRATE, "Error while trying to load settings from {:?}: {}. Using default settings instead.", path, err);
                LogEventsPluginSettings::default(path)
            }
        }
    }

    fn default(path: &Path) -> Self {
        Self {
            enabled: true,
            show_window: false,
            saved_settings: path.to_path_buf(),
            previous_settings: BTreeMap::new(),
        }
    }

    fn load_saved_settings(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let saved_settings: LoggedEventsSettings = from_reader(file)?;
        let new = Self {
            enabled: saved_settings.plugin_enabled,
            show_window: false,
            saved_settings: path.to_path_buf(),
            previous_settings: saved_settings.events_settings,
        };
        Ok(new)
    }
}

fn plugin_enabled(plugin_settings: Res<LogEventsPluginSettings>) -> bool {
    plugin_settings.enabled
}

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct LogSettingsIds(BTreeMap<String, ComponentId>);

impl LogSettingsIds {
    fn registered(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    fn register(&mut self, name: String, id: ComponentId) {
        self.insert(name, id);
    }

    pub(crate) fn iter_ids(&self) -> impl Iterator<Item = (&String, &ComponentId)> {
        self.iter()
    }
}

pub(crate) fn register_event<T>(world: &mut World, name: String) -> bool
where
    T: Resource + Default + DerefMut<Target = EventSettings>,
{
    world.resource_scope(|world, mut log_settings_ids: Mut<LogSettingsIds>| {
        if log_settings_ids.registered(&name) {
            false
        } else {
            world.insert_resource(T::default());
            world.resource_scope(|world, plugin_settings: Mut<LogEventsPluginSettings>| {
                if let Some(previous) = plugin_settings.previous_settings.get(&name) {
                    let mut event_settings = world.resource_mut::<T>();
                    **event_settings = *previous;
                }
            });
            let id = world.components().resource_id::<T>().unwrap();
            log_settings_ids.register(name, id);
            true
        }
    })
}

fn log(level: Level, to_log: &str) {
    match level {
        Level::ERROR => error!(target: CRATE, "{to_log}"),
        Level::WARN => warn!(target: CRATE, "{to_log}"),
        Level::INFO => info!(target: CRATE, "{to_log}"),
        Level::DEBUG => debug!(target: CRATE, "{to_log}"),
        Level::TRACE => trace!(target: CRATE, "{to_log}"),
    }
}

fn format_event<E>(
    settings: &EventSettings,
    event: &E,
    location: MaybeLocation,
) -> Result<String, Box<dyn Error>>
where
    E: std::fmt::Debug,
{
    let name = type_name::<E>();
    let mut to_log = String::new();
    to_log.write_str(name)?;
    if let Some(location) = location.into_option() {
        to_log.write_fmt(format_args!(" at {location}"))?;
    }
    to_log.write_str(": ")?;
    if settings.pretty {
        to_log.write_fmt(format_args!("{event:#?}"))?;
    } else {
        to_log.write_fmt(format_args!("{event:?}"))?;
    }
    Ok(to_log)
}

fn format_entity_and_component<C>(
    settings: &EventSettings,
    event_name: &str,
    entity_name: &Option<&Name>,
    entity: Entity,
    component: &C,
    location: MaybeLocation,
) -> Result<String, Box<dyn Error>>
where
    C: std::fmt::Debug,
{
    let mut to_log = String::new();
    to_log.write_fmt(format_args!("{event_name} on "))?;
    if let Some(name) = entity_name {
        to_log.write_fmt(format_args!("{name}({entity})"))?;
    } else {
        to_log.write_fmt(format_args!("{entity}"))?;
    }
    if let Some(location) = location.into_option() {
        to_log.write_fmt(format_args!(" at {location}"))?;
    }
    to_log.write_str(": ")?;
    if settings.pretty {
        to_log.write_fmt(format_args!("{component:#?}"))?;
    } else {
        to_log.write_fmt(format_args!("{component:?}"))?;
    }
    Ok(to_log)
}

pub(crate) fn log_message<M>(settings: Res<LoggedEventSettings<M>>, mut messages: MessageReader<M>)
where
    M: Message + std::fmt::Debug,
{
    if !settings.enabled {
        return;
    }
    for (message, id) in messages.read_with_id() {
        if let Ok(to_log) = format_event(&settings, message, id.caller) {
            log(settings.level, &to_log);
        }
    }
}

pub(crate) fn log_event<E>(
    event: On<E>,
    plugin_settings: Res<LogEventsPluginSettings>,
    settings: Res<LoggedEventSettings<E>>,
) where
    E: Event + std::fmt::Debug,
{
    if !plugin_settings.enabled || !settings.enabled {
        return;
    }
    if let Ok(to_log) = format_event(&settings, event.event(), event.caller()) {
        log(settings.level, &to_log);
    }
}

pub(crate) fn log_component<E, C>(
    event: On<E, C>,
    plugin_settings: Res<LogEventsPluginSettings>,
    settings: Res<LoggedEventSettings<E, C>>,
    query: Query<(&C, Option<&Name>)>,
) where
    E: EntityEvent,
    C: Component + std::fmt::Debug,
{
    if !plugin_settings.enabled || !settings.enabled {
        return;
    }
    let target = event.event_target();
    if let Ok((component, name)) = query.get(target)
        && let Ok(to_log) = format_entity_and_component::<C>(
            &settings,
            &trigger_name::<E, C>(),
            &name,
            target,
            component,
            event.caller(),
        )
    {
        log(settings.level, &to_log);
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

fn save_settings(world: &mut World) {
    let log_settings_ids = world.resource::<LogSettingsIds>();
    let mut all_settings = BTreeMap::new();
    for (name, id) in log_settings_ids.iter_ids() {
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
            target: CRATE,
            "Could not save {} at {:?} due to {:?}",
            type_name::<LoggedEventsSettings>(),
            path,
            e
        );
    }
}
