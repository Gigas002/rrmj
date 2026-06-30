use super::rules_line_count;

#[test]
fn cheatsheet_reexports_line_count() {
    assert!(rules_line_count() > 120);
}
