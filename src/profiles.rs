//! User profiles/roles that customize output based on use case.
//!
//! Each profile adjusts:
//! - Metric scoring weights (what matters most)
//! - Usage check targets (relevant benchmarks)
//! - Output section visibility
//! - Rating thresholds

use serde::{Deserialize, Serialize};

/// Pre-defined user profiles.
///
/// # Example
///
/// ```
/// use netspeed_cli::profiles::UserProfile;
///
/// // Parse from a string name
/// assert_eq!(UserProfile::from_name("gamer"), Some(UserProfile::Gamer));
/// assert_eq!(UserProfile::from_name("streamer"), Some(UserProfile::Streamer));
/// assert_eq!(UserProfile::from_name("invalid"), None);
///
/// // Round-trip: name() → from_name()
/// assert_eq!(UserProfile::from_name(UserProfile::RemoteWorker.name()), Some(UserProfile::RemoteWorker));
///
/// // Default is PowerUser
/// assert_eq!(UserProfile::default(), UserProfile::PowerUser);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UserProfile {
    /// Tech-savvy users who want all metrics and detailed analysis.
    #[default]
    PowerUser,
    /// Online gamers focused on latency, jitter, and bufferbloat.
    Gamer,
    /// Content consumers (Netflix, `YouTube`, etc.) focused on download speed.
    Streamer,
    /// Work-from-home professionals focused on upload and stability.
    RemoteWorker,
    /// Basic users who want a simple pass/fail assessment.
    Casual,
}

impl UserProfile {
    /// Get profile from string name (case-insensitive).
    ///
    /// Returns `Some(UserProfile)` for valid names (including aliases like
    /// `"poweruser"` and `"remote"`), or `None` for unrecognized names.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// // Canonical names
    /// assert_eq!(UserProfile::from_name("power-user"), Some(UserProfile::PowerUser));
    /// assert_eq!(UserProfile::from_name("gamer"), Some(UserProfile::Gamer));
    /// assert_eq!(UserProfile::from_name("streamer"), Some(UserProfile::Streamer));
    /// assert_eq!(UserProfile::from_name("remote-worker"), Some(UserProfile::RemoteWorker));
    /// assert_eq!(UserProfile::from_name("casual"), Some(UserProfile::Casual));
    ///
    /// // Aliases
    /// assert_eq!(UserProfile::from_name("poweruser"), Some(UserProfile::PowerUser));
    /// assert_eq!(UserProfile::from_name("remote"), Some(UserProfile::RemoteWorker));
    ///
    /// // Case-insensitive
    /// assert_eq!(UserProfile::from_name("GAMER"), Some(UserProfile::Gamer));
    /// assert_eq!(UserProfile::from_name("Casual"), Some(UserProfile::Casual));
    ///
    /// // Invalid names return None
    /// assert_eq!(UserProfile::from_name("admin"), None);
    /// ```
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::is_valid_name(name).then_some(Self::from_name_unchecked(name))
    }

    /// Check if a profile name is valid without returning the profile.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// // Canonical names are valid
    /// assert!(UserProfile::is_valid_name("power-user"));
    /// assert!(UserProfile::is_valid_name("gamer"));
    ///
    /// // Aliases are also valid
    /// assert!(UserProfile::is_valid_name("poweruser"));
    /// assert!(UserProfile::is_valid_name("remote"));
    ///
    /// // Case-insensitive
    /// assert!(UserProfile::is_valid_name("GAMER"));
    ///
    /// // Invalid names
    /// assert!(!UserProfile::is_valid_name("admin"));
    /// assert!(!UserProfile::is_valid_name(""));
    /// ```
    #[must_use]
    pub fn is_valid_name(name: &str) -> bool {
        matches!(
            name.to_lowercase().as_str(),
            "power-user"
                | "poweruser"
                | "gamer"
                | "streamer"
                | "remote-worker"
                | "remoteworker"
                | "remote"
                | "casual"
        )
    }

    /// Internal: convert validated name to profile (assumes valid input).
    fn from_name_unchecked(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "power-user" | "poweruser" => Self::PowerUser,
            "gamer" => Self::Gamer,
            "streamer" => Self::Streamer,
            "remote-worker" | "remoteworker" | "remote" => Self::RemoteWorker,
            "casual" => Self::Casual,
            _ => Self::PowerUser, // Safe default
        }
    }

    /// Validate this profile name and return error message if invalid.
    ///
    /// Returns `Ok(())` if valid, `Err(msg)` with the list of valid options if invalid.
    /// Use this for config-file validation where you need an error message;
    /// use [`from_name()`](UserProfile::from_name) if you just need the `UserProfile` value.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// // Valid names pass validation
    /// assert!(UserProfile::validate("power-user").is_ok());
    /// assert!(UserProfile::validate("gamer").is_ok());
    /// assert!(UserProfile::validate("streamer").is_ok());
    /// assert!(UserProfile::validate("remote-worker").is_ok());
    /// assert!(UserProfile::validate("casual").is_ok());
    ///
    /// // Invalid names produce a descriptive error
    /// let err = UserProfile::validate("admin").unwrap_err();
    /// assert!(err.contains("Invalid profile"));
    /// assert!(err.contains("admin"));
    /// assert!(err.contains("gamer"));  // lists valid options
    /// ```
    pub fn validate(name: &str) -> Result<(), String> {
        if Self::is_valid_name(name) {
            Ok(())
        } else {
            Err(format!(
                "Invalid profile '{}'. Valid options: {}",
                name,
                Self::VALID_NAMES.join(", ")
            ))
        }
    }

    /// Type identifier for error messages (DIP: shared validation pattern).
    pub const TYPE_NAME: &'static str = "profile";

    /// List of valid profile names for error messages.
    pub const VALID_NAMES: &'static [&'static str] =
        &["power-user", "gamer", "streamer", "remote-worker", "casual"];

    /// CLI-friendly name for the profile.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// assert_eq!(UserProfile::PowerUser.name(), "power-user");
    /// assert_eq!(UserProfile::Gamer.name(), "gamer");
    /// assert_eq!(UserProfile::Streamer.name(), "streamer");
    /// assert_eq!(UserProfile::RemoteWorker.name(), "remote-worker");
    /// assert_eq!(UserProfile::Casual.name(), "casual");
    /// ```
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::PowerUser => "power-user",
            Self::Gamer => "gamer",
            Self::Streamer => "streamer",
            Self::RemoteWorker => "remote-worker",
            Self::Casual => "casual",
        }
    }

    /// Display name with emoji for headers.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// assert_eq!(UserProfile::PowerUser.display_name(), "⚙️ Power User");
    /// assert_eq!(UserProfile::Gamer.display_name(), "🎮 Gamer");
    /// assert_eq!(UserProfile::Streamer.display_name(), "📺 Streamer");
    /// assert_eq!(UserProfile::RemoteWorker.display_name(), "💼 Remote Worker");
    /// assert_eq!(UserProfile::Casual.display_name(), "👤 Casual");
    /// ```
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::PowerUser => "⚙️ Power User",
            Self::Gamer => "🎮 Gamer",
            Self::Streamer => "📺 Streamer",
            Self::RemoteWorker => "💼 Remote Worker",
            Self::Casual => "👤 Casual",
        }
    }

    /// Description for help text.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// // Each description highlights the profile's focus
    /// assert!(UserProfile::Gamer.description().contains("Latency"));
    /// assert!(UserProfile::Gamer.description().contains("jitter"));
    ///
    /// assert!(UserProfile::Streamer.description().contains("Download"));
    /// assert!(UserProfile::Streamer.description().contains("streaming"));
    ///
    /// assert!(UserProfile::RemoteWorker.description().contains("Upload"));
    /// assert!(UserProfile::RemoteWorker.description().contains("video calls"));
    ///
    /// assert!(UserProfile::Casual.description().contains("pass/fail"));
    ///
    /// assert!(UserProfile::PowerUser.description().contains("All metrics"));
    /// ```
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::PowerUser => "All metrics, historical trends, percentiles, stability analysis",
            Self::Gamer => "Latency, jitter, bufferbloat — optimized for gaming performance",
            Self::Streamer => "Download speed, consistency — optimized for streaming quality",
            Self::RemoteWorker => {
                "Upload speed, stability — optimized for video calls and cloud work"
            }
            Self::Casual => "Simple pass/fail with overall rating only",
        }
    }

    /// Scoring weights for overall connection rating (ping, jitter, download, upload).
    ///
    /// Returns `(ping_weight, jitter_weight, download_weight, upload_weight)`.
    /// Weights always sum to ~1.0, but the distribution reflects each profile's
    /// priorities.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// // Gamer prioritizes latency and jitter
    /// let (ping, jitter, dl, ul) = UserProfile::Gamer.scoring_weights();
    /// assert!(ping > dl, "gamer weights ping over download");
    /// assert!(jitter > ul, "gamer weights jitter over upload");
    ///
    /// // Streamer prioritizes download speed
    /// let (_, _, dl, _) = UserProfile::Streamer.scoring_weights();
    /// assert!(dl >= 0.5, "streamer weights download highest");
    ///
    /// // RemoteWorker prioritizes upload speed
    /// let (_, _, _, ul) = UserProfile::RemoteWorker.scoring_weights();
    /// assert!(ul >= 0.35, "remote-worker weights upload highest");
    ///
    /// // All profiles' weights sum to ~1.0
    /// for profile in [UserProfile::PowerUser, UserProfile::Gamer,
    ///                UserProfile::Streamer, UserProfile::RemoteWorker,
    ///                UserProfile::Casual] {
    ///     let (p, j, d, u) = profile.scoring_weights();
    ///     assert!((p + j + d + u - 1.0).abs() < 0.01,
    ///             "weights must sum to ~1.0 for {profile:?}");
    /// }
    /// ```
    #[must_use]
    pub fn scoring_weights(&self) -> (f64, f64, f64, f64) {
        match self {
            Self::PowerUser => (0.25, 0.20, 0.30, 0.25), // Balanced
            Self::Gamer => (0.40, 0.30, 0.15, 0.15),     // Latency-focused
            Self::Streamer => (0.15, 0.15, 0.55, 0.15),  // Download-focused
            Self::RemoteWorker => (0.20, 0.15, 0.25, 0.40), // Upload-focused
            Self::Casual => (0.25, 0.15, 0.35, 0.25),    // Simplified balanced
        }
    }

    /// Speed rating thresholds for "Excellent" (in Mbps).
    ///
    /// Lower values = easier to achieve. PowerUser demands the highest
    /// bandwidth; Casual is satisfied with the least.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// // PowerUser requires 500 Mbps for "Excellent"
    /// assert_eq!(UserProfile::PowerUser.excellent_speed_threshold(), 500.0);
    ///
    /// // Gamer and RemoteWorker need only 100 Mbps (latency matters more)
    /// assert_eq!(UserProfile::Gamer.excellent_speed_threshold(), 100.0);
    /// assert_eq!(UserProfile::RemoteWorker.excellent_speed_threshold(), 100.0);
    ///
    /// // Streamer needs 200 Mbps (4K streaming headroom)
    /// assert_eq!(UserProfile::Streamer.excellent_speed_threshold(), 200.0);
    ///
    /// // Casual is happy with 50 Mbps
    /// assert_eq!(UserProfile::Casual.excellent_speed_threshold(), 50.0);
    /// ```
    #[must_use]
    pub fn excellent_speed_threshold(&self) -> f64 {
        match self {
            Self::PowerUser => 500.0,
            Self::Gamer | Self::RemoteWorker => 100.0, // Gamers/remote workers don't need massive bandwidth
            Self::Streamer => 200.0, // 4K streaming needs ~50 Mbps, 200 gives headroom
            Self::Casual => 50.0,
        }
    }

    /// Ping rating thresholds for "Excellent" (in ms).
    ///
    /// Lower values = harder to achieve. Gamer demands ultra-low latency;
    /// Casual/Streamer tolerate higher ping.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// // Gamer needs ≤5 ms for "Excellent" ping
    /// assert_eq!(UserProfile::Gamer.excellent_ping_threshold(), 5.0);
    ///
    /// // PowerUser needs ≤10 ms
    /// assert_eq!(UserProfile::PowerUser.excellent_ping_threshold(), 10.0);
    ///
    /// // RemoteWorker tolerates ≤20 ms
    /// assert_eq!(UserProfile::RemoteWorker.excellent_ping_threshold(), 20.0);
    ///
    /// // Streamer and Casual are fine with ≤30 ms
    /// assert_eq!(UserProfile::Streamer.excellent_ping_threshold(), 30.0);
    /// assert_eq!(UserProfile::Casual.excellent_ping_threshold(), 30.0);
    /// ```
    #[must_use]
    pub fn excellent_ping_threshold(&self) -> f64 {
        match self {
            Self::PowerUser => 10.0,
            Self::Gamer => 5.0, // Gamers need ultra-low latency
            Self::RemoteWorker => 20.0,
            Self::Streamer | Self::Casual => 30.0, // Streaming buffers / casual users tolerate higher ping
        }
    }

    /// Jitter rating thresholds for "Excellent" (in ms).
    ///
    /// Lower values = harder to achieve. Gamer needs the most consistent
    /// latency; Casual/Streamer tolerate more variation.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// // Gamer needs ≤1 ms jitter for "Excellent"
    /// assert_eq!(UserProfile::Gamer.excellent_jitter_threshold(), 1.0);
    ///
    /// // PowerUser needs ≤2 ms
    /// assert_eq!(UserProfile::PowerUser.excellent_jitter_threshold(), 2.0);
    ///
    /// // RemoteWorker tolerates ≤3 ms
    /// assert_eq!(UserProfile::RemoteWorker.excellent_jitter_threshold(), 3.0);
    ///
    /// // Streamer and Casual are fine with ≤5 ms
    /// assert_eq!(UserProfile::Streamer.excellent_jitter_threshold(), 5.0);
    /// assert_eq!(UserProfile::Casual.excellent_jitter_threshold(), 5.0);
    /// ```
    #[must_use]
    pub fn excellent_jitter_threshold(&self) -> f64 {
        match self {
            Self::PowerUser => 2.0,
            Self::Gamer => 1.0, // Gamers need consistent latency
            Self::RemoteWorker => 3.0,
            Self::Streamer | Self::Casual => 5.0,
        }
    }

    /// Whether to show detailed latency section.
    ///
    /// All profiles except [`Casual`](UserProfile::Casual) show latency details.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// assert!(UserProfile::PowerUser.show_latency_details());
    /// assert!(UserProfile::Gamer.show_latency_details());
    /// assert!(!UserProfile::Casual.show_latency_details()); // minimal output
    /// ```
    #[must_use]
    pub fn show_latency_details(&self) -> bool {
        !matches!(self, Self::Casual)
    }

    /// Whether to show bufferbloat grade.
    ///
    /// Only [`PowerUser`](UserProfile::PowerUser) and [`Gamer`](UserProfile::Gamer)
    /// care about bufferbloat.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// assert!(UserProfile::PowerUser.show_bufferbloat());
    /// assert!(UserProfile::Gamer.show_bufferbloat());
    /// assert!(!UserProfile::Streamer.show_bufferbloat());
    /// assert!(!UserProfile::Casual.show_bufferbloat());
    /// ```
    #[must_use]
    pub fn show_bufferbloat(&self) -> bool {
        matches!(self, Self::PowerUser | Self::Gamer)
    }

    /// Whether to show stability analysis (CV%).
    ///
    /// [`PowerUser`](UserProfile::PowerUser) and [`RemoteWorker`](UserProfile::RemoteWorker)
    /// need consistent connections for their use cases.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// assert!(UserProfile::PowerUser.show_stability());
    /// assert!(UserProfile::RemoteWorker.show_stability());
    /// assert!(!UserProfile::Gamer.show_stability());
    /// assert!(!UserProfile::Casual.show_stability());
    /// ```
    #[must_use]
    pub fn show_stability(&self) -> bool {
        matches!(self, Self::PowerUser | Self::RemoteWorker)
    }

    /// Whether to show latency percentiles.
    ///
    /// Only [`PowerUser`](UserProfile::PowerUser) sees percentile detail.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// assert!(UserProfile::PowerUser.show_percentiles());
    /// assert!(!UserProfile::Gamer.show_percentiles());
    /// assert!(!UserProfile::Casual.show_percentiles());
    /// ```
    #[must_use]
    pub fn show_percentiles(&self) -> bool {
        matches!(self, Self::PowerUser)
    }

    /// Whether to show usage check targets.
    ///
    /// All profiles except [`Casual`](UserProfile::Casual) show usage checks.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// assert!(UserProfile::PowerUser.show_usage_check());
    /// assert!(UserProfile::Gamer.show_usage_check());
    /// assert!(!UserProfile::Casual.show_usage_check()); // minimal output
    /// ```
    #[must_use]
    pub fn show_usage_check(&self) -> bool {
        !matches!(self, Self::Casual)
    }

    /// Whether to show download time estimates.
    ///
    /// [`PowerUser`](UserProfile::PowerUser) wants all metrics;
    /// [`Casual`](UserProfile::Casual) benefits from practical time estimates.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// assert!(UserProfile::PowerUser.show_estimates());
    /// assert!(UserProfile::Casual.show_estimates());
    /// assert!(!UserProfile::Gamer.show_estimates());
    /// assert!(!UserProfile::Streamer.show_estimates());
    /// ```
    #[must_use]
    pub fn show_estimates(&self) -> bool {
        matches!(self, Self::PowerUser | Self::Casual)
    }

    /// Whether to show historical comparison.
    ///
    /// [`PowerUser`](UserProfile::PowerUser) tracks trends;
    /// [`RemoteWorker`](UserProfile::RemoteWorker) monitors connection reliability over time.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// assert!(UserProfile::PowerUser.show_history());
    /// assert!(UserProfile::RemoteWorker.show_history());
    /// assert!(!UserProfile::Gamer.show_history());
    /// assert!(!UserProfile::Casual.show_history());
    /// ```
    #[must_use]
    pub fn show_history(&self) -> bool {
        matches!(self, Self::PowerUser | Self::RemoteWorker)
    }

    /// Whether to show UL/DL ratio.
    ///
    /// [`PowerUser`](UserProfile::PowerUser) wants all metrics;
    /// [`RemoteWorker`](UserProfile::RemoteWorker) cares about upload relative to download.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// assert!(UserProfile::PowerUser.show_ul_dl_ratio());
    /// assert!(UserProfile::RemoteWorker.show_ul_dl_ratio());
    /// assert!(!UserProfile::Streamer.show_ul_dl_ratio());
    /// assert!(!UserProfile::Casual.show_ul_dl_ratio());
    /// ```
    #[must_use]
    pub fn show_ul_dl_ratio(&self) -> bool {
        matches!(self, Self::PowerUser | Self::RemoteWorker)
    }

    /// Whether to show peak speeds.
    ///
    /// All profiles except [`Casual`](UserProfile::Casual) show peak speeds.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// assert!(UserProfile::PowerUser.show_peaks());
    /// assert!(UserProfile::Gamer.show_peaks());
    /// assert!(!UserProfile::Casual.show_peaks()); // minimal output
    /// ```
    #[must_use]
    pub fn show_peaks(&self) -> bool {
        !matches!(self, Self::Casual)
    }

    /// Whether to show latency under load.
    ///
    /// [`PowerUser`](UserProfile::PowerUser) wants all metrics;
    /// [`Gamer`](UserProfile::Gamer) needs to know if loaded latency spikes.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::UserProfile;
    ///
    /// assert!(UserProfile::PowerUser.show_latency_under_load());
    /// assert!(UserProfile::Gamer.show_latency_under_load());
    /// assert!(!UserProfile::Streamer.show_latency_under_load());
    /// assert!(!UserProfile::Casual.show_latency_under_load());
    /// ```
    #[must_use]
    pub fn show_latency_under_load(&self) -> bool {
        matches!(self, Self::PowerUser | Self::Gamer)
    }
}

/// Profile-specific usage check targets.
///
/// Each target represents a real-world use case (e.g., "4K streaming", "video calls")
/// with the minimum bandwidth required to support it.
///
/// # Example
///
/// ```
/// use netspeed_cli::profiles::{UserProfile, profile_usage_targets};
///
/// let targets = profile_usage_targets(UserProfile::Gamer);
/// assert!(!targets.is_empty());
///
/// // Each target has a name, required bandwidth, and icon
/// let first = &targets[0];
/// assert!(!first.name.is_empty());
/// assert!(first.required_mbps > 0.0);
/// assert!(!first.icon.is_empty());
/// ```
pub struct UsageTarget {
    /// Human-readable name of the use case (e.g., `"4K streaming"`, `"Video calls (1080p)"`).
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::{UserProfile, profile_usage_targets};
    ///
    /// let targets = profile_usage_targets(UserProfile::Streamer);
    /// let four_k = targets.iter().find(|t| t.name.contains("4K")).unwrap();
    /// assert_eq!(four_k.name, "4K streaming");
    ///
    /// let casual = profile_usage_targets(UserProfile::Casual);
    /// assert_eq!(casual[0].name, "Web browsing");
    /// ```
    pub name: &'static str,

    /// Minimum bandwidth in Mbps required for a good experience.
    ///
    /// Always positive. Higher values indicate more demanding use cases.
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::{UserProfile, profile_usage_targets};
    ///
    /// // 4K streaming needs at least 25 Mbps
    /// let streamer = profile_usage_targets(UserProfile::Streamer);
    /// let four_k = streamer.iter().find(|t| t.name.contains("4K")).unwrap();
    /// assert_eq!(four_k.required_mbps, 25.0);
    ///
    /// // Web browsing is lightweight — only 1 Mbps
    /// let casual = profile_usage_targets(UserProfile::Casual);
    /// assert_eq!(casual[0].required_mbps, 1.0);
    ///
    /// // Voice chat is extremely lightweight
    /// let gamer = profile_usage_targets(UserProfile::Gamer);
    /// let voice = gamer.iter().find(|t| t.name.contains("Voice")).unwrap();
    /// assert!(voice.required_mbps < 1.0);
    /// ```
    pub required_mbps: f64,

    /// Emoji icon for visual display in the usage check section.
    ///
    /// Always a single emoji character (e.g., `"📺"`, `"📹"`, `"☁️"`).
    ///
    /// # Example
    ///
    /// ```
    /// use netspeed_cli::profiles::{UserProfile, profile_usage_targets};
    ///
    /// let streamer = profile_usage_targets(UserProfile::Streamer);
    /// let four_k = streamer.iter().find(|t| t.name.contains("4K")).unwrap();
    /// assert_eq!(four_k.icon, "🎬");
    ///
    /// let casual = profile_usage_targets(UserProfile::Casual);
    /// assert_eq!(casual[0].icon, "🌐"); // Web browsing
    ///
    /// // All targets have non-empty icons
    /// for profile in [UserProfile::PowerUser, UserProfile::Gamer,
    ///                UserProfile::Streamer, UserProfile::RemoteWorker,
    ///                UserProfile::Casual] {
    ///     for target in &profile_usage_targets(profile) {
    ///         assert!(!target.icon.is_empty(),
    ///                 "icon must not be empty for {}", target.name);
    ///     }
    /// }
    /// ```
    pub icon: &'static str,
}

/// Get usage check targets for a profile.
///
/// Returns a list of [`UsageTarget`] entries relevant to the profile's use case.
/// Each profile has different targets reflecting its priorities.
///
/// # Example
///
/// ```
/// use netspeed_cli::profiles::{UserProfile, profile_usage_targets};
///
/// // Gamer targets include voice chat and cloud gaming
/// let gamer = profile_usage_targets(UserProfile::Gamer);
/// assert!(gamer.len() >= 3);
/// assert!(gamer.iter().any(|t| t.name.contains("gaming")));
///
/// // Streamer targets include streaming quality levels
/// let streamer = profile_usage_targets(UserProfile::Streamer);
/// assert!(streamer.iter().any(|t| t.name.contains("4K")));
///
/// // RemoteWorker targets include video calls and file uploads
/// let remote = profile_usage_targets(UserProfile::RemoteWorker);
/// assert!(remote.iter().any(|t| t.name.contains("Video calls")));
///
/// // Casual has the fewest targets
/// let casual = profile_usage_targets(UserProfile::Casual);
/// assert!(casual.len() < gamer.len());
///
/// // All profiles have at least one target
/// for profile in [UserProfile::PowerUser, UserProfile::Gamer,
///                UserProfile::Streamer, UserProfile::RemoteWorker,
///                UserProfile::Casual] {
///     assert!(!profile_usage_targets(profile).is_empty());
/// }
/// ```
#[must_use]
pub fn profile_usage_targets(profile: UserProfile) -> Vec<UsageTarget> {
    match profile {
        UserProfile::Gamer => vec![
            UsageTarget {
                name: "Online gaming (1080p)",
                required_mbps: 3.0,
                icon: "🎮",
            },
            UsageTarget {
                name: "Game downloads (50 GB)",
                required_mbps: 100.0,
                icon: "💿",
            },
            UsageTarget {
                name: "Game updates (5 GB)",
                required_mbps: 50.0,
                icon: "🔄",
            },
            UsageTarget {
                name: "Cloud gaming (Stadia)",
                required_mbps: 35.0,
                icon: "☁️",
            },
            UsageTarget {
                name: "Voice chat (Discord)",
                required_mbps: 0.1,
                icon: "🎙️",
            },
        ],
        UserProfile::Streamer => vec![
            UsageTarget {
                name: "SD streaming (480p)",
                required_mbps: 3.0,
                icon: "📺",
            },
            UsageTarget {
                name: "HD streaming (1080p)",
                required_mbps: 5.0,
                icon: "📺",
            },
            UsageTarget {
                name: "4K streaming",
                required_mbps: 25.0,
                icon: "🎬",
            },
            UsageTarget {
                name: "8K streaming",
                required_mbps: 80.0,
                icon: "🎬",
            },
            UsageTarget {
                name: "Multiple streams (3x)",
                required_mbps: 75.0,
                icon: "📺",
            },
        ],
        UserProfile::RemoteWorker => vec![
            UsageTarget {
                name: "Video calls (1080p)",
                required_mbps: 3.0,
                icon: "📹",
            },
            UsageTarget {
                name: "Video calls (4K)",
                required_mbps: 8.0,
                icon: "📹",
            },
            UsageTarget {
                name: "Screen sharing",
                required_mbps: 5.0,
                icon: "🖥️",
            },
            UsageTarget {
                name: "Large file upload",
                required_mbps: 50.0,
                icon: "📤",
            },
            UsageTarget {
                name: "Cloud backup",
                required_mbps: 20.0,
                icon: "☁️",
            },
        ],
        UserProfile::PowerUser => vec![
            UsageTarget {
                name: "Video calls (1080p)",
                required_mbps: 3.0,
                icon: "📹",
            },
            UsageTarget {
                name: "HD streaming",
                required_mbps: 5.0,
                icon: "📺",
            },
            UsageTarget {
                name: "4K streaming",
                required_mbps: 25.0,
                icon: "🎬",
            },
            UsageTarget {
                name: "Cloud gaming",
                required_mbps: 35.0,
                icon: "☁️",
            },
            UsageTarget {
                name: "Large file transfers",
                required_mbps: 100.0,
                icon: "📤",
            },
        ],
        UserProfile::Casual => vec![
            UsageTarget {
                name: "Web browsing",
                required_mbps: 1.0,
                icon: "🌐",
            },
            UsageTarget {
                name: "Email",
                required_mbps: 0.5,
                icon: "📧",
            },
            UsageTarget {
                name: "SD video",
                required_mbps: 3.0,
                icon: "📺",
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_name() {
        assert!(UserProfile::is_valid_name("gamer"));
        assert!(UserProfile::is_valid_name("GAMER"));
        assert!(UserProfile::is_valid_name("power-user"));
        // Aliases
        assert!(UserProfile::is_valid_name("remote")); // alias for remote-worker
        assert!(UserProfile::is_valid_name("poweruser")); // alias without hyphen
        assert!(!UserProfile::is_valid_name("invalid"));
    }

    #[test]
    fn test_validate_valid() {
        assert!(UserProfile::validate("gamer").is_ok());
        assert!(UserProfile::validate("streamer").is_ok());
        assert!(UserProfile::validate("casual").is_ok());
    }

    #[test]
    fn test_validate_invalid() {
        let result = UserProfile::validate("invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid profile"));
        assert!(err.contains("valid"));
    }

    #[test]
    fn test_profile_from_name() {
        assert!(UserProfile::from_name("gamer").is_some());
        assert!(UserProfile::from_name("GAMER").is_some());
        assert!(UserProfile::from_name("streamer").is_some());
        assert!(UserProfile::from_name("remote-worker").is_some());
        assert!(UserProfile::from_name("power-user").is_some());
        assert!(UserProfile::from_name("casual").is_some());
        assert!(UserProfile::from_name("invalid").is_none());
    }

    #[test]
    fn test_profile_name_roundtrip() {
        for profile in [
            UserProfile::PowerUser,
            UserProfile::Gamer,
            UserProfile::Streamer,
            UserProfile::RemoteWorker,
            UserProfile::Casual,
        ] {
            assert_eq!(UserProfile::from_name(profile.name()), Some(profile));
        }
    }

    #[test]
    fn test_scoring_weights_sum() {
        for profile in [
            UserProfile::PowerUser,
            UserProfile::Gamer,
            UserProfile::Streamer,
            UserProfile::RemoteWorker,
            UserProfile::Casual,
        ] {
            let (ping_w, jitter_w, dl_w, ul_w) = profile.scoring_weights();
            assert!(
                (ping_w + jitter_w + dl_w + ul_w - 1.0).abs() < 0.01,
                "Weights should sum to ~1.0 for {profile:?}"
            );
        }
    }

    #[test]
    fn test_gamer_profile_priorities() {
        let gamer = UserProfile::Gamer;
        let (ping_w, jitter_w, dl_w, ul_w) = gamer.scoring_weights();
        assert!(
            ping_w > dl_w,
            "Gamer: ping should weight more than download"
        );
        assert!(
            jitter_w > ul_w,
            "Gamer: jitter should weight more than upload"
        );
        assert!((gamer.excellent_ping_threshold() - 5.0).abs() < f64::EPSILON);
        assert!(gamer.show_bufferbloat());
    }

    #[test]
    fn test_streamer_profile_priorities() {
        let streamer = UserProfile::Streamer;
        let (_, _, dl_w, _) = streamer.scoring_weights();
        assert!(dl_w >= 0.5, "Streamer: download should have highest weight");
        assert!(streamer.show_usage_check());
    }

    #[test]
    fn test_remote_worker_profile_priorities() {
        let remote_worker = UserProfile::RemoteWorker;
        let (_, _, _, ul_w) = remote_worker.scoring_weights();
        assert!(ul_w >= 0.35, "RemoteWorker: upload should have high weight");
        assert!(remote_worker.show_stability());
        assert!(remote_worker.show_ul_dl_ratio());
    }

    #[test]
    fn test_casual_profile_minimal() {
        let casual = UserProfile::Casual;
        assert!(!casual.show_latency_details());
        assert!(!casual.show_bufferbloat());
        assert!(!casual.show_stability());
        assert!(!casual.show_percentiles());
        assert!(!casual.show_history());
        assert!(casual.show_estimates());
    }

    #[test]
    fn test_power_user_shows_all() {
        let power_user = UserProfile::PowerUser;
        assert!(power_user.show_latency_details());
        assert!(power_user.show_bufferbloat());
        assert!(power_user.show_stability());
        assert!(power_user.show_percentiles());
        assert!(power_user.show_history());
    }

    #[test]
    fn test_profile_usage_targets_not_empty() {
        for profile in [
            UserProfile::PowerUser,
            UserProfile::Gamer,
            UserProfile::Streamer,
            UserProfile::RemoteWorker,
            UserProfile::Casual,
        ] {
            let targets = profile_usage_targets(profile);
            assert!(!targets.is_empty(), "{profile:?} should have usage targets");
        }
    }
}
