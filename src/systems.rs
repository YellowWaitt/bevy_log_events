use std::{
    any::type_name,
    collections::BTreeMap,
    error::Error,
    fmt::Write,
    fs::{create_dir_all, File},
    ops::DerefMut,
    path::{Path, PathBuf},
};

use bevy::{ecs::component::ComponentId, log::Level, prelude::*};

use bitflags::bitflags;
use ron::{de::from_reader, ser::PrettyConfig};

use crate::{
    utils::{get_log_settings_by_id, trigger_name, LoggedEventsSettings},
    EventSettings, LogEventsPlugin, LogEventsPluginSettings, LogEventsSet, LoggedEventSettings,
};

impl Plugin for LogEventsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LogEventsPluginSettings::new(self))
            .insert_resource(LogSettingsIds::default())
            .configure_sets(Last, LogEventsSet.run_if(plugin_enabled))
            .add_systems(PostUpdate, save_settings.run_if(on_event::<AppExit>))
            .add_plugins(crate::settings_window::plugin);
    }
}

impl LogEventsPluginSettings {
    fn new(log_plugin: &LogEventsPlugin) -> Self {
        let path = &log_plugin.settings_path;
        match Self::load_saved_settings(path) {
            Ok(new) => new,
            Err(err) => {
                warn!(target: "bevy_log_events", "Error while trying to load settings from {:?}: {}. Using default settings instead.", path, err);
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

bitflags! {
    #[derive(Clone, Copy)]
    pub(crate) struct EventKind: u8 {
        const EVENT = 1;
        const TRIGGER = 1 << 1;
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct LogSettingsIds(BTreeMap<String, (ComponentId, EventKind)>);

impl LogSettingsIds {
    fn registered(&self, name: &str, kind: EventKind) -> bool {
        self.get(name).is_some_and(|entry| entry.1.contains(kind))
    }

    fn register(&mut self, name: String, id: ComponentId, kind: EventKind) {
        if let Some(entry) = self.get_mut(&name) {
            entry.1.insert(kind);
        } else {
            self.insert(name, (id, kind));
        }
    }
    pub(crate) fn iter_ids(&self) -> impl Iterator<Item = (&String, &ComponentId)> {
        self.iter().map(|(name, (id, _))| (name, id))
    }
}

pub(crate) fn register_event<T>(world: &mut World, name: String, kind: EventKind) -> bool
where
    T: Resource + Default + DerefMut<Target = EventSettings>,
{
    world.resource_scope(|world, mut log_settings_ids: Mut<LogSettingsIds>| {
        if log_settings_ids.registered(&name, kind) {
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
            log_settings_ids.register(name, id, kind);
            true
        }
    })
}

fn log(level: Level, to_log: &str) {
    match level {
        Level::ERROR => error!(target: "bevy_log_events", "{}", to_log),
        Level::WARN => warn!(target: "bevy_log_events", "{}", to_log),
        Level::INFO => info!(target: "bevy_log_events", "{}", to_log),
        Level::DEBUG => debug!(target: "bevy_log_events", "{}", to_log),
        Level::TRACE => trace!(target: "bevy_log_events", "{}", to_log),
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

pub(crate) fn log_event<E>(settings: Res<LoggedEventSettings<E>>, mut events: EventReader<E>)
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

pub(crate) fn log_triggered<E>(
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

pub(crate) fn log_component<E, C>(
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
            target: "bevy_log_events",
            "Could not save {} at {:?} due to {:?}",
            type_name::<LoggedEventsSettings>(),
            path,
            e
        );
    }
}
