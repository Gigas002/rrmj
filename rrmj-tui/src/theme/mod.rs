use ratatui::style::{Color, Modifier, Style};

/// Presentation palette for the TUI.
#[derive(Debug, Clone)]
pub struct Theme {
    #[allow(dead_code)]
    pub id: &'static str,
    pub label: &'static str,
    pub primary: Color,
    pub accent: Color,
    pub border: Color,
    pub selected: Color,
    pub actor: Color,
    pub riichi: Color,
    pub dora: Color,
    pub danger: Color,
    pub safe: Color,
    pub muted: Color,
    pub red_tile: Color,
    pub score: Color,
    pub logo: Color,
}

impl Theme {
    pub fn resolve(name: &str) -> Self {
        match name {
            "high-contrast" => high_contrast(),
            _ => default(),
        }
    }

    pub fn tile_style(
        &self,
        red: bool,
        selected: bool,
        drawn: bool,
        matched: bool,
        recent_discard: bool,
    ) -> Style {
        let mut style = if red {
            Style::default().fg(self.red_tile)
        } else {
            Style::default().fg(self.primary)
        };
        if recent_discard {
            style = style
                .fg(self.actor)
                .add_modifier(Modifier::UNDERLINED | Modifier::BOLD);
        }
        if drawn {
            style = style
                .fg(self.accent)
                .add_modifier(Modifier::UNDERLINED | Modifier::BOLD);
        }
        if matched || selected {
            style = style
                .fg(self.selected)
                .add_modifier(Modifier::REVERSED | Modifier::BOLD);
        }
        style
    }

    pub fn actor_style(&self, pulsing: bool) -> Style {
        let mut style = Style::default().fg(self.actor).add_modifier(Modifier::BOLD);
        if pulsing {
            style = style.add_modifier(Modifier::RAPID_BLINK);
        }
        style
    }

    pub fn riichi_style(&self, pulsing: bool) -> Style {
        let mut style = Style::default()
            .fg(self.riichi)
            .add_modifier(Modifier::BOLD);
        if pulsing {
            style = style.add_modifier(Modifier::SLOW_BLINK);
        }
        style
    }

    pub fn block_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn menu_selected_style(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn status_style(&self) -> Style {
        Style::default().fg(self.score)
    }

    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted).add_modifier(Modifier::DIM)
    }

    pub fn dora_style(&self) -> Style {
        Style::default().fg(self.dora)
    }
}

pub fn default() -> Theme {
    Theme {
        id: "default",
        label: "Default (dark)",
        primary: Color::White,
        accent: Color::Green,
        border: Color::Cyan,
        selected: Color::Yellow,
        actor: Color::Green,
        riichi: Color::Magenta,
        dora: Color::LightRed,
        danger: Color::Red,
        safe: Color::Blue,
        muted: Color::DarkGray,
        red_tile: Color::Red,
        score: Color::Yellow,
        logo: Color::Yellow,
    }
}

pub fn high_contrast() -> Theme {
    Theme {
        id: "high-contrast",
        label: "High contrast",
        primary: Color::White,
        accent: Color::White,
        border: Color::White,
        selected: Color::Black,
        actor: Color::White,
        riichi: Color::White,
        dora: Color::White,
        danger: Color::White,
        safe: Color::White,
        muted: Color::Gray,
        red_tile: Color::White,
        score: Color::White,
        logo: Color::White,
    }
}
