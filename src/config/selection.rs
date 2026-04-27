//! Selection configuration types for test execution.
//!
//! These types represent the processed configuration after merging
//! CLI args, config file, and defaults.

use crate::theme::Theme;
use serde::{Deserialize, Serialize};

/// Output display configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub bytes: bool,
    pub simple: bool,
    pub csv: bool,
    pub csv_delimiter: char,
    pub csv_header: bool,
    pub json: bool,
    pub minimal: bool,
    pub quiet: bool,
    pub list: bool,
    pub profile: Option<String>,
    pub theme: Theme,
    pub format: Option<super::Format>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            bytes: false,
            simple: false,
            csv: false,
            csv_delimiter: ',',
            csv_header: false,
            json: false,
            minimal: false,
            quiet: false,
            list: false,
            profile: None,
            theme: Theme::Dark,
            format: None,
        }
    }
}

impl OutputConfig {
    #[must_use]
    #[allow(deprecated)]
    pub(crate) fn from_source(
        source: &super::OutputSource,
        file_config: &super::File,
        merge_bool: impl Fn(Option<bool>, Option<bool>) -> bool,
    ) -> Self {
        let theme = if source.theme == "dark" {
            file_config
                .theme
                .as_ref()
                .and_then(|t| Theme::from_name(t))
                .unwrap_or_default()
        } else {
            Theme::from_name(&source.theme).unwrap_or_default()
        };

        Self {
            bytes: merge_bool(source.bytes, file_config.bytes),
            simple: merge_bool(source.simple, file_config.simple),
            csv: merge_bool(source.csv, file_config.csv),
            csv_delimiter: if source.csv_delimiter == ',' {
                file_config.csv_delimiter.unwrap_or(',')
            } else {
                source.csv_delimiter
            },
            csv_header: merge_bool(source.csv_header, file_config.csv_header),
            json: merge_bool(source.json, file_config.json),
            list: source.list,
            quiet: merge_bool(source.quiet, None),
            profile: source.profile.clone().or(file_config.profile.clone()),
            theme,
            minimal: merge_bool(source.minimal, None),
            format: source.format,
        }
    }
}

/// Test execution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSelection {
    pub no_download: bool,
    pub no_upload: bool,
    pub single: bool,
    pub timeout: u64,
}

impl Default for TestSelection {
    fn default() -> Self {
        Self {
            no_download: false,
            no_upload: false,
            single: false,
            timeout: 30,
        }
    }
}

impl TestSelection {
    #[must_use]
    pub(crate) fn from_source(
        source: &super::TestSource,
        file_config: &super::File,
        merge_bool: impl Fn(Option<bool>, Option<bool>) -> bool,
    ) -> Self {
        Self {
            no_download: merge_bool(source.no_download, file_config.no_download),
            no_upload: merge_bool(source.no_upload, file_config.no_upload),
            single: merge_bool(source.single, file_config.single),
            timeout: source.timeout.or(file_config.timeout).unwrap_or(30),
        }
    }
}

/// Network connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub bind: Option<String>,
    pub timeout: u64,
    pub ca_cert: Option<String>,
    pub tls_version: Option<String>,
    pub pin_certs: bool,
    pub insecure: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind: None,
            timeout: 30,
            ca_cert: None,
            tls_version: None,
            pin_certs: false,
            insecure: false,
        }
    }
}

impl NetworkConfig {
    #[must_use]
    pub(crate) fn from_source(
        source: &super::NetworkSource,
        file_config: &super::File,
    ) -> Self {
        Self {
            bind: source.source.clone().or(file_config.bind.clone()),
            timeout: source.timeout,
            ca_cert: source.ca_cert.clone().or(file_config.ca_cert.clone()),
            tls_version: source.tls_version.clone().or(file_config.tls_version.clone()),
            pin_certs: source.pin_certs.unwrap_or(file_config.pin_certs.unwrap_or(false)),
            insecure: source.insecure.unwrap_or(file_config.insecure.unwrap_or(false)),
        }
    }
}

/// Server selection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSelection {
    pub server_ids: Vec<u64>,
    pub exclude_ids: Vec<u64>,
}

impl Default for ServerSelection {
    fn default() -> Self {
        Self {
            server_ids: Vec::new(),
            exclude_ids: Vec::new(),
        }
    }
}

impl ServerSelection {
    #[must_use]
    pub(crate) fn from_source(
        source: &super::ServerSource,
        file_config: &super::File,
    ) -> Self {
        let mut server_ids = source.server_ids.clone();
        if let Some(ref file_ids) = file_config.server_ids {
            for id in file_ids {
                if !server_ids.contains(id) {
                    server_ids.push(*id);
                }
            }
        }
        
        let mut exclude_ids = source.exclude_ids.clone();
        if let Some(ref file_exclude) = file_config.exclude_ids {
            for id in file_exclude {
                if !exclude_ids.contains(id) {
                    exclude_ids.push(*id);
                }
            }
        }

        Self {
            server_ids,
            exclude_ids,
        }
    }
}