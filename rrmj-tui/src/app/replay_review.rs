use std::time::{Duration, Instant};

use librrmj::event::Event;
use librrmj::replay::RecordingPlayer;

use super::event_text::describe_event;
use crate::error::AppError;
use crate::save::RecordingEntry;

/// Observe-only replay of a finished recording (`RecordingPlayer` cursor).
pub struct ReplayReview {
    pub entry: RecordingEntry,
    pub player: RecordingPlayer,
    /// Seat whose concealed hand is shown (full information).
    pub view_seat: usize,
    pub event_scroll: usize,
    pub playing: bool,
    pub step_delay_ms: u64,
    step_wait_until: Option<Instant>,
}

impl ReplayReview {
    pub fn new(entry: RecordingEntry, player: RecordingPlayer, step_delay_ms: u64) -> Self {
        let view_seat = player.recording().human_seat.unwrap_or(0);
        Self {
            entry,
            player,
            view_seat,
            event_scroll: 0,
            playing: false,
            step_delay_ms,
            step_wait_until: None,
        }
    }

    pub fn title(&self) -> String {
        self.player
            .recording()
            .meta
            .title
            .clone()
            .unwrap_or_else(|| self.entry.label.clone())
    }

    pub fn event_lines(&self) -> Vec<String> {
        let mut lines = vec!["    · match start".into()];
        lines.extend(
            self.player
                .recording()
                .events
                .iter()
                .enumerate()
                .map(|(i, event)| format!("{:>4}. {}", i + 1, describe_event(event))),
        );
        lines
    }

    /// Index into [`event_lines`](Self::event_lines) for the playback cursor.
    pub fn cursor_line_index(&self) -> usize {
        self.player.event_index().map_or(0, |index| index + 1)
    }

    pub fn scroll_events(&mut self, delta: isize, visible: usize) {
        let len = self.event_lines().len();
        if len == 0 {
            self.event_scroll = 0;
            return;
        }
        let max = len.saturating_sub(visible.max(1));
        let next = self.event_scroll as isize + delta;
        self.event_scroll = next.clamp(0, max as isize) as usize;
    }

    pub fn sync_event_scroll_to_cursor(&mut self, visible: usize) {
        let cursor = self.cursor_line_index();
        if cursor < self.event_scroll {
            self.event_scroll = cursor;
        } else if cursor >= self.event_scroll + visible.max(1) {
            self.event_scroll = cursor.saturating_sub(visible.saturating_sub(1));
        }
    }

    pub fn cycle_view_seat(&mut self) {
        self.view_seat = (self.view_seat + 1) % 4;
    }

    pub fn set_view_seat(&mut self, seat: usize) {
        self.view_seat = seat % 4;
    }

    pub fn toggle_playback(&mut self) {
        if self.playing {
            self.playing = false;
            self.step_wait_until = None;
            return;
        }
        if self.player.at_end() {
            return;
        }
        self.playing = true;
        self.step_wait_until = Some(Instant::now());
    }

    pub fn tick_autoplay(&mut self) -> Result<(), AppError> {
        if !self.playing || self.player.at_end() {
            self.playing = false;
            self.step_wait_until = None;
            return Ok(());
        }
        if self
            .step_wait_until
            .is_some_and(|until| Instant::now() < until)
        {
            return Ok(());
        }
        if !self.player.step_forward().map_err(AppError::Engine)? {
            self.playing = false;
            self.step_wait_until = None;
            return Ok(());
        }
        self.step_wait_until = Some(Instant::now() + Duration::from_millis(self.step_delay_ms));
        Ok(())
    }

    pub fn step_forward(&mut self) -> Result<(), AppError> {
        self.playing = false;
        self.step_wait_until = None;
        self.player.step_forward().map_err(AppError::Engine)?;
        Ok(())
    }

    pub fn step_back(&mut self) -> Result<(), AppError> {
        self.playing = false;
        self.step_wait_until = None;
        self.player.step_back().map_err(AppError::Engine)?;
        Ok(())
    }

    pub fn seek_start(&mut self) -> Result<(), AppError> {
        self.playing = false;
        self.step_wait_until = None;
        self.player.seek(None).map_err(AppError::Engine)
    }

    pub fn seek_end(&mut self) -> Result<(), AppError> {
        self.playing = false;
        self.step_wait_until = None;
        let last = self.player.event_count().saturating_sub(1);
        if last == 0 && self.player.event_count() == 0 {
            return Ok(());
        }
        self.player.play_to_index(last).map_err(AppError::Engine)
    }

    fn current_hand_index(&self) -> usize {
        let cursor = self.player.event_index().unwrap_or(0);
        self.player
            .hand_boundaries()
            .iter()
            .rposition(|&boundary| boundary <= cursor)
            .unwrap_or(0)
    }

    pub fn seek_next_hand(&mut self) -> Result<(), AppError> {
        self.playing = false;
        self.step_wait_until = None;
        let next = self.current_hand_index() + 1;
        if next < self.player.hand_boundaries().len() {
            self.player.seek_hand(next).map_err(AppError::Engine)?;
        }
        Ok(())
    }

    pub fn seek_prev_hand(&mut self) -> Result<(), AppError> {
        self.playing = false;
        self.step_wait_until = None;
        let current = self.current_hand_index();
        if current == 0 {
            return self.seek_start();
        }
        self.player.seek_hand(current - 1).map_err(AppError::Engine)
    }

    pub fn recent_discard_highlight(&self) -> Option<(usize, usize)> {
        let index = self.player.event_index()?;
        match &self.player.recording().events[index] {
            Event::Discarded { seat, .. } => {
                let river_len = self.player.game().hand().discards(*seat).len();
                river_len
                    .checked_sub(1)
                    .map(|tile_index| (*seat, tile_index))
            }
            _ => None,
        }
    }

    pub fn status_text(&self) -> String {
        let total = self.player.event_count();
        let pos = self
            .player
            .event_index()
            .map(|i| format!("{}/{}", i + 1, total))
            .unwrap_or_else(|| format!("start / {total}"));
        let play = if self.playing { "playing" } else { "paused" };
        format!("Event {pos} · {play}")
    }
}

#[cfg(test)]
mod tests {
    use librrmj::replay::{MatchRecording, RecordingPlayer};

    use super::*;
    use crate::save::RecordingEntry;

    #[test]
    fn replay_player_matches_apply_until() {
        let text = include_str!("../../../examples/scenarios/dealer_tsumo.json");
        let recording = MatchRecording::from_json(text).unwrap();
        let entry = RecordingEntry {
            path: "dealer_tsumo.json".into(),
            recording_id: "dealer_tsumo".into(),
            label: "dealer tsumo".into(),
            detail: String::new(),
        };
        let player = RecordingPlayer::new(recording.clone()).unwrap();
        let mut review = ReplayReview::new(entry, player, 0);
        review.step_forward().unwrap();
        let at_index = recording.apply_until(0).unwrap();
        assert_eq!(review.player.game().snapshot(), at_index.snapshot());
        review.set_view_seat(2);
        assert_eq!(review.view_seat, 2);
    }
}
