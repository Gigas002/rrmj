use librrmj::action::Action;
use librrmj::tile::Tile;

/// Legal actions grouped for menu presentation.
#[derive(Debug, Clone, Default)]
pub struct ActionMenu {
    pub discards: Vec<Tile>,
    pub riichi: Vec<Tile>,
    pub closed_kans: Vec<Tile>,
    pub kakan: Vec<usize>,
    pub chi: Vec<[Tile; 3]>,
    pub can_ron: bool,
    pub can_pon: bool,
    pub can_open_kan: bool,
    pub can_pass: bool,
    pub can_tsumo: bool,
    pub can_abort_nine_terminals: bool,
}

impl ActionMenu {
    pub fn from_legal(legal: &[Action]) -> Self {
        let mut menu = Self::default();
        for &action in legal {
            match action {
                Action::Discard(tile) => menu.discards.push(tile),
                Action::Riichi { discard } => menu.riichi.push(discard),
                Action::ClosedKan { tile } => menu.closed_kans.push(tile),
                Action::Kakan { meld_index } => menu.kakan.push(meld_index),
                Action::Chi { tiles } => menu.chi.push(tiles),
                Action::Ron => menu.can_ron = true,
                Action::Pon => menu.can_pon = true,
                Action::OpenKan => menu.can_open_kan = true,
                Action::Pass => menu.can_pass = true,
                Action::Tsumo => menu.can_tsumo = true,
                Action::AbortiveNineTerminals => menu.can_abort_nine_terminals = true,
                Action::Draw => {}
            }
        }
        menu.discards.sort();
        menu.riichi.sort();
        menu.closed_kans.sort();
        menu.kakan.sort_unstable();
        menu.kakan.dedup();
        menu
    }

    pub fn is_reaction(&self) -> bool {
        self.can_ron || self.can_pon || self.can_open_kan || self.can_pass || !self.chi.is_empty()
    }

    /// Reaction window where the only legal choice is to pass.
    pub fn is_pass_only(&self) -> bool {
        self.can_pass && !self.can_ron && !self.can_pon && !self.can_open_kan && self.chi.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use librrmj::action::Action;

    use super::ActionMenu;

    #[test]
    fn from_legal_includes_kakan_meld_indices() {
        let menu = ActionMenu::from_legal(&[
            Action::Discard(librrmj::tile::Tile::man(1)),
            Action::Kakan { meld_index: 0 },
            Action::Kakan { meld_index: 2 },
        ]);
        assert_eq!(menu.kakan, vec![0, 2]);
    }
}
