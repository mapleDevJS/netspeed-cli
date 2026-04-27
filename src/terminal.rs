//! Terminal environment detection and abstraction.
//!
//! This module provides terminal display environment detection functions
//! and a trait abstraction for terminal capabilities.
//!
//! Functions:
//! - [`no_color()`] — Detect if colored output should be disabled (`NO_COLOR` env)
//! - [`no_emoji()`] — Detect if emojis should be disabled (`NO_EMOJI` env)
//! - [`no_animation()`] — Detect if animations should be skipped (`PREFER_REDUCED_MOTION`)
//!
//! Trait:
//! - [`Capabilities`] — Abstraction for terminal display capabilities

/// Detect if [`NO_COLOR`](https://no-color.org/) environment variable is set.
///
/// When set, all colorization should be disabled regardless of theme settings.
/// This follows the standard: <https://no-color.org/>
///
/// # Example
///
/// ```
/// use netspeed_cli::terminal::no_color;
///
/// if no_color() {
///     println!("Plain output");
/// } else {
///     println!("Colorized output");
/// }
/// ```
#[must_use]
pub fn no_color() -> bool {
    std::env::var("NO_COLOR").is_ok()
}

/// Detect if emojis should be disabled.
///
/// Checked via `NO_EMOJI` environment variable, or set programmatically
/// via `--no-emoji` flag in the CLI.
///
/// # Example
///
/// ```
/// use netspeed_cli::terminal::no_emoji;
///
/// let rating = if no_emoji() { "Good" } else { "✅ Good" };
/// ```
#[must_use]
pub fn no_emoji() -> bool {
    std::env::var("NO_EMOJI").is_ok()
}

/// Detect if animations should be skipped.
///
/// Skips intentional-friction delays and spinner animations for users with
/// vestibular disorders. Follows the `PREFER_REDUCED_MOTION` accessibility
/// media query convention.
///
/// # Example
///
/// ```
/// use netspeed_cli::terminal::no_animation;
///
/// if no_animation() {
///     print!("Result: A");
/// } else {
///     // Skip animation for accessibility
///     print!("Result: A (animation skipped)");
/// }
/// ```
#[must_use]
pub fn no_animation() -> bool {
    std::env::var("PREFER_REDUCED_MOTION").is_ok()
}

/// Trait for terminal display capabilities.
///
/// Implement this trait to provide custom terminal display behavior,
/// useful for testing or alternative terminal implementations.
pub trait Capabilities {
    /// Returns true if colored output should be disabled.
    fn is_color_disabled(&self) -> bool;

    /// Returns true if emojis should be disabled.
    fn is_emoji_disabled(&self) -> bool;

    /// Returns true if animations should be skipped.
    fn prefers_reduced_motion(&self) -> bool;
}

/// Default terminal display based on environment variables.
pub struct Env;

impl Capabilities for Env {
    fn is_color_disabled(&self) -> bool {
        no_color()
    }

    fn is_emoji_disabled(&self) -> bool {
        no_emoji()
    }

    fn prefers_reduced_motion(&self) -> bool {
        no_animation()
    }
}

/// Terminal settings resolved at startup.
///
/// Captures terminal capabilities once to avoid repeated env var lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Settings {
    /// Whether colors should be disabled.
    pub no_color: bool,
    /// Whether emojis should be disabled.
    pub no_emoji: bool,
    /// Whether animations should be skipped.
    pub no_animation: bool,
}

impl Settings {
    /// Create terminal settings from current environment.
    ///
    /// This captures the environment state at initialization time.
    /// For testing, construct manually or use `Settings::default()`.
    #[must_use]
    pub fn from_environment() -> Self {
        Self {
            no_color: no_color(),
            no_emoji: no_emoji(),
            no_animation: no_animation(),
        }
    }
}

impl Capabilities for Settings {
    fn is_color_disabled(&self) -> bool {
        self.no_color
    }

    fn is_emoji_disabled(&self) -> bool {
        self.no_emoji
    }

    fn prefers_reduced_motion(&self) -> bool {
        self.no_animation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_color_default() {
        // Just verify it doesn't panic - actual value depends on env
        let _ = no_color();
    }

    #[test]
    fn test_no_emoji_default() {
        let _ = no_emoji();
    }

    #[test]
    fn test_no_animation_default() {
        let _ = no_animation();
    }

    #[test]
    fn test_terminal_settings_default() {
        let settings = Settings::default();
        // Default is all false (unless env vars set)
        assert!(!settings.is_color_disabled() || no_color());
    }

    #[test]
    fn test_terminal_settings_from_environment() {
        let settings = Settings::from_environment();
        assert_eq!(settings.is_color_disabled(), no_color());
        assert_eq!(settings.is_emoji_disabled(), no_emoji());
        assert_eq!(settings.prefers_reduced_motion(), no_animation());
    }

    #[test]
    fn test_default_terminal_trait() {
        let terminal = Env;
        // Just verify trait methods don't panic
        let _ = terminal.is_color_disabled();
        let _ = terminal.is_emoji_disabled();
        let _ = terminal.prefers_reduced_motion();
    }
}
