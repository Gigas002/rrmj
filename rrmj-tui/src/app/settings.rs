#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    Theme,
    DefaultDifficulty,
    HumanSeat,
}

impl SettingsField {
    pub const ALL: [Self; 3] = [Self::Theme, Self::DefaultDifficulty, Self::HumanSeat];

    pub fn next(self) -> Self {
        let fields = Self::ALL;
        let idx = fields.iter().position(|f| *f == self).unwrap_or(0);
        fields[(idx + 1) % fields.len()]
    }

    pub fn prev(self) -> Self {
        let fields = Self::ALL;
        let idx = fields.iter().position(|f| *f == self).unwrap_or(0);
        fields[(idx + fields.len() - 1) % fields.len()]
    }
}
