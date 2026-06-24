use rand::Rng;
use rand::seq::IndexedRandom;

use crate::action::{Action, KanIntent};
use crate::agent::PlayerView;
use crate::hand::Hand;

use super::common::{
    hand_from_view, prefer_win, simulate_added_kan, simulate_call, simulate_closed_kan,
};
use super::defense::{any_opponent_riichi, tile_safety};
use super::efficiency::weighted_waiting_count;
use super::shanten::{
    HandStrength, evaluate_hand, hand_without_concealed_tile, shanten_to_tenpai, waiting_count,
};

/// How eagerly the agent calls tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallPolicy {
    /// Call only when shanten drops or ukeire strictly rises.
    Conservative,
    /// Also call when shanten is unchanged but ukeire is at least as good.
    Aggressive,
}

#[derive(Debug, Clone, Copy)]
pub struct DiscardPolicy {
    pub call_policy: CallPolicy,
    pub min_ukeire_for_riichi: usize,
    pub min_points_for_riichi: i32,
    pub defend_from_shanten: i8,
    pub prefer_riichi_at_tenpai: bool,
}

impl DiscardPolicy {
    pub const MEDIUM: Self = Self {
        call_policy: CallPolicy::Conservative,
        min_ukeire_for_riichi: 1,
        min_points_for_riichi: 1_000,
        defend_from_shanten: 2,
        prefer_riichi_at_tenpai: false,
    };

    pub const HARD: Self = Self {
        call_policy: CallPolicy::Aggressive,
        min_ukeire_for_riichi: 1,
        min_points_for_riichi: 1_000,
        defend_from_shanten: 2,
        prefer_riichi_at_tenpai: true,
    };
}

pub fn choose_reaction(view: &PlayerView, legal: &[Action], policy: CallPolicy) -> Action {
    if let Some(win) = prefer_win(legal) {
        return win;
    }

    let Some(hand) = hand_from_view(view) else {
        return Action::Pass;
    };
    let baseline = evaluate_hand(&hand, Some(view));

    let Some(pending) = view.turn.pending_call() else {
        return Action::Pass;
    };

    let accept_equal_ukeire = policy == CallPolicy::Aggressive;
    let mut best = Action::Pass;
    let mut best_strength = baseline;

    for action in legal {
        if matches!(action, Action::Pass | Action::Ron) {
            continue;
        }
        let Some(after_hand) = simulate_call(&hand, *action, pending.tile) else {
            continue;
        };
        let strength = evaluate_hand(&after_hand, Some(view));
        if strength.improves_over(baseline, accept_equal_ukeire) && strength.is_better_than(best_strength)
        {
            best_strength = strength;
            best = *action;
        }
    }

    if best == Action::Pass {
        Action::Pass
    } else {
        best
    }
}

pub fn should_defend(view: &PlayerView, hand: &Hand, from_shanten: i8) -> bool {
    any_opponent_riichi(view) && shanten_to_tenpai(hand) >= from_shanten
}

pub fn choose_discard(rng: &mut impl Rng, view: &PlayerView, legal: &[Action], policy: DiscardPolicy) -> Action {
    if let Some(kan) = choose_kan(rng, view, legal, policy) {
        return kan;
    }

    let Some(hand) = hand_from_view(view) else {
        return legal[0];
    };

    let defending = should_defend(view, &hand, policy.defend_from_shanten);
    let can_riichi = !defending
        && hand.melds().is_empty()
        && view.scores[view.seat] >= policy.min_points_for_riichi;

    let mut candidates: Vec<Action> = legal
        .iter()
        .copied()
        .filter(|action| matches!(action, Action::Discard(_)))
        .collect();
    if can_riichi {
        candidates.extend(
            legal
                .iter()
                .copied()
                .filter(|action| matches!(action, Action::Riichi { .. })),
        );
    }

    if candidates.is_empty() {
        return *legal
            .choose(rng)
            .expect("legal discard actions expected in discard phase");
    }

    let mut ranked: Vec<(Action, HandStrength, u16)> = Vec::new();
    for action in candidates {
        if let Some((strength, safety)) = score_discard(view, &hand, action, defending, policy) {
            ranked.push((action, strength, safety));
        }
    }

    if ranked.is_empty() {
        return *legal
            .iter()
            .find(|action| matches!(action, Action::Discard(_)))
            .expect("discard fallback");
    }

    let best_shanten = ranked.iter().map(|(_, s, _)| s.shanten).min().unwrap();
    ranked.retain(|(_, s, _)| s.shanten == best_shanten);

    let best_ukeire = ranked.iter().map(|(_, s, _)| s.ukeire).max().unwrap();
    ranked.retain(|(_, s, _)| s.ukeire == best_ukeire);

    if defending {
        let best_weighted = ranked.iter().map(|(_, s, _)| s.weighted).max().unwrap();
        ranked.retain(|(_, s, _)| s.weighted == best_weighted);
        ranked.sort_by_key(|(_, _, safety)| std::cmp::Reverse(*safety));
        return ranked[0].0;
    }

    if policy.prefer_riichi_at_tenpai {
        let riichi: Vec<Action> = ranked
            .iter()
            .filter(|(action, strength, _)| {
                matches!(action, Action::Riichi { .. }) && strength.shanten == 0
            })
            .map(|(action, _, _)| *action)
            .collect();
        if !riichi.is_empty() {
            return *riichi.choose(rng).expect("riichi candidate");
        }
    }

    let best_weighted = ranked.iter().map(|(_, s, _)| s.weighted).max().unwrap();
    let top: Vec<Action> = ranked
        .iter()
        .filter(|(_, s, _)| s.weighted == best_weighted)
        .map(|(action, _, _)| *action)
        .collect();

    if let Some(action) = top.choose(rng) {
        return *action;
    }
    ranked[0].0
}

fn choose_kan(
    rng: &mut impl Rng,
    view: &PlayerView,
    legal: &[Action],
    policy: DiscardPolicy,
) -> Option<Action> {
    let Some(hand) = hand_from_view(view) else {
        return None;
    };
    let baseline = evaluate_hand(&hand, Some(view));
    let accept_equal_ukeire = policy.call_policy == CallPolicy::Aggressive;

    let mut best: Option<(Action, HandStrength)> = None;
    for action in legal {
        let after = match action {
            Action::Kan(KanIntent::Closed { tile }) => simulate_closed_kan(&hand, *tile)?,
            Action::Kan(KanIntent::Added { meld_index }) => {
                simulate_added_kan(&hand, *meld_index)?
            }
            _ => continue,
        };
        let strength = evaluate_hand(&after, Some(view));
        if strength.improves_over(baseline, accept_equal_ukeire)
            && best.as_ref().is_none_or(|(_, best_strength)| strength.is_better_than(*best_strength))
        {
            best = Some((*action, strength));
        }
    }

    best.map(|(action, _)| action).or_else(|| {
        // Declare closed kan when one-shanten or closer and it does not worsen shanten.
        if baseline.shanten > 1 {
            return None;
        }
        let mut neutral: Vec<Action> = Vec::new();
        for action in legal {
            let after = match action {
                Action::Kan(KanIntent::Closed { tile }) => simulate_closed_kan(&hand, *tile)?,
                Action::Kan(KanIntent::Added { meld_index }) => {
                    simulate_added_kan(&hand, *meld_index)?
                }
                _ => continue,
            };
            let strength = evaluate_hand(&after, Some(view));
            if strength.shanten <= baseline.shanten && strength.ukeire + 1 >= baseline.ukeire {
                neutral.push(*action);
            }
        }
        neutral.choose(rng).copied()
    })
}

fn score_discard(
    view: &PlayerView,
    hand: &Hand,
    action: Action,
    defending: bool,
    policy: DiscardPolicy,
) -> Option<(HandStrength, u16)> {
    let tile = match action {
        Action::Discard(tile) | Action::Riichi { discard: tile } => tile,
        _ => return None,
    };
    let after = hand_without_concealed_tile(hand, tile)?;
    let plain = waiting_count(&after);
    if matches!(action, Action::Riichi { .. }) && plain < policy.min_ukeire_for_riichi {
        return None;
    }
    let mut strength = evaluate_hand(&after, Some(view));
    if matches!(action, Action::Riichi { .. }) && plain > 0 {
        strength.weighted = strength.weighted.saturating_add(weighted_waiting_count(&after, view));
    }
    let safety = if defending {
        tile_safety(view, tile)
    } else {
        0
    };
    Some((strength, safety))
}
