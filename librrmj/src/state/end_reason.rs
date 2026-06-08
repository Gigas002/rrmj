use crate::game::AbortiveDrawKind;

/// Why an in-progress hand ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandEndReason {
    Win { winner: usize },
    ExhaustiveDraw,
    AbortiveDraw(AbortiveDrawKind),
}
