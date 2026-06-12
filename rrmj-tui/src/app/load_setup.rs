use librrmj::ai::GameSetup;
use librrmj::replay::GameRecording;

use crate::save::RecordingEntry;
use crate::scenarios::ScenarioEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeSetupKind {
    SavedGame,
    Scenario,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadSetupField {
    HumanSeat,
    CpuStepDelay,
    TurnTimer,
    ResponseTimer,
    Confirm,
}

/// Seat picker shown after choosing a save to load.
#[derive(Debug, Clone)]
pub struct LoadGameSetup {
    pub entry: RecordingEntry,
    pub recording: GameRecording,
    /// Seat stored in the recording when the save was written.
    pub saved_human_seat: usize,
    pub selected_seat: usize,
    pub cpu_step_delay_ms: u64,
    pub turn_timer_ms: u64,
    pub response_timer_ms: u64,
    pub selected: LoadSetupField,
}

impl LoadGameSetup {
    pub fn from_scenario(
        entry: ScenarioEntry,
        recording: GameRecording,
        fallback_human_seat: usize,
        fallback_cpu_delay_ms: u64,
        fallback_turn_timer_ms: u64,
        fallback_response_timer_ms: u64,
    ) -> Self {
        let recording_entry = RecordingEntry {
            path: entry.path,
            recording_id: entry.id,
            label: entry.title,
            detail: entry.description,
        };
        Self::new(
            recording_entry,
            recording,
            fallback_human_seat,
            fallback_cpu_delay_ms,
            fallback_turn_timer_ms,
            fallback_response_timer_ms,
        )
    }

    pub fn new(
        entry: RecordingEntry,
        recording: GameRecording,
        fallback_human_seat: usize,
        fallback_cpu_delay_ms: u64,
        fallback_turn_timer_ms: u64,
        fallback_response_timer_ms: u64,
    ) -> Self {
        let saved_human_seat = recording.human_seat.unwrap_or(fallback_human_seat);
        let cpu_step_delay_ms = recording
            .cpu_step_delay_ms
            .map(crate::timers::normalize_cpu)
            .unwrap_or_else(|| crate::timers::normalize_cpu(fallback_cpu_delay_ms));
        let turn_timer_ms = recording
            .turn_timer_ms
            .map(crate::timers::normalize_turn)
            .unwrap_or_else(|| crate::timers::normalize_turn(fallback_turn_timer_ms));
        let response_timer_ms = recording
            .response_timer_ms
            .map(crate::timers::normalize_response)
            .unwrap_or_else(|| crate::timers::normalize_response(fallback_response_timer_ms));
        Self {
            entry,
            recording,
            saved_human_seat,
            selected_seat: saved_human_seat,
            cpu_step_delay_ms,
            turn_timer_ms,
            response_timer_ms,
            selected: LoadSetupField::HumanSeat,
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
            LoadSetupField::HumanSeat => LoadSetupField::CpuStepDelay,
            LoadSetupField::CpuStepDelay => LoadSetupField::TurnTimer,
            LoadSetupField::TurnTimer => LoadSetupField::ResponseTimer,
            LoadSetupField::ResponseTimer => LoadSetupField::Confirm,
            LoadSetupField::Confirm => LoadSetupField::HumanSeat,
        };
    }

    pub fn select_prev(&mut self) {
        self.selected = match self.selected {
            LoadSetupField::HumanSeat => LoadSetupField::Confirm,
            LoadSetupField::CpuStepDelay => LoadSetupField::HumanSeat,
            LoadSetupField::TurnTimer => LoadSetupField::CpuStepDelay,
            LoadSetupField::ResponseTimer => LoadSetupField::TurnTimer,
            LoadSetupField::Confirm => LoadSetupField::ResponseTimer,
        };
    }

    pub fn cycle_seat(&mut self) {
        self.selected_seat = (self.selected_seat + 1) % 4;
    }

    pub fn cycle_cpu_delay(&mut self) {
        self.cpu_step_delay_ms = crate::timers::cycle_cpu(self.cpu_step_delay_ms);
    }

    pub fn cycle_turn_timer(&mut self) {
        self.turn_timer_ms = crate::timers::cycle_turn(self.turn_timer_ms);
    }

    pub fn cycle_response_timer(&mut self) {
        self.response_timer_ms = crate::timers::cycle_response(self.response_timer_ms);
    }

    pub fn game_setup_for_load(&self) -> GameSetup {
        self.recording
            .game_setup()
            .with_human_seat(self.selected_seat)
    }
}

#[cfg(test)]
mod tests {
    use librrmj::agent::PlayerSlot;

    use super::*;

    #[test]
    fn rejects_finished_recording_for_load() {
        let text = include_str!("../../../examples/scenarios/match_finished.json");
        let recording = GameRecording::from_json(text).unwrap();
        assert_ne!(
            recording.game_status,
            librrmj::replay::GameStatus::InProgress
        );
    }

    #[test]
    fn remaps_agents_when_study_seat_chosen() {
        let text = include_str!("../../../examples/scenarios/dealer_tsumo.json");
        let recording = GameRecording::from_json(text).unwrap();
        let entry = RecordingEntry {
            path: "test.json".into(),
            recording_id: "test".into(),
            label: "test".into(),
            detail: String::new(),
        };
        let mut load = LoadGameSetup::new(entry, recording, 0, 300, 30_000, 5_000);
        load.selected_seat = 1;

        let setup = load.game_setup_for_load();
        assert_eq!(setup.slots[1], PlayerSlot::Human);
        assert_eq!(setup.slots[0], PlayerSlot::Cpu);
    }
}
