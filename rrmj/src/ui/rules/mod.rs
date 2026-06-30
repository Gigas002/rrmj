use super::rules_content::{self, CheatLine, LineKind, SectionTone, all_cheat_lines};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme::Theme;
use crate::ui::popup;

#[cfg(test)]
mod tests;

pub fn draw_rules_popup(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let popup = popup::open_popup(frame, area, 98, 96);

    let scroll = app.rules_scroll();
    let visible: Vec<Line> = all_cheat_lines()
        .into_iter()
        .skip(scroll)
        .take(popup.height.saturating_sub(2) as usize)
        .map(|line| style_line(&line, theme))
        .collect();

    let widget = Paragraph::new(visible).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.block_style())
            .title("Riichi Mahjong Cheatsheet"),
    );
    frame.render_widget(widget, popup);
}

fn style_line(line: &CheatLine, theme: &Theme) -> Line<'static> {
    if line.text.is_empty() {
        return Line::from("");
    }

    let base = tone_style(line.tone, theme);
    let style = match line.kind {
        LineKind::Legend | LineKind::Footer => Style::default().fg(theme.muted),
        LineKind::Title => theme
            .title_style()
            .add_modifier(Modifier::UNDERLINED | Modifier::BOLD),
        LineKind::Section => base.add_modifier(Modifier::BOLD),
        LineKind::Subsection => base.add_modifier(Modifier::BOLD | Modifier::ITALIC),
        LineKind::YakuHead => Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
        LineKind::Example => Style::default().fg(theme.accent),
        LineKind::Prose => Style::default().fg(theme.primary),
        LineKind::TableHead => base.add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        LineKind::TableRow => Style::default().fg(theme.score),
    };

    Line::from(Span::styled(line.text.clone(), style))
}

fn tone_style(tone: Option<SectionTone>, theme: &Theme) -> Style {
    let color = match tone {
        Some(SectionTone::Closed) => theme.danger,
        Some(SectionTone::Timing) => theme.riichi,
        Some(SectionTone::Pattern) => theme.safe,
        Some(SectionTone::Yakuman) => theme.dora,
        Some(SectionTone::Dora) => theme.logo,
        Some(SectionTone::Scoring) => theme.score,
        Some(SectionTone::Reference) => theme.border,
        None => theme.primary,
    };
    Style::default().fg(color)
}

pub fn rules_line_count() -> usize {
    rules_content::line_count()
}
