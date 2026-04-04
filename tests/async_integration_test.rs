// Integration tests for async modules using wiremock

#[cfg(test)]
mod tests {
    use netspeed_cli::cli::{CliArgs, ShellType};
    use netspeed_cli::config::Config;
    use netspeed_cli::discovery::ServerDiscovery;
    use netspeed_cli::error::SpeedtestError;
    use netspeed_cli::http::create_client;
    use netspeed_cli::mini::{detect_mini_server, mini_to_server};
    use netspeed_cli::presenter::ResultPresenter;
    use netspeed_cli::servers::select_best_server;
    use netspeed_cli::types::{Server, ServerInfo, TestResult};
    use netspeed_cli::download::download_test;
    use netspeed_cli::upload::upload_test;
    use netspeed_cli::runner::TestRunner;
    use netspeed_cli::servers::ping_test;

    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_server_with_url(url: &str) -> Server {
        Server {
            id: "1".to_string(),
            url: url.to_string(),
            name: "Test Server".to_string(),
            sponsor: "Test ISP".to_string(),
            country: "US".to_string(),
            lat: 40.0,
            lon: -74.0,
            distance: 100.0,
            latency: 0.0,
        }
    }

    // ===== Download Tests =====

    #[tokio::test]
    async fn test_download_test_all_streams_fail() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let server = create_test_server_with_url(&mock_server.uri());
        let client = create_client(&Config::from_args(&CliArgs::default())).unwrap();

        let result = download_test(&client, &server, false).await;
        assert!(result.is_err());
    }

    // ===== Upload Tests =====

    #[tokio::test]
    async fn test_upload_test_all_streams_fail() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/upload"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let server = create_test_server_with_url(&mock_server.uri());
        let client = create_client(&Config::from_args(&CliArgs::default())).unwrap();

        let result = upload_test(&client, &server, false).await;
        assert!(result.is_err());
    }

    // ===== Runner Tests =====

    #[tokio::test]
    async fn test_runner_both_disabled() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("1.2.3.4"))
            .mount(&mock_server)
            .await;

        let server = create_test_server_with_url(&mock_server.uri());
        let args = CliArgs { no_download: true, no_upload: true, simple: true, ..Default::default() };
        let config = Config::from_args(&args);
        let client = create_client(&config).unwrap();

        let result = TestRunner::run(&client, &server, &config).await;
        assert!(result.is_ok());
        let tr = result.unwrap();
        assert!(tr.download.is_none());
        assert!(tr.upload.is_none());
        assert!(tr.ping.is_none());
        assert!(!tr.timestamp.is_empty());
        assert!(tr.server.id == "1");
    }

    // ===== Discovery Tests =====

    #[tokio::test]
    async fn test_discover_mini_server() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let args = CliArgs { mini: Some(mock_server.uri()), ..Default::default() };
        let config = Config::from_args(&args);
        let client = create_client(&config).unwrap();

        let result = ServerDiscovery::discover(&client, &config).await;
        assert!(result.is_ok());
        let server = result.unwrap();
        assert_eq!(server.id, "0");
        assert_eq!(server.sponsor, "Mini");
    }

    #[tokio::test]
    async fn test_handle_list_flag_true() {
        let args = CliArgs { list: true, ..Default::default() };
        let config = Config::from_args(&args);
        assert!(config.list);
    }

    #[tokio::test]
    async fn test_handle_list_flag_false() {
        let args = CliArgs { list: false, ..Default::default() };
        let config = Config::from_args(&args);
        assert!(!config.list);
    }

    #[tokio::test]
    async fn test_discover_speedtest_empty_servers_error() {
        let servers: Vec<Server> = vec![];
        let result = select_best_server(&servers);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpeedtestError::ServerNotFound(_)));
    }

    #[tokio::test]
    async fn test_discover_speedtest_with_server_filter() {
        let servers = vec![
            Server { id: "1234".to_string(), url: "http://srv1.com".to_string(), name: "Server1".to_string(), sponsor: "ISP1".to_string(), country: "US".to_string(), lat: 40.0, lon: -74.0, distance: 100.0, latency: 0.0 },
            Server { id: "5678".to_string(), url: "http://srv2.com".to_string(), name: "Server2".to_string(), sponsor: "ISP2".to_string(), country: "DE".to_string(), lat: 50.0, lon: 0.0, distance: 200.0, latency: 0.0 },
        ];

        let args = CliArgs { server: vec!["1234".to_string()], ..Default::default() };
        let config = Config::from_args(&args);

        let mut filtered = servers;
        filtered.retain(|s| config.server_ids.contains(&s.id));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "1234");
    }

    #[tokio::test]
    async fn test_discover_speedtest_with_exclude_filter() {
        let args = CliArgs { exclude: vec!["5678".to_string()], ..Default::default() };
        let config = Config::from_args(&args);

        let servers = vec![
            Server { id: "1234".to_string(), url: "http://srv1.com".to_string(), name: "Server1".to_string(), sponsor: "ISP1".to_string(), country: "US".to_string(), lat: 40.0, lon: -74.0, distance: 100.0, latency: 0.0 },
            Server { id: "5678".to_string(), url: "http://srv2.com".to_string(), name: "Server2".to_string(), sponsor: "ISP2".to_string(), country: "DE".to_string(), lat: 50.0, lon: 0.0, distance: 200.0, latency: 0.0 },
        ];

        let mut filtered = servers;
        filtered.retain(|s| !config.exclude_ids.contains(&s.id));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "1234");
    }

    #[tokio::test]
    async fn test_discover_speedtest_all_filtered_out_error() {
        let args = CliArgs { server: vec!["9999".to_string()], ..Default::default() };
        let config = Config::from_args(&args);

        let servers = vec![Server { id: "1234".to_string(), url: "http://srv1.com".to_string(), name: "Server1".to_string(), sponsor: "ISP1".to_string(), country: "US".to_string(), lat: 40.0, lon: -74.0, distance: 100.0, latency: 0.0 }];

        let mut filtered = servers;
        filtered.retain(|s| config.server_ids.contains(&s.id));
        assert!(filtered.is_empty());
        assert!(select_best_server(&filtered).is_err());
    }

    // ===== Mini Server Tests =====

    #[tokio::test]
    async fn test_detect_mini_server_all_failures_defaults_to_php() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let client = create_client(&Config::from_args(&CliArgs::default())).unwrap();

        let result = detect_mini_server(&client, &mock_server.uri()).await;
        assert!(result.is_ok());
        let mini = result.unwrap();
        assert!(mini.url.ends_with("upload.php"));
        assert_eq!(mini.sponsor, "Mini");
        assert_eq!(mini.id, "0");
    }

    #[tokio::test]
    async fn test_detect_mini_server_first_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = create_client(&Config::from_args(&CliArgs::default())).unwrap();

        let result = detect_mini_server(&client, &mock_server.uri()).await;
        assert!(result.is_ok());
        let mini = result.unwrap();
        assert!(mini.url.contains("upload"));
    }

    #[tokio::test]
    async fn test_mini_to_server_integration() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST")).respond_with(ResponseTemplate::new(200)).mount(&mock_server).await;

        let client = create_client(&Config::from_args(&CliArgs::default())).unwrap();
        let mini = detect_mini_server(&client, &mock_server.uri()).await.unwrap();
        let server = mini_to_server(&mini);

        assert_eq!(server.id, "0");
        assert_eq!(server.country, "Unknown");
        assert!((server.lat - 0.0).abs() < f64::EPSILON);
    }

    // ===== Presenter Tests =====

    #[test]
    fn test_present_json_mode() {
        let args = CliArgs { json: true, ..Default::default() };
        let config = Config::from_args(&args);
        let result = TestResult {
            server: ServerInfo { id: "12345".to_string(), name: "Test".to_string(), sponsor: "Test ISP".to_string(), country: "US".to_string(), distance: 100.0 },
            ping: Some(25.0), download: Some(100_000_000.0), upload: Some(50_000_000.0),
            share_url: None, timestamp: "2024-01-01T00:00:00Z".to_string(), client_ip: None,
        };
        assert!(ResultPresenter::present(&result, &config).is_ok());
    }

    #[test]
    fn test_present_csv_mode() {
        let args = CliArgs { csv: true, ..Default::default() };
        let config = Config::from_args(&args);
        let result = TestResult {
            server: ServerInfo { id: "12345".to_string(), name: "Test".to_string(), sponsor: "Test ISP".to_string(), country: "US".to_string(), distance: 100.0 },
            ping: Some(25.0), download: Some(100_000_000.0), upload: Some(50_000_000.0),
            share_url: None, timestamp: "2024-01-01T00:00:00Z".to_string(), client_ip: None,
        };
        assert!(ResultPresenter::present(&result, &config).is_ok());
    }

    #[test]
    fn test_present_simple_mode() {
        let args = CliArgs { simple: true, ..Default::default() };
        let config = Config::from_args(&args);
        let result = TestResult {
            server: ServerInfo { id: "12345".to_string(), name: "Test".to_string(), sponsor: "Test ISP".to_string(), country: "US".to_string(), distance: 100.0 },
            ping: Some(25.0), download: Some(100_000_000.0), upload: Some(50_000_000.0),
            share_url: None, timestamp: "2024-01-01T00:00:00Z".to_string(), client_ip: None,
        };
        assert!(ResultPresenter::present(&result, &config).is_ok());
    }

    #[test]
    fn test_present_no_format_flags() {
        let config = Config::from_args(&CliArgs::default());
        let result = TestResult {
            server: ServerInfo { id: "12345".to_string(), name: "Test".to_string(), sponsor: "Test ISP".to_string(), country: "US".to_string(), distance: 100.0 },
            ping: Some(25.0), download: Some(100_000_000.0), upload: Some(50_000_000.0),
            share_url: None, timestamp: "2024-01-01T00:00:00Z".to_string(), client_ip: None,
        };
        assert!(ResultPresenter::present(&result, &config).is_ok());
    }

    #[test]
    fn test_handle_share_not_requested() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = Config::from_args(&CliArgs::default());
        let client = create_client(&config).unwrap();
        let result = TestResult {
            server: ServerInfo { id: "12345".to_string(), name: "Test".to_string(), sponsor: "Test ISP".to_string(), country: "US".to_string(), distance: 100.0 },
            ping: Some(25.0), download: Some(100_000_000.0), upload: Some(50_000_000.0),
            share_url: None, timestamp: "2024-01-01T00:00:00Z".to_string(), client_ip: None,
        };
        let res = rt.block_on(ResultPresenter::handle_share(&client, &result, false));
        assert!(res.is_ok());
    }

    #[test]
    fn test_handle_share_requested_error_on_network_failure() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let config = Config::from_args(&CliArgs::default());
        let client = create_client(&config).unwrap();
        let result = TestResult {
            server: ServerInfo { id: "12345".to_string(), name: "Test".to_string(), sponsor: "Test ISP".to_string(), country: "US".to_string(), distance: 100.0 },
            ping: Some(25.0), download: Some(100_000_000.0), upload: Some(50_000_000.0),
            share_url: None, timestamp: "2024-01-01T00:00:00Z".to_string(), client_ip: None,
        };
        let res = rt.block_on(ResultPresenter::handle_share(&client, &result, true));
        assert!(res.is_err());
    }

    // ===== Shell Completion Tests =====

    #[test]
    fn test_shell_type_value_enum() {
        let shells = [ShellType::Bash, ShellType::Zsh, ShellType::Fish, ShellType::PowerShell, ShellType::Elvish];
        for shell in shells {
            let _ = format!("{:?}", shell);
        }
    }

    #[test]
    fn test_generate_completion_cli_parsing() {
        use clap::Parser;
        let args = CliArgs::try_parse_from(["netspeed-cli", "--generate-completion", "bash"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.generate_completion.is_some());
        matches!(args.generate_completion.unwrap(), ShellType::Bash);
    }

    #[test]
    fn test_generate_completion_all_shells() {
        use clap::Parser;
        let shells = ["bash", "zsh", "fish", "power-shell", "elvish"];
        for shell in shells {
            let args = CliArgs::try_parse_from(["netspeed-cli", "--generate-completion", shell]).unwrap();
            assert!(args.generate_completion.is_some());
        }
    }

    // ===== Ping Test with Mock =====

    #[tokio::test]
    async fn test_ping_test_with_mock_server() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
            .mount(&mock_server)
            .await;

        let server = create_test_server_with_url(&mock_server.uri());
        let client = create_client(&Config::from_args(&CliArgs::default())).unwrap();

        let result = ping_test(&client, &server).await;
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
}
