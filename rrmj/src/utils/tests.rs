use super::{DEFAULT_CPU_MS, cycle_cpu, normalize_cpu};

#[test]
fn normalize_cpu_snaps_unknown_to_default() {
    assert_eq!(normalize_cpu(999), DEFAULT_CPU_MS);
    assert_eq!(normalize_cpu(300), 300);
}

#[test]
fn cycle_cpu_wraps_presets() {
    assert_eq!(cycle_cpu(0), 150);
    assert_eq!(cycle_cpu(2000), 0);
}
