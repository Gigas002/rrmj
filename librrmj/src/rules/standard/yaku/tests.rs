use crate::rules::standard::yaku::han_for_yaku;
use crate::scoring::Yaku;

#[test]
fn double_riichi_han_replaces_regular_riichi() {
    assert_eq!(han_for_yaku(Yaku::DoubleRiichi, false), 2);
    assert_eq!(han_for_yaku(Yaku::Riichi, false), 1);
    assert_eq!(han_for_yaku(Yaku::Ippatsu, false), 1);
}

#[test]
fn yakuman_han_values() {
    assert_eq!(han_for_yaku(Yaku::Kokushi, false), 13);
    assert_eq!(han_for_yaku(Yaku::Daisuushii, false), 26);
    assert_eq!(han_for_yaku(Yaku::Suukantsu, true), 13);
}

#[test]
fn open_han_penalty_applies_to_eligible_pattern_yaku() {
    assert_eq!(han_for_yaku(Yaku::Sanshoku, true), 1);
    assert_eq!(han_for_yaku(Yaku::Sanshoku, false), 2);
    assert_eq!(han_for_yaku(Yaku::Chinitsu, true), 5);
    assert_eq!(han_for_yaku(Yaku::Chinitsu, false), 6);
    assert_eq!(han_for_yaku(Yaku::Toitoi, true), 2);
    assert_eq!(han_for_yaku(Yaku::Ryanpeikou, false), 3);
}
