//! Color theme system for terminal output.
//!
//! Provides theme-aware coloring that adapts to different terminal backgrounds.
//! When a theme is set, all formatters use theme-aware colors instead of
//! hardcoded `green()`, `red()`, `cyan()` calls.
//!
//! ## Note
//!
//! Terminal environment detection (`no_color`) has been moved to the
//! [`crate::terminal`] module.

use owo_colors::OwoColorize;

use crate::terminal;

/// Color theme for terminal output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    /// Default dark terminal theme (bright colors)
    #[default]
    Dark,
    /// Light terminal background (darker colors for readability)
    Light,
    /// High contrast (bold colors, larger visual weight)
    HighContrast,
    /// Monochrome (no colors, bold/italic for emphasis)
    Monochrome,
}

impl Theme {
    /// Parse theme from string.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dark" => Some(Self::Dark),
            "light" => Some(Self::Light),
            "high-contrast" | "highcontrast" => Some(Self::HighContrast),
            "monochrome" | "mono" => Some(Self::Monochrome),
            _ => None,
        }
    }

    /// CLI-friendly name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::HighContrast => "high-contrast",
            Self::Monochrome => "monochrome",
        }
    }
}

/// Theme-aware color wrapper.
/// Use these instead of direct `.green()`, `.red()`, etc. to respect the active theme.
pub struct ThemeColors;

impl ThemeColors {
    /// Good/success color.
    pub fn good(s: &str, theme: Theme) -> String {
        if terminal::no_color() || theme == Theme::Monochrome {
            s.to_string()
        } else {
            match theme {
                Theme::Dark | Theme::HighContrast => s.green().bold().to_string(),
                Theme::Light => s.green().to_string(),
                Theme::Monochrome => s.bold().to_string(),
            }
        }
    }

    /// Warning/caution color.
    pub fn warn(s: &str, theme: Theme) -> String {
        if terminal::no_color() || theme == Theme::Monochrome {
            s.to_string()
        } else {
            match theme {
                Theme::Dark | Theme::HighContrast => s.yellow().bold().to_string(),
                Theme::Light => s.yellow().to_string(),
                Theme::Monochrome => s.italic().to_string(),
            }
        }
    }

    /// Error/bad color.
    pub fn bad(s: &str, theme: Theme) -> String {
        if terminal::no_color() || theme == Theme::Monochrome {
            s.to_string()
        } else {
            match theme {
                Theme::Dark | Theme::HighContrast => s.red().bold().to_string(),
                Theme::Light => s.red().to_string(),
                Theme::Monochrome => s.bold().to_string(),
            }
        }
    }

    /// Info/neutral color (cyan/blue).
    pub fn info(s: &str, theme: Theme) -> String {
        if terminal::no_color() || theme == Theme::Monochrome {
            s.to_string()
        } else {
            match theme {
                Theme::Dark => s.cyan().to_string(),
                Theme::Light => s.blue().to_string(),
                Theme::HighContrast => s.cyan().bold().to_string(),
                Theme::Monochrome => s.italic().to_string(),
            }
        }
    }

    /// Dimmed/secondary text.
    pub fn dimmed(s: &str, theme: Theme) -> String {
        if terminal::no_color() || theme == Theme::Monochrome {
            s.to_string()
        } else {
            match theme {
                Theme::Dark | Theme::Light => s.dimmed().to_string(),
                Theme::HighContrast | Theme::Monochrome => s.to_string(), // Keep readable in high contrast
            }
        }
    }

    /// Bold/emphasized text.
    pub fn bold(s: &str, _theme: Theme) -> String {
        s.bold().to_string()
    }

    /// Muted/secondary text (bright_black equivalent).
    pub fn muted(s: &str, theme: Theme) -> String {
        if terminal::no_color() || theme == Theme::Monochrome {
            s.to_string()
        } else {
            match theme {
                Theme::Dark | Theme::HighContrast => s.bright_black().to_string(),
                Theme::Light => s.dimmed().to_string(),
                Theme::Monochrome => s.to_string(),
            }
        }
    }

    /// Header/section title color.
    pub fn header(s: &str, theme: Theme) -> String {
        if terminal::no_color() || theme == Theme::Monochrome {
            s.to_string()
        } else {
            match theme {
                Theme::Dark | Theme::HighContrast => s.cyan().bold().underline().to_string(),
                Theme::Light => s.blue().bold().underline().to_string(),
                Theme::Monochrome => s.bold().to_string(),
            }
        }
    }
}

/// Resolve the active theme from config, CLI, and environment.
pub fn resolve_theme(config_theme: &str, minimal: bool) -> Theme {
    if minimal || terminal::no_color() {
        return Theme::Monochrome;
    }
    Theme::from_name(config_theme).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_from_name() {
        assert!(Theme::from_name("dark").is_some());
        assert!(Theme::from_name("light").is_some());
        assert!(Theme::from_name("high-contrast").is_some());
        assert!(Theme::from_name("highcontrast").is_some());
        assert!(Theme::from_name("monochrome").is_some());
        assert!(Theme::from_name("mono").is_some());
        assert!(Theme::from_name("invalid").is_none());
    }

    #[test]
    fn test_theme_name_roundtrip() {
        for theme in [
            Theme::Dark,
            Theme::Light,
            Theme::HighContrast,
            Theme::Monochrome,
        ] {
            assert_eq!(Theme::from_name(theme.name()), Some(theme));
        }
    }

    #[test]
    fn test_resolve_theme_minimal() {
        assert_eq!(resolve_theme("dark", true), Theme::Monochrome);
        assert_eq!(resolve_theme("light", true), Theme::Monochrome);
    }

    #[test]
    fn test_resolve_theme_default() {
        assert_eq!(resolve_theme("dark", false), Theme::Dark);
        assert_eq!(resolve_theme("invalid", false), Theme::Dark);
        assert_eq!(resolve_theme("light", false), Theme::Light);
        assert_eq!(resolve_theme("high-contrast", false), Theme::HighContrast);
    }
}
