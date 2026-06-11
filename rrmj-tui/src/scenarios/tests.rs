use super::*;

#[test]
fn load_scenario_from_path_reads_bundled_fixture() {
    let path = default_scenarios_dir().join("dealer_tsumo.json");
    if !path.exists() {
        return;
    }
    let (entry, recording) = load_scenario_from_path(path.to_str().unwrap()).expect("import");
    assert_eq!(entry.id, "dealer_tsumo");
    recording.validate().unwrap();
}

#[test]
fn load_scenario_from_path_appends_json_extension() {
    let path = default_scenarios_dir().join("dealer_tsumo");
    if !path.with_extension("json").exists() {
        return;
    }
    let (entry, _) = load_scenario_from_path(path.to_str().unwrap()).expect("import");
    assert_eq!(entry.id, "dealer_tsumo");
}

#[test]
fn lists_bundled_scenarios() {
    let dir = default_scenarios_dir();
    if !dir.exists() {
        return;
    }
    let entries = list_scenarios(&dir).expect("list scenarios");
    assert!(!entries.is_empty(), "expected bundled scenario fixtures");
}
