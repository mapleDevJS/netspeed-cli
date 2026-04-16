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
    /// Content consumers (Netflix, YouTube, etc.) focused on download speed.
    Streamer,
    /// Work-from-home professionals focused on upload and stability.
    RemoteWorker,
    /// Basic users who want a simple pass/fail assessment.
    Casual,
}

impl UserProfile {
    /// Get profile from string name (case-insensitive).
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
    /// Returns (ping_weight, jitter_weight, download_weight, upload_weight).
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
    pub fn excellent_speed_threshold(&self) -> f64 {
        match self {
            Self::PowerUser => 500.0,
            Self::Gamer => 100.0,    // Gamers don't need massive bandwidth
            Self::Streamer => 200.0, // 4K streaming needs ~50 Mbps, 200 gives headroom
            Self::RemoteWorker => 100.0,
            Self::Casual => 50.0,
        }
    }

    /// Ping rating thresholds for "Excellent" (in ms).
    pub fn excellent_ping_threshold(&self) -> f64 {
        match self {
            Self::PowerUser => 10.0,
            Self::Gamer => 5.0,     // Gamers need ultra-low latency
            Self::Streamer => 30.0, // Streaming buffers, so higher ping is OK
            Self::RemoteWorker => 20.0,
            Self::Casual => 30.0,
        }
    }

    /// Jitter rating thresholds for "Excellent" (in ms).
    pub fn excellent_jitter_threshold(&self) -> f64 {
        match self {
            Self::PowerUser => 2.0,
            Self::Gamer => 1.0, // Gamers need consistent latency
            Self::Streamer => 5.0,
            Self::RemoteWorker => 3.0,
            Self::Casual => 5.0,
        }
    }

    /// Whether to show detailed latency section.
    pub fn show_latency_details(&self) -> bool {
        !matches!(self, Self::Casual)
    }

    /// Whether to show bufferbloat grade.
    pub fn show_bufferbloat(&self) -> bool {
        matches!(self, Self::PowerUser | Self::Gamer)
    }

    /// Whether to show stability analysis (CV%).
    pub fn show_stability(&self) -> bool {
        matches!(self, Self::PowerUser | Self::RemoteWorker)
    }

    /// Whether to show latency percentiles.
    pub fn show_percentiles(&self) -> bool {
        matches!(self, Self::PowerUser)
    }

    /// Whether to show usage check targets.
    pub fn show_usage_check(&self) -> bool {
        !matches!(self, Self::Casual)
    }

    /// Whether to show download time estimates.
    pub fn show_estimates(&self) -> bool {
        matches!(self, Self::PowerUser | Self::Casual)
    }

    /// Whether to show historical comparison.
    pub fn show_history(&self) -> bool {
        matches!(self, Self::PowerUser | Self::RemoteWorker)
    }

    /// Whether to show UL/DL ratio.
    pub fn show_ul_dl_ratio(&self) -> bool {
        matches!(self, Self::PowerUser | Self::RemoteWorker)
    }

    /// Whether to show peak speeds.
    pub fn show_peaks(&self) -> bool {
        !matches!(self, Self::Casual)
    }

    /// Whether to show latency under load.
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
            let (p, j, d, u) = profile.scoring_weights();
            assert!(
                (p + j + d + u - 1.0).abs() < 0.01,
                "Weights should sum to ~1.0 for {profile:?}"
            );
        }
    }

    #[test]
    fn test_gamer_profile_priorities() {
        let g = UserProfile::Gamer;
        let (p, j, d, u) = g.scoring_weights();
        assert!(p > d, "Gamer: ping should weight more than download");
        assert!(j > u, "Gamer: jitter should weight more than upload");
        assert_eq!(g.excellent_ping_threshold(), 5.0);
        assert!(g.show_bufferbloat());
    }

    #[test]
    fn test_streamer_profile_priorities() {
        let s = UserProfile::Streamer;
        let (_, _, d, _) = s.scoring_weights();
        assert!(d >= 0.5, "Streamer: download should have highest weight");
        assert!(s.show_usage_check());
    }

    #[test]
    fn test_remote_worker_profile_priorities() {
        let r = UserProfile::RemoteWorker;
        let (_, _, _, u) = r.scoring_weights();
        assert!(u >= 0.35, "RemoteWorker: upload should have high weight");
        assert!(r.show_stability());
        assert!(r.show_ul_dl_ratio());
    }

    #[test]
    fn test_casual_profile_minimal() {
        let c = UserProfile::Casual;
        assert!(!c.show_latency_details());
        assert!(!c.show_bufferbloat());
        assert!(!c.show_stability());
        assert!(!c.show_percentiles());
        assert!(!c.show_history());
        assert!(c.show_estimates());
    }

    #[test]
    fn test_power_user_shows_all() {
        let p = UserProfile::PowerUser;
        assert!(p.show_latency_details());
        assert!(p.show_bufferbloat());
        assert!(p.show_stability());
        assert!(p.show_percentiles());
        assert!(p.show_history());
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
