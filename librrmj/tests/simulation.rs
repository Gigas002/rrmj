//! Whole-match simulation: plays seeded matches with CPU and random seats and
//! checks engine invariants after every action (docs/AUDIT.md §2.5).
//!
//! `cargo test -p librrmj --features ai --test simulation` runs a short set.
//! The long run is ignored by default; size it with `RRMJ_SIM_GAMES`:
//!
//! ```sh
//! RRMJ_SIM_GAMES=2000 cargo test --release -p librrmj --features ai \
//!     --test simulation -- --ignored --nocapture
//! ```
#![cfg(feature = "ai")]

use std::collections::BTreeMap;

use librrmj::ai::CpuAgent;
use librrmj::game::GameLength;
use librrmj::rules::RulesConfig;
use librrmj::scoring::{ScoringResult, WinType};
use librrmj::{Action, Agent, AiConfig, Difficulty, Event, Game, PlayerView};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

/// Hard cap on actions in one match; a hanchan takes a few thousand.
const MAX_ACTIONS: usize = 50_000;

enum Seat {
    Cpu(Box<CpuAgent>),
    /// Uniform choice among legal actions; reaches calls and kans the CPUs avoid.
    Random(Box<StdRng>),
}

impl Agent for Seat {
    fn decide(&mut self, view: &PlayerView, legal: &[Action]) -> Action {
        match self {
            Self::Cpu(agent) => agent.decide(view, legal),
            Self::Random(rng) => legal[rng.random_range(0..legal.len())],
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Mix {
    Cpu(Difficulty),
    Random,
}

fn seats(mix: [Mix; 4], seed: u64) -> [Seat; 4] {
    std::array::from_fn(|seat| match mix[seat] {
        Mix::Cpu(difficulty) => Seat::Cpu(Box::new(CpuAgent::new(
            AiConfig::new(difficulty, seed),
            seat,
            seed,
        ))),
        Mix::Random => Seat::Random(Box::new(StdRng::seed_from_u64(
            seed ^ ((seat as u64 + 1) * 0x51),
        ))),
    })
}

#[derive(Default, Debug)]
#[allow(dead_code)] // fields are read through the Debug dump of the long run
struct Stats {
    games: u64,
    hands: u64,
    actions: u64,
    rons: u64,
    tsumos: u64,
    exhaustive: u64,
    abortive: BTreeMap<String, u64>,
    yaku: BTreeMap<String, u64>,
    calls: BTreeMap<String, u64>,
    kakan: u64,
    riichi: u64,
    max_han: u8,
}

/// Plays one match and checks invariants after every action. Returns the event log.
fn play(config: &RulesConfig, mix: [Mix; 4], seed: u64, stats: &mut Stats) -> Vec<Event> {
    let mut game = Game::new(config.clone(), seed).expect("new game");
    let mut agents = seats(mix, seed);
    let total = config.starting_points * 4;
    let ctx = |game: &Game| format!("seed {seed} mix {mix:?} hand {}", game.hand_index());

    let mut actions = 0;
    while !game.is_ended() {
        actions += 1;
        assert!(actions < MAX_ACTIONS, "{}: match did not end", ctx(&game));

        let seat = game
            .pending_seat()
            .unwrap_or_else(|| panic!("{}: no pending seat in a live match", ctx(&game)));
        let legal = game.pending_legal_actions();
        assert!(
            !legal.is_empty(),
            "{}: seat {seat} must act but has no legal action (phase {:?})",
            ctx(&game),
            game.hand().phase()
        );

        let view = PlayerView::from_game(&game, seat);
        let action = agents[seat].decide(&view, &legal);
        assert!(
            legal.contains(&action),
            "{}: seat {seat} chose {action:?}, not in {legal:?}",
            ctx(&game)
        );

        let events = game.apply_action(seat, action).unwrap_or_else(|err| {
            panic!("{}: {action:?} by seat {seat} failed: {err}", ctx(&game))
        });

        // Also after the last action: the final hand is what a finished
        // recording stores, and replays validate it.
        game.hand()
            .validate_tile_conservation()
            .unwrap_or_else(|err| panic!("{}: after {action:?}: {err}", ctx(&game)));
        if !game.is_ended() {
            let hand = game.hand();
            let on_table: i32 =
                hand.scores().iter().sum::<i32>() + i32::from(hand.table_riichi_sticks()) * 1000;
            assert_eq!(
                on_table,
                total,
                "{}: points not conserved after {action:?}: scores {:?}, sticks {}",
                ctx(&game),
                hand.scores(),
                hand.table_riichi_sticks()
            );
        }

        for event in &events {
            check_event(event, &game, stats, &ctx);
        }
    }
    stats.games += 1;
    stats.actions += actions as u64;
    game.events().to_vec()
}

/// Han must be the yaku han (open −1 where it applies) plus all dora.
fn recomputed_han(scoring: &ScoringResult) -> u8 {
    let yaku: u8 = scoring
        .yaku
        .iter()
        .map(|y| {
            if scoring.open && y.open_han_penalty() {
                y.closed_han() - 1
            } else {
                y.closed_han()
            }
        })
        .sum();
    yaku + scoring.dora + scoring.ura_dora + scoring.aka_dora
}

fn check_event(event: &Event, game: &Game, stats: &mut Stats, ctx: &dyn Fn(&Game) -> String) {
    match event {
        Event::Won { seat, scoring } => {
            assert_eq!(scoring.winner, *seat, "{}: winner mismatch", ctx(game));
            assert!(
                !scoring.yaku.is_empty(),
                "{}: seat {seat} won with no yaku: {scoring:?}",
                ctx(game)
            );
            assert_eq!(
                scoring.han,
                recomputed_han(scoring),
                "{}: han doesn't add up: {scoring:?}",
                ctx(game)
            );
            assert!(
                scoring.yaku.iter().any(|y| y.is_yakuman())
                    || scoring.fu == 25
                    || scoring.fu % 10 == 0,
                "{}: fu not rounded: {scoring:?}",
                ctx(game)
            );
            assert!(
                scoring.deltas[*seat] > 0,
                "{}: winner didn't gain: {scoring:?}",
                ctx(game)
            );
            assert!(
                scoring.deltas.iter().sum::<i32>() >= 0,
                "{}: win deltas sum below zero: {scoring:?}",
                ctx(game)
            );
            match scoring.win_type {
                WinType::Ron { from } => {
                    assert_ne!(from, *seat, "{}: ron on own discard", ctx(game));
                    stats.rons += 1;
                }
                WinType::Tsumo => stats.tsumos += 1,
            }
            for yaku in &scoring.yaku {
                *stats.yaku.entry(format!("{yaku:?}")).or_default() += 1;
            }
            stats.max_han = stats.max_han.max(scoring.han);
        }
        Event::ExhaustiveDraw { deltas, .. } => {
            assert_eq!(
                deltas.iter().sum::<i32>(),
                0,
                "{}: noten payments",
                ctx(game)
            );
            stats.exhaustive += 1;
        }
        Event::AbortiveDraw { kind } => {
            *stats.abortive.entry(format!("{kind:?}")).or_default() += 1;
        }
        Event::Called { meld, .. } => {
            *stats.calls.entry(format!("{meld:?}")).or_default() += 1;
        }
        Event::KakanDeclared { .. } => stats.kakan += 1,
        Event::RiichiDeclared { .. } => stats.riichi += 1,
        Event::HandStarted { .. } => stats.hands += 1,
        Event::GameEnded { scores } => {
            // Leftover riichi sticks go to first place, so nothing is lost.
            assert_eq!(
                scores.iter().sum::<i32>(),
                game.config().starting_points * 4,
                "{}: final scores {scores:?} don't add up",
                ctx(game)
            );
        }
        _ => {}
    }
}

const MIXES: [[Mix; 4]; 6] = {
    use Difficulty::*;
    use Mix::*;
    [
        [Cpu(Easy); 4],
        [Cpu(Medium); 4],
        [Cpu(Hard); 4],
        [Cpu(Hard), Cpu(Medium), Cpu(Easy), Random],
        [Random; 4],
        [Random, Cpu(Medium), Random, Cpu(Medium)],
    ]
};

fn configs() -> Vec<RulesConfig> {
    let standard = RulesConfig::standard();
    let mut east = standard.clone();
    east.game_length = GameLength::EastOnly;
    let mut variant = standard.clone();
    variant.aka_dora = false;
    variant.kiriage = true;
    variant.triple_ron = true;
    let mut no_aborts = standard.clone();
    no_aborts.abortive_nine_terminals = false;
    no_aborts.abortive_four_winds = false;
    no_aborts.abortive_four_kongs = false;
    no_aborts.abortive_four_riichis = false;
    no_aborts.double_ron = false;
    vec![standard, east, variant, no_aborts]
}

#[test]
fn short_simulation_keeps_invariants() {
    // Easy and random seats only: Hard is slow in debug builds.
    let mut stats = Stats::default();
    for seed in 0..6 {
        let mix = [
            [Mix::Cpu(Difficulty::Easy); 4],
            [Mix::Random; 4],
            [
                Mix::Random,
                Mix::Cpu(Difficulty::Easy),
                Mix::Random,
                Mix::Cpu(Difficulty::Easy),
            ],
        ][seed as usize % 3];
        play(&configs()[seed as usize % 4], mix, seed, &mut stats);
    }
    assert_eq!(stats.games, 6);
}

#[test]
fn same_seed_same_match() {
    let config = RulesConfig::standard();
    let mix = [
        Mix::Cpu(Difficulty::Easy),
        Mix::Random,
        Mix::Cpu(Difficulty::Easy),
        Mix::Random,
    ];
    let a = play(&config, mix, 77, &mut Stats::default());
    let b = play(&config, mix, 77, &mut Stats::default());
    assert_eq!(a, b);
}

#[test]
#[ignore = "long; size with RRMJ_SIM_GAMES, run with --release"]
fn long_simulation_keeps_invariants() {
    let games: u64 = std::env::var("RRMJ_SIM_GAMES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(240);
    let mut stats = Stats::default();
    let configs = configs();
    for n in 0..games {
        let mix = MIXES[(n as usize) % MIXES.len()];
        let config = &configs[(n as usize / MIXES.len()) % configs.len()];
        play(config, mix, 10_000 + n, &mut stats);
    }
    println!("{stats:#?}");
}
