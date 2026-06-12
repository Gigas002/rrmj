#![cfg(feature = "debug-menu")]

use librrmj::ai::GameSetup;
use librrmj::replay::GameRecording;

use crate::scenarios::ScenarioEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSetupField {
    HumanSeat,
    Confirm,
}

/// Seat picker shown after choosing a debug scenario.
#[derive(Debug, Clone)]
pub struct DebugScenarioSetup {
    pub entry: ScenarioEntry,
    pub recording: GameRecording,
    pub saved_human_seat: usize,
    pub selected_seat: usize,
    pub selected: DebugSetupField,
}

impl DebugScenarioSetup {
    pub fn new(
        entry: ScenarioEntry,
        recording: GameRecording,
        fallback_human_seat: usize,
    ) -> Self {
        let saved_human_seat = recording.human_seat.unwrap_or(fallback_human_seat);
        Self {
            entry,
            recording,
            saved_human_seat,
            selected_seat: saved_human_seat,
            selected: DebugSetupField::HumanSeat,
        }
    }

    pub fn seat_name(seat: usize) -> &'static str {
        crate::app::NewGameSetup::seat_name(seat)
    }

    pub fn using_saved_seat(&self) -> bool {
        self.selected_seat == self.saved_human_seat
    }

    pub fn select_next(&mut self) {
        self.selected = match self.selected {
            DebugSetupField::HumanSeat => DebugSetupField::Confirm,
            DebugSetupField::Confirm => DebugSetupField::HumanSeat,
        };
    }

    pub fn select_prev(&mut self) {
        self.selected = match self.selected {
            DebugSetupField::HumanSeat => DebugSetupField::Confirm,
            DebugSetupField::Confirm => DebugSetupField::HumanSeat,
        };
    }

    pub fn cycle_seat(&mut self) {
        self.selected_seat = (self.selected_seat + 1) % 4;
    }

    pub fn game_setup_for_launch(&self) -> GameSetup {
        self.recording
            .game_setup()
            .with_human_seat(self.selected_seat)
    }
}
