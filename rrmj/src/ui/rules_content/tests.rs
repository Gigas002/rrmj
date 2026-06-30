use super::{LineKind, all_cheat_lines, line_count};

#[test]
fn cheatsheet_has_substantial_content() {
    assert!(line_count() > 120);
}

#[test]
fn every_yaku_has_example_combination() {
    let lines = all_cheat_lines();
    let mut saw_yaku = false;
    for window in lines.windows(2) {
        if window[0].kind == LineKind::YakuHead {
            saw_yaku = true;
            assert!(
                window[1].kind == LineKind::Example && window[1].text.starts_with("    ex:"),
                "yaku {:?} missing example line",
                window[0].text
            );
        }
    }
    assert!(saw_yaku, "expected at least one yaku row");
}

#[test]
fn includes_cheatsheet_sections_from_reference_image() {
    let text = all_cheat_lines()
        .into_iter()
        .map(|l| l.text)
        .collect::<Vec<_>>()
        .join("\n");
    for section in [
        "CLOSED HAND REQUIRED",
        "RIICHI & WIN TIMING",
        "YAKUMAN",
        "SCORING — HOW TO CALCULATE",
        "POINT TABLE — DEALER",
        "TILES",
    ] {
        assert!(text.contains(section), "missing section {section}");
    }
}
