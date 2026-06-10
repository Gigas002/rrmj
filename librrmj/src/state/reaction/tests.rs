use super::ReactionState;
use crate::action::Action;
use crate::tile::Tile;

#[test]
fn ron_beats_pon_when_both_claim() {
    let mut reaction = ReactionState::new(0, Tile::man(2));
    reaction.record(1, Action::Pon);
    reaction.record(2, Action::Ron);
    reaction.record(3, Action::Pass);

    assert!(reaction.winning_call().is_none());
    assert_eq!(reaction.ron_winners(1), vec![2]);
}

#[test]
fn double_ron_collects_seats_in_priority_order() {
    let mut reaction = ReactionState::new(0, Tile::pin(2));
    reaction.record(1, Action::Ron);
    reaction.record(2, Action::Pass);
    reaction.record(3, Action::Ron);

    assert_eq!(reaction.ron_winners(2), vec![1, 3]);
    assert_eq!(reaction.ron_winners(1), vec![1]);
}

#[test]
fn kakan_reaction_has_no_call_winner() {
    let mut reaction = ReactionState::new_kakan(1, Tile::sou(5), 0);
    reaction.record(0, Action::Pass);
    reaction.record(2, Action::Pass);
    reaction.record(3, Action::Pass);

    assert!(reaction.winning_call().is_none());
    assert!(reaction.ron_winners(2).is_empty());
}
