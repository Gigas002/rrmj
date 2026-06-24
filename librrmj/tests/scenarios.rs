//! Table-driven regression over `examples/scenarios/*.json`.

#![cfg(feature = "serde")]

use std::fs;
use std::path::PathBuf;

use librrmj::action::Action;
use librrmj::replay::GameRecording;
use librrmj::rules::{RulesConfig, RulesProfileId, RulesRegistry, WinContext, WinTimingFlags};
use librrmj::scoring::{WinType, Yaku};

fn scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("examples")
        .join("scenarios")
}

fn debug_catalog_ids() -> Vec<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("docs")
        .join("DEBUG_SCENARIOS.md");
    let text = fs::read_to_string(&path).expect("read DEBUG_SCENARIOS.md");
    let section = text
        .split("## Scenarios")
        .nth(1)
        .expect("scenarios section")
        .split("## Winning-hand")
        .next()
        .expect("winning-hand boundary");
    section
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix("| `")?;
            let id = rest.split('`').next()?;
            Some(id.to_string())
        })
        .collect()
}

fn verify_expected_yaku(recording: &GameRecording, game: &librrmj::game::Game) {
    let Some(expected) = recording.expected_yaku() else {
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
fn committed_scenarios_restore() {
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
        let recording = GameRecording::from_json(&text).expect("parse scenario");
        recording.validate().expect("validate scenario");
        let game = recording.restore().expect("restore scenario");

        if let Some(expected) = recording.expected_legal_actions() {
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
fn committed_scenarios_match_debug_catalog() {
    let dir = scenarios_dir();
    if !dir.exists() {
        return;
    }

    let catalog = debug_catalog_ids();
    assert_eq!(
        catalog.len(),
        50,
        "DEBUG_SCENARIOS.md scenario table should list 50 ids"
    );

    let mut on_disk = std::collections::HashSet::new();
    for entry in fs::read_dir(&dir).expect("read scenarios dir").flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            on_disk.insert(stem.to_string());
        }
    }

    for id in &catalog {
        assert!(
            on_disk.contains(id),
            "missing examples/scenarios/{id}.json for catalog row"
        );
        let path = dir.join(format!("{id}.json"));
        let text = fs::read_to_string(&path).expect("read scenario");
        let recording = GameRecording::from_json(&text).expect("parse scenario");
        recording.validate().expect("validate scenario");
        recording.restore().expect("restore scenario");
    }

    for id in &on_disk {
        assert!(
            catalog.iter().any(|c| c == id),
            "examples/scenarios/{id}.json is not listed in DEBUG_SCENARIOS.md"
        );
    }
}

#[test]
fn committed_scenarios_fixture_count() {
    let dir = scenarios_dir();
    if !dir.exists() {
        return;
    }

    let count = fs::read_dir(&dir)
        .expect("read scenarios dir")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        .count();
    assert_eq!(
        count, 50,
        "update docs/DEBUG_SCENARIOS.md when adding or removing fixtures"
    );
}

#[test]
fn committed_scenarios_use_assertions_object() {
    let dir = scenarios_dir();
    if !dir.exists() {
        return;
    }

    for entry in fs::read_dir(&dir).expect("read dir").flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read scenario");
        let value: serde_json::Value = serde_json::from_str(&text).expect("parse scenario json");
        let Some(object) = value.as_object() else {
            continue;
        };
        assert!(
            !object.contains_key("expected_legal_actions"),
            "{}: move expected_legal_actions under assertions",
            path.display()
        );
        assert!(
            !object.contains_key("expected_yaku"),
            "{}: move expected_yaku under assertions",
            path.display()
        );
    }
}

#[test]
fn committed_scenarios_cover_baseline_yaku() {
    let dir = scenarios_dir();
    if !dir.exists() {
        return;
    }

    let mut yaku_seen = std::collections::HashSet::new();
    for entry in fs::read_dir(&dir).expect("read dir").flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let text = fs::read_to_string(&path).expect("read");
            let recording = GameRecording::from_json(&text).expect("parse");
            if let Some(expected) = recording.expected_yaku() {
                yaku_seen.extend(expected.iter().copied());
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
