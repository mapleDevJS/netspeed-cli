use crate::cli::CliArgs;

pub struct Config {
    pub no_download: bool,
    pub no_upload: bool,
    pub single: bool,
    pub bytes: bool,
    pub share: bool,
    pub simple: bool,
    pub csv: bool,
    pub csv_delimiter: char,
    pub csv_header: bool,
    pub json: bool,
    pub list: bool,
    pub server_ids: Vec<String>,
    pub exclude_ids: Vec<String>,
    #[allow(dead_code)]
    pub mini_url: Option<String>,
    #[allow(dead_code)]
    pub source: Option<String>,
    pub timeout: u64,
    #[allow(dead_code)]
    pub secure: bool,
    #[allow(dead_code)]
    pub no_pre_allocate: bool,
    #[allow(dead_code)]
    pub client_ip: Option<String>,
}

impl Config {
    pub fn from_args(args: &CliArgs) -> Self {
        Self {
            no_download: args.no_download,
            no_upload: args.no_upload,
            single: args.single,
            bytes: args.bytes,
            share: args.share,
            simple: args.simple,
            csv: args.csv,
            csv_delimiter: args.csv_delimiter,
            csv_header: args.csv_header,
            json: args.json,
            list: args.list,
            server_ids: args.server.clone(),
            exclude_ids: args.exclude.clone(),
            mini_url: args.mini.clone(),
            source: args.source.clone(),
            timeout: args.timeout,
            secure: args.secure,
            no_pre_allocate: args.no_pre_allocate,
            client_ip: None,
        }
    }
}
