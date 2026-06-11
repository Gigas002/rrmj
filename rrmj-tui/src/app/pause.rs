#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseItem {
    Resume,
    ExportSave,
    MainMenu,
    Quit,
}

impl PauseItem {
    pub const ALL: [Self; 4] = [Self::Resume, Self::ExportSave, Self::MainMenu, Self::Quit];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Resume => "Resume",
            Self::ExportSave => "Export save…",
            Self::MainMenu => "Return to main menu",
            Self::Quit => "Quit application",
        }
    }

    pub fn next(self) -> Self {
        let items = Self::ALL;
        let idx = items.iter().position(|i| *i == self).unwrap_or(0);
        items[(idx + 1) % items.len()]
    }

    pub fn prev(self) -> Self {
        let items = Self::ALL;
        let idx = items.iter().position(|i| *i == self).unwrap_or(0);
        items[(idx + items.len() - 1) % items.len()]
    }
}
