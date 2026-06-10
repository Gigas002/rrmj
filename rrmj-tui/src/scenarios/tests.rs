use super::*;

#[test]
fn lists_bundled_scenarios() {
    let dir = default_scenarios_dir();
    if !dir.exists() {
        return;
    }
    let entries = list_scenarios(&dir).expect("list scenarios");
    assert!(!entries.is_empty(), "expected bundled scenario fixtures");
}
