use rand::SeedableRng;
use rand::rngs::StdRng;

use super::EasyAgent;
use crate::action::Action;
use crate::agent::{Agent, PlayerView};
use crate::ai::{AiConfig, MatchSetup};
use crate::game::Match;
use crate::rules::RulesConfig;

#[test]
fn easy_always_chooses_legal_action() {
    let seed = 1234;
    let mut agents = MatchSetup::all_easy(seed).build_agents(seed);
    let mut game = Match::new(RulesConfig::standard(), seed).unwrap();

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
        let mut game = Match::new(RulesConfig::standard(), seed).unwrap();
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
    let view = PlayerView::from_match(&Match::new(RulesConfig::standard(), 1).unwrap(), 0);
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
