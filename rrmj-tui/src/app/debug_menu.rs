#![cfg(feature = "debug-menu")]

use crossterm::event::{KeyCode, KeyEvent};

use super::scenario_menu::{ImportScenarioTarget, SCENARIOS_MENU_INDEX, SETTINGS_MENU_INDEX};
use super::{App, DebugScenarioSetup, DebugSetupField, MainMenuMode, Screen, TableMode};
use crate::error::AppError;
use crate::input::BindAction;
use crate::scenarios::{self, ScenarioEntry};

pub const DEBUG_MENU_INDEX: usize = 4;

impl App {
    pub(super) fn handle_debug_mode_menu(
        &mut self,
        key: &KeyEvent,
        action: Option<BindAction>,
    ) -> Result<(), AppError> {
        if self.main_menu_mode == MainMenuMode::LoadGame {
            return self.handle_load_game_menu(key, action);
        }
        if self.main_menu_mode == MainMenuMode::Replays {
            return self.handle_replays_menu(key, action);
        }
        if self.main_menu_mode == MainMenuMode::Scenarios {
            return self.handle_scenarios_menu(key, action);
        }
        if self.main_menu_mode == MainMenuMode::Debug {
            return self.handle_debug_menu(key, action);
        }

        if key.code == KeyCode::Char('f') {
            return self.cycle_debug_filter();
        }

        if self.is_activate(key) {
            return match self.menu_index {
                0 => {
                    self.setup = Some(super::NewGameSetup::new(
                        self.config.default_difficulty,
                        self.config.human_seat,
                        self.config.cpu_step_delay_ms,
                        self.config.turn_timer_ms,
                        self.config.response_timer_ms,
                    ));
                    Ok(())
                }
                1 => self.open_load_game_menu(),
                super::REPLAYS_MENU_INDEX => self.open_replays_menu(),
                SCENARIOS_MENU_INDEX => self.open_scenarios_menu(),
                DEBUG_MENU_INDEX => self.open_debug_menu(),
                SETTINGS_MENU_INDEX => {
                    self.settings_field = super::SettingsField::Theme;
                    self.settings_open = true;
                    Ok(())
                }
                _ => {
                    self.quit = true;
                    Ok(())
                }
            };
        }

        let max = super::ROOT_MENU_LEN.saturating_sub(1);
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

    pub(super) fn handle_debug_setup_key_if_open(&mut self, key: KeyEvent) -> Result<(), AppError> {
        if self.debug_setup.is_some() && self.screen == Screen::MainMenu {
            return self.handle_debug_setup_key(key);
        }
        Ok(())
    }

    fn cycle_debug_filter(&mut self) -> Result<(), AppError> {
        if self.main_menu_mode != MainMenuMode::Debug {
            return Ok(());
        }
        let tags = scenarios::all_tags(&self.debug_entries);
        self.debug_filter_tag = match &self.debug_filter_tag {
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

    fn open_debug_menu(&mut self) -> Result<(), AppError> {
        let dir = scenarios::bundled_debug_scenarios_dir();
        self.debug_entries = scenarios::list_scenarios(&dir)?;
        self.debug_filter_tag = None;
        self.main_menu_mode = MainMenuMode::Debug;
        self.menu_index = 0;
        if self.debug_entries.is_empty() {
            self.status = format!("No scenarios in {}", dir.display());
        } else {
            self.status.clear();
        }
        Ok(())
    }

    fn handle_debug_menu(
        &mut self,
        key: &KeyEvent,
        action: Option<BindAction>,
    ) -> Result<(), AppError> {
        if key.code == KeyCode::Char('i') {
            self.open_import_scenario(ImportScenarioTarget::Debug);
            return Ok(());
        }
        if self.keybinds.is_bound(key, BindAction::Back) {
            self.main_menu_mode = MainMenuMode::Root;
            self.menu_index = DEBUG_MENU_INDEX;
            self.debug_filter_tag = None;
            return Ok(());
        }
        let filtered = self.filtered_debug_entries();
        if filtered.is_empty() {
            if self.is_activate(key) || self.keybinds.is_bound(key, BindAction::Back) {
                self.main_menu_mode = MainMenuMode::Root;
                self.menu_index = DEBUG_MENU_INDEX;
            }
            return Ok(());
        }
        if self.is_activate(key) {
            let index = self.menu_index;
            return self.open_debug_seat_picker(index);
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

    fn open_debug_seat_picker(&mut self, index: usize) -> Result<(), AppError> {
        let entry = scenarios::filter_by_tag(&self.debug_entries, self.debug_filter_tag.as_deref())
            .into_iter()
            .nth(index)
            .cloned()
            .ok_or_else(|| AppError::Config {
                path: self.config_path.clone(),
                detail: "no scenario selected".into(),
            })?;
        let recording = scenarios::read_scenario(&entry.path)?;
        recording.validate().map_err(AppError::Engine)?;
        self.debug_setup = Some(DebugScenarioSetup::new(
            entry,
            recording,
            self.config.human_seat,
        ));
        self.main_menu_mode = MainMenuMode::Root;
        Ok(())
    }

    fn handle_debug_setup_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let action = self.keybinds.action_for(&key);
        if self.keybinds.is_bound(&key, BindAction::Back) {
            self.debug_setup = None;
            self.main_menu_mode = MainMenuMode::Debug;
            return Ok(());
        }
        if self.is_activate(&key) {
            let confirm = self
                .debug_setup
                .as_ref()
                .is_some_and(|setup| setup.selected == DebugSetupField::Confirm);
            if confirm {
                return self.confirm_debug_scenario();
            }
            if let Some(setup) = self.debug_setup.as_mut()
                && setup.selected == DebugSetupField::HumanSeat
            {
                setup.cycle_seat();
            }
            return Ok(());
        }
        let Some(setup) = self.debug_setup.as_mut() else {
            return Ok(());
        };
        match action {
            Some(BindAction::MenuUp) => setup.select_prev(),
            Some(BindAction::MenuDown) => setup.select_next(),
            Some(BindAction::MenuToggle) | Some(BindAction::MenuCycle) => {
                if setup.selected == DebugSetupField::HumanSeat {
                    setup.cycle_seat();
                }
            }
            Some(BindAction::Quit) => self.quit = true,
            _ => {}
        }
        Ok(())
    }

    fn confirm_debug_scenario(&mut self) -> Result<(), AppError> {
        let setup = self.debug_setup.take().expect("debug setup present");
        let human = setup.selected_seat;
        let title = setup.entry.title.clone();
        let match_setup = setup.match_setup_for_launch();
        let seed = setup.recording.seed;
        let agents = match_setup.build_agents(seed);
        let game = setup.recording.restore()?;
        self.setup_meta = Some(match_setup);
        self.agents = Some(agents);
        self.match_game = Some(game);
        self.human_seat_active = human;
        self.cpu_step_delay_ms = self.config.cpu_step_delay_ms;
        self.turn_timer_ms = self.config.turn_timer_ms;
        self.response_timer_ms = self.config.response_timer_ms;
        self.cpu_step_wait_until = None;
        self.action_timer.reset();
        self.active_recording_id = None;
        self.active_recording_meta = None;
        self.active_save_path = None;
        self.table_mode = TableMode::Normal;
        self.tile_index = 0;
        self.hand_result = None;
        self.match_summary = None;
        self.screen = Screen::Table;
        self.status = format!("Debug: {title}");
        Ok(())
    }

    pub fn debug_setup(&self) -> Option<&DebugScenarioSetup> {
        self.debug_setup.as_ref()
    }

    pub fn debug_setup_open(&self) -> bool {
        self.debug_setup.is_some()
    }

    pub fn debug_filter_tag(&self) -> Option<&str> {
        self.debug_filter_tag.as_deref()
    }

    pub fn filtered_debug_entries(&self) -> Vec<&ScenarioEntry> {
        scenarios::filter_by_tag(&self.debug_entries, self.debug_filter_tag.as_deref())
    }
}
