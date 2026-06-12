#[path = "easy/tests.rs"]
mod easy;
#[path = "hard/tests.rs"]
mod hard;
#[path = "medium/tests.rs"]
mod medium;

use rand::SeedableRng;
use rand::rngs::StdRng;

use super::EasyAgent;
use super::hard::HardAgent;
use super::medium::MediumAgent;
use crate::action::Action;
use crate::agent::{Agent, PlayerView};
use crate::ai::common::hand_from_view;
use crate::ai::efficiency::weighted_waiting_count;
use crate::ai::shanten::{hand_without_concealed_tile, waiting_count};
use crate::ai::{AiConfig, MatchSetup};
use crate::game::Game;
use crate::rules::RulesConfig;
use crate::state::HandPhase;

fn discard_tile(action: Action) -> Option<crate::tile::Tile> {
    match action {
        Action::Discard(tile) | Action::Riichi { discard: tile } => Some(tile),
        _ => None,
    }
}

fn discard_quality(view: &PlayerView, tile: crate::tile::Tile) -> Option<(usize, u32)> {
    let hand = hand_from_view(view)?;
    let after = hand_without_concealed_tile(&hand, tile)?;
    Some((waiting_count(&after), weighted_waiting_count(&after, view)))
}

#[test]
fn easy_always_chooses_legal_action() {
    let seed = 1234;
    let mut agents = MatchSetup::all_easy(seed).build_agents(seed);
    let mut game = Game::new(RulesConfig::standard(), seed).unwrap();

    for _ in 0..500 {
        if game.is_ended() {
            break;
        }
        let seat = match game.pending_seat() {
            Some(seat) => seat,
            None => break,
        };
        let legal = game.pending_legal_actions();
        if legal.is_empty() {
            break;
        }
        let result = game.step(&mut agents).unwrap().expect("step taken");
        assert!(
            legal.contains(&result.action),
            "easy agent returned illegal action {:?} from {legal:?}",
            result.action
        );
        assert_eq!(result.seat, seat);
    }
}

#[test]
fn ai_vs_ai_smoke_completes_without_panic() {
    for seed in [42u64, 99] {
        let setup = if seed == 42 {
            MatchSetup::all_easy(seed)
        } else {
            MatchSetup::all_medium(seed)
        };
        let mut agents = setup.build_agents(seed);
        let mut game = Game::new(RulesConfig::standard(), seed).unwrap();
        let initial_hand_index = game.events().len();

        for _ in 0..3000 {
            if game.is_ended() {
                break;
            }
            if game.step(&mut agents).unwrap().is_none() {
                break;
            }
        }

        assert!(
            game.events().len() > initial_hand_index,
            "seed {seed} produced no events"
        );
    }
}

#[test]
fn easy_agent_is_deterministic_with_seed() {
    let legal = [
        Action::Pass,
        Action::Discard(crate::tile::Tile::man(1)),
        Action::Discard(crate::tile::Tile::man(2)),
    ];
    let mut a = EasyAgent::new(5);
    let mut b = EasyAgent::new(5);
    let view = PlayerView::from_game(&Game::new(RulesConfig::standard(), 1).unwrap(), 0);
    assert_eq!(a.decide(&view, &legal), b.decide(&view, &legal));
}

#[test]
fn match_setup_mixed_seats() {
    let mut setup = MatchSetup::all_easy(1);
    setup.slots[0] = crate::agent::PlayerSlot::Human;
    setup.seat_ai[2] = Some(AiConfig::medium(2));
    let agents = setup.build_agents(1);
    assert!(matches!(agents[0], super::SeatAgent::HumanPending));
    assert!(matches!(agents[1], super::SeatAgent::Cpu(_)));
    assert!(matches!(agents[2], super::SeatAgent::Cpu(_)));
}

#[test]
fn tenpai_hand_has_positive_waiting_count() {
    use crate::ai::shanten;
    use crate::hand::{Concealed, Hand};
    use crate::tile::Tile;

    let concealed = vec![
        Tile::man(2),
        Tile::man(3),
        Tile::man(4),
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(5),
        Tile::sou(6),
        Tile::sou(7),
        Tile::sou(8),
        Tile::sou(9),
        Tile::sou(9),
        Tile::sou(9),
        Tile::pin(2),
    ];
    let hand = Hand::new(Concealed::from_tiles(concealed), vec![]).unwrap();
    assert!(shanten::waiting_count(&hand) > 0);
}

#[test]
fn easy_wins_when_only_tsumo_legal() {
    let mut rng = StdRng::seed_from_u64(0);
    let legal = [Action::Tsumo];
    assert_eq!(EasyAgent::decide_with_rng(&mut rng, &legal), Action::Tsumo);
}

/// Seeded sim: compare discard samples from live deals; hard should match or beat medium.
#[test]
fn hard_beats_medium_in_short_benchmark() {
    const GAMES: u64 = 24;
    const SAMPLES_PER_GAME: u32 = 40;
    let mut hard_better = 0u32;
    let mut comparisons = 0u32;

    for seed in 0..GAMES {
        let mut game = Game::new(RulesConfig::standard(), seed.wrapping_add(90_000)).unwrap();
        let mut agents = MatchSetup::all_medium(seed).build_agents(seed);
        let mut medium = MediumAgent::new(seed);
        let mut hard = HardAgent::new(seed);

        let mut samples = 0u32;
        while !game.is_ended() && samples < SAMPLES_PER_GAME {
            let seat = match game.pending_seat() {
                Some(seat) => seat,
                None => break,
            };

            if game.hand().phase() != HandPhase::Discard {
                game.step(&mut agents).unwrap();
                continue;
            }

            let view = PlayerView::from_game(&game, seat);
            let legal = game.pending_legal_actions();
            if !legal.iter().any(|a| matches!(a, Action::Discard(_))) {
                game.step(&mut agents).unwrap();
                continue;
            }

            let medium_action = medium.decide(&view, &legal);
            let hard_action = hard.decide(&view, &legal);
            if let (Some(m_tile), Some(h_tile)) =
                (discard_tile(medium_action), discard_tile(hard_action))
                && let (Some(m_score), Some(h_score)) = (
                    discard_quality(&view, m_tile),
                    discard_quality(&view, h_tile),
                )
            {
                if h_score >= m_score {
                    hard_better += 1;
                }
                comparisons += 1;
                samples += 1;
            }

            game.step(&mut agents).unwrap();
        }
    }

    assert!(comparisons > 100, "not enough samples: {comparisons}");
    assert!(
        hard_better * 2 > comparisons,
        "hard matched or beat medium on {hard_better}/{comparisons} sampled discards"
    );
}
