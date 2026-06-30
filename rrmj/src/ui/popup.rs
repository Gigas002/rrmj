use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Clear;

/// Centered rectangle with absolute pixel width and height (clamped to `area`).
pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.clamp(1, area.width.max(1));
    let height = height.clamp(1, area.height.max(1));
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(width),
            Constraint::Fill(1),
        ])
        .split(vertical[1])[1]
}

/// Centered rectangle sized as a percentage of `area`.
pub fn centered_rect_percent(width_pct: u16, height_pct: u16, area: Rect) -> Rect {
    centered_rect(
        area.width.saturating_mul(width_pct) / 100,
        area.height.saturating_mul(height_pct) / 100,
        area,
    )
}

/// Clear and return the centered popup bounds.
pub fn open_popup(frame: &mut ratatui::Frame, area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let popup = centered_rect_percent(width_pct, height_pct, area);
    frame.render_widget(Clear, popup);
    popup
}
