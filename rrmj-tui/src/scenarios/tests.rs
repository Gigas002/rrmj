use tempfile::TempDir;

use super::*;

#[test]
fn load_scenario_from_path_reads_bundled_fixture() {
    let path = bundled_debug_scenarios_dir().join("dealer_tsumo.json");
    if !path.exists() {
        return;
    }
    let (entry, recording) = load_scenario_from_path(path.to_str().unwrap()).expect("import");
    assert_eq!(entry.id, "dealer_tsumo");
    recording.validate().unwrap();
}

#[test]
fn load_scenario_from_path_appends_json_extension() {
    let path = bundled_debug_scenarios_dir().join("dealer_tsumo");
    if !path.with_extension("json").exists() {
        return;
    }
    let (entry, _) = load_scenario_from_path(path.to_str().unwrap()).expect("import");
    assert_eq!(entry.id, "dealer_tsumo");
}

#[test]
fn lists_bundled_debug_scenarios() {
    let dir = bundled_debug_scenarios_dir();
    if !dir.exists() {
        return;
    }
    let entries = list_scenarios(&dir).expect("list scenarios");
    assert!(!entries.is_empty(), "expected bundled scenario fixtures");
}

#[test]
fn list_scenarios_empty_when_dir_missing() {
    let tmp = TempDir::new().unwrap();
    let missing = tmp.path().join("does-not-exist");
    let entries = list_scenarios(&missing).expect("list");
    assert!(entries.is_empty());
}

#[test]
fn scenario_entry_uses_meta_title_and_tags() {
    let text = include_str!("../../../examples/scenarios/match_finished.json");
    let recording = MatchRecording::from_json(text).unwrap();
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("match_finished.json");
    std::fs::write(&path, text).unwrap();
    let entries = list_scenarios(tmp.path()).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title, "Match finished");
    assert_eq!(entries[0].tags, vec!["match-flow"]);
    assert!(entries[0].description.contains("completed"));
    assert_eq!(recording.meta.title.as_deref(), Some("Match finished"));
}
