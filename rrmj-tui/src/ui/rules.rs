use super::rules_content;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme::Theme;
use crate::ui::popup;

pub fn draw_rules_popup(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let popup = popup::open_popup(frame, area, 98, 96);

    let scroll = app.rules_scroll();
    let lines = rules_content::all_lines();
    let visible: Vec<Line> = lines
        .iter()
        .skip(scroll)
        .take(popup.height.saturating_sub(2) as usize)
        .map(|line| style_line(line, theme))
        .collect();

    let widget = Paragraph::new(visible).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.block_style())
            .title("Riichi Mahjong Reference"),
    );
    frame.render_widget(widget, popup);
}

fn style_line(line: &str, theme: &Theme) -> Line<'static> {
    if line.is_empty() {
        return Line::from("");
    }
    if line == rules_content::LEGEND {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(theme.muted),
        ));
    }
    if rules_content::SECTIONS
        .iter()
        .any(|(title, _)| *title == line)
    {
        return Line::from(Span::styled(
            line.to_string(),
            theme.status_style().add_modifier(Modifier::BOLD),
        ));
    }
    if line.starts_with("Implemented yaku:") || line.starts_with("Press ?") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(theme.muted),
        ));
    }
    Line::from(Span::styled(
        line.to_string(),
        Style::default().fg(theme.primary),
    ))
}

pub fn rules_line_count() -> usize {
    rules_content::line_count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cheatsheet_has_substantial_content() {
        assert!(rules_line_count() > 80);
    }
}
