//! Riichi cheatsheet for the `?` overlay — layout mirrors `examples/cs.png`.

mod reference;
mod scoring;
mod yaku;

#[cfg(test)]
mod tests;

use reference::{footer_lines, legend_lines, reference_lines};
use scoring::scoring_lines;
use yaku::yaku_lines;

/// Styling hint for [`CheatLine`] rendering (maps to cheatsheet band colors).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionTone {
    Closed,
    Timing,
    Pattern,
    Yakuman,
    Dora,
    Scoring,
    Reference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Legend,
    Title,
    Section,
    Subsection,
    YakuHead,
    Example,
    Prose,
    TableHead,
    TableRow,
    Footer,
}

#[derive(Debug, Clone)]
pub struct CheatLine {
    pub kind: LineKind,
    pub tone: Option<SectionTone>,
    pub text: String,
}

impl CheatLine {
    fn legend(text: impl Into<String>) -> Self {
        Self {
            kind: LineKind::Legend,
            tone: None,
            text: text.into(),
        }
    }

    fn title(text: impl Into<String>) -> Self {
        Self {
            kind: LineKind::Title,
            tone: None,
            text: text.into(),
        }
    }

    pub(crate) fn section(tone: SectionTone, text: impl Into<String>) -> Self {
        Self {
            kind: LineKind::Section,
            tone: Some(tone),
            text: text.into(),
        }
    }

    pub(crate) fn subsection(tone: SectionTone, text: impl Into<String>) -> Self {
        Self {
            kind: LineKind::Subsection,
            tone: Some(tone),
            text: text.into(),
        }
    }

    pub(crate) fn yaku_head(tone: SectionTone, text: impl Into<String>) -> Self {
        Self {
            kind: LineKind::YakuHead,
            tone: Some(tone),
            text: text.into(),
        }
    }

    pub(crate) fn example(tone: SectionTone, text: impl Into<String>) -> Self {
        Self {
            kind: LineKind::Example,
            tone: Some(tone),
            text: text.into(),
        }
    }

    pub(crate) fn prose(tone: Option<SectionTone>, text: impl Into<String>) -> Self {
        Self {
            kind: LineKind::Prose,
            tone,
            text: text.into(),
        }
    }

    pub(crate) fn table_head(tone: SectionTone, text: impl Into<String>) -> Self {
        Self {
            kind: LineKind::TableHead,
            tone: Some(tone),
            text: text.into(),
        }
    }

    pub(crate) fn table_row(tone: SectionTone, text: impl Into<String>) -> Self {
        Self {
            kind: LineKind::TableRow,
            tone: Some(tone),
            text: text.into(),
        }
    }

    fn footer(text: impl Into<String>) -> Self {
        Self {
            kind: LineKind::Footer,
            tone: None,
            text: text.into(),
        }
    }

    fn blank() -> Self {
        Self::prose(None, "")
    }
}

pub fn all_cheat_lines() -> Vec<CheatLine> {
    let mut out = Vec::new();
    out.extend(legend_lines());
    out.push(CheatLine::blank());
    out.push(CheatLine::title("RIICHI MAHJONG CHEATSHEET"));
    out.push(CheatLine::blank());
    out.extend(yaku_lines());
    out.push(CheatLine::blank());
    out.extend(scoring_lines());
    out.push(CheatLine::blank());
    out.extend(reference_lines());
    out.extend(footer_lines());
    out
}

pub fn line_count() -> usize {
    all_cheat_lines().len()
}
