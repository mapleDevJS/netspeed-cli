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
///
/// # Example
///
/// ```
/// use netspeed_cli::theme::Theme;
///
/// // Parse from a string name
/// assert_eq!(Theme::from_name("dark"), Some(Theme::Dark));
/// assert_eq!(Theme::from_name("light"), Some(Theme::Light));
/// assert_eq!(Theme::from_name("invalid"), None);
///
/// // Round-trip: name() → from_name()
/// assert_eq!(Theme::from_name(Theme::HighContrast.name()), Some(Theme::HighContrast));
///
/// // Default is Dark
/// assert_eq!(Theme::default(), Theme::Dark);
/// ```
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
    ///
    /// Returns `Some(Theme)` for valid names (including aliases like `"mono"`
    /// and `"highcontrast"`), or `None` for unrecognized names.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::theme::Theme;
    ///
    /// // Canonical names
    /// assert_eq!(Theme::from_name("dark"), Some(Theme::Dark));
    /// assert_eq!(Theme::from_name("light"), Some(Theme::Light));
    /// assert_eq!(Theme::from_name("high-contrast"), Some(Theme::HighContrast));
    /// assert_eq!(Theme::from_name("monochrome"), Some(Theme::Monochrome));
    ///
    /// // Aliases
    /// assert_eq!(Theme::from_name("mono"), Some(Theme::Monochrome));
    /// assert_eq!(Theme::from_name("highcontrast"), Some(Theme::HighContrast));
    ///
    /// // Case-insensitive
    /// assert_eq!(Theme::from_name("DARK"), Some(Theme::Dark));
    /// assert_eq!(Theme::from_name("Light"), Some(Theme::Light));
    ///
    /// // Invalid names return None
    /// assert_eq!(Theme::from_name("solarized"), None);
    /// ```
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::is_valid_name(name).then_some(Self::from_name_unchecked(name))
    }

    /// Check if a theme name is valid without returning the theme.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::theme::Theme;
    ///
    /// // Canonical names are valid
    /// assert!(Theme::is_valid_name("dark"));
    /// assert!(Theme::is_valid_name("high-contrast"));
    ///
    /// // Aliases are also valid
    /// assert!(Theme::is_valid_name("mono"));
    /// assert!(Theme::is_valid_name("highcontrast"));
    ///
    /// // Case-insensitive
    /// assert!(Theme::is_valid_name("DARK"));
    ///
    /// // Invalid names
    /// assert!(!Theme::is_valid_name("neon"));
    /// assert!(!Theme::is_valid_name(""));
    /// ```
    #[must_use]
    pub fn is_valid_name(name: &str) -> bool {
        matches!(
            name.to_lowercase().as_str(),
            "dark" | "light" | "high-contrast" | "highcontrast" | "monochrome" | "mono"
        )
    }

    /// Internal: convert validated name to theme (assumes valid input).
    fn from_name_unchecked(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "dark" => Self::Dark,
            "light" => Self::Light,
            "high-contrast" | "highcontrast" => Self::HighContrast,
            "monochrome" | "mono" => Self::Monochrome,
            _ => Self::Dark, // Safe default
        }
    }

    /// Validate this theme name and return error message if invalid.
    ///
    /// Returns `Ok(())` if valid, `Err(msg)` with the list of valid options if invalid.
    /// Use this for config-file validation where you need an error message;
    /// use [`from_name()`](Theme::from_name) if you just need the `Theme` value.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::theme::Theme;
    ///
    /// // Valid names pass validation
    /// assert!(Theme::validate("dark").is_ok());
    /// assert!(Theme::validate("light").is_ok());
    /// assert!(Theme::validate("high-contrast").is_ok());
    /// assert!(Theme::validate("monochrome").is_ok());
    ///
    /// // Invalid names produce a descriptive error
    /// let err = Theme::validate("neon").unwrap_err();
    /// assert!(err.contains("Invalid theme"));
    /// assert!(err.contains("neon"));
    /// assert!(err.contains("dark"));  // lists valid options
    /// ```
    pub fn validate(name: &str) -> Result<(), String> {
        if Self::is_valid_name(name) {
            Ok(())
        } else {
            Err(format!(
                "Invalid theme '{}'. Valid options: {}",
                name,
                Self::VALID_NAMES.join(", ")
            ))
        }
    }

    /// Type identifier for error messages (DIP: shared validation pattern).
    pub const TYPE_NAME: &'static str = "theme";

    /// List of valid theme names for error messages.
    pub const VALID_NAMES: &'static [&'static str] =
        &["dark", "light", "high-contrast", "monochrome"];

    /// CLI-friendly name.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::theme::Theme;
    ///
    /// assert_eq!(Theme::Dark.name(), "dark");
    /// assert_eq!(Theme::Light.name(), "light");
    /// assert_eq!(Theme::HighContrast.name(), "high-contrast");
    /// assert_eq!(Theme::Monochrome.name(), "monochrome");
    /// ```
    #[must_use]
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
///
/// Use these instead of direct `.green()`, `.red()`, etc. to respect the active theme.
/// Each method takes a string and a [`Theme`], returning a styled string that
/// adapts to the theme's color palette.
///
/// # Example
///
/// ```
/// use netspeed_cli::theme::{Colors, Theme};
///
/// // Monochrome always returns plain text (no ANSI escapes)
/// assert_eq!(Colors::good("OK", Theme::Monochrome), "OK");
/// assert_eq!(Colors::warn("caution", Theme::Monochrome), "caution");
/// assert_eq!(Colors::bad("FAIL", Theme::Monochrome), "FAIL");
/// assert_eq!(Colors::info("note", Theme::Monochrome), "note");
///
/// // Other themes add ANSI styling but always preserve the original text
/// assert!(Colors::good("OK", Theme::Dark).contains("OK"));
/// assert!(Colors::bad("FAIL", Theme::Light).contains("FAIL"));
/// ```
pub struct Colors;

impl Colors {
    /// Good/success color.
    ///
    /// Green in Dark/HighContrast/Light themes, plain text in Monochrome.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::theme::{Colors, Theme};
    ///
    /// // Monochrome: plain text
    /// assert_eq!(Colors::good("100 Mbps", Theme::Monochrome), "100 Mbps");
    ///
    /// // Dark/Light/HighContrast: styled with green (contains the text)
    /// assert!(Colors::good("100 Mbps", Theme::Dark).contains("100 Mbps"));
    /// assert!(Colors::good("100 Mbps", Theme::Light).contains("100 Mbps"));
    /// ```
    #[must_use]
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
    ///
    /// Yellow in Dark/HighContrast/Light themes, plain text in Monochrome.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::theme::{Colors, Theme};
    ///
    /// assert_eq!(Colors::warn("high latency", Theme::Monochrome), "high latency");
    /// assert!(Colors::warn("high latency", Theme::Dark).contains("high latency"));
    /// ```
    #[must_use]
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
    ///
    /// Red in Dark/HighContrast/Light themes, plain text in Monochrome.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::theme::{Colors, Theme};
    ///
    /// assert_eq!(Colors::bad("FAILED", Theme::Monochrome), "FAILED");
    /// assert!(Colors::bad("FAILED", Theme::Dark).contains("FAILED"));
    /// ```
    #[must_use]
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
    ///
    /// Cyan in Dark/HighContrast, blue in Light, plain text in Monochrome.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::theme::{Colors, Theme};
    ///
    /// assert_eq!(Colors::info("Server: 1234", Theme::Monochrome), "Server: 1234");
    /// assert!(Colors::info("Server: 1234", Theme::Dark).contains("Server: 1234"));
    /// assert!(Colors::info("Server: 1234", Theme::Light).contains("Server: 1234"));
    /// ```
    #[must_use]
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
    ///
    /// Dimmed in Dark/Light, plain text in HighContrast/Monochrome
    /// (kept readable at high contrast).
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::theme::{Colors, Theme};
    ///
    /// // Monochrome and HighContrast: plain text (readability over style)
    /// assert_eq!(Colors::dimmed("secondary", Theme::Monochrome), "secondary");
    /// assert_eq!(Colors::dimmed("secondary", Theme::HighContrast), "secondary");
    ///
    /// // Dark/Light: dimmed styling (contains the text)
    /// assert!(Colors::dimmed("secondary", Theme::Dark).contains("secondary"));
    /// assert!(Colors::dimmed("secondary", Theme::Light).contains("secondary"));
    /// ```
    #[must_use]
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
    ///
    /// Always applies bold regardless of theme (theme parameter reserved
    /// for future theme-specific bold behavior).
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::theme::{Colors, Theme};
    ///
    /// // Bold always contains the original text
    /// assert!(Colors::bold("important", Theme::Dark).contains("important"));
    /// assert!(Colors::bold("important", Theme::Light).contains("important"));
    /// assert!(Colors::bold("important", Theme::Monochrome).contains("important"));
    /// ```
    #[must_use]
    pub fn bold(s: &str, _theme: Theme) -> String {
        s.bold().to_string()
    }

    /// Muted/secondary text (`bright_black` equivalent).
    ///
    /// Bright black in Dark/HighContrast, dimmed in Light, plain in Monochrome.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::theme::{Colors, Theme};
    ///
    /// // Monochrome: plain text
    /// assert_eq!(Colors::muted("hint", Theme::Monochrome), "hint");
    ///
    /// // Other themes contain the original text
    /// assert!(Colors::muted("hint", Theme::Dark).contains("hint"));
    /// assert!(Colors::muted("hint", Theme::Light).contains("hint"));
    /// assert!(Colors::muted("hint", Theme::HighContrast).contains("hint"));
    /// ```
    #[must_use]
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
    ///
    /// Cyan+bold+underline in Dark/HighContrast, blue+bold+underline in Light,
    /// plain text in Monochrome (no color/underline).
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::theme::{Colors, Theme};
    ///
    /// // Monochrome: plain text (no color/underline)
    /// assert!(Colors::header("Results", Theme::Monochrome).contains("Results"));
    ///
    /// // Dark: cyan + bold + underline
    /// assert!(Colors::header("Results", Theme::Dark).contains("Results"));
    ///
    /// // Light: blue + bold + underline
    /// assert!(Colors::header("Results", Theme::Light).contains("Results"));
    /// ```
    #[must_use]
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
///
/// Priority order:
/// 1. **`minimal=true`** → always [`Monochrome`](Theme::Monochrome)
/// 2. **`NO_COLOR` env var set** → [`Monochrome`](Theme::Monochrome)
/// 3. **Valid `config_theme`** → the matching [`Theme`]
/// 4. **Invalid `config_theme`** → [`Dark`](Theme::Dark) (default)
///
/// # Example
///
/// ```
/// use netspeed_cli::theme::{Theme, resolve};
///
/// // minimal=true always forces Monochrome (deterministic)
/// assert_eq!(resolve("dark", true), Theme::Monochrome);
/// assert_eq!(resolve("light", true), Theme::Monochrome);
/// assert_eq!(resolve("high-contrast", true), Theme::Monochrome);
/// ```
///
/// ```ignore
/// // These depend on NO_COLOR not being set in the environment:
/// assert_eq!(resolve("dark", false), Theme::Dark);
/// assert_eq!(resolve("light", false), Theme::Light);
/// assert_eq!(resolve("high-contrast", false), Theme::HighContrast);
/// assert_eq!(resolve("monochrome", false), Theme::Monochrome);
///
/// // Invalid theme falls back to Dark (the default)
/// assert_eq!(resolve("invalid", false), Theme::Dark);
/// ```
#[must_use]
pub fn resolve(config_theme: &str, minimal: bool) -> Theme {
    if minimal || terminal::no_color() {
        return Theme::Monochrome;
    }
    Theme::from_name(config_theme).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_name() {
        assert!(Theme::is_valid_name("dark"));
        assert!(Theme::is_valid_name("DARK"));
        assert!(Theme::is_valid_name("high-contrast"));
        assert!(Theme::is_valid_name("monochrome"));
        // Aliases
        assert!(Theme::is_valid_name("mono")); // alias for monochrome
        assert!(Theme::is_valid_name("highcontrast")); // alias without hyphen
        assert!(!Theme::is_valid_name("invalid"));
    }

    #[test]
    fn test_validate_valid() {
        assert!(Theme::validate("dark").is_ok());
        assert!(Theme::validate("light").is_ok());
        assert!(Theme::validate("high-contrast").is_ok());
    }

    #[test]
    fn test_validate_invalid() {
        let result = Theme::validate("invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid theme"));
        assert!(err.contains("valid"));
    }

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
        assert_eq!(resolve("dark", true), Theme::Monochrome);
        assert_eq!(resolve("light", true), Theme::Monochrome);
    }

    #[test]
    fn test_resolve_theme_default() {
        assert_eq!(resolve("dark", false), Theme::Dark);
        assert_eq!(resolve("invalid", false), Theme::Dark);
        assert_eq!(resolve("light", false), Theme::Light);
        assert_eq!(resolve("high-contrast", false), Theme::HighContrast);
    }
}
