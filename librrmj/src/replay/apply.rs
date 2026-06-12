use crate::Error;
use crate::event::Event;
use crate::game::Game;
use crate::state::HandPhase;

/// Rebuild match state by applying events from the log, optionally stopping early.
pub fn apply_events(game: &mut Game, events: &[Event], until: Option<usize>) -> Result<(), Error> {
    let mut hand_starts = 0usize;
    for (index, event) in events.iter().enumerate() {
        apply_one_event(game, event, events.get(index + 1), &mut hand_starts)?;
        if until == Some(index) {
            break;
        }
    }
    Ok(())
}

/// Apply a single event at `index` from a fresh or partially replayed match.
pub fn apply_one_event(
    game: &mut Game,
    event: &Event,
    next: Option<&Event>,
    hand_starts: &mut usize,
) -> Result<(), Error> {
    match event {
        Event::HandStarted {
            dealer,
            round_wind,
            kyoku,
            honba,
        } => {
            *hand_starts += 1;
            if *hand_starts > 1 {
                game.begin_hand_from_event(*dealer, *round_wind, *kyoku, *honba)?;
            } else {
                game.assert_hand_metadata(*dealer, *round_wind, *kyoku, *honba)?;
            }
        }
        Event::Dealt { dealer } => {
            if *hand_starts > 1 && game.dealer() != *dealer {
                return Err(Error::ReplayMismatch {
                    detail: "dealt dealer mismatch on new hand",
                });
            }
        }
        Event::MatchEnded { scores } => {
            game.end_with_scores(*scores);
        }
        Event::Discarded { seat, .. } => {
            game.hand_mut().apply_event(event)?;
            if game.hand().phase() != HandPhase::Ended {
                game.hand_mut().apply_discard_followup(*seat)?;
                if game.hand().phase() == HandPhase::Reaction
                    && matches!(next, Some(Event::Drawn { .. }))
                {
                    game.hand_mut().resolve_all_passed_reaction()?;
                }
            }
            game.sync_scores_from_hand();
        }
        _ => {
            game.hand_mut().apply_event(event)?;
            game.sync_scores_from_hand();
        }
    }
    Ok(())
}
