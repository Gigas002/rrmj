mod concealed;
mod meld;

#[cfg(test)]
mod tests;

pub use concealed::Concealed;
pub use meld::{Meld, MeldKind, MeldTileCount};

use crate::Error;

/// A player's hand: concealed tiles plus open melds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hand {
    concealed: Concealed,
    melds: Vec<Meld>,
}

impl Hand {
    pub fn new(concealed: Concealed, melds: Vec<Meld>) -> Result<Self, Error> {
        let hand = Self { concealed, melds };
        hand.validate()?;
        Ok(hand)
    }

    pub fn empty() -> Self {
        Self {
            concealed: Concealed::empty(),
            melds: Vec::new(),
        }
    }

    pub fn concealed(&self) -> &Concealed {
        &self.concealed
    }

    pub fn concealed_mut(&mut self) -> &mut Concealed {
        &mut self.concealed
    }

    pub fn melds(&self) -> &[Meld] {
        &self.melds
    }

    pub fn melds_mut(&mut self) -> &mut Vec<Meld> {
        &mut self.melds
    }

    pub fn total_tiles(&self) -> usize {
        self.concealed.len() + self.melds.iter().map(|m| m.tiles().len()).sum::<usize>()
    }

    pub fn validate(&self) -> Result<(), Error> {
        for meld in &self.melds {
            meld.validate()?;
        }
        Ok(())
    }

    pub fn sort_concealed(&mut self) {
        self.concealed.sort();
    }
}

/// Number of tiles a complete four-player hand should contain at deal time.
pub const DEALER_HAND_SIZE: usize = 14;
pub const NON_DEALER_HAND_SIZE: usize = 13;

pub fn validate_deal_counts(hands: &[Hand; 4], dealer: usize) -> Result<(), Error> {
    if dealer >= 4 {
        return Err(Error::InvalidSeat(dealer));
    }

    for (seat, hand) in hands.iter().enumerate() {
        hand.validate()?;
        let expected = if seat == dealer {
            DEALER_HAND_SIZE
        } else {
            NON_DEALER_HAND_SIZE
        };
        if hand.total_tiles() != expected {
            return Err(Error::InvalidHandTileCount {
                seat,
                expected,
                actual: hand.total_tiles(),
            });
        }
    }

    Ok(())
}
