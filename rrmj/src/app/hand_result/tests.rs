use librrmj::action::Action;
use librrmj::ai::{AiConfig, CpuAgent, Difficulty};
use librrmj::event::Event;
use librrmj::game::Game;
use librrmj::replay::GameRecording;
use librrmj::rules::RulesConfig;

use super::summary_from_events;

/// Plays a committed scenario to the end of its hand: every seat rons when it
/// can, otherwise passes or discards its first legal tile.
fn play_scenario_hand(name: &str) -> Vec<Event> {
    let path = format!(
        "{}/../examples/scenarios/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).expect("read scenario");
    let recording = GameRecording::from_json(&text).expect("parse scenario");
    let mut game = recording.restore().expect("restore scenario");
    let start = game.hand_index();
    let mut events = Vec::new();
    while game.hand_index() == start && !game.is_ended() {
        let seat = game.pending_seat().expect("pending seat");
        let legal = game.hand().legal_actions_for(seat);
        let action = [Action::Ron, Action::Draw, Action::Pass]
            .into_iter()
            .find(|action| legal.contains(action))
            .or_else(|| {
                legal
                    .iter()
                    .copied()
                    .find(|a| matches!(a, Action::Discard(_)))
            })
            .expect("an action to take");
        events.extend(game.apply_action(seat, action).expect("legal action"));
    }
    events
}

fn assert_multi_ron(scenario: &str, title: &str, count: usize) {
    let events = play_scenario_hand(scenario);
    let winners: Vec<usize> = events
        .iter()
        .filter_map(|e| match e {
            Event::Won { seat, .. } => Some(*seat),
            _ => None,
        })
        .collect();
    assert_eq!(winners.len(), count, "{scenario}: winners {winners:?}");

    let summary = summary_from_events(&events).expect("summary");
    assert_eq!(summary.title, title);
    for seat in winners {
        let name = crate::app::NewGameSetup::seat_name(seat);
        assert!(
            summary.lines.iter().any(|line| line.starts_with(name)),
            "{name} missing from {:?}",
            summary.lines
        );
    }
}

#[test]
fn double_ron_lists_both_winners() {
    assert_multi_ron("double_ron", "Double ron", 2);
}

#[test]
fn triple_ron_lists_all_winners() {
    assert_multi_ron("triple_ron", "Triple ron", 3);
}

/// The events of the first hand of a seeded all-CPU match that ends in an
/// exhaustive draw with both tenpai and noten seats.
fn mixed_exhaustive_draw() -> Vec<Event> {
    let config = RulesConfig::standard();
    for seed in 0..50 {
        let mut game = Game::new(config.clone(), seed).expect("new game");
        let mut agents: [CpuAgent; 4] = std::array::from_fn(|seat| {
            CpuAgent::new(AiConfig::new(Difficulty::Easy, seed), seat, seed)
        });
        let mut hand = Vec::new();
        while let Some(step) = game.step(&mut agents).expect("step") {
            hand.extend(step.events);
            if let Some(Event::ExhaustiveDraw { tenpai, .. }) = hand
                .iter()
                .find(|e| matches!(e, Event::ExhaustiveDraw { .. }))
                && tenpai.iter().any(|t| *t)
                && !tenpai.iter().all(|t| *t)
            {
                return hand;
            }
            if hand.iter().any(|e| matches!(e, Event::HandStarted { .. })) {
                hand.clear();
            }
        }
    }
    panic!("no mixed exhaustive draw in 50 seeds");
}

#[test]
fn exhaustive_draw_shows_tenpai_seats_and_deltas() {
    let events = mixed_exhaustive_draw();
    let Some(Event::ExhaustiveDraw { deltas, tenpai }) = events
        .iter()
        .find(|e| matches!(e, Event::ExhaustiveDraw { .. }))
    else {
        panic!("scenario should end in an exhaustive draw: {events:?}");
    };
    assert!(tenpai.iter().any(|t| *t) && !tenpai.iter().all(|t| *t));

    let summary = summary_from_events(&events).expect("summary");
    let text = summary.lines.join("\n");
    for seat in 0..4 {
        let name = crate::app::NewGameSetup::seat_name(seat);
        let row = if tenpai[seat] { "Tenpai:" } else { "Noten:" };
        let line = summary
            .lines
            .iter()
            .find(|line| line.starts_with(row))
            .expect("tenpai/noten row");
        assert!(line.contains(name), "{name} not in {line:?}");
        if deltas[seat] != 0 {
            assert!(
                text.contains(&format!("{name}: {:+}", deltas[seat])),
                "{text}"
            );
        }
    }
}
