use std::io::{Read, Write};

use crate::action::Action;
use crate::agent::PlayerSlot;
use crate::ai::{AiConfig, GameSetup};
use crate::error::Error;
use crate::event::Event;
use crate::game::{Game, GamePhase, RoundWind};
use crate::rules::{RulesConfig, RulesProfileId};
use crate::state::HandState;

use super::Replay;
use super::apply::apply_events;
use super::snapshot::HandSnapshot;

pub const FORMAT_VERSION: u32 = 3;

/// Whether a recording can be resumed or is a completed game session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
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
    pub fn from_game_setup(setup: &GameSetup, seat: usize) -> Self {
        Self {
            slot: setup.slots[seat],
            display_name: None,
            ai: setup.ai_config_for(seat),
        }
    }

    pub fn to_game_setup(players: &[PlayerSetup; 4], default_ai: AiConfig) -> GameSetup {
        let mut slots = [PlayerSlot::Cpu; 4];
        let mut seat_ai = [None; 4];
        for (seat, player) in players.iter().enumerate() {
            slots[seat] = player.slot;
            seat_ai[seat] = player.ai;
        }
        GameSetup {
            slots,
            default_ai,
            seat_ai,
        }
    }
}

/// CI-only assertions for debug scenarios (`examples/scenarios/*.json`).
///
/// Ignored by the TUI and player-authored scenarios. See `docs/REPLAY.md`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecordingAssertions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_legal_actions: Option<Vec<Action>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_yaku: Option<Vec<crate::scoring::Yaku>>,
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

/// Complete game save point — tiles, history, flow, and player setup.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GameRecording {
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
    #[serde(rename = "game_status", alias = "match_status")]
    pub game_status: GameStatus,
    pub dealer: usize,
    pub round_wind: RoundWind,
    pub kyoku: u8,
    pub honba: u8,
    pub scores: [i32; 4],
    pub table_riichi_sticks: u8,
    pub hand_index: u32,
    #[serde(rename = "game_phase", alias = "match_phase")]
    pub game_phase: GamePhase,
    pub hand: HandSnapshot,
    pub events: Vec<Event>,
    pub event_index: usize,
    /// CI-only checks for debug scenarios; absent in replays, saves, and player scenarios.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assertions: Option<RecordingAssertions>,
}

impl GameRecording {
    /// Snapshot the current game for persistence or dev scenarios.
    pub fn capture(
        game: &Game,
        setup: &GameSetup,
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
            players: std::array::from_fn(|seat| PlayerSetup::from_game_setup(setup, seat)),
            human_seat: Some(human_seat),
            cpu_step_delay_ms: Some(cpu_step_delay_ms),
            turn_timer_ms: Some(turn_timer_ms),
            response_timer_ms: Some(response_timer_ms),
            game_status: if game.is_ended() {
                GameStatus::Finished
            } else {
                GameStatus::InProgress
            },
            dealer: game.dealer(),
            round_wind: game.round_wind(),
            kyoku: game.kyoku(),
            honba: game.honba(),
            scores: *game.scores(),
            table_riichi_sticks: game.table_riichi_sticks(),
            hand_index: game.hand_index(),
            game_phase: game.phase(),
            hand: game.hand().to_snapshot(),
            events,
            event_index,
            assertions: None,
        }
    }

    pub fn expected_legal_actions(&self) -> Option<&Vec<Action>> {
        self.assertions
            .as_ref()
            .and_then(|assertions| assertions.expected_legal_actions.as_ref())
    }

    pub fn expected_yaku(&self) -> Option<&Vec<crate::scoring::Yaku>> {
        self.assertions
            .as_ref()
            .and_then(|assertions| assertions.expected_yaku.as_ref())
    }

    pub fn game_setup(&self) -> GameSetup {
        let default_ai = self.players[0]
            .ai
            .or(self.players[1].ai)
            .or(self.players[2].ai)
            .or(self.players[3].ai)
            .unwrap_or(AiConfig::medium(self.seed));
        PlayerSetup::to_game_setup(&self.players, default_ai)
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

    /// Restore a live game from this save point.
    pub fn restore(&self) -> Result<Game, Error> {
        self.validate()?;
        let hand = HandState::from_snapshot(self.hand.clone(), self.rules_config.clone())?;
        Ok(Game::restore_from_hand(self, hand))
    }

    /// Replay events from the start through `index` (for regression / scenario tests).
    pub fn apply_until(&self, index: usize) -> Result<Game, Error> {
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
        let mut game = Game::new(self.rules_config.clone(), self.seed)?;
        apply_events(&mut game, &replay.events, Some(index))?;
        Ok(game)
    }

    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(self).map_err(|err| Error::InvalidRecording {
            detail: err.to_string(),
        })
    }

    pub fn from_json(text: &str) -> Result<Self, Error> {
        let mut value: serde_json::Value =
            serde_json::from_str(text).map_err(|err| Error::InvalidRecording {
                detail: err.to_string(),
            })?;
        Self::migrate_wire_value(&mut value);
        serde_json::from_value(value).map_err(|err| Error::InvalidRecording {
            detail: err.to_string(),
        })
    }

    /// Normalize legacy top-level assertion fields and bump `format_version` on read.
    fn migrate_wire_value(value: &mut serde_json::Value) {
        let Some(object) = value.as_object_mut() else {
            return;
        };

        if object.get("assertions").is_none() {
            let legal = object.remove("expected_legal_actions");
            let yaku = object.remove("expected_yaku");
            if legal.is_some() || yaku.is_some() {
                let mut assertions = serde_json::Map::new();
                if let Some(legal) = legal {
                    assertions.insert("expected_legal_actions".into(), legal);
                }
                if let Some(yaku) = yaku {
                    assertions.insert("expected_yaku".into(), yaku);
                }
                object.insert("assertions".into(), serde_json::Value::Object(assertions));
            }
        }

        if object.get("game_status").is_none()
            && let Some(status) = object.remove("match_status")
        {
            object.insert("game_status".into(), status);
        }
        if object.get("game_phase").is_none()
            && let Some(phase) = object.remove("match_phase")
        {
            object.insert("game_phase".into(), phase);
        }

        if let Some(rules) = object
            .get_mut("rules_config")
            .and_then(|v| v.as_object_mut())
            && rules.get("game_length").is_none()
            && let Some(length) = rules.remove("match_length")
        {
            rules.insert("game_length".into(), length);
        }

        if let Some(version) = object.get("format_version").and_then(|v| v.as_u64())
            && version < FORMAT_VERSION as u64
        {
            object.insert("format_version".into(), serde_json::json!(FORMAT_VERSION));
        }
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
