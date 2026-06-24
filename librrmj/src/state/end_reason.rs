use crate::game::AbortiveDrawKind;

/// Why an in-progress hand ended.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HandEndReason {
    Win { winners: Vec<usize> },
    ExhaustiveDraw,
    AbortiveDraw(AbortiveDrawKind),
}

impl HandEndReason {
    pub fn primary_winner(&self) -> Option<usize> {
        match self {
            Self::Win { winners } => winners.first().copied(),
            _ => None,
        }
    }
}
