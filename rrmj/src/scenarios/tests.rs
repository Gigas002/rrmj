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
    let recording = GameRecording::from_json(text).unwrap();
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

#[test]
fn import_copies_into_the_scenarios_dir_without_clobbering() {
    let tmp = tempfile::TempDir::new().unwrap();
    let outside = tmp.path().join("downloads");
    let dir = tmp.path().join("scenarios");
    std::fs::create_dir_all(&outside).unwrap();
    let a = include_str!("../../../examples/scenarios/dealer_tsumo.json");
    let b = include_str!("../../../examples/scenarios/pinfu_ron.json");

    let source = outside.join("hand.json");
    std::fs::write(&source, a).unwrap();
    let copied = super::import_into(&dir, &source).unwrap();
    assert_eq!(copied, dir.join("hand.json"));
    assert_eq!(std::fs::read_to_string(&copied).unwrap(), a);

    // Same content again: no duplicate.
    assert_eq!(super::import_into(&dir, &source).unwrap(), copied);

    // Same name, different content: a new name, the first file untouched.
    std::fs::write(&source, b).unwrap();
    let second = super::import_into(&dir, &source).unwrap();
    assert_eq!(second, dir.join("hand-2.json"));
    assert_eq!(std::fs::read_to_string(&copied).unwrap(), a);

    // A file already in the folder is used where it is.
    assert_eq!(super::import_into(&dir, &copied).unwrap(), copied);
    assert_eq!(super::list_scenarios(&dir).unwrap().len(), 2);
}

#[test]
fn a_broken_scenario_file_does_not_hide_the_others() {
    let tmp = tempfile::TempDir::new().unwrap();
    let dir = tmp.path();
    std::fs::write(
        dir.join("good.json"),
        include_str!("../../../examples/scenarios/dealer_tsumo.json"),
    )
    .unwrap();
    std::fs::write(dir.join("broken.json"), "{ not json").unwrap();
    assert_eq!(super::list_scenarios(dir).unwrap().len(), 1);
}
