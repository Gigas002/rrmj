//! Match timer presets (milliseconds). `0` = disabled for turn/response limits.

/// Pause between CPU decisions (presentation pacing).
pub const CPU_PRESETS_MS: &[u64] = &[0, 150, 300, 500, 1000, 2000];

/// Maximum thinking time to choose a discard (draw is automatic).
pub const TURN_PRESETS_MS: &[u64] = &[0, 5_000, 10_000, 15_000, 30_000, 60_000];

/// Maximum time to answer a discard (chi / pon / ron / pass).
pub const RESPONSE_PRESETS_MS: &[u64] = &[0, 1_000, 3_000, 5_000, 10_000, 15_000];

pub const DEFAULT_CPU_MS: u64 = 300;
pub const DEFAULT_TURN_MS: u64 = 30_000;
pub const DEFAULT_RESPONSE_MS: u64 = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerKind {
    Turn,
    Response,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeatTimer {
    pub kind: TimerKind,
    pub remaining_ms: u64,
    pub total_ms: u64,
}

pub fn normalize_cpu(ms: u64) -> u64 {
    normalize_in(ms, CPU_PRESETS_MS, DEFAULT_CPU_MS)
}

pub fn normalize_turn(ms: u64) -> u64 {
    normalize_in(ms, TURN_PRESETS_MS, DEFAULT_TURN_MS)
}

pub fn normalize_response(ms: u64) -> u64 {
    normalize_in(ms, RESPONSE_PRESETS_MS, DEFAULT_RESPONSE_MS)
}

fn normalize_in(ms: u64, presets: &[u64], default: u64) -> u64 {
    if presets.contains(&ms) { ms } else { default }
}

pub fn cycle_cpu(ms: u64) -> u64 {
    cycle_in(ms, CPU_PRESETS_MS)
}

pub fn cycle_turn(ms: u64) -> u64 {
    cycle_in(ms, TURN_PRESETS_MS)
}

pub fn cycle_response(ms: u64) -> u64 {
    cycle_in(ms, RESPONSE_PRESETS_MS)
}

fn cycle_in(ms: u64, presets: &[u64]) -> u64 {
    let idx = presets.iter().position(|&v| v == ms).unwrap_or(0);
    presets[(idx + 1) % presets.len()]
}

pub fn label_cpu(ms: u64) -> String {
    label_ms(ms, "instant")
}

pub fn label_turn(ms: u64) -> String {
    label_ms(ms, "off")
}

pub fn label_response(ms: u64) -> String {
    label_ms(ms, "off")
}

fn label_ms(ms: u64, zero: &str) -> String {
    if ms == 0 {
        zero.into()
    } else if ms >= 1000 && ms.is_multiple_of(1000) {
        format!("{} s", ms / 1000)
    } else {
        format!("{ms} ms")
    }
}

/// Whole seconds remaining (rounded up) for the action bar countdown.
pub fn format_decision_timer(timer: SeatTimer) -> String {
    let secs = timer.remaining_ms.div_ceil(1000);
    let label = match timer.kind {
        TimerKind::Turn => "turn",
        TimerKind::Response => "call",
    };
    format!("{label} {secs}s")
}
