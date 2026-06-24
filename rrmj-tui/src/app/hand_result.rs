use librrmj::event::Event;
use librrmj::game::AbortiveDrawKind;
use librrmj::scoring::ScoringResult;

/// Summary shown between hands.
#[derive(Debug, Clone)]
pub struct HandResultSummary {
    pub title: String,
    pub lines: Vec<String>,
}

pub fn summary_from_events(events: &[Event]) -> Option<HandResultSummary> {
    if let Some(Event::Won { seat, scoring }) =
        events.iter().find(|e| matches!(e, Event::Won { .. }))
    {
        return Some(win_summary(*seat, scoring));
    }

    if events
        .iter()
        .any(|e| matches!(e, Event::ExhaustiveDraw { .. }))
    {
        let mut lines = vec!["No winner — exhaustive draw".into()];
        if let Some(Event::ExhaustiveDraw { deltas }) = events
            .iter()
            .find(|e| matches!(e, Event::ExhaustiveDraw { .. }))
        {
            lines.push(format!("Deltas: {:?}", deltas));
        }
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

fn win_summary(seat: usize, scoring: &ScoringResult) -> HandResultSummary {
    let mut lines = vec![
        format!("Winner: {}", crate::app::NewGameSetup::seat_name(seat)),
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
    for line in scoring.delta_lines() {
        lines.push(format!("  {line}"));
    }
    HandResultSummary {
        title: "Hand won".into(),
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
