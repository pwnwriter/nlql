// theme support for the tui

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind {
    // default themes
    Dark,
    Light,
    Dracula,
    Nord,
    // catppuccin
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    // rose pine
    RosePine,
    RosePineMoon,
    RosePineDawn,
}

impl ThemeKind {
    pub const ALL: &'static [ThemeKind] = &[
        Self::Dark,
        Self::Light,
        Self::Dracula,
        Self::Nord,
        Self::CatppuccinLatte,
        Self::CatppuccinFrappe,
        Self::CatppuccinMacchiato,
        Self::CatppuccinMocha,
        Self::RosePine,
        Self::RosePineMoon,
        Self::RosePineDawn,
    ];

    pub fn next(self) -> Self {
        let all = Self::ALL;
        let idx = all.iter().position(|&t| t == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    pub fn prev(self) -> Self {
        let all = Self::ALL;
        let idx = all.iter().position(|&t| t == self).unwrap_or(0);
        if idx == 0 {
            all[all.len() - 1]
        } else {
            all[idx - 1]
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Dracula => "dracula",
            Self::Nord => "nord",
            Self::CatppuccinLatte => "catppuccin latte",
            Self::CatppuccinFrappe => "catppuccin frappe",
            Self::CatppuccinMacchiato => "catppuccin macchiato",
            Self::CatppuccinMocha => "catppuccin mocha",
            Self::RosePine => "rose pine",
            Self::RosePineMoon => "rose pine moon",
            Self::RosePineDawn => "rose pine dawn",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub border: Color,
    pub selection: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub muted: Color,
}

impl Theme {
    pub fn from_kind(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Dark => Self::dark(),
            ThemeKind::Light => Self::light(),
            ThemeKind::Dracula => Self::dracula(),
            ThemeKind::Nord => Self::nord(),
            ThemeKind::CatppuccinLatte => Self::catppuccin_latte(),
            ThemeKind::CatppuccinFrappe => Self::catppuccin_frappe(),
            ThemeKind::CatppuccinMacchiato => Self::catppuccin_macchiato(),
            ThemeKind::CatppuccinMocha => Self::catppuccin_mocha(),
            ThemeKind::RosePine => Self::rose_pine(),
            ThemeKind::RosePineMoon => Self::rose_pine_moon(),
            ThemeKind::RosePineDawn => Self::rose_pine_dawn(),
        }
    }

    fn dark() -> Self {
        Self {
            bg: Color::Rgb(20, 20, 30),
            fg: Color::Rgb(220, 220, 230),
            accent: Color::Rgb(100, 150, 255),
            border: Color::Rgb(60, 60, 80),
            selection: Color::Rgb(50, 50, 70),
            error: Color::Rgb(255, 100, 100),
            success: Color::Rgb(100, 255, 150),
            warning: Color::Rgb(255, 200, 100),
            muted: Color::Rgb(120, 120, 140),
        }
    }

    fn light() -> Self {
        Self {
            bg: Color::Rgb(250, 250, 252),
            fg: Color::Rgb(30, 30, 40),
            accent: Color::Rgb(50, 100, 200),
            border: Color::Rgb(200, 200, 210),
            selection: Color::Rgb(230, 240, 255),
            error: Color::Rgb(200, 50, 50),
            success: Color::Rgb(50, 150, 80),
            warning: Color::Rgb(200, 150, 50),
            muted: Color::Rgb(140, 140, 150),
        }
    }

    fn dracula() -> Self {
        Self {
            bg: Color::Rgb(40, 42, 54),
            fg: Color::Rgb(248, 248, 242),
            accent: Color::Rgb(189, 147, 249),
            border: Color::Rgb(68, 71, 90),
            selection: Color::Rgb(68, 71, 90),
            error: Color::Rgb(255, 85, 85),
            success: Color::Rgb(80, 250, 123),
            warning: Color::Rgb(255, 184, 108),
            muted: Color::Rgb(98, 114, 164),
        }
    }

    fn nord() -> Self {
        Self {
            bg: Color::Rgb(46, 52, 64),
            fg: Color::Rgb(236, 239, 244),
            accent: Color::Rgb(136, 192, 208),
            border: Color::Rgb(67, 76, 94),
            selection: Color::Rgb(67, 76, 94),
            error: Color::Rgb(191, 97, 106),
            success: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            muted: Color::Rgb(76, 86, 106),
        }
    }

    // catppuccin latte (light)
    fn catppuccin_latte() -> Self {
        Self {
            bg: Color::Rgb(239, 241, 245),
            fg: Color::Rgb(76, 79, 105),
            accent: Color::Rgb(114, 135, 253),
            border: Color::Rgb(204, 208, 218),
            selection: Color::Rgb(188, 192, 204),
            error: Color::Rgb(210, 15, 57),
            success: Color::Rgb(64, 160, 43),
            warning: Color::Rgb(223, 142, 29),
            muted: Color::Rgb(108, 111, 133),
        }
    }

    // catppuccin frappe
    fn catppuccin_frappe() -> Self {
        Self {
            bg: Color::Rgb(48, 52, 70),
            fg: Color::Rgb(198, 208, 245),
            accent: Color::Rgb(186, 187, 241),
            border: Color::Rgb(65, 69, 89),
            selection: Color::Rgb(81, 87, 109),
            error: Color::Rgb(231, 130, 132),
            success: Color::Rgb(166, 209, 137),
            warning: Color::Rgb(229, 200, 144),
            muted: Color::Rgb(165, 173, 206),
        }
    }

    // catppuccin macchiato
    fn catppuccin_macchiato() -> Self {
        Self {
            bg: Color::Rgb(36, 39, 58),
            fg: Color::Rgb(202, 211, 245),
            accent: Color::Rgb(183, 189, 248),
            border: Color::Rgb(54, 58, 79),
            selection: Color::Rgb(73, 77, 100),
            error: Color::Rgb(237, 135, 150),
            success: Color::Rgb(166, 218, 149),
            warning: Color::Rgb(238, 212, 159),
            muted: Color::Rgb(165, 173, 203),
        }
    }

    // catppuccin mocha
    fn catppuccin_mocha() -> Self {
        Self {
            bg: Color::Rgb(30, 30, 46),
            fg: Color::Rgb(205, 214, 244),
            accent: Color::Rgb(180, 190, 254),
            border: Color::Rgb(49, 50, 68),
            selection: Color::Rgb(69, 71, 90),
            error: Color::Rgb(243, 139, 168),
            success: Color::Rgb(166, 227, 161),
            warning: Color::Rgb(249, 226, 175),
            muted: Color::Rgb(166, 173, 200),
        }
    }

    // rose pine
    fn rose_pine() -> Self {
        Self {
            bg: Color::Rgb(25, 23, 36),
            fg: Color::Rgb(224, 222, 244),
            accent: Color::Rgb(196, 167, 231),
            border: Color::Rgb(38, 35, 58),
            selection: Color::Rgb(57, 53, 82),
            error: Color::Rgb(235, 111, 146),
            success: Color::Rgb(156, 207, 216),
            warning: Color::Rgb(246, 193, 119),
            muted: Color::Rgb(110, 106, 134),
        }
    }

    // rose pine moon
    fn rose_pine_moon() -> Self {
        Self {
            bg: Color::Rgb(35, 33, 54),
            fg: Color::Rgb(224, 222, 244),
            accent: Color::Rgb(196, 167, 231),
            border: Color::Rgb(57, 53, 82),
            selection: Color::Rgb(68, 65, 90),
            error: Color::Rgb(235, 111, 146),
            success: Color::Rgb(156, 207, 216),
            warning: Color::Rgb(246, 193, 119),
            muted: Color::Rgb(110, 106, 134),
        }
    }

    // rose pine dawn (light)
    fn rose_pine_dawn() -> Self {
        Self {
            bg: Color::Rgb(250, 244, 237),
            fg: Color::Rgb(87, 82, 121),
            accent: Color::Rgb(144, 122, 169),
            border: Color::Rgb(242, 233, 225),
            selection: Color::Rgb(223, 218, 217),
            error: Color::Rgb(180, 99, 122),
            success: Color::Rgb(86, 148, 159),
            warning: Color::Rgb(234, 157, 52),
            muted: Color::Rgb(152, 147, 165),
        }
    }

    // style helpers
    pub fn base(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg)
    }

    pub fn accent(&self) -> Style {
        Style::default().fg(self.accent)
    }

    pub fn border(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn selected(&self) -> Style {
        Style::default()
            .bg(self.selection)
            .add_modifier(Modifier::BOLD)
    }

    pub fn error(&self) -> Style {
        Style::default().fg(self.error)
    }

    pub fn success(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn muted(&self) -> Style {
        Style::default().fg(self.muted)
    }

    pub fn title(&self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }
}
