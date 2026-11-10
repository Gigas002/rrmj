use librrmj::action::Action;
use librrmj::state::HandPhase;
use librrmj::tile::Tile;

use super::{forced_action, timeout_action};
use crate::app::ActionMenu;

fn discards(tiles: &[Tile]) -> Vec<Action> {
    tiles.iter().map(|&t| Action::Discard(t)).collect()
}

#[test]
fn discard_timeout_throws_the_drawn_tile() {
    let legal = discards(&[Tile::man(1), Tile::pin(5), Tile::sou(4), Tile::sou(9)]);
    let menu = ActionMenu::from_legal(&legal);
    assert_eq!(
        timeout_action(&legal, HandPhase::Discard, &menu, Some(Tile::sou(4))),
        Some(Action::Discard(Tile::sou(4)))
    );
}

#[test]
fn discard_timeout_without_draw_throws_rightmost_tile() {
    let legal = discards(&[Tile::sou(9), Tile::man(1), Tile::pin(5)]);
    let menu = ActionMenu::from_legal(&legal);
    assert_eq!(
        timeout_action(&legal, HandPhase::Discard, &menu, None),
        Some(Action::Discard(Tile::sou(9)))
    );
}

#[test]
fn discard_timeout_skips_a_drawn_tile_that_is_not_legal() {
    let legal = discards(&[Tile::man(1), Tile::pin(5)]);
    let menu = ActionMenu::from_legal(&legal);
    assert_eq!(
        timeout_action(&legal, HandPhase::Discard, &menu, Some(Tile::sou(4))),
        Some(Action::Discard(Tile::pin(5)))
    );
}

#[test]
fn timeouts_take_an_offered_win() {
    let mut legal = discards(&[Tile::man(1)]);
    legal.push(Action::Tsumo);
    let menu = ActionMenu::from_legal(&legal);
    assert_eq!(
        timeout_action(&legal, HandPhase::Discard, &menu, Some(Tile::man(1))),
        Some(Action::Tsumo)
    );

    let legal = vec![Action::Pon, Action::Ron, Action::Pass];
    let menu = ActionMenu::from_legal(&legal);
    assert_eq!(
        timeout_action(&legal, HandPhase::Reaction, &menu, None),
        Some(Action::Ron)
    );

    let legal = vec![Action::Pon, Action::Pass];
    let menu = ActionMenu::from_legal(&legal);
    assert_eq!(
        timeout_action(&legal, HandPhase::Reaction, &menu, None),
        Some(Action::Pass)
    );
}

#[test]
fn riichi_tsumogiri_is_forced() {
    let legal = discards(&[Tile::pin(3)]);
    let menu = ActionMenu::from_legal(&legal);
    assert_eq!(
        forced_action(&legal, HandPhase::Discard, &menu),
        Some(Action::Discard(Tile::pin(3)))
    );
}

#[test]
fn a_real_choice_is_not_forced() {
    let mut legal = discards(&[Tile::pin(3)]);
    legal.push(Action::Tsumo);
    let menu = ActionMenu::from_legal(&legal);
    assert_eq!(forced_action(&legal, HandPhase::Discard, &menu), None);

    let legal = discards(&[Tile::pin(3), Tile::pin(4)]);
    let menu = ActionMenu::from_legal(&legal);
    assert_eq!(forced_action(&legal, HandPhase::Discard, &menu), None);

    let legal = vec![Action::Ron, Action::Pass];
    let menu = ActionMenu::from_legal(&legal);
    assert_eq!(forced_action(&legal, HandPhase::Reaction, &menu), None);

    let legal = vec![Action::Pass];
    let menu = ActionMenu::from_legal(&legal);
    assert_eq!(
        forced_action(&legal, HandPhase::Reaction, &menu),
        Some(Action::Pass)
    );
}
