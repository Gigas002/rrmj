mod actions;
#[cfg(feature = "debug-menu")]
mod debug_menu;
#[cfg(feature = "debug-menu")]
mod debug_setup;
mod event_text;
mod hand_result;
mod load_setup;
mod path_input;
mod pause;
mod replay_review;
mod scenario_menu;
mod settings;
mod setup;
mod timers;

pub use actions::ActionMenu;
#[cfg(feature = "debug-menu")]
pub use debug_setup::{DebugScenarioSetup, DebugSetupField};
pub use hand_result::HandResultSummary;
pub use load_setup::{LoadGameSetup, LoadSetupField, ResumeSetupKind};
pub use path_input::PathInputDialog;
pub use pause::PauseItem;
pub use replay_review::ReplayReview;
pub use settings::SettingsField;
pub use setup::{NewGameSetup, SetupField, difficulty_label};

use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use librrmj::action::{Action, KanIntent};
use librrmj::agent::{PlayerSlot, PlayerView};
use librrmj::ai::{GameSetup, SeatAgent};
use librrmj::event::Event as GameEvent;
use librrmj::game::Game;
use librrmj::rules::Recommendation;
use librrmj::state::HandPhase;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::config::{AppConfig, cycle_theme};
use crate::error::AppError;
use crate::input::{BindAction, Keybinds, normalize_key_event};
use crate::save::{
    RecordingEntry, SavePaths, list_finished, list_in_progress, read_recording,
    unix_timestamp_string, write_recording_async,
};

use self::path_input::PathInputAction;
use crate::theme::Theme;
use crate::timers::{TimerKind, format_decision_timer};
use crate::ui;
use librrmj::game::GamePhase;
use librrmj::replay::{GameRecording, GameStatus, RecordingMeta, RecordingPlayer};

use self::timers::{ActionTimerState, timeout_action};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    Table,
    ReplayReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuMode {
    Root,
    LoadGame,
    Replays,
    Scenarios,
    #[cfg(feature = "debug-menu")]
    Debug,
}

pub(crate) const REPLAYS_MENU_INDEX: usize = 2;
pub(crate) const ROOT_MENU_LEN: usize = if cfg!(feature = "debug-menu") { 7 } else { 6 };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableMode {
    Normal,
    PickDiscard,
    PickRiichi,
    PickClosedKan,
    PickKakan,
    PickChi,
}

/// Top-level application state.
pub struct App {
    screen: Screen,
    keybinds: Keybinds,
    keybinds_path: Option<PathBuf>,
    config: AppConfig,
    config_path: PathBuf,
    settings_field: SettingsField,
    menu_index: usize,
    main_menu_mode: MainMenuMode,
    load_entries: Vec<RecordingEntry>,
    replay_entries: Vec<RecordingEntry>,
    replay_review: Option<ReplayReview>,
    scenario_entries: Vec<crate::scenarios::ScenarioEntry>,
    scenario_filter_tag: Option<String>,
    scenario_setup: Option<LoadGameSetup>,
    import_scenario: Option<PathInputDialog>,
    import_scenario_target: scenario_menu::ImportScenarioTarget,
    #[cfg(feature = "debug-menu")]
    debug_entries: Vec<crate::scenarios::ScenarioEntry>,
    #[cfg(feature = "debug-menu")]
    debug_filter_tag: Option<String>,
    active_recording_id: Option<String>,
    active_recording_meta: Option<RecordingMeta>,
    active_save_path: Option<PathBuf>,
    setup: Option<NewGameSetup>,
    load_setup: Option<LoadGameSetup>,
    #[cfg(feature = "debug-menu")]
    debug_setup: Option<DebugScenarioSetup>,
    active_game: Option<Game>,
    agents: Option<[SeatAgent; 4]>,
    setup_meta: Option<GameSetup>,
    human_seat_active: usize,
    cpu_step_delay_ms: u64,
    turn_timer_ms: u64,
    response_timer_ms: u64,
    cpu_step_wait_until: Option<Instant>,
    action_timer: ActionTimerState,
    table_mode: TableMode,
    tile_index: usize,
    chi_index: usize,
    hand_result: Option<HandResultSummary>,
    game_summary: Option<[i32; 4]>,
    help_open: bool,
    pause_open: bool,
    pause_index: PauseItem,
    export_save: Option<PathInputDialog>,
    scores_open: bool,
    recommendations_open: bool,
    recommendations_scroll: usize,
    recommendations_cache: Vec<Recommendation>,
    settings_open: bool,
    rules_open: bool,
    rules_scroll: usize,
    status: String,
    quit: bool,
}

impl App {
    pub fn new(
        keybinds: Keybinds,
        keybinds_path: Option<PathBuf>,
        config: AppConfig,
        config_path: PathBuf,
    ) -> Self {
        let cpu_step_delay_ms = config.cpu_step_delay_ms;
        let turn_timer_ms = config.turn_timer_ms;
        let response_timer_ms = config.response_timer_ms;
        Self {
            screen: Screen::MainMenu,
            keybinds,
            keybinds_path,
            config,
            config_path,
            settings_field: SettingsField::Theme,
            menu_index: 0,
            main_menu_mode: MainMenuMode::Root,
            load_entries: Vec::new(),
            replay_entries: Vec::new(),
            replay_review: None,
            scenario_entries: Vec::new(),
            scenario_filter_tag: None,
            scenario_setup: None,
            import_scenario: None,
            import_scenario_target: scenario_menu::ImportScenarioTarget::Player,
            #[cfg(feature = "debug-menu")]
            debug_entries: Vec::new(),
            #[cfg(feature = "debug-menu")]
            debug_filter_tag: None,
            active_recording_id: None,
            active_recording_meta: None,
            active_save_path: None,
            setup: None,
            load_setup: None,
            #[cfg(feature = "debug-menu")]
            debug_setup: None,
            active_game: None,
            agents: None,
            setup_meta: None,
            human_seat_active: 0,
            cpu_step_delay_ms,
            turn_timer_ms,
            response_timer_ms,
            cpu_step_wait_until: None,
            action_timer: ActionTimerState::default(),
            table_mode: TableMode::Normal,
            tile_index: 0,
            chi_index: 0,
            hand_result: None,
            game_summary: None,
            help_open: false,
            pause_open: false,
            pause_index: PauseItem::Resume,
            export_save: None,
            scores_open: false,
            recommendations_open: false,
            recommendations_scroll: 0,
            recommendations_cache: Vec::new(),
            settings_open: false,
            rules_open: false,
            rules_scroll: 0,
            status: String::new(),
            quit: false,
        }
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_loop(&mut terminal);

        disable_raw_mode()?;
        terminal.backend_mut().execute(LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        result
    }

    fn run_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), AppError> {
        while !self.quit {
            self.tick_cpu()?;
            self.tick_replay()?;
            self.tick_action_timers()?;
            let theme = self.theme();
            terminal.draw(|frame| ui::draw(frame, self, &theme))?;

            if event::poll(Duration::from_millis(50))? {
                while let Ok(Event::Key(key)) = event::read() {
                    if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                        self.handle_key(key)?;
                    }
                    if !event::poll(Duration::ZERO)? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn tick_cpu(&mut self) -> Result<(), AppError> {
        if self.screen != Screen::Table
            || self.hand_result.is_some()
            || self.pause_open
            || self.export_save.is_some()
            || self.scores_open
            || self.recommendations_open
        {
            return Ok(());
        }
        if let Some(until) = self.cpu_step_wait_until {
            if Instant::now() < until {
                return Ok(());
            }
            self.cpu_step_wait_until = None;
        }
        while let Some(seat) = self
            .active_game
            .as_ref()
            .and_then(|game| game.pending_seat())
        {
            let is_cpu = self
                .setup_meta
                .as_ref()
                .is_some_and(|setup| setup.slots[seat] == PlayerSlot::Cpu);
            if !is_cpu {
                break;
            }

            let events = {
                let game = self.active_game.as_mut().expect("match present");
                let agents = self.agents.as_mut().expect("agents present");
                match game.step(agents)? {
                    Some(step) => step.events,
                    None => break,
                }
            };
            let ended = self.active_game.as_ref().is_some_and(|game| game.is_ended());
            self.on_game_events(&events);
            if self.hand_result.is_some() || ended {
                break;
            }
            if self.cpu_step_delay_ms > 0 {
                self.cpu_step_wait_until =
                    Some(Instant::now() + Duration::from_millis(self.cpu_step_delay_ms));
                break;
            }
        }
        Ok(())
    }

    fn tick_action_timers(&mut self) -> Result<(), AppError> {
        if self.screen != Screen::Table
            || self.pause_open
            || self.export_save.is_some()
            || self.scores_open
            || self.recommendations_open
            || self.help_open
            || self.rules_open
            || self.hand_result.is_some()
            || self.game_summary.is_some()
        {
            self.action_timer.reset();
            return Ok(());
        }

        let Some(game) = self.active_game.as_ref() else {
            self.action_timer.reset();
            return Ok(());
        };
        let Some(seat) = game.pending_seat() else {
            self.action_timer.reset();
            return Ok(());
        };
        let phase = game.hand().phase();
        let menu = ActionMenu::from_legal(&game.hand().legal_actions_for(seat));

        // Nothing to decide — pass immediately, no response timer.
        if phase == HandPhase::Reaction && menu.is_pass_only() {
            return self.apply_action_for_seat(seat, Action::Pass);
        }

        // Draw is automatic at turn start (no timer, no hotkey).
        if phase == HandPhase::Draw {
            return self.apply_action_for_seat(seat, Action::Draw);
        }

        self.action_timer
            .sync(seat, phase, self.turn_timer_ms, self.response_timer_ms);

        if !self.action_timer.is_expired() {
            return Ok(());
        }

        self.force_pending_action()
    }

    fn force_pending_action(&mut self) -> Result<(), AppError> {
        let game = self.active_game.as_ref().expect("match present");
        let seat = game.pending_seat().expect("pending seat");
        let phase = game.hand().phase();
        let legal = game.hand().legal_actions_for(seat);
        let menu = ActionMenu::from_legal(&legal);
        let Some(action) = timeout_action(&legal, phase, &menu) else {
            return Ok(());
        };

        if self.is_human_turn() {
            self.apply_action_for_seat(seat, action)?;
        } else {
            self.cpu_step_wait_until = None;
            self.apply_action_for_seat(seat, action)?;
        }
        Ok(())
    }

    fn apply_action_for_seat(&mut self, seat: usize, action: Action) -> Result<(), AppError> {
        let events = self
            .active_game
            .as_mut()
            .expect("match present")
            .apply_action(seat, action)?;
        self.on_game_events(&events);
        self.table_mode = TableMode::Normal;
        self.action_timer.reset();
        Ok(())
    }

    fn apply_human_action(&mut self, action: Action) -> Result<(), AppError> {
        let seat = self
            .active_game
            .as_ref()
            .and_then(|g| g.pending_seat())
            .expect("human action without pending seat");
        self.apply_action_for_seat(seat, action)
    }

    fn open_pause_menu(&mut self) {
        self.scores_open = false;
        self.recommendations_open = false;
        self.pause_open = true;
        self.pause_index = PauseItem::Resume;
        self.action_timer.reset();
    }

    fn close_pause_menu(&mut self) {
        self.pause_open = false;
        self.pause_index = PauseItem::Resume;
        self.export_save = None;
    }

    fn return_to_main_menu(&mut self) {
        self.active_game = None;
        self.agents = None;
        self.setup_meta = None;
        self.hand_result = None;
        self.game_summary = None;
        self.active_recording_id = None;
        self.active_recording_meta = None;
        self.active_save_path = None;
        self.screen = Screen::MainMenu;
        self.main_menu_mode = MainMenuMode::Root;
        self.menu_index = 0;
        self.table_mode = TableMode::Normal;
        self.cpu_step_wait_until = None;
        self.action_timer.reset();
        self.scores_open = false;
        self.recommendations_open = false;
        self.recommendations_cache.clear();
        self.close_pause_menu();
        self.status = "Returned to main menu".into();
    }

    fn try_exit_table_to_main_menu(&mut self, action: Option<BindAction>) -> bool {
        if matches!(action, Some(BindAction::MainMenu)) {
            self.return_to_main_menu();
            return true;
        }
        false
    }

    fn default_export_path(&self) -> PathBuf {
        if let Some(path) = &self.active_save_path {
            return path.clone();
        }
        let seed = self
            .active_game
            .as_ref()
            .map(|game| game.seed())
            .unwrap_or(0);
        self.save_paths().recording_path(&format!("match-{seed}"))
    }

    fn open_export_save(&mut self) {
        self.export_save = Some(PathInputDialog::new(self.default_export_path()));
    }

    fn close_export_save(&mut self) {
        self.export_save = None;
    }

    fn persist_recording_meta(&self) -> RecordingMeta {
        let id = self.active_recording_id.clone().unwrap_or_else(|| {
            format!(
                "match-{}",
                self.active_game.as_ref().map(|g| g.seed()).unwrap_or(0)
            )
        });
        let mut meta = self.active_recording_meta.clone().unwrap_or_default();
        meta.recording_id = Some(id);
        meta.updated_at = Some(unix_timestamp_string());
        meta.client_version = Some(format!("rrmj-tui {}", librrmj::VERSION));
        if meta.created_at.is_none() {
            meta.created_at = Some(unix_timestamp_string());
        }
        meta
    }

    fn capture_current_recording(&self) -> Option<GameRecording> {
        let game = self.active_game.as_ref()?;
        let setup = self.setup_meta.as_ref()?;
        let meta = self.persist_recording_meta();
        Some(GameRecording::capture(
            game,
            setup,
            self.human_seat_active,
            self.cpu_step_delay_ms,
            self.turn_timer_ms,
            self.response_timer_ms,
            meta,
        ))
    }

    fn handle_export_save_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let Some(recording) = self.capture_current_recording() else {
            self.close_export_save();
            self.status = "No match to save".into();
            return Ok(());
        };
        let is_activate = self.is_activate(&key);
        let is_back = self.keybinds.is_bound(&key, BindAction::Back);
        let dialog = self.export_save.as_mut().expect("export dialog open");
        match dialog.handle_key(key, is_activate, is_back) {
            PathInputAction::Continue => {}
            PathInputAction::Cancel => self.close_export_save(),
            PathInputAction::Confirm(path) => {
                if path.is_empty() {
                    self.status = "Enter a file path to save".into();
                    return Ok(());
                }
                let path =
                    crate::save::ensure_recording_extension(crate::save::resolve_user_path(&path));
                match crate::save::write_recording(&path, &recording) {
                    Ok(()) => {
                        self.active_save_path = Some(path.clone());
                        self.active_recording_id =
                            recording.meta.recording_id.clone().or_else(|| {
                                path.file_stem()
                                    .map(|stem| stem.to_string_lossy().into_owned())
                            });
                        self.active_recording_meta = Some(recording.meta);
                        self.status = format!("Saved to {}", path.display());
                        self.close_export_save();
                    }
                    Err(err) => self.status = format!("Save failed: {err}"),
                }
            }
        }
        Ok(())
    }

    fn handle_pause_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        if self.export_save.is_some() {
            return self.handle_export_save_key(key);
        }
        let action = self.keybinds.action_for(&key);
        if self.try_exit_table_to_main_menu(action) {
            return Ok(());
        }
        if self.keybinds.is_bound(&key, BindAction::Back) {
            self.close_pause_menu();
            return Ok(());
        }
        if self.is_activate(&key) {
            return match self.pause_index {
                PauseItem::Resume => {
                    self.close_pause_menu();
                    Ok(())
                }
                PauseItem::ExportSave => {
                    self.open_export_save();
                    Ok(())
                }
                PauseItem::MainMenu => {
                    self.return_to_main_menu();
                    Ok(())
                }
                PauseItem::Quit => {
                    self.quit = true;
                    Ok(())
                }
            };
        }
        match action {
            Some(BindAction::MenuUp) => {
                self.pause_index = self.pause_index.prev();
            }
            Some(BindAction::MenuDown) => {
                self.pause_index = self.pause_index.next();
            }
            Some(BindAction::Quit) => self.quit = true,
            _ => {}
        }
        Ok(())
    }

    fn save_paths(&self) -> SavePaths {
        SavePaths {
            recordings_dir: self.config.resolved_recordings_dir(),
        }
    }

    fn finalize_finished_match(&mut self) {
        let (Some(game), Some(setup)) = (&self.active_game, &self.setup_meta) else {
            return;
        };
        if !game.is_ended() {
            return;
        }
        let meta = self.persist_recording_meta();
        // Same file path; `capture` sets `game_status = finished` when the game ended.
        let recording = GameRecording::capture(
            game,
            setup,
            self.human_seat_active,
            self.cpu_step_delay_ms,
            self.turn_timer_ms,
            self.response_timer_ms,
            meta,
        );
        let fallback_id = recording
            .meta
            .recording_id
            .clone()
            .unwrap_or_else(|| format!("match-{}", game.seed()));
        let path = self
            .active_save_path
            .clone()
            .unwrap_or_else(|| self.save_paths().recording_path(&fallback_id));
        write_recording_async(path, recording);
        self.active_recording_id = None;
        self.active_recording_meta = None;
        self.active_save_path = None;
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let key = normalize_key_event(key);

        if self.import_scenario.is_some() {
            return self.handle_import_scenario_key(key);
        }
        if self.pause_open {
            return self.handle_pause_key(key);
        }
        if self.scores_open {
            return self.handle_scores_key(key);
        }
        if self.recommendations_open {
            return self.handle_recommendations_key(key);
        }
        if self.help_open {
            return self.handle_help_key(key);
        }
        if self.rules_open {
            return self.handle_rules_key(key);
        }
        if self.settings_open {
            return self.handle_settings_key(key);
        }
        if self.setup.is_some() && self.screen == Screen::MainMenu {
            return self.handle_setup_key(key);
        }
        if self.load_setup.is_some() && self.screen == Screen::MainMenu {
            return self.handle_load_setup_key(key);
        }
        self.handle_scenario_setup_key_if_open(key)?;
        #[cfg(feature = "debug-menu")]
        self.handle_debug_setup_key_if_open(key)?;

        let action = self.keybinds.action_for(&key);
        if matches!(action, Some(BindAction::Help)) {
            self.help_open = true;
            return Ok(());
        }
        let rules_hotkey = matches!(action, Some(BindAction::RulesReference))
            || (key.code == KeyCode::Char('y') && key.modifiers == KeyModifiers::empty());
        if rules_hotkey {
            self.rules_open = true;
            self.rules_scroll = 0;
            return Ok(());
        }
        if matches!(action, Some(BindAction::Scores))
            && self.screen == Screen::Table
            && self.hand_result.is_none()
            && self.game_summary.is_none()
        {
            self.scores_open = true;
            return Ok(());
        }
        if matches!(action, Some(BindAction::Recommendations))
            && self.screen == Screen::Table
            && self.is_human_pending()
            && self.hand_result.is_none()
            && self.game_summary.is_none()
        {
            self.refresh_recommendations();
            self.recommendations_open = true;
            self.recommendations_scroll = 0;
            return Ok(());
        }

        match self.screen {
            Screen::MainMenu => self.handle_main_menu(&key, action),
            Screen::Table => self.handle_table(&key, action),
            Screen::ReplayReview => self.handle_replay_review_key(&key, action),
        }
    }

    fn is_activate(&self, key: &KeyEvent) -> bool {
        self.keybinds.is_any_bound(
            key,
            &[
                BindAction::MenuSelect,
                BindAction::Confirm,
                BindAction::Continue,
            ],
        )
    }

    fn handle_scores_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let action = self.keybinds.action_for(&key);
        if matches!(action, Some(BindAction::Quit)) {
            self.quit = true;
        }
        if matches!(
            action,
            Some(BindAction::Scores) | Some(BindAction::Back) | Some(BindAction::Quit)
        ) {
            self.scores_open = false;
        }
        Ok(())
    }

    fn refresh_recommendations(&mut self) {
        self.recommendations_cache = self
            .active_game
            .as_ref()
            .map(|game| game.recommendations(self.human_seat_active, 8))
            .unwrap_or_default();
    }

    fn handle_recommendations_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let action = self.keybinds.action_for(&key);
        if matches!(action, Some(BindAction::Quit)) {
            self.quit = true;
        }
        if matches!(
            action,
            Some(BindAction::Recommendations) | Some(BindAction::Back) | Some(BindAction::Quit)
        ) {
            self.recommendations_open = false;
            return Ok(());
        }
        let page = 8usize;
        match key.code {
            KeyCode::Up => {
                self.recommendations_scroll = self.recommendations_scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                let max = ui::recommendation_line_count(self).saturating_sub(1);
                self.recommendations_scroll = (self.recommendations_scroll + 1).min(max);
            }
            KeyCode::PageUp => {
                self.recommendations_scroll = self.recommendations_scroll.saturating_sub(page);
            }
            KeyCode::PageDown => {
                let max = ui::recommendation_line_count(self).saturating_sub(1);
                self.recommendations_scroll = (self.recommendations_scroll + page).min(max);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let action = self.keybinds.action_for(&key);
        if matches!(
            action,
            Some(BindAction::Help) | Some(BindAction::Back) | Some(BindAction::Quit)
        ) {
            self.help_open = false;
        }
        Ok(())
    }

    fn handle_rules_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let action = self.keybinds.action_for(&key);
        let close = matches!(
            action,
            Some(BindAction::RulesReference) | Some(BindAction::Back) | Some(BindAction::Quit)
        ) || (key.code == KeyCode::Char('y') && key.modifiers == KeyModifiers::empty());
        if close {
            self.rules_open = false;
            return Ok(());
        }
        let page = 12usize;
        match key.code {
            KeyCode::Up => {
                self.rules_scroll = self.rules_scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                let max = ui::rules_line_count().saturating_sub(1);
                self.rules_scroll = (self.rules_scroll + 1).min(max);
            }
            KeyCode::PageUp => {
                self.rules_scroll = self.rules_scroll.saturating_sub(page);
            }
            KeyCode::PageDown => {
                let max = ui::rules_line_count().saturating_sub(1);
                self.rules_scroll = (self.rules_scroll + page).min(max);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_main_menu(
        &mut self,
        key: &KeyEvent,
        action: Option<BindAction>,
    ) -> Result<(), AppError> {
        #[cfg(feature = "debug-menu")]
        {
            self.handle_debug_mode_menu(key, action)
        }

        #[cfg(not(feature = "debug-menu"))]
        {
            if self.main_menu_mode == MainMenuMode::LoadGame {
                return self.handle_load_game_menu(key, action);
            }
            if self.main_menu_mode == MainMenuMode::Replays {
                return self.handle_replays_menu(key, action);
            }
            if self.main_menu_mode == MainMenuMode::Scenarios {
                return self.handle_scenarios_menu(key, action);
            }

            if self.is_activate(key) {
                return match self.menu_index {
                    0 => {
                        self.setup = Some(NewGameSetup::new(
                            self.config.default_difficulty,
                            self.config.human_seat,
                            self.config.cpu_step_delay_ms,
                            self.config.turn_timer_ms,
                            self.config.response_timer_ms,
                        ));
                        Ok(())
                    }
                    1 => self.open_load_game_menu(),
                    REPLAYS_MENU_INDEX => self.open_replays_menu(),
                    scenario_menu::SCENARIOS_MENU_INDEX => self.open_scenarios_menu(),
                    scenario_menu::SETTINGS_MENU_INDEX => {
                        self.settings_field = SettingsField::Theme;
                        self.settings_open = true;
                        Ok(())
                    }
                    _ => {
                        self.quit = true;
                        Ok(())
                    }
                };
            }

            let max = ROOT_MENU_LEN.saturating_sub(1);
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
    }

    fn open_load_game_menu(&mut self) -> Result<(), AppError> {
        self.load_entries = list_in_progress(&self.save_paths())?;
        self.main_menu_mode = MainMenuMode::LoadGame;
        self.menu_index = 0;
        if self.load_entries.is_empty() {
            self.status = "No in-progress saves found".into();
        } else {
            self.status.clear();
        }
        Ok(())
    }

    fn open_replays_menu(&mut self) -> Result<(), AppError> {
        self.replay_entries = list_finished(&self.save_paths())?;
        self.main_menu_mode = MainMenuMode::Replays;
        self.menu_index = 0;
        if self.replay_entries.is_empty() {
            self.status = "No finished replays found".into();
        } else {
            self.status.clear();
        }
        Ok(())
    }

    fn handle_load_game_menu(
        &mut self,
        key: &KeyEvent,
        action: Option<BindAction>,
    ) -> Result<(), AppError> {
        if self.keybinds.is_bound(key, BindAction::Back) {
            self.main_menu_mode = MainMenuMode::Root;
            self.menu_index = 1;
            return Ok(());
        }
        if self.load_entries.is_empty() {
            if self.is_activate(key) || self.keybinds.is_bound(key, BindAction::Back) {
                self.main_menu_mode = MainMenuMode::Root;
                self.menu_index = 1;
            }
            return Ok(());
        }
        if self.is_activate(key) {
            return self.open_load_seat_picker();
        }
        let max = self.load_entries.len().saturating_sub(1);
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

    fn handle_replays_menu(
        &mut self,
        key: &KeyEvent,
        action: Option<BindAction>,
    ) -> Result<(), AppError> {
        if self.keybinds.is_bound(key, BindAction::Back) {
            self.main_menu_mode = MainMenuMode::Root;
            self.menu_index = REPLAYS_MENU_INDEX;
            return Ok(());
        }
        if self.replay_entries.is_empty() {
            if self.is_activate(key) || self.keybinds.is_bound(key, BindAction::Back) {
                self.main_menu_mode = MainMenuMode::Root;
                self.menu_index = REPLAYS_MENU_INDEX;
            }
            return Ok(());
        }
        if self.is_activate(key) {
            return self.open_replay_review();
        }
        let max = self.replay_entries.len().saturating_sub(1);
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

    fn open_replay_review(&mut self) -> Result<(), AppError> {
        let entry = self
            .replay_entries
            .get(self.menu_index)
            .cloned()
            .ok_or_else(|| AppError::Config {
                path: self.config_path.clone(),
                detail: "no replay selected".into(),
            })?;
        let recording = read_recording(&entry.path)?;
        let title = recording
            .meta
            .title
            .clone()
            .unwrap_or_else(|| entry.label.clone());
        let step_delay_ms = recording
            .cpu_step_delay_ms
            .map(crate::timers::normalize_cpu)
            .unwrap_or_else(|| crate::timers::normalize_cpu(self.config.cpu_step_delay_ms));
        let player = RecordingPlayer::new(recording).map_err(AppError::Engine)?;
        self.replay_review = Some(ReplayReview::new(entry, player, step_delay_ms));
        self.screen = Screen::ReplayReview;
        self.status = format!("Replay: {title}");
        Ok(())
    }

    fn close_replay_review(&mut self) {
        self.replay_review = None;
        self.screen = Screen::MainMenu;
        self.main_menu_mode = MainMenuMode::Replays;
    }

    fn tick_replay(&mut self) -> Result<(), AppError> {
        if self.screen != Screen::ReplayReview {
            return Ok(());
        }
        let visible = 12usize;
        if let Some(review) = self.replay_review.as_mut() {
            review.tick_autoplay()?;
            review.sync_event_scroll_to_cursor(visible);
            if let Some(index) = review.player.event_index() {
                if let Some(event) = review.player.recording().events.get(index) {
                    self.status = event_text::describe_event(event);
                }
            } else {
                self.status = "Match start".into();
            }
        }
        Ok(())
    }

    fn handle_replay_review_key(
        &mut self,
        key: &KeyEvent,
        action: Option<BindAction>,
    ) -> Result<(), AppError> {
        if self.keybinds.is_bound(key, BindAction::Back) {
            self.close_replay_review();
            return Ok(());
        }
        if matches!(action, Some(BindAction::Quit)) {
            self.quit = true;
            return Ok(());
        }
        let page = 8isize;
        let visible = 12usize;
        let Some(review) = self.replay_review.as_mut() else {
            return Ok(());
        };

        if key.code == KeyCode::Char(' ') {
            review.toggle_playback();
            return Ok(());
        }
        if let KeyCode::Char(seat @ ('1'..='4')) = key.code {
            review.set_view_seat((seat as u8 - b'1') as usize);
            return Ok(());
        }

        match action {
            Some(BindAction::TilePrev) => review.step_back()?,
            Some(BindAction::TileNext) => review.step_forward()?,
            Some(BindAction::MenuCycle) => review.cycle_view_seat(),
            Some(BindAction::MenuUp) => review.scroll_events(-1, visible),
            Some(BindAction::MenuDown) => review.scroll_events(1, visible),
            _ => match key.code {
                KeyCode::Home => review.seek_start()?,
                KeyCode::End => review.seek_end()?,
                KeyCode::Char('n') => review.seek_next_hand()?,
                KeyCode::Char('b') => review.seek_prev_hand()?,
                KeyCode::PageUp => review.scroll_events(-page, visible),
                KeyCode::PageDown => review.scroll_events(page, visible),
                _ => {}
            },
        }
        review.sync_event_scroll_to_cursor(visible);
        if let Some(index) = review.player.event_index() {
            if let Some(event) = review.player.recording().events.get(index) {
                self.status = event_text::describe_event(event);
            }
        } else {
            self.status = "Match start".into();
        }
        Ok(())
    }

    fn open_load_seat_picker(&mut self) -> Result<(), AppError> {
        let entry = self
            .load_entries
            .get(self.menu_index)
            .cloned()
            .ok_or_else(|| AppError::Config {
                path: self.config_path.clone(),
                detail: "no save selected".into(),
            })?;
        let recording = read_recording(&entry.path)?;
        if recording.game_status != GameStatus::InProgress {
            self.status = "Only in-progress saves can be loaded".into();
            return Ok(());
        }
        if recording.game_phase == GamePhase::Ended {
            self.status = "Save is already finished".into();
            return Ok(());
        }
        self.load_setup = Some(LoadGameSetup::new(
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

    fn handle_load_setup_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let action = self.keybinds.action_for(&key);
        if self.keybinds.is_bound(&key, BindAction::Back) {
            self.load_setup = None;
            self.main_menu_mode = MainMenuMode::LoadGame;
            return Ok(());
        }
        if self.is_activate(&key) {
            let confirm = self
                .load_setup
                .as_ref()
                .is_some_and(|load| load.selected == LoadSetupField::Confirm);
            if confirm {
                return self.confirm_load_game();
            }
            if let Some(load) = self.load_setup.as_mut() {
                match load.selected {
                    LoadSetupField::HumanSeat => load.cycle_seat(),
                    LoadSetupField::CpuStepDelay => load.cycle_cpu_delay(),
                    LoadSetupField::TurnTimer => load.cycle_turn_timer(),
                    LoadSetupField::ResponseTimer => load.cycle_response_timer(),
                    LoadSetupField::Confirm => {}
                }
            }
            return Ok(());
        }
        let Some(load) = self.load_setup.as_mut() else {
            return Ok(());
        };
        match action {
            Some(BindAction::MenuUp) => load.select_prev(),
            Some(BindAction::MenuDown) => load.select_next(),
            Some(BindAction::MenuToggle) | Some(BindAction::MenuCycle) => match load.selected {
                LoadSetupField::HumanSeat => load.cycle_seat(),
                LoadSetupField::CpuStepDelay => load.cycle_cpu_delay(),
                LoadSetupField::TurnTimer => load.cycle_turn_timer(),
                LoadSetupField::ResponseTimer => load.cycle_response_timer(),
                LoadSetupField::Confirm => {}
            },
            Some(BindAction::Quit) => self.quit = true,
            _ => {}
        }
        Ok(())
    }

    fn confirm_load_game(&mut self) -> Result<(), AppError> {
        let load = self.load_setup.take().expect("load setup present");
        let human = load.selected_seat;
        let saved_human_seat = load.saved_human_seat;
        let using_saved = load.using_saved_seat();
        let status = if using_saved {
            format!("Loaded save as {}", LoadGameSetup::seat_name(human))
        } else {
            format!(
                "Loaded save as {} (saved seat was {})",
                LoadGameSetup::seat_name(human),
                LoadGameSetup::seat_name(saved_human_seat)
            )
        };
        let setup = load.game_setup_for_load();
        let seed = load.recording.seed;
        let agents = setup.build_agents(seed);
        if load.recording.game_status != GameStatus::InProgress {
            self.status = "Only in-progress saves can be loaded".into();
            return Ok(());
        }
        if load.recording.game_phase == GamePhase::Ended {
            self.status = "Save is already finished".into();
            return Ok(());
        }
        let game = load.recording.restore()?;
        self.setup_meta = Some(setup);
        self.agents = Some(agents);
        self.active_game = Some(game);
        self.human_seat_active = human;
        self.cpu_step_delay_ms = load.cpu_step_delay_ms;
        self.turn_timer_ms = load.turn_timer_ms;
        self.response_timer_ms = load.response_timer_ms;
        self.cpu_step_wait_until = None;
        self.action_timer.reset();
        self.active_recording_id = Some(load.entry.recording_id);
        self.active_recording_meta = Some(load.recording.meta);
        self.active_save_path = Some(load.entry.path);
        self.table_mode = TableMode::Normal;
        self.tile_index = 0;
        self.hand_result = None;
        self.game_summary = None;
        self.screen = Screen::Table;
        self.status = status;
        Ok(())
    }

    fn handle_settings_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let action = self.keybinds.action_for(&key);
        if self.keybinds.is_bound(&key, BindAction::Back) {
            self.save_config()?;
            self.settings_open = false;
            return Ok(());
        }
        match action {
            Some(BindAction::Quit) => self.quit = true,
            Some(BindAction::MenuUp) => {
                self.settings_field = self.settings_field.prev();
            }
            Some(BindAction::MenuDown) => {
                self.settings_field = self.settings_field.next();
            }
            Some(BindAction::MenuCycle) | Some(BindAction::MenuToggle) => {
                self.apply_settings_change();
            }
            _ if self.is_activate(&key) => {
                self.apply_settings_change();
            }
            _ => {}
        }
        Ok(())
    }

    fn apply_settings_change(&mut self) {
        match self.settings_field {
            SettingsField::Theme => {
                self.config.theme = cycle_theme(&self.config.theme);
            }
            SettingsField::RulesProfile => {
                self.config.rules_profile = self.config.rules_profile.next();
            }
            SettingsField::DefaultDifficulty => {
                self.config.default_difficulty =
                    setup::cycle_difficulty(self.config.default_difficulty);
            }
            SettingsField::HumanSeat => {
                self.config.human_seat = (self.config.human_seat + 1) % 4;
            }
            SettingsField::CpuStepDelay => {
                self.config.cpu_step_delay_ms =
                    crate::timers::cycle_cpu(self.config.cpu_step_delay_ms);
            }
            SettingsField::TurnTimer => {
                self.config.turn_timer_ms = crate::timers::cycle_turn(self.config.turn_timer_ms);
            }
            SettingsField::ResponseTimer => {
                self.config.response_timer_ms =
                    crate::timers::cycle_response(self.config.response_timer_ms);
            }
        }
    }

    fn save_config(&mut self) -> Result<(), AppError> {
        self.config.save(&self.config_path)
    }

    fn handle_setup_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let action = self.keybinds.action_for(&key);
        if self.keybinds.is_bound(&key, BindAction::Back) {
            self.setup = None;
            return Ok(());
        }
        if self.is_activate(&key) {
            let confirm = self
                .setup
                .as_ref()
                .is_some_and(|setup| setup.selected == SetupField::Confirm);
            if confirm {
                return self.start_match();
            }
            if let Some(setup) = self.setup.as_mut() {
                setup.toggle_selected();
            }
            return Ok(());
        }
        let Some(setup) = self.setup.as_mut() else {
            return Ok(());
        };
        match action {
            Some(BindAction::MenuUp) => setup.select_prev(),
            Some(BindAction::MenuDown) => setup.select_next(),
            Some(BindAction::MenuToggle) => setup.toggle_selected(),
            Some(BindAction::MenuCycle) => setup.cycle_selected(),
            Some(BindAction::Quit) => self.quit = true,
            _ => {}
        }
        Ok(())
    }

    fn start_match(&mut self) -> Result<(), AppError> {
        let setup = self.setup.take().expect("setup present");
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(1);
        let game_setup = setup.to_game_setup(seed);
        let agents = game_setup.build_agents(seed);
        let game = Game::new(self.config.rules_config(), seed)?;
        self.human_seat_active = setup.human_seat;
        self.cpu_step_delay_ms = setup.cpu_step_delay_ms;
        self.turn_timer_ms = setup.turn_timer_ms;
        self.response_timer_ms = setup.response_timer_ms;
        self.cpu_step_wait_until = None;
        self.action_timer.reset();
        self.setup_meta = Some(game_setup);
        self.agents = Some(agents);
        self.active_game = Some(game);
        self.active_recording_id = Some(format!("match-{seed}"));
        self.active_recording_meta = None;
        self.active_save_path = None;
        self.table_mode = TableMode::Normal;
        self.tile_index = 0;
        self.hand_result = None;
        self.game_summary = None;
        self.screen = Screen::Table;
        self.status = "Match started (pause menu to save)".into();
        Ok(())
    }

    fn handle_hand_result(
        &mut self,
        key: &KeyEvent,
        action: Option<BindAction>,
    ) -> Result<(), AppError> {
        if self.try_exit_table_to_main_menu(action) {
            return Ok(());
        }
        if self.is_activate(key) {
            self.hand_result = None;
            self.table_mode = TableMode::Normal;
            return Ok(());
        }
        if matches!(action, Some(BindAction::Quit)) {
            self.quit = true;
        }
        Ok(())
    }

    fn handle_game_summary(
        &mut self,
        key: &KeyEvent,
        action: Option<BindAction>,
    ) -> Result<(), AppError> {
        if self.try_exit_table_to_main_menu(action) {
            return Ok(());
        }
        if self.is_activate(key) || self.keybinds.is_bound(key, BindAction::Back) {
            self.return_to_main_menu();
            return Ok(());
        }
        if matches!(action, Some(BindAction::Quit)) {
            self.quit = true;
        }
        Ok(())
    }

    fn handle_table(&mut self, key: &KeyEvent, action: Option<BindAction>) -> Result<(), AppError> {
        if self.hand_result.is_some() {
            return self.handle_hand_result(key, action);
        }
        if self.game_summary.is_some() {
            return self.handle_game_summary(key, action);
        }
        if self.try_exit_table_to_main_menu(action) {
            return Ok(());
        }

        if matches!(action, Some(BindAction::Back)) && self.table_mode == TableMode::Normal {
            self.open_pause_menu();
            return Ok(());
        }

        let human_turn = self.is_human_turn();
        if !human_turn {
            match action {
                Some(BindAction::Back) => self.open_pause_menu(),
                Some(BindAction::Quit) => self.quit = true,
                _ => {}
            }
            return Ok(());
        }

        if let Some(chosen) = self.map_table_action(key, action)? {
            self.apply_human_action(chosen)?;
        }
        Ok(())
    }

    fn map_table_action(
        &mut self,
        key: &KeyEvent,
        action: Option<BindAction>,
    ) -> Result<Option<Action>, AppError> {
        let menu = self.current_action_menu();
        if self.is_activate(key) {
            if let Some(action) = self.confirm_table_pick()? {
                return Ok(Some(action));
            }
            return Ok(None);
        }
        match action {
            Some(BindAction::Back) => {
                self.table_mode = TableMode::Normal;
            }
            Some(BindAction::Pass) if menu.can_pass => return Ok(Some(Action::Pass)),
            Some(BindAction::Ron) if menu.can_ron => return Ok(Some(Action::Ron)),
            Some(BindAction::Pon) if menu.can_pon => return Ok(Some(Action::Pon)),
            Some(BindAction::OpenKan) if menu.can_open_kan => {
                return Ok(Some(Action::Kan(KanIntent::Open)));
            }
            Some(BindAction::Tsumo) if menu.can_tsumo => return Ok(Some(Action::Tsumo)),
            Some(BindAction::AbortNineTerminals) if menu.can_abort_nine_terminals => {
                return Ok(Some(Action::AbortiveNineTerminals));
            }
            Some(BindAction::Riichi) if !menu.riichi.is_empty() => {
                self.table_mode = TableMode::PickRiichi;
                self.tile_index = 0;
            }
            Some(BindAction::ClosedKan) if !menu.closed_kans.is_empty() => {
                self.table_mode = TableMode::PickClosedKan;
                self.tile_index = 0;
            }
            Some(BindAction::Kakan) if !menu.kakan.is_empty() => {
                if menu.kakan.len() == 1 {
                    return Ok(Some(Action::Kan(KanIntent::Added {
                        meld_index: menu.kakan[0],
                    })));
                }
                self.table_mode = TableMode::PickKakan;
                self.tile_index = 0;
            }
            Some(BindAction::Chi) if !menu.chi.is_empty() => {
                if menu.chi.len() == 1 {
                    return Ok(Some(Action::Chi { tiles: menu.chi[0] }));
                }
                if self.table_mode == TableMode::PickChi {
                    self.bump_tile_index(1);
                } else {
                    self.table_mode = TableMode::PickChi;
                    self.chi_index = 0;
                    self.tile_index = 0;
                }
            }
            Some(BindAction::Discard) if !menu.discards.is_empty() => {
                self.table_mode = TableMode::PickDiscard;
                self.tile_index = 0;
            }
            Some(BindAction::TilePrev) => self.bump_tile_index(-1),
            Some(BindAction::TileNext) => self.bump_tile_index(1),
            Some(BindAction::Quit) => self.quit = true,
            _ => {}
        }
        Ok(None)
    }

    fn confirm_table_pick(&mut self) -> Result<Option<Action>, AppError> {
        let menu = self.current_action_menu();
        match self.table_mode {
            TableMode::PickDiscard => {
                let tile =
                    *menu
                        .discards
                        .get(self.tile_index)
                        .ok_or_else(|| AppError::Keybinds {
                            path: PathBuf::from("<table>"),
                            detail: "no discard selected".into(),
                        })?;
                Ok(Some(Action::Discard(tile)))
            }
            TableMode::PickRiichi => {
                let tile = *menu
                    .riichi
                    .get(self.tile_index)
                    .ok_or_else(|| AppError::Keybinds {
                        path: PathBuf::from("<table>"),
                        detail: "no riichi discard selected".into(),
                    })?;
                Ok(Some(Action::Riichi { discard: tile }))
            }
            TableMode::PickClosedKan => {
                let tile =
                    *menu
                        .closed_kans
                        .get(self.tile_index)
                        .ok_or_else(|| AppError::Keybinds {
                            path: PathBuf::from("<table>"),
                            detail: "no closed kan selected".into(),
                        })?;
                Ok(Some(Action::Kan(KanIntent::Closed { tile })))
            }
            TableMode::PickKakan => {
                let meld_index =
                    *menu
                        .kakan
                        .get(self.tile_index)
                        .ok_or_else(|| AppError::Keybinds {
                            path: PathBuf::from("<table>"),
                            detail: "no kakan meld selected".into(),
                        })?;
                Ok(Some(Action::Kan(KanIntent::Added { meld_index })))
            }
            TableMode::PickChi => {
                let tiles = menu.chi[self.chi_index];
                Ok(Some(Action::Chi { tiles }))
            }
            _ => Ok(None),
        }
    }

    fn bump_tile_index(&mut self, delta: isize) {
        let len = match self.table_mode {
            TableMode::PickDiscard => self.current_action_menu().discards.len(),
            TableMode::PickRiichi => self.current_action_menu().riichi.len(),
            TableMode::PickClosedKan => self.current_action_menu().closed_kans.len(),
            TableMode::PickKakan => self.current_action_menu().kakan.len(),
            TableMode::PickChi => self.current_action_menu().chi.len(),
            _ => 0,
        };
        if len == 0 {
            return;
        }
        let current = if self.table_mode == TableMode::PickChi {
            self.chi_index
        } else {
            self.tile_index
        };
        let wrapped = ((current as isize + delta).rem_euclid(len as isize)) as usize;
        if self.table_mode == TableMode::PickChi {
            self.chi_index = wrapped;
        } else {
            self.tile_index = wrapped;
        }
    }

    fn on_game_events(&mut self, events: &[GameEvent]) {
        if let Some(summary) = hand_result::summary_from_events(events) {
            self.hand_result = Some(summary);
        }
        if let Some(GameEvent::GameEnded { scores }) = events
            .iter()
            .find(|e| matches!(e, GameEvent::GameEnded { .. }))
        {
            self.game_summary = Some(*scores);
            self.finalize_finished_match();
        }
        if let Some(last) = events.last() {
            self.status = event_text::describe_event(last);
        }
    }

    fn is_human_turn(&self) -> bool {
        let Some(game) = self.active_game.as_ref() else {
            return false;
        };
        let Some(seat) = game.pending_seat() else {
            return false;
        };
        self.setup_meta
            .as_ref()
            .is_some_and(|s| s.slots[seat] == PlayerSlot::Human)
    }

    fn current_action_menu(&self) -> ActionMenu {
        let Some(game) = self.active_game.as_ref() else {
            return ActionMenu::default();
        };
        let Some(seat) = game.pending_seat() else {
            return ActionMenu::default();
        };
        ActionMenu::from_legal(&game.hand().legal_actions_for(seat))
    }

    // --- accessors for UI ---

    pub const fn screen(&self) -> Screen {
        self.screen
    }

    pub fn keybinds(&self) -> &Keybinds {
        &self.keybinds
    }

    pub fn keybinds_path(&self) -> Option<&PathBuf> {
        self.keybinds_path.as_ref()
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    pub const fn settings_field(&self) -> SettingsField {
        self.settings_field
    }

    pub fn theme(&self) -> Theme {
        Theme::resolve(&self.config.theme)
    }

    pub const fn menu_index(&self) -> usize {
        self.menu_index
    }

    pub const fn main_menu_mode(&self) -> MainMenuMode {
        self.main_menu_mode
    }

    pub fn load_entries(&self) -> &[RecordingEntry] {
        &self.load_entries
    }

    pub fn replay_entries(&self) -> &[RecordingEntry] {
        &self.replay_entries
    }

    pub fn replay_review(&self) -> Option<&ReplayReview> {
        self.replay_review.as_ref()
    }

    pub fn setup(&self) -> Option<&NewGameSetup> {
        self.setup.as_ref()
    }

    pub fn setup_open(&self) -> bool {
        self.setup.is_some()
    }

    pub fn resume_setup_open(&self) -> bool {
        self.load_setup.is_some() || self.scenario_setup.is_some()
    }

    pub const fn debug_menu_enabled(&self) -> bool {
        cfg!(feature = "debug-menu")
    }

    pub const fn human_seat_active(&self) -> usize {
        self.human_seat_active
    }

    pub const fn table_mode(&self) -> TableMode {
        self.table_mode
    }

    pub const fn tile_index(&self) -> usize {
        self.tile_index
    }

    pub const fn chi_index(&self) -> usize {
        self.chi_index
    }

    pub fn hand_result(&self) -> Option<&HandResultSummary> {
        self.hand_result.as_ref()
    }

    pub fn game_summary(&self) -> Option<&[i32; 4]> {
        self.game_summary.as_ref()
    }

    pub const fn help_open(&self) -> bool {
        self.help_open
    }

    pub const fn pause_open(&self) -> bool {
        self.pause_open
    }

    pub const fn scores_open(&self) -> bool {
        self.scores_open
    }

    pub const fn recommendations_open(&self) -> bool {
        self.recommendations_open
    }

    pub const fn recommendations_scroll(&self) -> usize {
        self.recommendations_scroll
    }

    pub fn recommendations(&self) -> &[Recommendation] {
        &self.recommendations_cache
    }

    pub const fn pause_index(&self) -> PauseItem {
        self.pause_index
    }

    pub fn export_save_open(&self) -> bool {
        self.export_save.is_some()
    }

    pub fn export_save_path(&self) -> Option<&str> {
        self.export_save.as_ref().map(PathInputDialog::path)
    }

    pub const fn settings_open(&self) -> bool {
        self.settings_open
    }

    pub const fn rules_open(&self) -> bool {
        self.rules_open
    }

    pub const fn rules_scroll(&self) -> usize {
        self.rules_scroll
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn player_view(&self) -> Option<PlayerView> {
        let game = self.active_game.as_ref()?;
        Some(PlayerView::from_game(game, self.human_seat_active))
    }

    pub fn action_menu(&self) -> ActionMenu {
        self.current_action_menu()
    }

    pub fn is_human_pending(&self) -> bool {
        self.screen == Screen::Table && self.is_human_turn() && self.hand_result.is_none()
    }

    pub fn wall_remaining(&self) -> Option<usize> {
        Some(self.active_game.as_ref()?.hand().wall().live_remaining())
    }

    /// Whose turn it is on the table (discarder until reaction window closes).
    pub fn turn_highlight_seat(&self) -> Option<usize> {
        let game = self.active_game.as_ref()?;
        let hand = game.hand();
        match hand.phase() {
            HandPhase::Reaction => hand.pending_call().map(|call| call.discarder),
            HandPhase::Draw | HandPhase::Discard => Some(hand.current_actor()),
            HandPhase::Ended => None,
        }
    }

    /// Countdown for the human's pending decision (actions bar only).
    pub fn human_decision_timer_label(&self) -> Option<String> {
        if !self.is_human_pending() {
            return None;
        }
        let phase = self.active_game.as_ref()?.hand().phase();
        if phase == HandPhase::Reaction && self.action_menu().is_pass_only() {
            return None;
        }
        let kind = match phase {
            HandPhase::Draw => return None,
            HandPhase::Discard => TimerKind::Turn,
            HandPhase::Reaction => TimerKind::Response,
            HandPhase::Ended => return None,
        };
        self.action_timer
            .seat_timer(kind)
            .map(format_decision_timer)
    }
}

impl TableMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::PickDiscard => "pick discard",
            Self::PickRiichi => "pick riichi discard",
            Self::PickClosedKan => "pick closed kan",
            Self::PickKakan => "pick kakan meld",
            Self::PickChi => "pick chi",
        }
    }
}

pub fn seat_label(seat: usize, human: usize) -> String {
    let names = ["East", "South", "West", "North"];
    let rel = (seat + 4 - human) % 4;
    let pos = match rel {
        0 => "You",
        1 => "Right",
        2 => "Across",
        3 => "Left",
        _ => unreachable!(),
    };
    format!("{} ({})", names[seat], pos)
}

pub fn phase_label(phase: HandPhase) -> &'static str {
    match phase {
        HandPhase::Draw => "Draw",
        HandPhase::Discard => "Discard",
        HandPhase::Reaction => "Reaction",
        HandPhase::Ended => "Ended",
    }
}
