use librrmj::event::Event;
use librrmj::game::AbortiveDrawKind;

/// Short human-readable description of an engine event for the status bar and replay log.
pub fn describe_event(event: &Event) -> String {
    match event {
        Event::Dealt { dealer } => format!("Dealt — dealer {}", seat_name(*dealer)),
        Event::Drawn { seat, tile } => format!("{} drew {tile}", seat_name(*seat)),
        Event::Discarded { seat, tile } => format!("{} discarded {tile}", seat_name(*seat)),
        Event::RiichiDeclared { seat, discard } => {
            format!("{} declared riichi on {discard}", seat_name(*seat))
        }
        Event::Called {
            seat,
            from,
            meld,
            tiles,
        } => format!(
            "{} called {:?} from {} ({})",
            seat_name(*seat),
            meld,
            seat_name(*from),
            tiles
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        ),
        Event::DoraRevealed { tile } => format!("Dora revealed — indicator {tile}"),
        Event::RinshanDrawn { seat, tile } => format!("{} rinshan {tile}", seat_name(*seat)),
        Event::KakanDeclared {
            seat,
            meld_index,
            tile,
        } => format!(
            "{} added kan on meld {meld_index} ({tile})",
            seat_name(*seat)
        ),
        Event::Won { seat, han, fu } => {
            format!("{} won — {han} han {fu} fu", seat_name(*seat))
        }
        Event::ScoresAdjusted { deltas } => format!("Scores adjusted {deltas:?}"),
        Event::ExhaustiveDraw { deltas } => format!("Exhaustive draw {deltas:?}"),
        Event::HandStarted {
            dealer,
            round_wind,
            kyoku,
            honba,
        } => format!(
            "Hand started — {}-{} honba {}, dealer {}",
            round_wind.as_str(),
            kyoku,
            honba,
            seat_name(*dealer)
        ),
        Event::AbortiveDraw { kind } => {
            format!("Abortive draw — {}", abortive_label(*kind))
        }
        Event::MatchEnded { scores } => format!("Match ended — final scores {scores:?}"),
    }
}

fn seat_name(seat: usize) -> &'static str {
    match seat {
        0 => "East",
        1 => "South",
        2 => "West",
        3 => "North",
        _ => "Seat",
    }
}

fn abortive_label(kind: AbortiveDrawKind) -> &'static str {
    match kind {
        AbortiveDrawKind::NineTerminals => "nine terminals",
        AbortiveDrawKind::FourWinds => "four winds",
        AbortiveDrawKind::FourKongs => "four kongs",
        AbortiveDrawKind::FourRiichis => "four riichis",
    }
}
