use librrmj::event::Event;
use librrmj::game::AbortiveDrawKind;
use librrmj::scoring::ScoringResult;
use librrmj::state::HandEndReason;

/// Summary shown between hands.
#[derive(Debug, Clone)]
pub struct HandResultSummary {
    pub title: String,
    pub lines: Vec<String>,
}

pub fn summary_from_events(events: &[Event]) -> Option<HandResultSummary> {
    let wins: Vec<(usize, &ScoringResult)> = events
        .iter()
        .filter_map(|event| match event {
            Event::Won { seat, scoring } => Some((*seat, scoring)),
            _ => None,
        })
        .collect();
    let adjusted = events.iter().find_map(|event| match event {
        Event::ScoresAdjusted { deltas } => Some(*deltas),
        _ => None,
    });
    match wins.as_slice() {
        [] => {}
        [(seat, scoring)] => return Some(win_summary(*seat, scoring, adjusted)),
        _ => return Some(multi_win_summary(&wins, adjusted)),
    }

    if let Some((deltas, tenpai)) = events.iter().find_map(|event| match event {
        Event::ExhaustiveDraw { deltas, tenpai } => Some((*deltas, *tenpai)),
        _ => None,
    }) {
        let seats = |want: bool| {
            let names: Vec<&str> = (0..4)
                .filter(|&seat| tenpai[seat] == want)
                .map(seat_name)
                .collect();
            if names.is_empty() {
                "—".into()
            } else {
                names.join(", ")
            }
        };
        let mut lines = vec![
            "No winner — the wall ran out".into(),
            format!("Tenpai: {}", seats(true)),
            format!("Noten: {}", seats(false)),
            String::new(),
            "Score changes:".into(),
        ];
        lines.extend(delta_lines(&deltas));
        return Some(HandResultSummary {
            title: "Exhaustive draw".into(),
            lines,
        });
    }

    if let Some(Event::AbortiveDraw { kind }) = events
        .iter()
        .find(|e| matches!(e, Event::AbortiveDraw { .. }))
    {
        return Some(HandResultSummary {
            title: "Abortive draw".into(),
            lines: vec![format!("Kind: {}", abortive_label(*kind))],
        });
    }

    None
}

/// Summary for a loaded game whose hand had already ended (no event log for
/// that ending, only the reason).
pub fn summary_from_end_reason(reason: &HandEndReason) -> HandResultSummary {
    let note = "This hand had already ended when it was loaded.".to_string();
    match reason {
        HandEndReason::Win { winners } => HandResultSummary {
            title: "Hand won".into(),
            lines: vec![
                format!(
                    "Winner: {}",
                    winners
                        .iter()
                        .map(|&seat| seat_name(seat))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                note,
            ],
        },
        HandEndReason::ExhaustiveDraw => HandResultSummary {
            title: "Exhaustive draw".into(),
            lines: vec!["No winner — the wall ran out".into(), note],
        },
        HandEndReason::AbortiveDraw(kind) => HandResultSummary {
            title: "Abortive draw".into(),
            lines: vec![format!("Kind: {}", abortive_label(*kind)), note],
        },
    }
}

fn seat_name(seat: usize) -> &'static str {
    crate::app::NewGameSetup::seat_name(seat)
}

/// One line per seat whose score changed; "No change" when none did.
fn delta_lines(deltas: &[i32; 4]) -> Vec<String> {
    let lines: Vec<String> = deltas
        .iter()
        .enumerate()
        .filter(|(_, delta)| **delta != 0)
        .map(|(seat, delta)| format!("  {}: {delta:+}", seat_name(seat)))
        .collect();
    if lines.is_empty() {
        vec!["  No change".into()]
    } else {
        lines
    }
}

fn win_summary(
    seat: usize,
    scoring: &ScoringResult,
    adjusted: Option<[i32; 4]>,
) -> HandResultSummary {
    let mut lines = vec![
        format!("Winner: {}", seat_name(seat)),
        format!("Win type: {}", scoring.win_type_label()),
        String::new(),
        "Yaku:".into(),
    ];
    for line in scoring.yaku_lines() {
        lines.push(format!("  {line}"));
    }
    for line in scoring.dora_lines() {
        lines.push(format!("  {line}"));
    }
    lines.push(String::new());
    lines.push(format!(
        "Total: {} han, {} fu ({})",
        scoring.han,
        scoring.fu,
        scoring.limit_label()
    ));
    lines.push(String::new());
    lines.push("Payments:".into());
    for line in scoring.payment_lines() {
        lines.push(format!("  {line}"));
    }
    lines.push(String::new());
    lines.push("Score changes:".into());
    lines.extend(delta_lines(&adjusted.unwrap_or(scoring.deltas)));
    HandResultSummary {
        title: "Hand won".into(),
        lines,
    }
}

/// Double or triple ron: one compact block per winner, then the combined
/// score changes (the closest winner also took honba and riichi sticks).
fn multi_win_summary(
    wins: &[(usize, &ScoringResult)],
    adjusted: Option<[i32; 4]>,
) -> HandResultSummary {
    let mut lines = Vec::new();
    for (seat, scoring) in wins {
        lines.push(format!(
            "{} — {}: {} han, {} fu ({})",
            seat_name(*seat),
            scoring.win_type_label(),
            scoring.han,
            scoring.fu,
            scoring.limit_label()
        ));
        let yaku: Vec<String> = scoring
            .yaku_lines()
            .into_iter()
            .chain(scoring.dora_lines())
            .collect();
        lines.push(format!("  {}", yaku.join(", ")));
    }
    lines.push(String::new());
    lines.push("Score changes:".into());
    let combined = adjusted.unwrap_or_else(|| {
        let mut sum = [0; 4];
        for (_, scoring) in wins {
            for (total, delta) in sum.iter_mut().zip(scoring.deltas) {
                *total += delta;
            }
        }
        sum
    });
    lines.extend(delta_lines(&combined));
    HandResultSummary {
        title: if wins.len() == 3 {
            "Triple ron".into()
        } else {
            "Double ron".into()
        },
        lines,
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

#[cfg(test)]
mod tests;
