use librrmj::game::Match;
use librrmj::replay::MatchRecording;

use super::event_text::describe_event;
use crate::save::RecordingEntry;

/// Static review of a finished match recording (no playback controls).
pub struct ReplayReview {
    pub entry: RecordingEntry,
    pub recording: MatchRecording,
    pub match_game: Match,
    pub view_seat: usize,
    pub event_scroll: usize,
}

impl ReplayReview {
    pub fn new(entry: RecordingEntry, recording: MatchRecording, match_game: Match) -> Self {
        let view_seat = recording.human_seat.unwrap_or(0);
        Self {
            entry,
            recording,
            match_game,
            view_seat,
            event_scroll: 0,
        }
    }

    pub fn title(&self) -> String {
        self.recording
            .meta
            .title
            .clone()
            .unwrap_or_else(|| self.entry.label.clone())
    }

    pub fn event_lines(&self) -> Vec<String> {
        self.recording
            .events
            .iter()
            .enumerate()
            .map(|(i, event)| format!("{:>4}. {}", i + 1, describe_event(event)))
            .collect()
    }

    pub fn scroll_events(&mut self, delta: isize, visible: usize) {
        let len = self.recording.events.len();
        if len == 0 {
            self.event_scroll = 0;
            return;
        }
        let max = len.saturating_sub(visible.max(1));
        let next = self.event_scroll as isize + delta;
        self.event_scroll = next.clamp(0, max as isize) as usize;
    }
}
