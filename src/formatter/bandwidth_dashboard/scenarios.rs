//! Data types, constants, and scenario definitions for the bandwidth dashboard.

use std::io::IsTerminal;

// ── Constants ────────────────────────────────────────────────────────────────

/// Total available bandwidth in Mbps (contextual for this network)
pub const TOTAL_BANDWIDTH_MBPS: f64 = 277.0;

/// Column widths for expanded mode
pub const ICON_WIDTH: usize = 2;
pub const NAME_WIDTH: usize = 28;
pub const BANDWIDTH_WIDTH: usize = 8;
pub const CAPACITY_BADGE_WIDTH: usize = 6;
pub const FIXED_COLUMNS_WIDTH: usize =
    ICON_WIDTH + 1 + NAME_WIDTH + 1 + BANDWIDTH_WIDTH + 1 + CAPACITY_BADGE_WIDTH + 1;

/// Minimum bar width
pub const MIN_BAR_WIDTH: usize = 10;

/// Thresholds for capacity badges
pub const OPTIMAL_MULTIPLIER: u32 = 10;
pub const MODERATE_MULTIPLIER: u32 = 2;

// ── Data Structures ─────────────────────────────────────────────────────────

/// Responsive layout mode based on terminal width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponsiveLayout {
    /// >=120 columns: full layout with all columns
    Expanded,
    /// 90-119 columns: hide multipliers, keep bars
    Standard,
    /// 80-89 columns: vertical stack
    Compact,
    /// <80 columns: ASCII-only, minimal
    Minimal,
}

impl ResponsiveLayout {
    /// Detect layout mode from terminal width.
    pub fn from_width(width: u16) -> Self {
        if width >= 120 {
            Self::Expanded
        } else if width >= 90 {
            Self::Standard
        } else if width >= 80 {
            Self::Compact
        } else {
            Self::Minimal
        }
    }

    /// Detect current terminal width and layout mode.
    pub fn detect() -> (u16, Self) {
        let width = if std::io::stdout().is_terminal() {
            terminal_size::terminal_size()
                .map(|(w, _)| w.0)
                .unwrap_or(100)
        } else {
            100 // Default for piped output
        };
        (width, Self::from_width(width))
    }

    /// Check if we should use ASCII-only output.
    pub fn is_ascii_only(&self) -> bool {
        matches!(self, Self::Minimal)
    }

    /// Check if we should show multiplier column.
    pub fn show_multiplier(&self) -> bool {
        matches!(self, Self::Expanded)
    }

    /// Check if we should use vertical stack layout.
    pub fn is_compact(&self) -> bool {
        matches!(self, Self::Compact | Self::Minimal)
    }
}

/// A single usage scenario with bandwidth requirements.
pub struct BandwidthScenario {
    pub name: &'static str,
    pub required_mbps: f64,
    pub icon: &'static str,
    pub concurrent_label: &'static str,
    pub description: Option<&'static str>,
}

/// A category grouping scenarios.
pub struct ScenarioCategory {
    pub name: &'static str,
    pub icon: &'static str,
    pub scenarios: &'static [BandwidthScenario],
}

/// Computed status for a scenario.
pub struct ScenarioStatus {
    pub scenario: &'static BandwidthScenario,
    pub concurrent: u32,
    pub headroom_pct: f64,
    pub is_met: bool,
    pub usage_pct: f64,
}

/// Status level for capacity badges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityLevel {
    Optimal,  // >10x
    Moderate, // 2-10x
    Limited,  // ~1x
    Exceeded, // <1x (not met)
}

impl CapacityLevel {
    pub fn from_concurrent(concurrent: u32, is_met: bool) -> Self {
        if !is_met {
            Self::Exceeded
        } else if concurrent > OPTIMAL_MULTIPLIER {
            Self::Optimal
        } else if concurrent >= MODERATE_MULTIPLIER {
            Self::Moderate
        } else {
            Self::Limited
        }
    }
}

// ── Scenario Definitions ─────────────────────────────────────────────────────
// Scenarios ordered by bandwidth consumption (highest first) within each category

pub static CAT_COMMUNICATION: ScenarioCategory = ScenarioCategory {
    name: "COMMUNICATION & COLLABORATION",
    icon: "💬",
    scenarios: &[
        BandwidthScenario {
            name: "4K Video Calls (FaceTime/Meet)",
            required_mbps: 25.0,
            icon: "📹",
            concurrent_label: "calls",
            description: None,
        },
        BandwidthScenario {
            name: "HD Video Calls (Zoom/Teams+Share)",
            required_mbps: 8.0,
            icon: "📹",
            concurrent_label: "calls",
            description: None,
        },
        BandwidthScenario {
            name: "VoIP + Encrypted VPN",
            required_mbps: 2.0,
            icon: "🔒",
            concurrent_label: "sessions",
            description: None,
        },
    ],
};

pub static CAT_STREAMING: ScenarioCategory = ScenarioCategory {
    name: "STREAMING & ENTERTAINMENT",
    icon: "🎬",
    scenarios: &[
        BandwidthScenario {
            name: "Cloud Gaming (GeForce Now/Xbox)",
            required_mbps: 50.0,
            icon: "🎮",
            concurrent_label: "sessions",
            description: None,
        },
        BandwidthScenario {
            name: "4K HDR Streaming (Netflix/Disney+)",
            required_mbps: 35.0,
            icon: "📺",
            concurrent_label: "streams",
            description: None,
        },
        BandwidthScenario {
            name: "Live Broadcast Upload (Twitch/YT)",
            required_mbps: 30.0,
            icon: "📡",
            concurrent_label: "streams",
            description: None,
        },
    ],
};

pub static CAT_PRODUCTIVITY: ScenarioCategory = ScenarioCategory {
    name: "WORK & PRODUCTIVITY",
    icon: "💼",
    scenarios: &[
        BandwidthScenario {
            name: "4K Video Upload (YouTube Creator)",
            required_mbps: 80.0,
            icon: "🎥",
            concurrent_label: "uploads",
            description: None,
        },
        BandwidthScenario {
            name: "Cloud Sync Bulk Upload (Drive/Dropbox)",
            required_mbps: 50.0,
            icon: "☁️",
            concurrent_label: "syncs",
            description: None,
        },
        BandwidthScenario {
            name: "Remote Desktop HD (Parsec/TeamViewer)",
            required_mbps: 30.0,
            icon: "🖥️",
            concurrent_label: "sessions",
            description: None,
        },
    ],
};

pub static CAT_SMART_HOME: ScenarioCategory = ScenarioCategory {
    name: "SMART HOME & IOT",
    icon: "🏠",
    scenarios: &[
        BandwidthScenario {
            name: "4x 1080p Security Cameras",
            required_mbps: 20.0,
            icon: "📷",
            concurrent_label: "arrays",
            description: None,
        },
        BandwidthScenario {
            name: "50+ IoT Devices Hub",
            required_mbps: 5.0,
            icon: "🔌",
            concurrent_label: "hubs",
            description: None,
        },
    ],
};

pub static CAT_NEXTGEN: ScenarioCategory = ScenarioCategory {
    name: "NEXT-GEN / HEAVY USAGE",
    icon: "🚀",
    scenarios: &[
        BandwidthScenario {
            name: "AI Model Download (7-70GB LLM)",
            required_mbps: 200.0,
            icon: "🤖",
            concurrent_label: "downloads",
            description: Some("Uses 72% of total bandwidth — near connection limit"),
        },
        BandwidthScenario {
            name: "4x Simultaneous 4K Streams",
            required_mbps: 140.0,
            icon: "👨\u{200d}👩\u{200d}👧\u{200d}👦",
            concurrent_label: "households",
            description: Some("Uses 50% of total bandwidth"),
        },
        BandwidthScenario {
            name: "8K Streaming (YouTube 8K/AV1)",
            required_mbps: 100.0,
            icon: "🎬",
            concurrent_label: "streams",
            description: Some("Uses 36% of total bandwidth"),
        },
        BandwidthScenario {
            name: "VR/AR Streaming (Quest 3/Vision Pro)",
            required_mbps: 80.0,
            icon: "🥽",
            concurrent_label: "sessions",
            description: Some("Uses 29% of total bandwidth"),
        },
    ],
};

const ALL_CATEGORIES: &[&ScenarioCategory] = &[
    &CAT_COMMUNICATION,
    &CAT_STREAMING,
    &CAT_PRODUCTIVITY,
    &CAT_SMART_HOME,
    &CAT_NEXTGEN,
];

/// Get all scenario categories.
pub fn all_categories() -> &'static [&'static ScenarioCategory] {
    ALL_CATEGORIES
}

// ── Status Computation ───────────────────────────────────────────────────────

/// Compute status for all scenarios given download speed in Mbps.
pub fn compute_all_statuses(dl_mbps: f64) -> Vec<Vec<ScenarioStatus>> {
    all_categories()
        .iter()
        .map(|cat| {
            cat.scenarios
                .iter()
                .map(|s| {
                    let concurrent = if s.required_mbps > 0.0 {
                        (dl_mbps / s.required_mbps).floor() as u32
                    } else {
                        0
                    };
                    let headroom_pct = if s.required_mbps > 0.0 {
                        ((dl_mbps - s.required_mbps) / s.required_mbps * 100.0).max(0.0)
                    } else {
                        100.0
                    };
                    let is_met = dl_mbps >= s.required_mbps;
                    let usage_pct = if TOTAL_BANDWIDTH_MBPS > 0.0 {
                        (s.required_mbps / TOTAL_BANDWIDTH_MBPS) * 100.0
                    } else {
                        0.0
                    };

                    ScenarioStatus {
                        scenario: s,
                        concurrent,
                        headroom_pct,
                        is_met,
                        usage_pct,
                    }
                })
                .collect()
        })
        .collect()
}
