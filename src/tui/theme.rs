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
