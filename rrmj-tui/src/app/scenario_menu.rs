use crossterm::event::{KeyCode, KeyEvent};

use super::path_input::{PathInputAction, PathInputDialog};
use super::{App, LoadGameSetup, LoadSetupField, MainMenuMode, ResumeSetupKind, Screen, TableMode};
use crate::error::AppError;
use crate::input::BindAction;
use crate::scenarios::{self, ScenarioEntry};

pub const SCENARIOS_MENU_INDEX: usize = 3;

pub const SETTINGS_MENU_INDEX: usize = if cfg!(feature = "debug-menu") { 5 } else { 4 };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportScenarioTarget {
    Player,
    #[cfg(feature = "debug-menu")]
    Debug,
}

impl App {
    pub(super) fn handle_scenario_setup_key_if_open(
        &mut self,
        key: KeyEvent,
    ) -> Result<(), AppError> {
        if self.scenario_setup.is_some() && self.screen == Screen::MainMenu {
            return self.handle_scenario_setup_key(key);
        }
        Ok(())
    }

    pub(super) fn open_scenarios_menu(&mut self) -> Result<(), AppError> {
        let dir = self.config.resolved_scenarios_dir();
        self.scenario_entries = scenarios::list_scenarios(&dir)?;
        self.scenario_filter_tag = None;
        self.main_menu_mode = MainMenuMode::Scenarios;
        self.menu_index = 0;
        if self.scenario_entries.is_empty() {
            self.status = format!("No scenarios in {} (press i to import)", dir.display());
        } else {
            self.status.clear();
        }
        Ok(())
    }

    fn cycle_scenario_filter(&mut self) -> Result<(), AppError> {
        if self.main_menu_mode != MainMenuMode::Scenarios {
            return Ok(());
        }
        let tags = scenarios::all_tags(&self.scenario_entries);
        self.scenario_filter_tag = match &self.scenario_filter_tag {
            None => tags.first().cloned(),
            Some(current) => {
                let pos = tags.iter().position(|t| t == current);
                match pos {
                    Some(i) if i + 1 < tags.len() => Some(tags[i + 1].clone()),
                    _ => None,
                }
            }
        };
        self.menu_index = 0;
        Ok(())
    }

    fn default_import_scenario_path(&self, target: ImportScenarioTarget) -> std::path::PathBuf {
        let dir = match target {
            ImportScenarioTarget::Player => self.config.resolved_scenarios_dir(),
            #[cfg(feature = "debug-menu")]
            ImportScenarioTarget::Debug => scenarios::bundled_debug_scenarios_dir(),
        };
        dir.join("import.rrmj.json")
    }

    pub(super) fn open_import_scenario(&mut self, target: ImportScenarioTarget) {
        self.import_scenario_target = target;
        let path = self.default_import_scenario_path(target);
        self.import_scenario = Some(PathInputDialog::new(path));
    }

    fn close_import_scenario(&mut self) {
        self.import_scenario = None;
    }

    pub(super) fn handle_import_scenario_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let is_activate = self.is_activate(&key);
        let is_back = self.keybinds.is_bound(&key, BindAction::Back);
        let dialog = self.import_scenario.as_mut().expect("import dialog open");
        match dialog.handle_key(key, is_activate, is_back) {
            PathInputAction::Continue => {}
            PathInputAction::Cancel => self.close_import_scenario(),
            PathInputAction::Confirm(path) => {
                if path.is_empty() {
                    self.status = "Enter a scenario file path".into();
                    return Ok(());
                }
                match scenarios::load_scenario_from_path(&path) {
                    Ok((entry, recording)) => {
                        let setup = LoadGameSetup::from_scenario(
                            entry,
                            recording,
                            self.config.human_seat,
                            self.config.cpu_step_delay_ms,
                            self.config.turn_timer_ms,
                            self.config.response_timer_ms,
                        );
                        match self.import_scenario_target {
                            ImportScenarioTarget::Player => {
                                self.scenario_setup = Some(setup);
                                self.main_menu_mode = MainMenuMode::Root;
                            }
                            #[cfg(feature = "debug-menu")]
                            ImportScenarioTarget::Debug => {
                                let entry = ScenarioEntry {
                                    path: setup.entry.path,
                                    id: setup.entry.recording_id,
                                    title: setup.entry.label,
                                    description: setup.entry.detail,
                                    tags: setup.recording.meta.tags.clone(),
                                };
                                self.debug_setup = Some(super::DebugScenarioSetup::new(
                                    entry,
                                    setup.recording,
                                    self.config.human_seat,
                                ));
                                self.main_menu_mode = MainMenuMode::Root;
                            }
                        }
                        self.close_import_scenario();
                        self.status.clear();
                    }
                    Err(err) => self.status = format!("Import failed: {err}"),
                }
            }
        }
        Ok(())
    }

    pub(super) fn handle_scenarios_menu(
        &mut self,
        key: &KeyEvent,
        action: Option<BindAction>,
    ) -> Result<(), AppError> {
        if key.code == KeyCode::Char('i') {
            self.open_import_scenario(ImportScenarioTarget::Player);
            return Ok(());
        }
        if key.code == KeyCode::Char('f') {
            return self.cycle_scenario_filter();
        }
        if self.keybinds.is_bound(key, BindAction::Back) {
            self.main_menu_mode = MainMenuMode::Root;
            self.menu_index = SCENARIOS_MENU_INDEX;
            self.scenario_filter_tag = None;
            return Ok(());
        }
        let filtered = self.filtered_scenario_entries();
        if filtered.is_empty() {
            if self.is_activate(key) || self.keybinds.is_bound(key, BindAction::Back) {
                self.main_menu_mode = MainMenuMode::Root;
                self.menu_index = SCENARIOS_MENU_INDEX;
            }
            return Ok(());
        }
        if self.is_activate(key) {
            let index = self.menu_index;
            return self.open_scenario_seat_picker(index);
        }
        let max = filtered.len().saturating_sub(1);
        match action {
            Some(BindAction::MenuUp) => {
                self.menu_index = self.menu_index.saturating_sub(1);
            }
            Some(BindAction::MenuDown) => {
                self.menu_index = (self.menu_index + 1).min(max);
            }
            Some(BindAction::Quit) => self.quit = true,
            _ => {}
        }
        Ok(())
    }

    fn open_scenario_seat_picker(&mut self, index: usize) -> Result<(), AppError> {
        let entry =
            scenarios::filter_by_tag(&self.scenario_entries, self.scenario_filter_tag.as_deref())
                .into_iter()
                .nth(index)
                .cloned()
                .ok_or_else(|| AppError::Config {
                    path: self.config_path.clone(),
                    detail: "no scenario selected".into(),
                })?;
        let recording = scenarios::read_scenario(&entry.path)?;
        recording.validate().map_err(AppError::Engine)?;
        self.scenario_setup = Some(LoadGameSetup::from_scenario(
            entry,
            recording,
            self.config.human_seat,
            self.config.cpu_step_delay_ms,
            self.config.turn_timer_ms,
            self.config.response_timer_ms,
        ));
        self.main_menu_mode = MainMenuMode::Root;
        Ok(())
    }

    fn handle_scenario_setup_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let action = self.keybinds.action_for(&key);
        if self.keybinds.is_bound(&key, BindAction::Back) {
            self.scenario_setup = None;
            self.main_menu_mode = MainMenuMode::Scenarios;
            return Ok(());
        }
        if self.is_activate(&key) {
            let confirm = self
                .scenario_setup
                .as_ref()
                .is_some_and(|setup| setup.selected == LoadSetupField::Confirm);
            if confirm {
                return self.confirm_scenario_launch();
            }
            if let Some(setup) = self.scenario_setup.as_mut() {
                match setup.selected {
                    LoadSetupField::HumanSeat => setup.cycle_seat(),
                    LoadSetupField::CpuStepDelay => setup.cycle_cpu_delay(),
                    LoadSetupField::TurnTimer => setup.cycle_turn_timer(),
                    LoadSetupField::ResponseTimer => setup.cycle_response_timer(),
                    LoadSetupField::Confirm => {}
                }
            }
            return Ok(());
        }
        let Some(setup) = self.scenario_setup.as_mut() else {
            return Ok(());
        };
        match action {
            Some(BindAction::MenuUp) => setup.select_prev(),
            Some(BindAction::MenuDown) => setup.select_next(),
            Some(BindAction::MenuToggle) | Some(BindAction::MenuCycle) => match setup.selected {
                LoadSetupField::HumanSeat => setup.cycle_seat(),
                LoadSetupField::CpuStepDelay => setup.cycle_cpu_delay(),
                LoadSetupField::TurnTimer => setup.cycle_turn_timer(),
                LoadSetupField::ResponseTimer => setup.cycle_response_timer(),
                LoadSetupField::Confirm => {}
            },
            Some(BindAction::Quit) => self.quit = true,
            _ => {}
        }
        Ok(())
    }

    fn confirm_scenario_launch(&mut self) -> Result<(), AppError> {
        let setup = self.scenario_setup.take().expect("scenario setup present");
        let human = setup.selected_seat;
        let title = setup.entry.label.clone();
        let match_setup = setup.match_setup_for_load();
        let seed = setup.recording.seed;
        let agents = match_setup.build_agents(seed);
        let game = setup.recording.restore()?;
        self.setup_meta = Some(match_setup);
        self.agents = Some(agents);
        self.match_game = Some(game);
        self.human_seat_active = human;
        self.cpu_step_delay_ms = setup.cpu_step_delay_ms;
        self.turn_timer_ms = setup.turn_timer_ms;
        self.response_timer_ms = setup.response_timer_ms;
        self.cpu_step_wait_until = None;
        self.action_timer.reset();
        self.active_recording_id = setup
            .recording
            .meta
            .recording_id
            .clone()
            .or_else(|| Some(format!("scenario-{}", seed)));
        self.active_recording_meta = Some(setup.recording.meta);
        self.active_save_path = None;
        self.table_mode = TableMode::Normal;
        self.tile_index = 0;
        self.hand_result = None;
        self.match_summary = None;
        self.screen = Screen::Table;
        self.status = format!("Scenario: {title}");
        Ok(())
    }

    pub fn scenario_filter_tag(&self) -> Option<&str> {
        self.scenario_filter_tag.as_deref()
    }

    pub fn filtered_scenario_entries(&self) -> Vec<&ScenarioEntry> {
        scenarios::filter_by_tag(&self.scenario_entries, self.scenario_filter_tag.as_deref())
    }

    pub fn resume_setup(&self) -> Option<(&LoadGameSetup, ResumeSetupKind)> {
        if let Some(load) = self.load_setup.as_ref() {
            return Some((load, ResumeSetupKind::SavedGame));
        }
        self.scenario_setup
            .as_ref()
            .map(|load| (load, ResumeSetupKind::Scenario))
    }

    pub fn import_scenario_open(&self) -> bool {
        self.import_scenario.is_some()
    }

    pub fn import_scenario_path(&self) -> Option<&str> {
        self.import_scenario.as_ref().map(PathInputDialog::path)
    }
}
