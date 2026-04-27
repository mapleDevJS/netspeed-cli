//! Integration tests for constructing [`Config`] from [`ConfigSource`].
//!
//! These tests verify the public API that external test crates would use:
//! building a [`Config`] from a hand-built [`ConfigSource`] instead of
//! going through CLI argument parsing. This is the testability payoff
//! of the DIP refactoring (ConfigSource as CLI→config bridge).

use netspeed_cli::config::{
    ConfigSource, Format, NetworkSource, OutputSource, ServerSource, TestSource,
};

// ── Default Construction ─────────────────────────────────────────────

#[test]
fn test_config_from_default_source() {
    let source = ConfigSource::default();
    let config = netspeed_cli::config::Config::from_source(&source);

    // Output defaults
    assert!(!config.bytes());
    assert!(!config.simple());
    assert!(!config.csv());
    assert!(!config.json());
    assert!(!config.list());
    assert!(!config.quiet());
    assert!(!config.minimal());
    assert_eq!(config.csv_delimiter(), ',');
    assert!(!config.csv_header());
    assert!(config.profile().is_none());
    assert!(config.format().is_none());

    // Test defaults
    assert!(!config.no_download());
    assert!(!config.no_upload());
    assert!(!config.single());

    // Network defaults
    assert_eq!(config.timeout(), 10);
    assert!(config.source().is_none());
    assert!(config.ca_cert().is_none());
    assert!(config.tls_version().is_none());
    assert!(!config.pin_certs());

    // Server defaults
    assert!(config.server_ids().is_empty());
    assert!(config.exclude_ids().is_empty());
}

// ── Partial Construction ─────────────────────────────────────────────

#[test]
fn test_config_from_source_only_output() {
    let source = ConfigSource {
        output: OutputSource {
            json: Some(true),
            profile: Some("gamer".to_string()),
            format: Some(Format::Compact),
            ..Default::default()
        },
        ..Default::default()
    };
    let config = netspeed_cli::config::Config::from_source(&source);

    // Output fields set
    assert!(config.json());
    assert_eq!(config.profile(), Some("gamer"));
    assert_eq!(config.format(), Some(Format::Compact));

    // Everything else at defaults
    assert!(!config.no_download());
    assert_eq!(config.timeout(), 10);
    assert!(
        config.server_ids().is_empty(),
        "server_ids should default to empty"
    );
}

#[test]
fn test_config_from_source_only_network() {
    let source = ConfigSource {
        network: NetworkSource {
            timeout: 60,
            source: Some("0.0.0.0".to_string()),
            ca_cert: Some("/path/to/ca.pem".to_string()),
            tls_version: Some("1.3".to_string()),
            pin_certs: Some(true),
        },
        ..Default::default()
    };
    let config = netspeed_cli::config::Config::from_source(&source);

    // Network fields set
    assert_eq!(config.timeout(), 60);
    assert_eq!(config.source(), Some("0.0.0.0"));
    assert_eq!(config.ca_cert(), Some("/path/to/ca.pem"));
    assert_eq!(config.tls_version(), Some("1.3"));
    assert!(config.pin_certs());

    // Everything else at defaults
    assert!(!config.json());
    assert!(!config.no_download());
}

#[test]
fn test_config_from_source_only_test_selection() {
    let source = ConfigSource {
        test: TestSource {
            no_download: Some(true),
            no_upload: Some(true),
            single: Some(true),
        },
        ..Default::default()
    };
    let config = netspeed_cli::config::Config::from_source(&source);

    assert!(config.no_download());
    assert!(config.no_upload());
    assert!(config.single());

    // Everything else at defaults
    assert_eq!(config.timeout(), 10);
    assert!(!config.json());
}

#[test]
fn test_config_from_source_only_servers() {
    let source = ConfigSource {
        servers: ServerSource {
            server_ids: vec!["1234".to_string(), "5678".to_string()],
            exclude_ids: vec!["9999".to_string()],
        },
        ..Default::default()
    };
    let config = netspeed_cli::config::Config::from_source(&source);

    assert_eq!(config.server_ids(), ["1234", "5678"]);
    assert_eq!(config.exclude_ids(), ["9999"]);

    // Everything else at defaults
    assert!(!config.no_download());
    assert_eq!(config.timeout(), 10);
}

// ── Full Custom Construction ─────────────────────────────────────────

#[test]
fn test_config_from_source_fully_custom() {
    let source = ConfigSource {
        output: OutputSource {
            bytes: Some(true),
            simple: None,
            csv: None,
            csv_delimiter: ';',
            csv_header: Some(true),
            json: None,
            list: false,
            quiet: Some(true),
            minimal: None,
            profile: Some("streamer".to_string()),
            theme: "light".to_string(),
            format: Some(Format::Json),
        },
        test: TestSource {
            no_download: Some(false),
            no_upload: Some(true),
            single: Some(true),
        },
        network: NetworkSource {
            source: Some("192.168.1.100".to_string()),
            timeout: 30,
            ca_cert: None,
            tls_version: Some("1.2".to_string()),
            pin_certs: None,
        },
        servers: ServerSource {
            server_ids: vec!["42".to_string()],
            exclude_ids: Vec::new(),
        },
        strict_config: Some(true),
    };
    let config = netspeed_cli::config::Config::from_source(&source);

    // Output
    assert!(config.bytes());
    assert_eq!(config.csv_delimiter(), ';');
    assert!(config.csv_header());
    assert!(config.quiet());
    assert_eq!(config.profile(), Some("streamer"));
    assert_eq!(config.format(), Some(Format::Json));

    // Test selection
    assert!(!config.no_download()); // Some(false) → false
    assert!(config.no_upload());
    assert!(config.single());

    // Network
    assert_eq!(config.timeout(), 30);
    assert_eq!(config.source(), Some("192.168.1.100"));
    assert_eq!(config.tls_version(), Some("1.2"));

    // Servers
    assert_eq!(config.server_ids(), ["42"]);
    assert!(config.exclude_ids().is_empty());

    // Top-level
    assert!(config.strict());
}

// ── Format Variant Coverage ──────────────────────────────────────────

#[test]
fn test_config_from_source_all_format_variants() {
    let formats = [
        (Format::Json, true),  // machine-readable
        (Format::Jsonl, true), // machine-readable
        (Format::Csv, true),   // machine-readable
        (Format::Minimal, false),
        (Format::Simple, false),
        (Format::Compact, false),
        (Format::Detailed, false),
        (Format::Dashboard, false),
    ];

    for (fmt, expect_machine) in formats {
        let source = ConfigSource {
            output: OutputSource {
                format: Some(fmt),
                ..Default::default()
            },
            ..Default::default()
        };
        let config = netspeed_cli::config::Config::from_source(&source);

        assert_eq!(
            config.format(),
            Some(fmt),
            "Format {fmt:?} not preserved through Config::from_source"
        );
        assert_eq!(
            fmt.is_machine_readable(),
            expect_machine,
            "is_machine_readable mismatch for {fmt:?}"
        );
    }
}

// ── Sub-source Independence ──────────────────────────────────────────

#[test]
fn test_sub_sources_can_be_built_independently() {
    // Verify that each sub-source can be constructed on its own,
    // confirming the sub-source struct pattern works for external consumers
    let _output = OutputSource {
        format: Some(Format::Dashboard),
        ..Default::default()
    };
    let _test = TestSource {
        no_download: Some(true),
        ..Default::default()
    };
    let _network = NetworkSource {
        timeout: 45,
        ..Default::default()
    };
    let _servers = ServerSource {
        server_ids: vec!["1".to_string()],
        ..Default::default()
    };
    // If this compiles, the sub-source types are independently accessible
}

#[test]
fn test_config_source_composes_sub_sources() {
    // Verify that ConfigSource correctly delegates to its sub-source fields
    let source = ConfigSource {
        output: OutputSource {
            json: Some(true),
            ..Default::default()
        },
        test: TestSource {
            single: Some(true),
            ..Default::default()
        },
        network: NetworkSource {
            timeout: 20,
            ..Default::default()
        },
        servers: ServerSource {
            server_ids: vec!["99".to_string()],
            ..Default::default()
        },
        strict_config: None,
    };

    assert_eq!(source.output.json, Some(true));
    assert_eq!(source.test.single, Some(true));
    assert_eq!(source.network.timeout, 20);
    assert_eq!(source.servers.server_ids, vec!["99".to_string()]);
    assert!(source.strict_config.is_none());
}
