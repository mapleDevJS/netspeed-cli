use crate::cli::CliArgs;
use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct ConfigFile {
    pub no_download: Option<bool>,
    pub no_upload: Option<bool>,
    pub single: Option<bool>,
    pub bytes: Option<bool>,
    pub simple: Option<bool>,
    pub csv: Option<bool>,
    pub csv_delimiter: Option<char>,
    pub csv_header: Option<bool>,
    pub json: Option<bool>,
    pub timeout: Option<u64>,
}

#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    pub no_download: bool,
    pub no_upload: bool,
    pub single: bool,
    pub bytes: bool,
    pub simple: bool,
    pub csv: bool,
    pub csv_delimiter: char,
    pub csv_header: bool,
    pub json: bool,
    pub list: bool,
    pub server_ids: Vec<String>,
    pub exclude_ids: Vec<String>,
    pub source: Option<String>,
    pub timeout: u64,
}

impl Config {
    #[must_use]
    pub fn from_args(args: &CliArgs) -> Self {
        let file_config = load_config_file().unwrap_or_default();

        Self {
            no_download: args.no_download || file_config.no_download.unwrap_or(false),
            no_upload: args.no_upload || file_config.no_upload.unwrap_or(false),
            single: args.single || file_config.single.unwrap_or(false),
            bytes: args.bytes || file_config.bytes.unwrap_or(false),
            simple: args.simple || file_config.simple.unwrap_or(false),
            csv: args.csv || file_config.csv.unwrap_or(false),
            csv_delimiter: if args.csv_delimiter == ',' {
                file_config.csv_delimiter.unwrap_or(',')
            } else {
                args.csv_delimiter
            },
            csv_header: args.csv_header || file_config.csv_header.unwrap_or(false),
            json: args.json || file_config.json.unwrap_or(false),
            list: args.list,
            server_ids: args.server.clone(),
            exclude_ids: args.exclude.clone(),
            source: args.source.clone(),
            timeout: if args.timeout == 10 {
                file_config.timeout.unwrap_or(10)
            } else {
                args.timeout
            },
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    ProjectDirs::from("dev", "vibe", "netspeed-cli").map(|proj_dirs| {
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir).ok();
        config_dir.join("config.toml")
    })
}

fn load_config_file() -> Option<ConfigFile> {
    let path = get_config_path()?;
    if !path.exists() {
        return None;
    }

    let content = fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_config_from_args_defaults() {
        let args = CliArgs::parse_from(["netspeed-cli"]);
        let config = Config::from_args(&args);

        assert!(!config.no_download);
        assert!(!config.no_upload);
        assert!(!config.single);
        assert!(!config.bytes);
        assert!(!config.simple);
        assert!(!config.csv);
        assert!(!config.json);
        assert!(!config.list);
        assert_eq!(config.timeout, 10);
        assert_eq!(config.csv_delimiter, ',');
        assert!(!config.csv_header);
        assert!(config.server_ids.is_empty());
        assert!(config.exclude_ids.is_empty());
    }

    #[test]
    fn test_config_from_args_no_download() {
        let args = CliArgs::parse_from(["netspeed-cli", "--no-download"]);
        let config = Config::from_args(&args);
        assert!(config.no_download);
        assert!(!config.no_upload);
    }
}
