use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub text_normal: Color,
    pub text_muted: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub background: Color, // Used sparingly to avoid overriding terminal bg
}

impl Default for Theme {
    fn default() -> Self {
        Theme::parse("default")
    }
}

impl Theme {
    pub fn parse(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "dracula" => Theme {
                primary: Color::Rgb(189, 147, 249),     // Purple
                secondary: Color::Rgb(255, 121, 198),   // Pink
                text_normal: Color::Rgb(248, 248, 242), // White
                text_muted: Color::Rgb(98, 114, 164),   // Comment
                success: Color::Rgb(80, 250, 123),      // Green
                warning: Color::Rgb(241, 250, 140),     // Yellow
                error: Color::Rgb(255, 85, 85),         // Red
                info: Color::Rgb(139, 233, 253),        // Cyan
                background: Color::Black,
            },
            "nord" => Theme {
                primary: Color::Rgb(136, 192, 208),     // Frost Light Blue
                secondary: Color::Rgb(129, 161, 193),   // Frost Blue
                text_normal: Color::Rgb(216, 222, 233), // Snow Storm Light
                text_muted: Color::Rgb(76, 86, 106),    // Polar Night Light
                success: Color::Rgb(163, 190, 140),     // Aurora Green
                warning: Color::Rgb(235, 203, 139),     // Aurora Yellow
                error: Color::Rgb(191, 97, 106),        // Aurora Red
                info: Color::Rgb(143, 188, 187),        // Frost Cyan
                background: Color::Rgb(46, 52, 64),     // Polar Night
            },
            "monokai" => Theme {
                primary: Color::Rgb(253, 151, 31),      // Orange
                secondary: Color::Rgb(102, 217, 239),   // Blue
                text_normal: Color::Rgb(248, 248, 242), // White
                text_muted: Color::Rgb(117, 113, 94),   // Comment
                success: Color::Rgb(166, 226, 46),      // Green
                warning: Color::Rgb(230, 219, 116),     // Yellow
                error: Color::Rgb(249, 38, 114),        // Pink/Red
                info: Color::Rgb(174, 129, 255),        // Purple
                background: Color::Rgb(39, 40, 34),     // Monokai bg
            },
            "solarized-dark" => Theme {
                primary: Color::Rgb(38, 139, 210),      // Blue
                secondary: Color::Rgb(42, 161, 152),    // Cyan
                text_normal: Color::Rgb(131, 148, 150), // Base0
                text_muted: Color::Rgb(88, 110, 117),   // Base01
                success: Color::Rgb(133, 153, 0),       // Green
                warning: Color::Rgb(181, 137, 0),       // Yellow
                error: Color::Rgb(220, 50, 47),         // Red
                info: Color::Rgb(108, 113, 196),        // Violet
                background: Color::Rgb(0, 43, 54),      // Base03
            },
            "oceanic" => Theme {
                primary: Color::Rgb(102, 153, 204),     // Blue
                secondary: Color::Rgb(197, 148, 197),   // Purple
                text_normal: Color::Rgb(216, 222, 233), // White
                text_muted: Color::Rgb(101, 115, 126),  // Grey
                success: Color::Rgb(153, 199, 148),     // Green
                warning: Color::Rgb(250, 200, 99),      // Yellow
                error: Color::Rgb(236, 95, 103),        // Red
                info: Color::Rgb(95, 187, 175),         // Cyan
                background: Color::Rgb(40, 44, 52),     // Material bg
            },
            "catppuccin-mocha" => Theme {
                primary: Color::Rgb(203, 166, 247),     // Mauve
                secondary: Color::Rgb(245, 194, 231),   // Pink
                text_normal: Color::Rgb(205, 214, 244), // Text
                text_muted: Color::Rgb(147, 153, 178),  // Overlay0
                success: Color::Rgb(166, 227, 161),     // Green
                warning: Color::Rgb(249, 226, 175),     // Yellow
                error: Color::Rgb(243, 139, 168),       // Red
                info: Color::Rgb(137, 220, 235),        // Sky
                background: Color::Rgb(30, 30, 46),     // Base
            },
            "catppuccin-latte" => Theme {
                primary: Color::Rgb(136, 57, 239),     // Mauve
                secondary: Color::Rgb(234, 118, 203),  // Pink
                text_normal: Color::Rgb(76, 79, 105),  // Text
                text_muted: Color::Rgb(156, 160, 176), // Overlay0
                success: Color::Rgb(64, 160, 43),      // Green
                warning: Color::Rgb(223, 142, 29),     // Yellow
                error: Color::Rgb(210, 15, 57),        // Red
                info: Color::Rgb(4, 165, 229),         // Sky
                background: Color::Rgb(239, 241, 245), // Base
            },
            "gruvbox-dark" => Theme {
                primary: Color::Rgb(214, 153, 77),      // Orange
                secondary: Color::Rgb(177, 98, 134),    // Purple
                text_normal: Color::Rgb(235, 219, 178), // Fg4
                text_muted: Color::Rgb(146, 131, 116),  // Gray
                success: Color::Rgb(184, 187, 38),      // Green
                warning: Color::Rgb(215, 153, 33),      // Yellow
                error: Color::Rgb(251, 73, 52),         // Red
                info: Color::Rgb(104, 157, 106),        // Aqua
                background: Color::Rgb(40, 40, 40),     // Bg0
            },
            "gruvbox-light" => Theme {
                primary: Color::Rgb(214, 153, 77),     // Orange
                secondary: Color::Rgb(177, 98, 134),   // Purple
                text_normal: Color::Rgb(60, 56, 54),   // Fg
                text_muted: Color::Rgb(146, 131, 116), // Gray
                success: Color::Rgb(184, 187, 38),     // Green
                warning: Color::Rgb(215, 153, 33),     // Yellow
                error: Color::Rgb(251, 73, 52),        // Red
                info: Color::Rgb(104, 157, 106),       // Aqua
                background: Color::Rgb(251, 241, 199), // Bg0 light
            },
            "tokyo-night" => Theme {
                primary: Color::Rgb(122, 162, 247),     // Blue
                secondary: Color::Rgb(187, 154, 247),   // Purple
                text_normal: Color::Rgb(169, 177, 214), // Foreground
                text_muted: Color::Rgb(86, 95, 137),    // Comment
                success: Color::Rgb(158, 206, 106),     // Green
                warning: Color::Rgb(224, 175, 104),     // Yellow
                error: Color::Rgb(247, 118, 142),       // Red
                info: Color::Rgb(125, 207, 255),        // Cyan
                background: Color::Rgb(26, 27, 38),     // Bg
            },
            "rose-pine" => Theme {
                primary: Color::Rgb(196, 167, 231),     // Iris
                secondary: Color::Rgb(235, 188, 186),   // Rose
                text_normal: Color::Rgb(224, 222, 244), // Text
                text_muted: Color::Rgb(110, 106, 134),  // Muted
                success: Color::Rgb(156, 207, 216),     // Foam
                warning: Color::Rgb(246, 193, 119),     // Gold
                error: Color::Rgb(235, 111, 146),       // Love
                info: Color::Rgb(49, 116, 143),         // Pine
                background: Color::Rgb(25, 23, 36),     // Base
            },
            "light" => Theme {
                primary: Color::Blue,
                secondary: Color::Magenta,
                text_normal: Color::Black,
                text_muted: Color::DarkGray,
                success: Color::Green,
                warning: Color::Yellow,
                error: Color::Red,
                info: Color::Cyan,
                background: Color::White,
            },
            "dark" => Theme {
                primary: Color::Cyan,
                secondary: Color::Magenta,
                text_normal: Color::White,
                text_muted: Color::DarkGray,
                success: Color::Green,
                warning: Color::Yellow,
                error: Color::Red,
                info: Color::Blue,
                background: Color::Black,
            },
            // Default
            _ => Theme {
                primary: Color::Cyan,
                secondary: Color::Blue,
                text_normal: Color::White,
                text_muted: Color::DarkGray,
                success: Color::Green,
                warning: Color::Yellow,
                error: Color::Red,
                info: Color::LightCyan,
                background: Color::Black,
            },
        }
    }
}
