use crate::Error;
use crate::action::Action;
use crate::event::Event;
use crate::hand::{Hand, Meld, MeldKind};
use crate::rules::RulesConfig;
use crate::scoring::WinType;
use crate::state::calls::{
    CallKind, chi_actions, closed_kan_options, kamicha, resolve_chi, resolve_closed_kan,
    resolve_open_kan, resolve_pon,
};
use crate::state::end_reason::HandEndReason;
use crate::state::reaction::{ReactionState, call_kind};
use crate::tile::Tile;
use crate::wall::{DealResult, WALL_SIZE, Wall};

pub const SEAT_COUNT: usize = 4;

/// Phase of an in-progress hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandPhase {
    Draw,
    Discard,
    Reaction,
    Ended,
}

/// In-progress hand: seats, rivers, wall, and turn order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandState {
    dealer: usize,
    current_actor: usize,
    pub(crate) phase: HandPhase,
    hands: [Hand; 4],
    discards: [Vec<Tile>; SEAT_COUNT],
    wall: Wall,
    pub(crate) reaction: Option<ReactionState>,
    pub(crate) config: RulesConfig,
    pub(crate) scores: [i32; 4],
    pub(crate) riichi: [bool; 4],
    pub(crate) table_riichi_sticks: u8,
    pub(crate) honba: u8,
    pub(crate) last_draw: Option<Tile>,
    pub(crate) first_discards: [Option<Tile>; SEAT_COUNT],
    pub(crate) is_dealer_first_turn: bool,
    pub(crate) end_reason: Option<HandEndReason>,
}

impl HandState {
    pub fn from_deal(wall: Wall, deal: DealResult, config: RulesConfig) -> Self {
        let starting = config.starting_points;
        Self::from_deal_with_carry(wall, deal, config, [starting; 4], 0, 0)
    }

    pub fn from_deal_with_carry(
        wall: Wall,
        deal: DealResult,
        config: RulesConfig,
        scores: [i32; 4],
        honba: u8,
        table_riichi_sticks: u8,
    ) -> Self {
        Self {
            dealer: deal.dealer,
            current_actor: deal.dealer,
            phase: HandPhase::Discard,
            hands: deal.hands,
            discards: std::array::from_fn(|_| Vec::new()),
            wall,
            reaction: None,
            config,
            scores,
            riichi: [false; 4],
            table_riichi_sticks,
            honba,
            last_draw: None,
            first_discards: [None; SEAT_COUNT],
            is_dealer_first_turn: true,
            end_reason: None,
        }
    }

    pub fn end_reason(&self) -> Option<HandEndReason> {
        self.end_reason
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

    pub fn first_discard(&self, seat: usize) -> Option<Tile> {
        self.first_discards[seat]
    }

    pub fn is_ended(&self) -> bool {
        self.phase == HandPhase::Ended
    }

    pub fn legal_actions_for(&self, seat: usize) -> Vec<Action> {
        match self.phase {
            HandPhase::Draw if seat == self.current_actor => vec![Action::Draw],
            HandPhase::Discard if seat == self.current_actor => {
                let mut actions = Vec::new();
                if self.can_abort_nine_terminals(seat) {
                    actions.push(Action::AbortiveNineTerminals);
                }
                if self.can_tsumo(seat) {
                    actions.push(Action::Tsumo);
                }
                if self.can_declare_riichi(seat) {
                    for &tile in self.hands[seat].concealed().tiles() {
                        actions.push(Action::Riichi { discard: tile });
                    }
                }
                for tile in self.hands[seat].concealed().tiles().iter().copied() {
                    actions.push(Action::Discard(tile));
                }
                for tile in closed_kan_options(self.hands[seat].concealed()) {
                    actions.push(Action::ClosedKan { tile });
                }
                actions
            }
            HandPhase::Reaction => self.legal_reaction_actions(seat),
            _ => Vec::new(),
        }
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        self.legal_actions_for(self.current_actor)
    }

    /// Next seat that must submit a reaction, if any.
    pub fn pending_reaction_seat(&self) -> Option<usize> {
        if self.phase != HandPhase::Reaction {
            return None;
        }
        (0..SEAT_COUNT).find(|&seat| self.can_respond(seat))
    }

    pub(crate) fn resolve_all_passed_reaction(&mut self) -> Result<(), Error> {
        let reaction = self.reaction.take().ok_or(Error::ReplayMismatch {
            detail: "reaction resolve without reaction state",
        })?;
        self.phase = HandPhase::Draw;
        self.current_actor = super::calls::kamicha(reaction.discarder);
        Ok(())
    }

    pub fn apply(&mut self, seat: usize, action: Action) -> Result<Vec<Event>, Error> {
        if self.phase == HandPhase::Ended {
            return Err(Error::HandEnded);
        }

        match self.phase {
            HandPhase::Ended => Err(Error::HandEnded),
            HandPhase::Reaction => self.apply_reaction(seat, action),
            HandPhase::Draw | HandPhase::Discard => {
                if seat != self.current_actor {
                    return Err(Error::WrongActor {
                        expected: self.current_actor,
                        actual: seat,
                    });
                }
                match action {
                    Action::Draw => self.apply_draw(seat),
                    Action::Discard(tile) => self.apply_discard(seat, tile),
                    Action::Riichi { discard } => self.apply_riichi(seat, discard),
                    Action::Tsumo => {
                        let tile = self.last_draw.ok_or(Error::CannotWin)?;
                        self.resolve_win(seat, WinType::Tsumo, tile)
                    }
                    Action::ClosedKan { tile } => self.apply_closed_kan(seat, tile),
                    Action::AbortiveNineTerminals => self.apply_abortive_nine_terminals(seat),
                    _ => Err(Error::IllegalAction {
                        action,
                        phase: self.phase,
                    }),
                }
            }
        }
    }

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
                HandPhase::Reaction => {
                    for seat in 0..SEAT_COUNT {
                        if self.can_respond(seat) {
                            events.extend(self.apply(seat, Action::Pass)?);
                        }
                    }
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

    #[cfg(test)]
    pub fn set_concealed(&mut self, seat: usize, tiles: Vec<Tile>) {
        *self.hands[seat].concealed_mut() = crate::hand::Concealed::from_tiles(tiles);
    }

    #[cfg(test)]
    pub fn push_discard_for_test(&mut self, seat: usize, tile: Tile) {
        self.discards[seat].push(tile);
    }

    fn can_respond(&self, seat: usize) -> bool {
        self.reaction
            .as_ref()
            .is_some_and(|reaction| reaction.can_respond(seat))
    }

    fn legal_reaction_actions(&self, seat: usize) -> Vec<Action> {
        let Some(reaction) = &self.reaction else {
            return Vec::new();
        };
        if !reaction.can_respond(seat) {
            return Vec::new();
        }

        let mut actions = vec![Action::Pass];
        if self.can_ron(seat) {
            actions.push(Action::Ron);
        }

        let concealed = self.hands[seat].concealed();
        let called = reaction.tile;

        if seat == kamicha(reaction.discarder) {
            actions.extend(chi_actions(concealed, called));
        }
        if crate::state::calls::can_pon(concealed, called) {
            actions.push(Action::Pon);
        }
        if crate::state::calls::can_open_kan(concealed, called) {
            actions.push(Action::OpenKan);
        }

        actions
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
        self.last_draw = Some(tile);
        self.phase = HandPhase::Discard;

        Ok(vec![Event::Drawn { seat, tile }])
    }

    fn apply_discard(&mut self, seat: usize, tile: Tile) -> Result<Vec<Event>, Error> {
        self.apply_discard_inner(seat, tile, false)
    }

    pub(crate) fn apply_discard_inner(
        &mut self,
        seat: usize,
        tile: Tile,
        riichi: bool,
    ) -> Result<Vec<Event>, Error> {
        if self.phase != HandPhase::Discard {
            return Err(Error::WrongPhase {
                expected: HandPhase::Discard,
                actual: self.phase,
            });
        }

        self.hands[seat].concealed_mut().remove(tile)?;
        self.discards[seat].push(tile);
        self.last_draw = None;

        if self.first_discards[seat].is_none() {
            self.first_discards[seat] = Some(tile);
        }
        if seat == self.dealer {
            self.is_dealer_first_turn = false;
        }

        let events = if riichi {
            vec![
                Event::RiichiDeclared {
                    seat,
                    discard: tile,
                },
                Event::Discarded { seat, tile },
            ]
        } else {
            vec![Event::Discarded { seat, tile }]
        };

        if self.wall.live_remaining() == 0 {
            self.current_actor = super::next_seat(seat);
            return self.resolve_exhaustive_draw(events);
        }

        if let Some(kind) = self.check_abortive_after_discard(seat, tile) {
            return self.resolve_abortive_draw(kind, events);
        }

        if riichi && let Some(kind) = self.check_abortive_after_riichi() {
            return self.resolve_abortive_draw(kind, events);
        }

        self.current_actor = super::next_seat(seat);
        self.phase = HandPhase::Reaction;
        self.reaction = Some(ReactionState::new(seat, tile));
        Ok(events)
    }

    fn apply_closed_kan(&mut self, seat: usize, tile: Tile) -> Result<Vec<Event>, Error> {
        let meld = resolve_closed_kan(self.hands[seat].concealed(), tile)?;
        let tiles: Vec<Tile> = meld.tiles().to_vec();

        for meld_tile in meld.tiles() {
            self.hands[seat].concealed_mut().remove(*meld_tile)?;
        }
        self.hands[seat].melds_mut().push(meld);

        let (rinshan, dora) = self.wall.apply_kan()?;
        self.hands[seat].concealed_mut().push(rinshan);
        self.hands[seat].sort_concealed();
        self.last_draw = Some(rinshan);

        let mut events = vec![
            Event::Called {
                seat,
                from: seat,
                meld: MeldKind::ClosedKan,
                tiles,
            },
            Event::DoraRevealed { tile: dora },
            Event::RinshanDrawn {
                seat,
                tile: rinshan,
            },
        ];

        if let Some(kind) = self.check_abortive_after_kan() {
            events.extend(self.resolve_abortive_draw(kind, Vec::new())?);
        }

        Ok(events)
    }

    fn apply_reaction(&mut self, seat: usize, action: Action) -> Result<Vec<Event>, Error> {
        let Some(reaction) = &self.reaction else {
            return Err(Error::WrongPhase {
                expected: HandPhase::Reaction,
                actual: self.phase,
            });
        };

        if seat == reaction.discarder {
            return Err(Error::NotReactingSeat { seat });
        }
        if !reaction.can_respond(seat) {
            return Err(Error::AlreadyResponded { seat });
        }

        if action == Action::Ron {
            if self.is_furiten(seat, reaction.tile) {
                return Err(Error::Furiten);
            }
            let from = reaction.discarder;
            let win_tile = reaction.tile;
            self.reaction.take();
            self.pop_called_discard(from)?;
            return self.resolve_win(seat, WinType::Ron { from }, win_tile);
        }

        if action != Action::Pass {
            self.validate_call_action(seat, action)?;
        }

        self.reaction.as_mut().unwrap().record(seat, action);

        if !self.reaction.as_ref().unwrap().is_complete() {
            return Ok(Vec::new());
        }

        self.resolve_reaction()
    }

    fn validate_call_action(&self, seat: usize, action: Action) -> Result<(), Error> {
        let reaction = self.reaction.as_ref().expect("reaction");
        let called = reaction.tile;

        match action {
            Action::Chi { tiles } => {
                if seat != kamicha(reaction.discarder) {
                    return Err(Error::InvalidCall {
                        kind: CallKind::Chi,
                        reason: "chii only from kamicha",
                    });
                }
                resolve_chi(self.hands[seat].concealed(), called, tiles)?;
            }
            Action::Pon => {
                if !crate::state::calls::can_pon(self.hands[seat].concealed(), called) {
                    return Err(Error::InvalidCall {
                        kind: CallKind::Pon,
                        reason: "cannot pon",
                    });
                }
            }
            Action::OpenKan => {
                if !crate::state::calls::can_open_kan(self.hands[seat].concealed(), called) {
                    return Err(Error::InvalidCall {
                        kind: CallKind::OpenKan,
                        reason: "cannot open kan",
                    });
                }
            }
            _ => {
                return Err(Error::IllegalAction {
                    action,
                    phase: HandPhase::Reaction,
                });
            }
        }

        Ok(())
    }

    fn resolve_reaction(&mut self) -> Result<Vec<Event>, Error> {
        let reaction = self.reaction.take().expect("reaction state");
        let Some((caller, action)) = reaction.winning_call() else {
            self.phase = HandPhase::Draw;
            self.current_actor = kamicha(reaction.discarder);
            return Ok(Vec::new());
        };

        let called = reaction.tile;
        let discarder = reaction.discarder;
        let resolved = match action {
            Action::Chi { tiles } => resolve_chi(self.hands[caller].concealed(), called, tiles)?,
            Action::Pon => resolve_pon(self.hands[caller].concealed(), called)?,
            Action::OpenKan => resolve_open_kan(self.hands[caller].concealed(), called)?,
            _ => unreachable!("winning call must be chi/pon/open kan"),
        };

        let meld_kind = resolved.meld.kind();
        let meld_tiles: Vec<Tile> = resolved.meld.tiles().to_vec();

        self.pop_called_discard(discarder)?;

        for tile in resolved.remove_from_concealed {
            self.hands[caller].concealed_mut().remove(tile)?;
        }

        self.hands[caller].melds_mut().push(resolved.meld);

        let mut events = vec![Event::Called {
            seat: caller,
            from: discarder,
            meld: meld_kind,
            tiles: meld_tiles,
        }];

        if matches!(call_kind(action), Some(CallKind::OpenKan)) {
            let (rinshan, dora) = self.wall.apply_kan()?;
            self.hands[caller].concealed_mut().push(rinshan);
            self.hands[caller].sort_concealed();
            self.last_draw = Some(rinshan);
            events.push(Event::DoraRevealed { tile: dora });
            events.push(Event::RinshanDrawn {
                seat: caller,
                tile: rinshan,
            });

            if let Some(kind) = self.check_abortive_after_kan() {
                events.extend(self.resolve_abortive_draw(kind, Vec::new())?);
                return Ok(events);
            }
        }

        self.current_actor = caller;
        self.phase = HandPhase::Discard;
        Ok(events)
    }

    fn pop_called_discard(&mut self, discarder: usize) -> Result<(), Error> {
        self.discards[discarder].pop().ok_or(Error::InvalidCall {
            kind: CallKind::Pon,
            reason: "no discard to call",
        })?;
        Ok(())
    }

    /// Applies one recorded event (for replay). Does not validate wall tile order.
    pub fn apply_event(&mut self, event: &Event) -> Result<(), Error> {
        match event {
            Event::Dealt { dealer } => {
                if self.dealer != *dealer {
                    return Err(Error::ReplayMismatch {
                        detail: "dealer does not match dealt event",
                    });
                }
            }
            Event::HandStarted { .. } => {}
            Event::Drawn { seat, tile } => {
                self.hands[*seat].concealed_mut().push(*tile);
                self.hands[*seat].sort_concealed();
                self.last_draw = Some(*tile);
                self.current_actor = *seat;
                self.phase = HandPhase::Discard;
                let _ = self.wall.draw_live();
            }
            Event::Discarded { seat, tile } => {
                self.hands[*seat].concealed_mut().remove(*tile)?;
                self.discards[*seat].push(*tile);
                self.last_draw = None;
                if self.first_discards[*seat].is_none() {
                    self.first_discards[*seat] = Some(*tile);
                }
                if *seat == self.dealer {
                    self.is_dealer_first_turn = false;
                }
            }
            Event::RiichiDeclared { seat, discard: _ } => {
                self.scores[*seat] -= 1_000;
                self.table_riichi_sticks += 1;
                self.riichi[*seat] = true;
            }
            Event::Called {
                seat,
                from,
                meld,
                tiles,
            } => {
                let called = *self.discards[*from].last().ok_or(Error::ReplayMismatch {
                    detail: "no discard to complete call event",
                })?;
                self.discards[*from].pop().ok_or(Error::ReplayMismatch {
                    detail: "no discard to complete call event",
                })?;
                let meld = replay_build_meld(*meld, tiles, Some(called))?;
                for tile in meld.tiles() {
                    if meld.called() != Some(*tile) {
                        self.hands[*seat].concealed_mut().remove(*tile)?;
                    }
                }
                self.hands[*seat].melds_mut().push(meld);
                self.current_actor = *seat;
                self.phase = HandPhase::Discard;
                self.reaction = None;
            }
            Event::DoraRevealed { tile: _ } => {
                let _ = self.wall.apply_kan();
            }
            Event::RinshanDrawn { seat, tile } => {
                self.hands[*seat].concealed_mut().push(*tile);
                self.hands[*seat].sort_concealed();
                self.last_draw = Some(*tile);
            }
            Event::Won { seat, .. } => {
                self.phase = HandPhase::Ended;
                self.reaction = None;
                self.end_reason = Some(HandEndReason::Win { winner: *seat });
            }
            Event::ScoresAdjusted { deltas } => {
                self.apply_deltas(deltas);
            }
            Event::ExhaustiveDraw { deltas } => {
                self.apply_deltas(deltas);
                self.phase = HandPhase::Ended;
                self.reaction = None;
                self.end_reason = Some(HandEndReason::ExhaustiveDraw);
            }
            Event::AbortiveDraw { kind } => {
                self.phase = HandPhase::Ended;
                self.reaction = None;
                self.end_reason = Some(HandEndReason::AbortiveDraw(*kind));
            }
            Event::MatchEnded { .. } => {}
        }
        Ok(())
    }

    /// Completes reaction-phase bookkeeping after a discard event in replay.
    pub fn apply_discard_followup(&mut self, seat: usize) -> Result<(), Error> {
        if self.wall.live_remaining() == 0 {
            self.current_actor = super::next_seat(seat);
            let profile = crate::rules::RulesRegistry::get(self.config.profile)?;
            let deltas = profile.score_exhaustive_draw(self, &self.config);
            self.apply_deltas(&deltas);
            self.phase = HandPhase::Ended;
            self.reaction = None;
            self.end_reason = Some(HandEndReason::ExhaustiveDraw);
            return Ok(());
        }
        self.current_actor = super::next_seat(seat);
        self.phase = HandPhase::Reaction;
        let tile = *self.discards[seat].last().ok_or(Error::ReplayMismatch {
            detail: "discard follow-up without tile",
        })?;
        self.reaction = Some(ReactionState::new(seat, tile));
        Ok(())
    }
}

fn replay_build_meld(kind: MeldKind, tiles: &[Tile], called: Option<Tile>) -> Result<Meld, Error> {
    match kind {
        MeldKind::Chi => {
            let arr: [Tile; 3] = tiles.try_into().map_err(|_| Error::ReplayMismatch {
                detail: "chi event has wrong tile count",
            })?;
            let called = called.ok_or(Error::ReplayMismatch {
                detail: "chi event missing called tile",
            })?;
            Meld::chi(arr, called)
        }
        MeldKind::Pon => {
            let arr: [Tile; 3] = tiles.try_into().map_err(|_| Error::ReplayMismatch {
                detail: "pon event has wrong tile count",
            })?;
            let called = called.ok_or(Error::ReplayMismatch {
                detail: "pon event missing called tile",
            })?;
            Meld::pon(arr, called)
        }
        MeldKind::OpenKan => {
            let arr: [Tile; 4] = tiles.try_into().map_err(|_| Error::ReplayMismatch {
                detail: "open kan event has wrong tile count",
            })?;
            let called = called.ok_or(Error::ReplayMismatch {
                detail: "open kan event missing called tile",
            })?;
            Meld::open_kan(arr, called)
        }
        MeldKind::ClosedKan => {
            let arr: [Tile; 4] = tiles.try_into().map_err(|_| Error::ReplayMismatch {
                detail: "closed kan event has wrong tile count",
            })?;
            Meld::closed_kan(arr)
        }
        MeldKind::AddedKan => Meld::added_kan(tiles[0]),
    }
}
