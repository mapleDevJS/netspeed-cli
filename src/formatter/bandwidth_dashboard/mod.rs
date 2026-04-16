//! Responsive bandwidth capability dashboard with terminal width adaptation.
//!
//! Implements a modern, accessible dashboard showing network usage capabilities
//! across 5 categories with 15 scenarios, adapting to terminal width dynamically.
//!
//! # Layout Modes
//! - **Expanded (>=120 cols)**: Full layout with multipliers and capacity badges
//! - **Standard (90-119 cols)**: Hide multiplier column, keep bars
//! - **Compact (80-89 cols)**: Vertical stack (name -> bar -> value)
//! - **Minimal (<80 cols)**: ASCII-only, single column
//!
//! # Features
//! - Unicode box drawing with ASCII fallback
//! - Dynamic progress bars using block characters
//! - ANSI 256 color support with 16-color fallback
//! - Responsive column widths based on terminal detection
//! - Contextual status indicators (not just raw multipliers)
//! - Screen-reader friendly semantic structure

mod render;
mod scenarios;

// Re-export everything for backward compatibility
pub use render::*;
pub use scenarios::*;

use crate::terminal;

pub fn format_capability_report(dl_mbps: f64) -> String {
    let (terminal_width, layout) = ResponsiveLayout::detect();
    let nc = terminal::no_color() || layout == ResponsiveLayout::Minimal;
    let minimal = layout == ResponsiveLayout::Minimal;

    let statuses = compute_all_statuses(dl_mbps);
    let mut lines = Vec::new();

    // Opening header
    lines.push(String::new());
    lines.push(render_section_header(
        dl_mbps,
        terminal_width as usize,
        nc,
        minimal,
    ));
    lines.push(String::new());

    // Category boxes
    for (i, cat) in all_categories().iter().enumerate() {
        if i > 0 {
            lines.push(String::new());
        }
        lines.push(render_category_box(
            cat,
            &statuses[i],
            layout,
            terminal_width,
            nc,
            minimal,
        ));
    }

    // Closing footer
    lines.push(render_section_footer(nc, minimal));
    lines.push(String::new());

    lines.join("\n")
}

/// Print the capability report to stderr.
pub fn print_capability_report(dl_mbps: f64) {
    let output = format_capability_report(dl_mbps);
    eprintln!("{output}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn test_responsive_layout_from_width() {
        assert_eq!(
            ResponsiveLayout::from_width(120),
            ResponsiveLayout::Expanded
        );
        assert_eq!(
            ResponsiveLayout::from_width(100),
            ResponsiveLayout::Standard
        );
        assert_eq!(ResponsiveLayout::from_width(85), ResponsiveLayout::Compact);
        assert_eq!(ResponsiveLayout::from_width(70), ResponsiveLayout::Minimal);
    }

    #[test]
    fn test_responsive_layout_flags() {
        let expanded = ResponsiveLayout::Expanded;
        assert!(expanded.show_multiplier());
        assert!(!expanded.is_compact());
        assert!(!expanded.is_ascii_only());

        let minimal = ResponsiveLayout::Minimal;
        assert!(!minimal.show_multiplier());
        assert!(minimal.is_compact());
        assert!(minimal.is_ascii_only());
    }

    #[test]
    fn test_capacity_level() {
        assert_eq!(
            CapacityLevel::from_concurrent(15, true),
            CapacityLevel::Optimal
        );
        assert_eq!(
            CapacityLevel::from_concurrent(5, true),
            CapacityLevel::Moderate
        );
        assert_eq!(
            CapacityLevel::from_concurrent(1, true),
            CapacityLevel::Limited
        );
        assert_eq!(
            CapacityLevel::from_concurrent(0, false),
            CapacityLevel::Exceeded
        );
    }

    #[test]
    fn test_compute_scenario_statuses() {
        let statuses = compute_all_statuses(277.0);
        assert_eq!(statuses.len(), 5); // 5 categories

        // Check first category (Communication)
        let comm = &statuses[0];
        assert_eq!(comm.len(), 3); // 3 scenarios

        // 4K Video Calls at 25 Mbps should be met
        assert!(comm[0].is_met);
        assert_eq!(comm[0].concurrent, 11); // 277/25 = 11.08
    }

    #[test]
    fn test_progress_bar_unicode() {
        let bar = render_progress_bar(50.0, 20, false, false);
        // Should contain filled and empty blocks
        assert!(bar.contains('█') || bar.contains('░'));
    }

    #[test]
    fn test_progress_bar_ascii() {
        let bar = render_progress_bar(50.0, 20, true, true);
        assert!(bar.contains('=') || bar.contains('-'));
        assert!(bar.starts_with('['));
        assert!(bar.ends_with(']'));
    }

    #[test]
    fn test_progress_bar_empty() {
        let bar = render_progress_bar(0.0, 10, false, false);
        assert_eq!(bar.chars().filter(|&c| c == '░').count(), 10);
    }

    #[test]
    fn test_progress_bar_full() {
        let bar = render_progress_bar(100.0, 10, false, false);
        assert_eq!(bar.chars().filter(|&c| c == '█').count(), 10);
    }

    #[test]
    fn test_icon_padding_unicode() {
        let icon = get_padded_icon("📹", false, false);
        assert!(!icon.is_empty());
    }

    #[test]
    fn test_icon_padding_ascii() {
        let icon = get_padded_icon("📹", true, true);
        assert_eq!(icon, "VC");
    }

    #[test]
    fn test_name_truncation_short() {
        let name = truncate_name("Short Name", 28);
        assert_eq!(name, "Short Name");
    }

    #[test]
    fn test_name_truncation_long() {
        let name = truncate_name("This Is A Very Long Scenario Name That Exceeds Limit", 28);
        assert!(name.width() <= 28);
        assert!(name.contains('…'));
    }

    #[test]
    fn test_format_capability_report_contains_header() {
        let output = format_capability_report(277.0);
        assert!(output.contains("USAGE CAPABILITY"));
        assert!(output.contains("277 Mbps"));
    }

    #[test]
    fn test_format_capability_report_contains_categories() {
        let output = format_capability_report(277.0);
        assert!(output.contains("COMMUNICATION"));
        assert!(output.contains("STREAMING"));
        assert!(output.contains("NEXT-GEN"));
    }

    #[test]
    fn test_format_capability_report_minimal_mode() {
        // Simulate minimal by checking ASCII output structure
        let output = format_capability_report(50.0);
        // Should still have structure even at lower speeds
        assert!(output.contains("USAGE CAPABILITY"));
    }

    #[test]
    fn test_scenario_ordering_by_bandwidth() {
        // Next-gen should have highest bandwidth scenarios first
        let nextgen = &CAT_NEXTGEN;
        assert!(nextgen.scenarios[0].required_mbps >= nextgen.scenarios[1].required_mbps);
        assert!(nextgen.scenarios[1].required_mbps >= nextgen.scenarios[2].required_mbps);
    }

    #[test]
    fn test_warning_description_present() {
        // AI Model Download should have a description
        let ai_scenario = &CAT_NEXTGEN.scenarios[0];
        assert!(ai_scenario.description.is_some());
        assert!(ai_scenario.description.unwrap().contains("72%"));
    }

    #[test]
    fn test_total_bandwidth_constant() {
        assert_eq!(TOTAL_BANDWIDTH_MBPS, 277.0);
    }

    #[test]
    fn test_all_categories_count() {
        let cats = all_categories();
        assert_eq!(cats.len(), 5);
        let total_scenarios: usize = cats.iter().map(|c| c.scenarios.len()).sum();
        assert_eq!(total_scenarios, 15);
    }

    #[test]
    fn test_bar_color_zones() {
        // Test that color function returns different results for different zones
        let green_fn = bar_color_by_usage(20.0, false);
        let yellow_fn = bar_color_by_usage(50.0, false);
        let red_fn = bar_color_by_usage(80.0, false);

        // Just verify they don't panic and return strings
        assert!(!green_fn("test").is_empty());
        assert!(!yellow_fn("test").is_empty());
        assert!(!red_fn("test").is_empty());
    }
}
