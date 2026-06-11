use std::io::{Read, Write};

use crate::action::Action;
use crate::agent::PlayerSlot;
use crate::ai::{AiConfig, MatchSetup};
use crate::error::Error;
use crate::event::Event;
use crate::game::{Match, MatchPhase, RoundWind};
use crate::rules::{RulesConfig, RulesProfileId};
use crate::state::HandState;

use super::Replay;
use super::apply::apply_events;
use super::hand_snapshot::HandSnapshot;

pub const FORMAT_VERSION: u32 = 2;

/// Whether a recording can be resumed or is a completed match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(rename_all = "snake_case")]
pub enum MatchStatus {
    InProgress,
    Finished,
}

/// Per-seat player binding stored in a recording.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerSetup {
    pub slot: PlayerSlot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai: Option<AiConfig>,
}

impl PlayerSetup {
    pub fn from_match_setup(setup: &MatchSetup, seat: usize) -> Self {
        Self {
            slot: setup.slots[seat],
            display_name: None,
            ai: setup.ai_config_for(seat),
        }
    }

    pub fn to_match_setup(players: &[PlayerSetup; 4], default_ai: AiConfig) -> MatchSetup {
        let mut slots = [PlayerSlot::Cpu; 4];
        let mut seat_ai = [None; 4];
        for (seat, player) in players.iter().enumerate() {
            slots[seat] = player.slot;
            seat_ai[seat] = player.ai;
        }
        MatchSetup {
            slots,
            default_ai,
            seat_ai,
        }
    }
}

/// Optional client-provided metadata for a recording.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecordingMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recording_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Complete match save point — tiles, history, flow, and player setup.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchRecording {
    pub format_version: u32,
    #[serde(flatten)]
    pub meta: RecordingMeta,
    pub rules_profile: RulesProfileId,
    pub rules_config: RulesConfig,
    pub seed: u64,
    pub players: [PlayerSetup; 4],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_seat: Option<usize>,
    /// Pause between CPU steps in the TUI (milliseconds); client-only preference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_step_delay_ms: Option<u64>,
    /// Per-turn thinking limit for draw/discard (ms); `0` = off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_timer_ms: Option<u64>,
    /// Reaction window limit for chi/pon/ron/pass (ms); `0` = off.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "reaction_pass_delay_ms"
    )]
    pub response_timer_ms: Option<u64>,
    pub match_status: MatchStatus,
    pub dealer: usize,
    pub round_wind: RoundWind,
    pub kyoku: u8,
    pub honba: u8,
    pub scores: [i32; 4],
    pub table_riichi_sticks: u8,
    pub hand_index: u32,
    pub match_phase: MatchPhase,
    pub hand: HandSnapshot,
    pub events: Vec<Event>,
    pub event_index: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_legal_actions: Option<Vec<Action>>,
    /// When set, CI scores the pending win and checks yaku (debug scenarios).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_yaku: Option<Vec<crate::scoring::Yaku>>,
}

impl MatchRecording {
    /// Snapshot the current match for persistence or dev scenarios.
    pub fn capture(
        game: &Match,
        setup: &MatchSetup,
        human_seat: usize,
        cpu_step_delay_ms: u64,
        turn_timer_ms: u64,
        response_timer_ms: u64,
        mut meta: RecordingMeta,
    ) -> Self {
        let events = game.events().to_vec();
        let event_index = events.len().saturating_sub(1);
        if meta.recording_id.is_none() {
            meta.recording_id = Some(format!("{event_index}-{}", game.seed()));
        }

        Self {
            format_version: FORMAT_VERSION,
            meta,
            rules_profile: game.config().profile,
            rules_config: game.config().clone(),
            seed: game.seed(),
            players: std::array::from_fn(|seat| PlayerSetup::from_match_setup(setup, seat)),
            human_seat: Some(human_seat),
            cpu_step_delay_ms: Some(cpu_step_delay_ms),
            turn_timer_ms: Some(turn_timer_ms),
            response_timer_ms: Some(response_timer_ms),
            match_status: if game.is_ended() {
                MatchStatus::Finished
            } else {
                MatchStatus::InProgress
            },
            dealer: game.dealer(),
            round_wind: game.round_wind(),
            kyoku: game.kyoku(),
            honba: game.honba(),
            scores: *game.scores(),
            table_riichi_sticks: game.table_riichi_sticks(),
            hand_index: game.hand_index(),
            match_phase: game.phase(),
            hand: game.hand().to_snapshot(),
            events,
            event_index,
            expected_legal_actions: None,
            expected_yaku: None,
        }
    }

    pub fn match_setup(&self) -> MatchSetup {
        let default_ai = self.players[0]
            .ai
            .or(self.players[1].ai)
            .or(self.players[2].ai)
            .or(self.players[3].ai)
            .unwrap_or(AiConfig::medium(self.seed));
        PlayerSetup::to_match_setup(&self.players, default_ai)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.format_version != FORMAT_VERSION {
            return Err(Error::UnsupportedFormatVersion(self.format_version));
        }
        if self.event_index >= self.events.len() && !self.events.is_empty() {
            return Err(Error::InvalidRecording {
                detail: format!(
                    "event_index {} out of range for {} events",
                    self.event_index,
                    self.events.len()
                ),
            });
        }
        if self.rules_config.profile != self.rules_profile {
            return Err(Error::InvalidRecording {
                detail: "rules_profile does not match rules_config.profile".into(),
            });
        }
        self.hand.restore(self.rules_config.clone())?;
        Ok(())
    }

    /// Restore a live match from this save point.
    pub fn restore(&self) -> Result<Match, Error> {
        self.validate()?;
        let hand = HandState::from_snapshot(self.hand.clone(), self.rules_config.clone())?;
        Ok(Match::restore_from_hand(self, hand))
    }

    /// Replay events from the start through `index` (for regression / scenario tests).
    pub fn apply_until(&self, index: usize) -> Result<Match, Error> {
        if index >= self.events.len() {
            return Err(Error::InvalidRecording {
                detail: format!("apply_until index {index} >= {}", self.events.len()),
            });
        }
        let replay = Replay {
            rules_profile: self.rules_profile,
            rules_config: self.rules_config.clone(),
            seed: self.seed,
            events: self.events.clone(),
        };
        let mut game = Match::new(self.rules_config.clone(), self.seed)?;
        apply_events(&mut game, &replay.events, Some(index))?;
        Ok(game)
    }

    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(self).map_err(|err| Error::InvalidRecording {
            detail: err.to_string(),
        })
    }

    pub fn from_json(text: &str) -> Result<Self, Error> {
        serde_json::from_str(text).map_err(|err| Error::InvalidRecording {
            detail: err.to_string(),
        })
    }

    pub fn to_writer(&self, writer: &mut impl Write) -> Result<(), Error> {
        let json = self.to_json()?;
        writer.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn from_reader(reader: &mut impl Read) -> Result<Self, Error> {
        let mut text = String::new();
        reader.read_to_string(&mut text)?;
        Self::from_json(&text)
    }
}
