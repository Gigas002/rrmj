use librrmj::event::Event;
use librrmj::game::AbortiveDrawKind;

/// Summary shown between hands.
#[derive(Debug, Clone)]
pub struct HandResultSummary {
    pub title: String,
    pub lines: Vec<String>,
}

pub fn summary_from_events(events: &[Event]) -> Option<HandResultSummary> {
    if let Some(Event::Won { seat, han, fu }) =
        events.iter().find(|e| matches!(e, Event::Won { .. }))
    {
        let seat = *seat;
        let mut lines = vec![
            format!("Winner: seat {seat}"),
            format!("Han: {han}, fu: {fu}"),
        ];
        if let Some(Event::ScoresAdjusted { deltas }) = events
            .iter()
            .find(|e| matches!(e, Event::ScoresAdjusted { .. }))
        {
            lines.push(format!("Deltas: {:?}", deltas));
        }
        return Some(HandResultSummary {
            title: "Hand won".into(),
            lines,
        });
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

fn abortive_label(kind: AbortiveDrawKind) -> &'static str {
    match kind {
        AbortiveDrawKind::NineTerminals => "nine terminals",
        AbortiveDrawKind::FourWinds => "four winds",
        AbortiveDrawKind::FourKongs => "four kongs",
        AbortiveDrawKind::FourRiichis => "four riichis",
    }
}
