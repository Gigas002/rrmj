use librrmj::agent::PlayerSlot;
use librrmj::ai::{AiConfig, Difficulty, MatchSetup};

const SEAT_NAMES: [&str; 4] = ["East", "South", "West", "North"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupField {
    SeatType(usize),
    SeatDifficulty(usize),
    HumanSeat,
    Confirm,
}

/// New-game configuration screen state.
#[derive(Debug, Clone)]
pub struct NewGameSetup {
    pub slots: [PlayerSlot; 4],
    pub difficulties: [Difficulty; 4],
    pub human_seat: usize,
    pub default_difficulty: Difficulty,
    pub selected: SetupField,
}

impl NewGameSetup {
    pub fn new(default_difficulty: Difficulty, human_seat: usize) -> Self {
        let mut slots = [PlayerSlot::Cpu; 4];
        slots[human_seat] = PlayerSlot::Human;
        Self {
            slots,
            difficulties: [default_difficulty; 4],
            human_seat,
            default_difficulty,
            selected: SetupField::SeatType(0),
        }
    }

    pub fn fields() -> [SetupField; 10] {
        [
            SetupField::SeatType(0),
            SetupField::SeatType(1),
            SetupField::SeatType(2),
            SetupField::SeatType(3),
            SetupField::SeatDifficulty(0),
            SetupField::SeatDifficulty(1),
            SetupField::SeatDifficulty(2),
            SetupField::SeatDifficulty(3),
            SetupField::HumanSeat,
            SetupField::Confirm,
        ]
    }

    pub fn select_next(&mut self) {
        let fields = Self::fields();
        let idx = fields.iter().position(|f| *f == self.selected).unwrap_or(0);
        self.selected = fields[(idx + 1) % fields.len()];
    }

    pub fn select_prev(&mut self) {
        let fields = Self::fields();
        let idx = fields.iter().position(|f| *f == self.selected).unwrap_or(0);
        self.selected = fields[(idx + fields.len() - 1) % fields.len()];
    }

    pub fn toggle_selected(&mut self) {
        match self.selected {
            SetupField::SeatType(seat) => {
                self.slots[seat] = match self.slots[seat] {
                    PlayerSlot::Human => PlayerSlot::Cpu,
                    PlayerSlot::Cpu | PlayerSlot::Remote => PlayerSlot::Human,
                };
                self.reconcile_human_seat();
            }
            SetupField::SeatDifficulty(seat) if self.slots[seat] == PlayerSlot::Cpu => {
                self.difficulties[seat] = cycle_difficulty(self.difficulties[seat]);
            }
            SetupField::SeatDifficulty(_) => {}
            SetupField::HumanSeat => {
                self.human_seat = (self.human_seat + 1) % 4;
                self.ensure_one_human();
            }
            SetupField::Confirm => {}
        }
    }

    pub fn cycle_selected(&mut self) {
        match self.selected {
            SetupField::SeatDifficulty(seat) if self.slots[seat] == PlayerSlot::Cpu => {
                self.difficulties[seat] = cycle_difficulty(self.difficulties[seat]);
            }
            SetupField::HumanSeat => {
                self.human_seat = (self.human_seat + 1) % 4;
                self.ensure_one_human();
            }
            _ => self.toggle_selected(),
        }
    }

    fn reconcile_human_seat(&mut self) {
        if self.slots[self.human_seat] != PlayerSlot::Human {
            if let Some(seat) = self.slots.iter().position(|&s| s == PlayerSlot::Human) {
                self.human_seat = seat;
            } else {
                self.slots[self.human_seat] = PlayerSlot::Human;
            }
        }
        for seat in 0..4 {
            if seat != self.human_seat && self.slots[seat] == PlayerSlot::Human {
                self.slots[seat] = PlayerSlot::Cpu;
            }
        }
    }

    fn ensure_one_human(&mut self) {
        for seat in 0..4 {
            self.slots[seat] = if seat == self.human_seat {
                PlayerSlot::Human
            } else {
                PlayerSlot::Cpu
            };
        }
    }

    pub fn to_match_setup(&self, seed: u64) -> MatchSetup {
        let default_ai = ai_config(self.default_difficulty, seed);
        let seat_ai = std::array::from_fn(|seat| {
            if self.slots[seat] == PlayerSlot::Cpu {
                Some(ai_config(
                    self.difficulties[seat],
                    seed.wrapping_add(seat as u64),
                ))
            } else {
                None
            }
        });
        MatchSetup {
            slots: self.slots,
            default_ai,
            seat_ai,
        }
    }

    pub fn seat_name(seat: usize) -> &'static str {
        SEAT_NAMES[seat]
    }
}

pub const fn cycle_difficulty(d: Difficulty) -> Difficulty {
    match d {
        Difficulty::Easy => Difficulty::Medium,
        Difficulty::Medium => Difficulty::Hard,
        Difficulty::Hard => Difficulty::Easy,
    }
}

pub const fn difficulty_label(d: Difficulty) -> &'static str {
    match d {
        Difficulty::Easy => "Easy",
        Difficulty::Medium => "Medium",
        Difficulty::Hard => "Hard",
    }
}

fn ai_config(difficulty: Difficulty, seed: u64) -> AiConfig {
    match difficulty {
        Difficulty::Easy => AiConfig::easy(seed),
        Difficulty::Medium => AiConfig::medium(seed),
        Difficulty::Hard => AiConfig::hard(seed),
    }
}
