//! User profiles/roles that customize output based on use case.
//!
//! Each profile adjusts:
//! - Metric scoring weights (what matters most)
//! - Usage check targets (relevant benchmarks)
//! - Output section visibility
//! - Rating thresholds

use serde::{Deserialize, Serialize};

/// Pre-defined user profiles.
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
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "power-user" | "poweruser" => Some(Self::PowerUser),
            "gamer" => Some(Self::Gamer),
            "streamer" => Some(Self::Streamer),
            "remote-worker" | "remoteworker" | "remote" => Some(Self::RemoteWorker),
            "casual" => Some(Self::Casual),
            _ => None,
        }
    }

    /// CLI-friendly name for the profile.
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
    /// Returns (`ping_weight`, `jitter_weight`, `download_weight`, `upload_weight`).
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
    /// Lower values = easier to achieve.
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
    #[must_use]
    pub fn show_latency_details(&self) -> bool {
        !matches!(self, Self::Casual)
    }

    /// Whether to show bufferbloat grade.
    #[must_use]
    pub fn show_bufferbloat(&self) -> bool {
        matches!(self, Self::PowerUser | Self::Gamer)
    }

    /// Whether to show stability analysis (CV%).
    #[must_use]
    pub fn show_stability(&self) -> bool {
        matches!(self, Self::PowerUser | Self::RemoteWorker)
    }

    /// Whether to show latency percentiles.
    #[must_use]
    pub fn show_percentiles(&self) -> bool {
        matches!(self, Self::PowerUser)
    }

    /// Whether to show usage check targets.
    #[must_use]
    pub fn show_usage_check(&self) -> bool {
        !matches!(self, Self::Casual)
    }

    /// Whether to show download time estimates.
    #[must_use]
    pub fn show_estimates(&self) -> bool {
        matches!(self, Self::PowerUser | Self::Casual)
    }

    /// Whether to show historical comparison.
    #[must_use]
    pub fn show_history(&self) -> bool {
        matches!(self, Self::PowerUser | Self::RemoteWorker)
    }

    /// Whether to show UL/DL ratio.
    #[must_use]
    pub fn show_ul_dl_ratio(&self) -> bool {
        matches!(self, Self::PowerUser | Self::RemoteWorker)
    }

    /// Whether to show peak speeds.
    #[must_use]
    pub fn show_peaks(&self) -> bool {
        !matches!(self, Self::Casual)
    }

    /// Whether to show latency under load.
    #[must_use]
    pub fn show_latency_under_load(&self) -> bool {
        matches!(self, Self::PowerUser | Self::Gamer)
    }
}

/// Profile-specific usage check targets.
pub struct UsageTarget {
    pub name: &'static str,
    pub required_mbps: f64,
    pub icon: &'static str,
}

/// Get usage check targets for a profile.
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
