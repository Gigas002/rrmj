//! Table-driven regression over `examples/scenarios/*.json`.

#![cfg(feature = "serde")]

use std::fs;
use std::path::PathBuf;

use librrmj::action::Action;
use librrmj::replay::MatchRecording;
use librrmj::rules::{RulesConfig, RulesProfileId, RulesRegistry, WinContext, WinTimingFlags};
use librrmj::scoring::{WinType, Yaku};

fn scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("examples")
        .join("scenarios")
}

fn verify_expected_yaku(recording: &MatchRecording, game: &librrmj::game::Game) {
    let Some(expected) = &recording.expected_yaku else {
        return;
    };
    let seat = recording.human_seat.expect("human seat for expected_yaku");
    let hand = game.hand();
    let config = RulesConfig::standard();
    let profile = RulesRegistry::get(RulesProfileId::Standard).unwrap();

    let (win_type, win_tile) = if hand.legal_actions_for(seat).contains(&Action::Tsumo) {
        let win_tile = recording
            .hand
            .last_draw
            .or_else(|| hand.last_drawn_tile())
            .expect("tsumo win tile");
        (WinType::Tsumo, win_tile)
    } else if hand.legal_actions_for(seat).contains(&Action::Ron) {
        let reaction = recording
            .hand
            .reaction
            .as_ref()
            .expect("ron reaction in snapshot");
        (
            WinType::Ron {
                from: reaction.discarder,
            },
            reaction.tile,
        )
    } else {
        panic!(
            "scenario {:?}: expected_yaku but seat {seat} cannot tsumo/ron",
            recording.meta.title
        );
    };

    let is_chankan = recording
        .hand
        .reaction
        .as_ref()
        .is_some_and(|r| r.kind == librrmj::state::ReactionKind::Kakan);
    let ctx = WinContext::new(
        hand,
        seat,
        win_type,
        win_tile,
        WinTimingFlags { is_chankan },
    );
    let result = profile.score_win(&ctx, &config);
    for yaku in expected {
        assert!(
            result.yaku.contains(yaku),
            "{}: expected {:?} in scored {:?} (han={})",
            recording.meta.title.as_deref().unwrap_or("scenario"),
            yaku,
            result.yaku,
            result.han
        );
    }
}

#[test]
fn scenario_fixtures_restore() {
    let dir = scenarios_dir();
    if !dir.exists() {
        return;
    }

    let mut paths: Vec<_> = fs::read_dir(&dir)
        .expect("read scenarios dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();

    assert!(!paths.is_empty(), "expected at least one scenario fixture");

    for path in paths {
        let text = fs::read_to_string(&path).expect("read scenario");
        let recording = MatchRecording::from_json(&text).expect("parse scenario");
        recording.validate().expect("validate scenario");
        let game = recording.restore().expect("restore scenario");

        if let Some(expected) = &recording.expected_legal_actions {
            let seat = game.pending_seat().expect("pending seat for scenario");
            let legal = game.hand().legal_actions_for(seat);
            for action in expected {
                assert!(
                    legal.contains(action),
                    "{}: expected {:?} in legal {:?}",
                    path.display(),
                    action,
                    legal
                );
            }
        }

        verify_expected_yaku(&recording, &game);
    }
}

#[test]
fn win_scenario_fixtures_cover_all_v0_yaku() {
    let dir = scenarios_dir();
    if !dir.exists() {
        return;
    }

    let mut yaku_seen = std::collections::HashSet::new();
    for entry in fs::read_dir(&dir).expect("read dir").flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let text = fs::read_to_string(&path).expect("read");
            let recording = MatchRecording::from_json(&text).expect("parse");
            if let Some(expected) = recording.expected_yaku {
                yaku_seen.extend(expected);
            }
        }
    }

    for yaku in [
        Yaku::Riichi,
        Yaku::MenzenTsumo,
        Yaku::Tanyao,
        Yaku::Pinfu,
        Yaku::Yakuhai,
        Yaku::Chiitoitsu,
    ] {
        assert!(
            yaku_seen.contains(&yaku),
            "no scenario fixture scores {:?}",
            yaku
        );
    }
}
