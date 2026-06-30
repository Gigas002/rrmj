use super::rules_line_count;

#[test]
fn cheatsheet_has_substantial_content() {
    assert!(rules_line_count() > 80);
}
