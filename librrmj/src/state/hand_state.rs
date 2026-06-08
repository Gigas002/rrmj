use crate::action::Action;
use crate::event::Event;
use crate::hand::Hand;
use crate::tile::Tile;
use crate::wall::{DealResult, Wall, WALL_SIZE};
use crate::Error;

pub const SEAT_COUNT: usize = 4;

/// Phase of an in-progress hand (phase-2 subset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandPhase {
    /// Active seat must draw from the live wall.
    Draw,
    /// Active seat must discard (dealer's first turn skips draw).
    Discard,
    /// Live wall exhausted; no further actions.
    Ended,
}

/// In-progress hand: seats, rivers, wall, and turn order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandState {
    dealer: usize,
    current_actor: usize,
    phase: HandPhase,
    hands: [Hand; 4],
    discards: [Vec<Tile>; SEAT_COUNT],
    wall: Wall,
}

impl HandState {
    pub fn from_deal(wall: Wall, deal: DealResult) -> Self {
        Self {
            dealer: deal.dealer,
            current_actor: deal.dealer,
            phase: HandPhase::Discard,
            hands: deal.hands,
            discards: std::array::from_fn(|_| Vec::new()),
            wall,
        }
    }

    pub const fn dealer(&self) -> usize {
        self.dealer
    }

    pub const fn current_actor(&self) -> usize {
        self.current_actor
    }

    pub const fn phase(&self) -> HandPhase {
        self.phase
    }

    pub fn hand(&self, seat: usize) -> &Hand {
        &self.hands[seat]
    }

    pub fn discards(&self, seat: usize) -> &[Tile] {
        &self.discards[seat]
    }

    pub fn wall(&self) -> &Wall {
        &self.wall
    }

    pub fn wall_mut(&mut self) -> &mut Wall {
        &mut self.wall
    }

    pub fn is_ended(&self) -> bool {
        self.phase == HandPhase::Ended
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        match self.phase {
            HandPhase::Draw => vec![Action::Draw],
            HandPhase::Discard => self.hands[self.current_actor]
                .concealed()
                .tiles()
                .iter()
                .copied()
                .map(Action::Discard)
                .collect(),
            HandPhase::Ended => Vec::new(),
        }
    }

    pub fn apply(&mut self, seat: usize, action: Action) -> Result<Vec<Event>, Error> {
        if self.phase == HandPhase::Ended {
            return Err(Error::HandEnded);
        }
        if seat != self.current_actor {
            return Err(Error::WrongActor {
                expected: self.current_actor,
                actual: seat,
            });
        }

        match action {
            Action::Draw => self.apply_draw(seat),
            Action::Discard(tile) => self.apply_discard(seat, tile),
            Action::Pass => Err(Error::IllegalAction {
                action,
                phase: self.phase,
            }),
        }
    }

    /// Applies [`Action::Draw`] then [`Action::Discard`] until the live wall is empty.
    pub fn play_out_discards<F>(&mut self, mut pick_discard: F) -> Result<Vec<Event>, Error>
    where
        F: FnMut(&HandState, usize) -> Tile,
    {
        let mut events = vec![Event::Dealt {
            dealer: self.dealer,
        }];

        while !self.is_ended() {
            match self.phase {
                HandPhase::Draw => {
                    events.extend(self.apply(self.current_actor, Action::Draw)?);
                }
                HandPhase::Discard => {
                    let seat = self.current_actor;
                    let tile = pick_discard(self, seat);
                    events.extend(self.apply(seat, Action::Discard(tile))?);
                }
                HandPhase::Ended => break,
            }
        }

        Ok(events)
    }

    pub fn accounted_tile_count(&self) -> usize {
        let in_hands: usize = self.hands.iter().map(|h| h.total_tiles()).sum();
        let in_rivers: usize = self.discards.iter().map(|d| d.len()).sum();
        let in_wall = self.wall.live_remaining() + self.wall.dead_wall().len();
        in_hands + in_rivers + in_wall
    }

    pub fn validate_tile_conservation(&self) -> Result<(), Error> {
        let count = self.accounted_tile_count();
        if count != WALL_SIZE {
            return Err(Error::TileConservation {
                expected: WALL_SIZE,
                actual: count,
            });
        }
        Ok(())
    }

    fn apply_draw(&mut self, seat: usize) -> Result<Vec<Event>, Error> {
        if self.phase != HandPhase::Draw {
            return Err(Error::WrongPhase {
                expected: HandPhase::Draw,
                actual: self.phase,
            });
        }

        let tile = self.wall.draw_live()?;
        self.hands[seat].concealed_mut().push(tile);
        self.hands[seat].sort_concealed();
        self.phase = HandPhase::Discard;

        Ok(vec![Event::Drawn { seat, tile }])
    }

    fn apply_discard(&mut self, seat: usize, tile: Tile) -> Result<Vec<Event>, Error> {
        if self.phase != HandPhase::Discard {
            return Err(Error::WrongPhase {
                expected: HandPhase::Discard,
                actual: self.phase,
            });
        }

        self.hands[seat].concealed_mut().remove(tile)?;
        self.discards[seat].push(tile);

        let next = super::next_seat(seat);
        let mut events = vec![Event::Discarded { seat, tile }];

        if self.wall.live_remaining() == 0 {
            self.current_actor = next;
            self.phase = HandPhase::Ended;
            events.push(Event::HandEnded);
        } else {
            self.current_actor = next;
            self.phase = HandPhase::Draw;
        }

        Ok(events)
    }
}
