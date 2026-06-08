use librrmj::hand::MeldKind;
use librrmj::tile::Tile;

pub fn tile_label(tile: Tile) -> String {
    tile.to_string()
}

pub fn meld_kind_label(kind: MeldKind) -> &'static str {
    match kind {
        MeldKind::Chi => "chi",
        MeldKind::Pon => "pon",
        MeldKind::OpenKan => "minkan",
        MeldKind::ClosedKan => "ankan",
        MeldKind::AddedKan => "kakan",
    }
}
