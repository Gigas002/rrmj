#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    Theme,
    DefaultDifficulty,
    HumanSeat,
    CpuStepDelay,
    TurnTimer,
    ResponseTimer,
}

impl SettingsField {
    pub const ALL: [Self; 6] = [
        Self::Theme,
        Self::DefaultDifficulty,
        Self::HumanSeat,
        Self::CpuStepDelay,
        Self::TurnTimer,
        Self::ResponseTimer,
    ];

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
