use librrmj::action::{Action, KanIntent};

use super::ActionMenu;

#[test]
fn from_legal_includes_kakan_meld_indices() {
    let menu = ActionMenu::from_legal(&[
        Action::Discard(librrmj::tile::Tile::man(1)),
        Action::Kan(KanIntent::Added { meld_index: 0 }),
        Action::Kan(KanIntent::Added { meld_index: 2 }),
    ]);
    assert_eq!(menu.kakan, vec![0, 2]);
}
