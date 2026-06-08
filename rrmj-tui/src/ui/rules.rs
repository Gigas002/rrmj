use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;
use crate::theme::Theme;
use crate::ui::popup;

const RULES_LINES: &[&str] = &[
    "Rules / yaku reference (standard profile)",
    "",
    "Winning",
    "  • Standard: four melds + pair, or seven pairs (menzen).",
    "  • Tsumo — self-draw on your turn after drawing.",
    "  • Ron — win on another player's discard in reaction.",
    "  • Tenpai — one tile away from winning (riichi / exhaustive draw).",
    "  • Furiten — cannot ron on a tile you already discarded.",
    "",
    "Yaku (v0)",
    "  Riichi          1 han   menzen, tenpai, 1,000 stick",
    "  Menzen tsumo    1 han   closed self-draw",
    "  Tanyao          1 han   all simples (2–8)",
    "  Pinfu           1 han   menzen sequences, non-yakuhai wait",
    "  Yakuhai         1 han   seat wind, round wind, or dragon pon",
    "",
    "Dora",
    "  Indicator tiles add han; ura dora after riichi win;",
    "  kan reveals another indicator; aka dora counts red fives.",
    "",
    "Riichi",
    "  Menzen + tenpai + 1,000 points; stick on table until won.",
    "",
    "Exhaustive draw",
    "  Tenpai players split 3,000 from noten; no pay if all same.",
    "",
    "Match flow",
    "  Honba sticks on dealer tenpai/win; renchan keeps dealer;",
    "  dealer rotates on child win or dealer noten at exhaustive.",
    "",
    "Abortive draws (when enabled)",
    "  Nine terminals, four winds, four kongs, four riichis.",
    "",
    "Press ? or y to close. ↑/↓ scroll.",
];

pub fn draw_rules_popup(frame: &mut ratatui::Frame, area: Rect, app: &App, theme: &Theme) {
    let popup = popup::open_popup(frame, area, 85, 80);

    let scroll = app.rules_scroll();
    let visible: Vec<Line> = RULES_LINES
        .iter()
        .skip(scroll)
        .take(popup.height.saturating_sub(2) as usize)
        .map(|line| Line::from(*line))
        .collect();

    let widget = Paragraph::new(visible)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.block_style())
                .title("Rules reference"),
        );
    frame.render_widget(widget, popup);
}

pub fn rules_line_count() -> usize {
    RULES_LINES.len()
}
