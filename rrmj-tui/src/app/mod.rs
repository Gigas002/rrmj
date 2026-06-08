mod actions;
mod hand_result;
mod settings;
mod setup;

pub use actions::ActionMenu;
pub use hand_result::HandResultSummary;
pub use settings::SettingsField;
pub use setup::{NewGameSetup, SetupField, difficulty_label};

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use librrmj::action::Action;
use librrmj::agent::{PlayerSlot, PlayerView};
use librrmj::ai::{MatchSetup, SeatAgent};
use librrmj::event::Event as GameEvent;
use librrmj::game::Match;
use librrmj::rules::RulesConfig;
use librrmj::state::HandPhase;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::config::{AppConfig, cycle_theme};
use crate::error::AppError;
use crate::input::{BindAction, Keybinds, normalize_key_event};
use crate::theme::Theme;
use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableMode {
    Normal,
    PickDiscard,
    PickRiichi,
    PickClosedKan,
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
    setup: Option<NewGameSetup>,
    match_game: Option<Match>,
    agents: Option<[SeatAgent; 4]>,
    setup_meta: Option<MatchSetup>,
    human_seat_active: usize,
    table_mode: TableMode,
    tile_index: usize,
    chi_index: usize,
    hand_result: Option<HandResultSummary>,
    match_summary: Option<[i32; 4]>,
    help_open: bool,
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
        Self {
            screen: Screen::MainMenu,
            keybinds,
            keybinds_path,
            config,
            config_path,
            settings_field: SettingsField::Theme,
            menu_index: 0,
            setup: None,
            match_game: None,
            agents: None,
            setup_meta: None,
            human_seat_active: 0,
            table_mode: TableMode::Normal,
            tile_index: 0,
            chi_index: 0,
            hand_result: None,
            match_summary: None,
            help_open: false,
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
        if self.screen != Screen::Table || self.hand_result.is_some() {
            return Ok(());
        }
        while let Some(seat) = self
            .match_game
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
                let game = self.match_game.as_mut().expect("match present");
                let agents = self.agents.as_mut().expect("agents present");
                match game.step(agents)? {
                    Some(step) => step.events,
                    None => break,
                }
            };
            let ended = self.match_game.as_ref().is_some_and(|game| game.is_ended());
            self.on_game_events(&events);
            if self.hand_result.is_some() || ended {
                break;
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<(), AppError> {
        let key = normalize_key_event(key);

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

        match self.screen {
            Screen::MainMenu => self.handle_main_menu(&key, action),
            Screen::Table => self.handle_table(&key, action),
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
        match key.code {
            KeyCode::Up => {
                self.rules_scroll = self.rules_scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                let max = ui::rules_line_count().saturating_sub(1);
                self.rules_scroll = (self.rules_scroll + 1).min(max);
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
        if self.is_activate(key) {
            return match self.menu_index {
                0 => {
                    self.setup = Some(NewGameSetup::new(
                        self.config.default_difficulty,
                        self.config.human_seat,
                    ));
                    Ok(())
                }
                1 => {
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

        match action {
            Some(BindAction::MenuUp) => {
                self.menu_index = self.menu_index.saturating_sub(1);
            }
            Some(BindAction::MenuDown) => {
                self.menu_index = (self.menu_index + 1).min(2);
            }
            Some(BindAction::Quit) => self.quit = true,
            _ => {}
        }
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
            SettingsField::DefaultDifficulty => {
                self.config.default_difficulty =
                    setup::cycle_difficulty(self.config.default_difficulty);
            }
            SettingsField::HumanSeat => {
                self.config.human_seat = (self.config.human_seat + 1) % 4;
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
        let match_setup = setup.to_match_setup(seed);
        let agents = match_setup.build_agents(seed);
        let game = Match::new(RulesConfig::standard(), seed)?;
        self.human_seat_active = setup.human_seat;
        self.setup_meta = Some(match_setup);
        self.agents = Some(agents);
        self.match_game = Some(game);
        self.table_mode = TableMode::Normal;
        self.tile_index = 0;
        self.hand_result = None;
        self.match_summary = None;
        self.screen = Screen::Table;
        self.status = "Match started".into();
        Ok(())
    }

    fn handle_hand_result(
        &mut self,
        key: &KeyEvent,
        action: Option<BindAction>,
    ) -> Result<(), AppError> {
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

    fn handle_match_summary(
        &mut self,
        key: &KeyEvent,
        action: Option<BindAction>,
    ) -> Result<(), AppError> {
        if self.is_activate(key) || self.keybinds.is_bound(key, BindAction::Back) {
            self.match_game = None;
            self.agents = None;
            self.setup_meta = None;
            self.match_summary = None;
            self.screen = Screen::MainMenu;
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
        if self.match_summary.is_some() {
            return self.handle_match_summary(key, action);
        }

        let human_turn = self.is_human_turn();
        if !human_turn {
            match action {
                Some(BindAction::Back) | Some(BindAction::Quit) => self.quit = true,
                _ => {}
            }
            return Ok(());
        }

        if let Some(chosen) = self.map_table_action(key, action)? {
            let seat = self
                .match_game
                .as_ref()
                .and_then(|g| g.pending_seat())
                .expect("human turn");
            let events = self
                .match_game
                .as_mut()
                .unwrap()
                .apply_action(seat, chosen)?;
            self.on_game_events(&events);
            self.table_mode = TableMode::Normal;
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
            Some(BindAction::OpenKan) if menu.can_open_kan => return Ok(Some(Action::OpenKan)),
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
            Some(BindAction::Chi) if !menu.chi.is_empty() => {
                if menu.chi.len() == 1 {
                    return Ok(Some(Action::Chi { tiles: menu.chi[0] }));
                }
                self.table_mode = TableMode::PickChi;
                self.chi_index = 0;
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
                Ok(Some(Action::ClosedKan { tile }))
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
            TableMode::PickChi => self.current_action_menu().chi.len(),
            _ => 0,
        };
        if len == 0 {
            return;
        }
        let idx = self.tile_index as isize + delta;
        let wrapped = (idx.rem_euclid(len as isize)) as usize;
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
        if let Some(GameEvent::MatchEnded { scores }) = events
            .iter()
            .find(|e| matches!(e, GameEvent::MatchEnded { .. }))
        {
            self.match_summary = Some(*scores);
        }
        if let Some(last) = events.last() {
            self.status = format!("{last:?}");
        }
    }

    fn is_human_turn(&self) -> bool {
        let Some(game) = self.match_game.as_ref() else {
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
        let Some(game) = self.match_game.as_ref() else {
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

    pub fn setup(&self) -> Option<&NewGameSetup> {
        self.setup.as_ref()
    }

    pub fn setup_open(&self) -> bool {
        self.setup.is_some()
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

    pub fn match_summary(&self) -> Option<&[i32; 4]> {
        self.match_summary.as_ref()
    }

    pub const fn help_open(&self) -> bool {
        self.help_open
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
        let game = self.match_game.as_ref()?;
        Some(PlayerView::from_match(game, self.human_seat_active))
    }

    pub fn action_menu(&self) -> ActionMenu {
        self.current_action_menu()
    }

    pub fn is_human_pending(&self) -> bool {
        self.screen == Screen::Table && self.is_human_turn() && self.hand_result.is_none()
    }

    pub fn wall_remaining(&self) -> Option<usize> {
        Some(self.match_game.as_ref()?.hand().wall().live_remaining())
    }

    pub fn actor_seat(&self) -> Option<usize> {
        let game = self.match_game.as_ref()?;
        let hand = game.hand();
        if hand.phase() == HandPhase::Reaction {
            hand.pending_reaction_seat()
        } else {
            Some(hand.current_actor())
        }
    }
}

impl TableMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::PickDiscard => "pick discard",
            Self::PickRiichi => "pick riichi discard",
            Self::PickClosedKan => "pick closed kan",
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
