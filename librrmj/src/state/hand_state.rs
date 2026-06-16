use crate::Error;
use crate::action::{Action, KanIntent};
use crate::event::Event;
use crate::hand::{Hand, KanForm, Meld, MeldKind};
use crate::rules::{RulesConfig, RulesRegistry, WinTimingFlags};
use crate::scoring::WinType;
use crate::state::calls::{
    CallKind, can_kakan, chi_actions, closed_kan_options, kakan_options, kamicha, resolve_chi,
    resolve_closed_kan, resolve_kakan_tile, resolve_open_kan, resolve_pon, upgrade_pon_to_open_kan,
};
use crate::state::end_reason::HandEndReason;
use crate::state::reaction::{ReactionKind, ReactionState, call_kind};
use crate::tile::Tile;
use crate::wall::{DealResult, WALL_SIZE, Wall};

pub const SEAT_COUNT: usize = 4;

/// Phase of an in-progress hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Most recent table discard (stays visible until called or replaced).
    pub(crate) last_discard: Option<(usize, Tile)>,
    pub(crate) config: RulesConfig,
    pub(crate) scores: [i32; 4],
    pub(crate) riichi: [bool; 4],
    pub(crate) table_riichi_sticks: u8,
    pub(crate) honba: u8,
    pub(crate) last_draw: Option<Tile>,
    pub(crate) first_discards: [Option<Tile>; SEAT_COUNT],
    pub(crate) is_dealer_first_turn: bool,
    pub(crate) temporary_furiten: [bool; 4],
    pub(crate) riichi_furiten: [bool; 4],
    /// Ippatsu still possible after riichi (voided by any call or kan).
    pub(crate) ippatsu_live: [bool; 4],
    pub(crate) double_riichi: [bool; 4],
    /// Any chi/pon/open-kan call has resolved this hand (blocks double riichi).
    pub(crate) calls_made: bool,
    /// True when `last_draw` came from a rinshan replacement tile.
    pub(crate) is_rinshan_draw: bool,
    /// Live-wall draws taken during play (excludes initial deal and rinshan).
    pub(crate) live_draws: [u8; 4],
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
            last_discard: None,
            config,
            scores,
            riichi: [false; 4],
            table_riichi_sticks,
            honba,
            last_draw: None,
            first_discards: [None; SEAT_COUNT],
            is_dealer_first_turn: true,
            temporary_furiten: [false; 4],
            riichi_furiten: [false; 4],
            ippatsu_live: [false; 4],
            double_riichi: [false; 4],
            calls_made: false,
            is_rinshan_draw: false,
            live_draws: [0; SEAT_COUNT],
            end_reason: None,
        }
    }

    pub fn end_reason(&self) -> Option<HandEndReason> {
        self.end_reason.clone()
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

    pub fn pending_call(&self) -> Option<crate::agent::PendingCall> {
        self.reaction
            .as_ref()
            .map(|reaction| crate::agent::PendingCall {
                discarder: reaction.discarder,
                tile: reaction.tile,
            })
    }

    pub fn last_discard(&self) -> Option<crate::agent::PendingCall> {
        self.last_discard
            .map(|(discarder, tile)| crate::agent::PendingCall { discarder, tile })
    }

    /// Tile just drawn (or rinshan) for the current actor during discard phase.
    pub fn last_drawn_tile(&self) -> Option<Tile> {
        self.last_draw
    }

    pub(crate) fn is_chankan_window(&self) -> bool {
        self.reaction
            .as_ref()
            .is_some_and(|reaction| reaction.kind == ReactionKind::Kakan)
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
                    let profile = RulesRegistry::get(self.config.profile).ok();
                    for &tile in self.hands[seat].concealed().tiles() {
                        if profile.is_some_and(|p| {
                            p.is_riichi_discard(self.hand(seat), tile, &self.config)
                        }) {
                            actions.push(Action::Riichi { discard: tile });
                        }
                    }
                }
                for tile in self.hands[seat].concealed().tiles().iter().copied() {
                    actions.push(Action::Discard(tile));
                }
                for tile in closed_kan_options(self.hands[seat].concealed()) {
                    actions.push(Action::Kan(KanIntent::Closed { tile }));
                }
                for meld_index in kakan_options(&self.hands[seat]) {
                    actions.push(Action::Kan(KanIntent::Added { meld_index }));
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
                    Action::Kan(KanIntent::Closed { tile }) => self.apply_closed_kan(seat, tile),
                    Action::Kan(KanIntent::Added { meld_index }) => {
                        self.apply_kakan(seat, meld_index)
                    }
                    Action::Kan(KanIntent::Open) => Err(Error::IllegalAction {
                        action,
                        phase: self.phase,
                    }),
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
        // Kakan reactions hold a tile removed from hand but not yet in a river.
        // Discard reactions reference the last river tile — already counted above.
        let in_reaction = self
            .reaction
            .as_ref()
            .filter(|r| r.kind == ReactionKind::Kakan)
            .map(|_| 1)
            .unwrap_or(0);
        in_hands + in_rivers + in_wall + in_reaction
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

    /// Lossless hand snapshot for match recordings.
    #[cfg(feature = "serde")]
    pub fn to_snapshot(&self) -> crate::replay::HandSnapshot {
        crate::replay::HandSnapshot {
            dealer: self.dealer,
            current_actor: self.current_actor,
            phase: self.phase,
            hands: self.hands.clone(),
            discards: self.discards.clone(),
            wall: self.wall.snapshot(),
            reaction: self.reaction.clone(),
            last_discard: self.last_discard,
            scores: self.scores,
            riichi: self.riichi,
            table_riichi_sticks: self.table_riichi_sticks,
            honba: self.honba,
            last_draw: self.last_draw,
            first_discards: self.first_discards,
            is_dealer_first_turn: self.is_dealer_first_turn,
            temporary_furiten: self.temporary_furiten,
            riichi_furiten: self.riichi_furiten,
            ippatsu_live: self.ippatsu_live,
            double_riichi: self.double_riichi,
            calls_made: self.calls_made,
            is_rinshan_draw: self.is_rinshan_draw,
            live_draws: self.live_draws,
            end_reason: self.end_reason.clone(),
        }
    }

    #[cfg(feature = "serde")]
    pub fn from_snapshot(
        snapshot: crate::replay::HandSnapshot,
        config: RulesConfig,
    ) -> Result<Self, Error> {
        let wall = Wall::from_snapshot(snapshot.wall)?;
        let state = Self {
            dealer: snapshot.dealer,
            current_actor: snapshot.current_actor,
            phase: snapshot.phase,
            hands: snapshot.hands,
            discards: snapshot.discards,
            wall,
            reaction: snapshot.reaction,
            last_discard: snapshot.last_discard,
            config,
            scores: snapshot.scores,
            riichi: snapshot.riichi,
            table_riichi_sticks: snapshot.table_riichi_sticks,
            honba: snapshot.honba,
            last_draw: snapshot.last_draw,
            first_discards: snapshot.first_discards,
            is_dealer_first_turn: snapshot.is_dealer_first_turn,
            temporary_furiten: snapshot.temporary_furiten,
            riichi_furiten: snapshot.riichi_furiten,
            ippatsu_live: snapshot.ippatsu_live,
            double_riichi: snapshot.double_riichi,
            calls_made: snapshot.calls_made,
            is_rinshan_draw: snapshot.is_rinshan_draw,
            live_draws: snapshot.live_draws,
            end_reason: snapshot.end_reason,
        };
        state.validate_tile_conservation()?;
        Ok(state)
    }

    #[cfg(test)]
    pub fn set_concealed(&mut self, seat: usize, tiles: Vec<Tile>) {
        *self.hands[seat].concealed_mut() = crate::hand::Concealed::from_tiles(tiles);
    }

    pub(crate) fn replace_hand(&mut self, seat: usize, hand: Hand) {
        self.hands[seat] = hand;
    }

    #[cfg(test)]
    pub fn set_hand(&mut self, seat: usize, hand: Hand) {
        self.replace_hand(seat, hand);
    }

    #[cfg(test)]
    pub fn push_discard_for_test(&mut self, seat: usize, tile: Tile) {
        self.discards[seat].push(tile);
    }

    #[cfg(test)]
    pub fn set_honba_for_test(&mut self, honba: u8) {
        self.honba = honba;
    }

    #[cfg(test)]
    pub fn set_table_riichi_sticks_for_test(&mut self, sticks: u8) {
        self.table_riichi_sticks = sticks;
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

        if reaction.kind == ReactionKind::Kakan {
            return actions;
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
            actions.push(Action::Kan(KanIntent::Open));
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
        self.is_rinshan_draw = false;
        self.live_draws[seat] += 1;
        self.clear_temporary_furiten(seat);
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
        self.last_discard = Some((seat, tile));

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
        self.is_rinshan_draw = true;
        self.void_ippatsu();

        let mut events = vec![
            Event::Called {
                seat,
                from: seat,
                meld: MeldKind::Kan(KanForm::Closed),
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

    fn apply_kakan(&mut self, seat: usize, meld_index: usize) -> Result<Vec<Event>, Error> {
        if self.phase != HandPhase::Discard {
            return Err(Error::WrongPhase {
                expected: HandPhase::Discard,
                actual: self.phase,
            });
        }
        if !can_kakan(&self.hands[seat], meld_index) {
            return Err(Error::InvalidCall {
                kind: CallKind::OpenKan,
                reason: "cannot kakan",
            });
        }

        let tile = resolve_kakan_tile(&self.hands[seat], meld_index)?;
        self.hands[seat].concealed_mut().remove(tile)?;

        self.phase = HandPhase::Reaction;
        self.reaction = Some(ReactionState::new_kakan(seat, tile, meld_index));
        self.void_ippatsu();

        Ok(vec![Event::KakanDeclared {
            seat,
            meld_index,
            tile,
        }])
    }

    fn complete_kakan(
        &mut self,
        seat: usize,
        meld_index: usize,
        tile: Tile,
    ) -> Result<Vec<Event>, Error> {
        let pon = self.hands[seat].melds()[meld_index].clone();
        let upgraded = upgrade_pon_to_open_kan(&pon, tile)?;
        let meld_tiles: Vec<Tile> = upgraded.tiles().to_vec();
        self.hands[seat].melds_mut()[meld_index] = upgraded;

        let (rinshan, dora) = self.wall.apply_kan()?;
        self.hands[seat].concealed_mut().push(rinshan);
        self.hands[seat].sort_concealed();
        self.last_draw = Some(rinshan);
        self.is_rinshan_draw = true;
        self.void_ippatsu();

        let mut events = vec![
            Event::Called {
                seat,
                from: seat,
                meld: MeldKind::Kan(KanForm::Open),
                tiles: meld_tiles,
            },
            Event::DoraRevealed { tile: dora },
            Event::RinshanDrawn {
                seat,
                tile: rinshan,
            },
        ];

        if let Some(kind) = self.check_abortive_after_kan() {
            events.extend(self.resolve_abortive_draw(kind, Vec::new())?);
            return Ok(events);
        }

        self.current_actor = seat;
        self.phase = HandPhase::Discard;
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
            if !self.can_ron(seat) {
                return Err(Error::CannotWin);
            }
        }

        if action == Action::Pass && self.can_ron(seat) {
            self.mark_passed_win(seat);
        }

        if action != Action::Pass && action != Action::Ron {
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
            Action::Kan(KanIntent::Open) => {
                if !crate::state::calls::can_open_kan(self.hands[seat].concealed(), called) {
                    return Err(Error::InvalidCall {
                        kind: CallKind::OpenKan,
                        reason: "cannot open kan",
                    });
                }
            }
            Action::Kan(_) => {
                return Err(Error::IllegalAction {
                    action,
                    phase: HandPhase::Reaction,
                });
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
        let max_rons = self.config.max_rons();
        let ron_winners = reaction.ron_winners(max_rons);
        let is_chankan = reaction.kind == ReactionKind::Kakan;
        if !ron_winners.is_empty() {
            let from = reaction.discarder;
            let win_tile = reaction.tile;
            if reaction.kind == ReactionKind::Discard {
                self.pop_called_discard(from)?;
            }
            return self.resolve_multi_ron(
                ron_winners,
                WinType::Ron { from },
                win_tile,
                WinTimingFlags { is_chankan },
            );
        }

        if reaction.kind == ReactionKind::Kakan {
            let seat = reaction.discarder;
            let meld_index = reaction
                .kakan_meld_index
                .expect("kakan reaction missing meld index");
            return self.complete_kakan(seat, meld_index, reaction.tile);
        }

        let Some((caller, action)) = reaction.winning_call() else {
            if self.wall.live_remaining() == 0 {
                return self.resolve_exhaustive_draw(Vec::new());
            }
            self.phase = HandPhase::Draw;
            self.current_actor = kamicha(reaction.discarder);
            return Ok(Vec::new());
        };

        let called = reaction.tile;
        let discarder = reaction.discarder;
        let resolved = match action {
            Action::Chi { tiles } => resolve_chi(self.hands[caller].concealed(), called, tiles)?,
            Action::Pon => resolve_pon(self.hands[caller].concealed(), called)?,
            Action::Kan(KanIntent::Open) => {
                resolve_open_kan(self.hands[caller].concealed(), called)?
            }
            _ => unreachable!("winning call must be chi/pon/open kan"),
        };

        let meld_kind = resolved.meld.kind();
        let meld_tiles: Vec<Tile> = resolved.meld.tiles().to_vec();

        self.pop_called_discard(discarder)?;

        for tile in resolved.remove_from_concealed {
            self.hands[caller].concealed_mut().remove(tile)?;
        }

        self.hands[caller].melds_mut().push(resolved.meld);
        self.calls_made = true;
        self.void_ippatsu();

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
            self.is_rinshan_draw = true;
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
        let tile = self.discards[discarder]
            .last()
            .copied()
            .ok_or(Error::InvalidCall {
                kind: CallKind::Pon,
                reason: "no discard to call",
            })?;
        self.discards[discarder].pop().ok_or(Error::InvalidCall {
            kind: CallKind::Pon,
            reason: "no discard to call",
        })?;
        if self.last_discard == Some((discarder, tile)) {
            self.last_discard = None;
        }
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
            Event::HandStarted { .. } => {
                self.last_discard = None;
            }
            Event::Drawn { seat, tile } => {
                self.hands[*seat].concealed_mut().push(*tile);
                self.hands[*seat].sort_concealed();
                self.last_draw = Some(*tile);
                self.is_rinshan_draw = false;
                self.live_draws[*seat] += 1;
                self.current_actor = *seat;
                self.phase = HandPhase::Discard;
                let _ = self.wall.draw_live();
            }
            Event::Discarded { seat, tile } => {
                self.hands[*seat].concealed_mut().remove(*tile)?;
                self.discards[*seat].push(*tile);
                self.last_draw = None;
                self.last_discard = Some((*seat, *tile));
                if self.first_discards[*seat].is_none() {
                    self.first_discards[*seat] = Some(*tile);
                }
                if *seat == self.dealer {
                    self.is_dealer_first_turn = false;
                }
            }
            Event::RiichiDeclared { seat, discard: _ } => {
                let is_first_discard = self.first_discards[*seat].is_none();
                self.scores[*seat] -= 1_000;
                self.table_riichi_sticks += 1;
                self.riichi[*seat] = true;
                self.ippatsu_live[*seat] = true;
                if is_first_discard && !self.calls_made {
                    self.double_riichi[*seat] = true;
                }
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
                if self.last_discard == Some((*from, called)) {
                    self.last_discard = None;
                }
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
                self.calls_made = true;
                self.void_ippatsu();
            }
            Event::DoraRevealed { tile: _ } => {
                let _ = self.wall.apply_kan();
                self.void_ippatsu();
            }
            Event::RinshanDrawn { seat, tile } => {
                self.hands[*seat].concealed_mut().push(*tile);
                self.hands[*seat].sort_concealed();
                self.last_draw = Some(*tile);
                self.is_rinshan_draw = true;
            }
            Event::KakanDeclared {
                seat,
                meld_index: _,
                tile,
            } => {
                self.hands[*seat].concealed_mut().remove(*tile)?;
                self.phase = HandPhase::Reaction;
                self.reaction = None;
                self.void_ippatsu();
            }
            Event::Won { seat, .. } => {
                self.phase = HandPhase::Ended;
                self.reaction = None;
                if !matches!(self.end_reason, Some(HandEndReason::Win { .. })) {
                    self.end_reason = Some(HandEndReason::Win {
                        winners: vec![*seat],
                    });
                }
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
            Event::GameEnded { .. } => {}
        }
        Ok(())
    }

    /// Completes reaction-phase bookkeeping after a discard event in replay.
    pub fn apply_discard_followup(&mut self, seat: usize) -> Result<(), Error> {
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
        MeldKind::Kan(KanForm::Open) => {
            let arr: [Tile; 4] = tiles.try_into().map_err(|_| Error::ReplayMismatch {
                detail: "open kan event has wrong tile count",
            })?;
            let called = called.ok_or(Error::ReplayMismatch {
                detail: "open kan event missing called tile",
            })?;
            Meld::open_kan(arr, called)
        }
        MeldKind::Kan(KanForm::Closed) => {
            let arr: [Tile; 4] = tiles.try_into().map_err(|_| Error::ReplayMismatch {
                detail: "closed kan event has wrong tile count",
            })?;
            Meld::closed_kan(arr)
        }
    }
}
