use crate::error::Error;
use crate::event::Event;
use crate::game::Game;

use super::apply::apply_events;
use super::recording::GameRecording;

/// Stepped playback over a recording's `events[]` with a derived `Game` at each cursor.
///
/// Observe-only: does not call `apply_action`. Used by replay mode (TUI Phase 14.4).
pub struct RecordingPlayer {
    recording: GameRecording,
    /// Last applied event index; `None` = fresh match before any event.
    cursor: Option<usize>,
    game: Game,
}

impl RecordingPlayer {
    pub fn new(recording: GameRecording) -> Result<Self, Error> {
        recording.validate()?;
        let game = Game::new(recording.rules_config.clone(), recording.seed)?;
        Ok(Self {
            recording,
            cursor: None,
            game,
        })
    }

    pub fn recording(&self) -> &GameRecording {
        &self.recording
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    /// Last applied event index, or `None` before the first event.
    pub fn event_index(&self) -> Option<usize> {
        self.cursor
    }

    pub fn event_count(&self) -> usize {
        self.recording.events.len()
    }

    pub fn at_start(&self) -> bool {
        self.cursor.is_none()
    }

    pub fn at_end(&self) -> bool {
        self.recording
            .events
            .len()
            .checked_sub(1)
            .is_none_or(|last| self.cursor == Some(last))
    }

    /// Jump to `index` (`None` = before first event).
    pub fn seek(&mut self, index: Option<usize>) -> Result<(), Error> {
        match index {
            None => {
                self.game = Game::new(self.recording.rules_config.clone(), self.recording.seed)?;
                self.cursor = None;
            }
            Some(index) => {
                if index >= self.recording.events.len() {
                    return Err(Error::InvalidRecording {
                        detail: format!(
                            "seek index {index} >= {} events",
                            self.recording.events.len()
                        ),
                    });
                }
                self.game = Game::new(self.recording.rules_config.clone(), self.recording.seed)?;
                apply_events(&mut self.game, &self.recording.events, Some(index))?;
                self.cursor = Some(index);
            }
        }
        Ok(())
    }

    /// Apply the next event. Returns `false` when already at the last event.
    pub fn step_forward(&mut self) -> Result<bool, Error> {
        let next = self.cursor.map_or(0, |index| index + 1);
        if next >= self.recording.events.len() {
            return Ok(false);
        }
        self.seek(Some(next))?;
        Ok(true)
    }

    /// Undo the last applied event. Returns `false` when already at the start.
    pub fn step_back(&mut self) -> Result<bool, Error> {
        match self.cursor {
            None => Ok(false),
            Some(0) => {
                self.seek(None)?;
                Ok(true)
            }
            Some(index) => {
                self.seek(Some(index - 1))?;
                Ok(true)
            }
        }
    }

    /// Alias for [`seek`](Self::seek) with a concrete index.
    pub fn play_to_index(&mut self, index: usize) -> Result<(), Error> {
        self.seek(Some(index))
    }

    /// Event indices where a new hand begins (`Event::HandStarted`).
    pub fn hand_boundaries(&self) -> Vec<usize> {
        self.recording
            .events
            .iter()
            .enumerate()
            .filter_map(|(index, event)| {
                matches!(event, Event::HandStarted { .. }).then_some(index)
            })
            .collect()
    }

    /// Seek to the `hand_index`-th hand (`0` = first `HandStarted` event).
    pub fn seek_hand(&mut self, hand_index: usize) -> Result<(), Error> {
        let boundary = self
            .hand_boundaries()
            .get(hand_index)
            .copied()
            .ok_or_else(|| Error::InvalidRecording {
                detail: format!("hand_index {hand_index} out of range"),
            })?;
        self.seek(Some(boundary))
    }
}
