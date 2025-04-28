//! Provide the window for editing the [LoggedEventSettings](crate::LoggedEventSettings).

use bevy::{log::Level, prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContext, EguiContextPass, EguiPlugin};

use regex::Regex;

use crate::{
    systems::LogSettingsIds, utils::get_log_settings_mut_by_id, EventSettings,
    LogEventsPluginSettings,
};

const WINDOW_NAME: &str = "Logged Events Settings";

pub(crate) fn plugin(app: &mut App) {
    assert!(app.is_plugin_added::<EguiPlugin>());
    app.insert_resource(LogEventsWindowState::default())
        .add_systems(EguiContextPass, show_settings_window);
}

const ALL_LEVELS: [Level; 5] = [
    Level::ERROR,
    Level::WARN,
    Level::INFO,
    Level::DEBUG,
    Level::TRACE,
];

fn level_color(level: Level) -> egui::Color32 {
    match level {
        Level::INFO => egui::Color32::from_rgb(45, 193, 40),
        Level::WARN => egui::Color32::from_rgb(249, 201, 24),
        Level::ERROR => egui::Color32::from_rgb(219, 23, 2),
        Level::DEBUG => egui::Color32::from_rgb(49, 140, 231),
        Level::TRACE => egui::Color32::from_rgb(189, 51, 164),
    }
}

fn colored_text_level(level: Level) -> egui::RichText {
    egui::RichText::new(level.as_str()).color(level_color(level))
}

#[derive(Default, PartialEq, Clone, Copy)]
enum EnabledFilter {
    #[default]
    All,
    Enabled,
    Disabled,
}

impl EnabledFilter {
    fn iter() -> impl Iterator<Item = Self> {
        [Self::All, Self::Enabled, Self::Disabled].into_iter()
    }

    fn contains(&self, enabled: bool) -> bool {
        match self {
            EnabledFilter::All => true,
            EnabledFilter::Enabled => enabled,
            EnabledFilter::Disabled => !enabled,
        }
    }
}

impl std::fmt::Display for EnabledFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            EnabledFilter::All => "All",
            EnabledFilter::Enabled => "Enabled",
            EnabledFilter::Disabled => "Disabled",
        };
        write!(f, "{}", str)
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
enum LevelFilter {
    #[default]
    All,
    Level(Level),
}

impl LevelFilter {
    fn contains(&self, level: Level) -> bool {
        match self {
            LevelFilter::All => true,
            LevelFilter::Level(lvl) => *lvl == level,
        }
    }

    fn to_label(self) -> egui::RichText {
        match self {
            LevelFilter::All => "All".into(),
            LevelFilter::Level(level) => colored_text_level(level),
        }
    }
}

#[derive(Default, Resource)]
struct LogEventsWindowState {
    name_filter: String,
    case_sensitive: bool,
    use_regex: bool,
    enabled_filter: EnabledFilter,
    level_filter: LevelFilter,
    regex: Option<Regex>,
    shown: usize,
}

impl LogEventsWindowState {
    fn name_contains_filter(&self, name: &str) -> bool {
        let (name, filter) = if self.case_sensitive {
            (name.to_string(), self.name_filter.clone())
        } else {
            (name.to_lowercase(), self.name_filter.to_lowercase())
        };
        if self.use_regex {
            self.regex.as_ref().is_some_and(|re| re.is_match(&name))
        } else {
            name.contains(&filter)
        }
    }

    fn update_regex(&mut self) {
        if self.use_regex {
            let re = if self.case_sensitive {
                self.name_filter.clone()
            } else {
                self.name_filter.to_lowercase()
            };
            self.regex = Regex::new(&re).ok();
        } else {
            self.regex = None;
        }
    }

    fn must_show(&self, log_settings: &EventSettings) -> bool {
        self.enabled_filter.contains(log_settings.enabled)
            && self.level_filter.contains(log_settings.level)
    }
}

macro_rules! selectable_label_switch {
    ($switch:expr, $ui:expr, $label:expr, $hover:expr) => {{
        let current = $switch;
        if $ui
            .selectable_label(current, $label)
            .on_hover_text($hover)
            .clicked()
        {
            $switch = !current;
        }
    }};
}

/// If you want to integrate the ui for editing the [LoggedEventSettings](crate::LoggedEventSettings)
/// in a custom way you can use this function. In such case you can ignore the `show_window` field
/// of [LogEventsPluginSettings].
pub fn log_events_window_ui(world: &mut World, ui: &mut egui::Ui) {
    world.resource_scope(|world, mut state: Mut<LogEventsWindowState>| {
        let mut plugin_settings = world.resource_mut::<LogEventsPluginSettings>();
        ui.strong("Plugin settings");
        ui.checkbox(&mut plugin_settings.enabled, "Enabled");

        ui.separator();

        ui.strong("🔍 Search");
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut state.name_filter);
            selectable_label_switch!(state.case_sensitive, ui, "Aa", "Match Case");
            selectable_label_switch!(state.use_regex, ui, ".*", "Use Regular Expression");
            state.update_regex();
        });
        ui.horizontal(|ui| {
            ui.label("Enabled");
            egui::ComboBox::from_id_salt("enabled_filter")
                .selected_text(state.enabled_filter.to_string())
                .show_ui(ui, |ui| {
                    for filter in EnabledFilter::iter() {
                        ui.selectable_value(&mut state.enabled_filter, filter, filter.to_string());
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label("Level");
            egui::ComboBox::from_id_salt("level_filter")
                .selected_text(state.level_filter.to_label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut state.level_filter,
                        LevelFilter::All,
                        LevelFilter::All.to_label(),
                    );
                    for level in ALL_LEVELS {
                        let level = LevelFilter::Level(level);
                        ui.selectable_value(&mut state.level_filter, level, level.to_label());
                    }
                });
        });
        world.resource_scope(|world, log_settings_ids: Mut<LogSettingsIds>| {
            ui.label(format!(
                "Displayed : {}/{}",
                state.shown,
                log_settings_ids.len()
            ));

            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink(true)
                .show(ui, |ui| {
                    state.shown = 0;
                    for (name, id) in log_settings_ids.iter_ids() {
                        if !state.name_contains_filter(name) {
                            continue;
                        }
                        let event_settings = get_log_settings_mut_by_id(world, id);
                        if !state.must_show(event_settings) {
                            continue;
                        }
                        if state.shown != 0 {
                            ui.separator();
                        }
                        state.shown += 1;
                        ui.strong(name);
                        ui.checkbox(&mut event_settings.enabled, "Enabled");
                        ui.checkbox(&mut event_settings.pretty, "Pretty Debug");
                        egui::ComboBox::from_id_salt(id.index())
                            .selected_text(colored_text_level(event_settings.level))
                            .show_ui(ui, |ui| {
                                for level in ALL_LEVELS {
                                    ui.selectable_value(
                                        &mut event_settings.level,
                                        level,
                                        colored_text_level(level),
                                    );
                                }
                            });
                    }
                });
        });
    });
}

fn show_settings_window(world: &mut World) {
    let mut open = world.resource::<LogEventsPluginSettings>().show_window;
    if let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
    {
        let mut egui_context = egui_context.clone();
        egui::Window::new(WINDOW_NAME)
            .open(&mut open)
            .show(egui_context.get_mut(), |ui| {
                log_events_window_ui(world, ui);
            });
        world.resource_mut::<LogEventsPluginSettings>().show_window = open;
    }
}
