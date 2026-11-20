#![cfg(unix)]

use async_trait::async_trait;
use netspeed_cli::cli::Args;
use netspeed_cli::error::Error;
use netspeed_cli::orchestrator::Orchestrator;
use netspeed_cli::services::{
    IpDiscoverer, ServerFetcher, ServerPinger, ServerSelector, ServerService, Services,
};
use netspeed_cli::types::{ClientLocation, Server};
use serial_test::serial;
use std::fs::File;
use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct FixtureServerService {
    client: reqwest::Client,
    discovery_url: String,
    server: Server,
}

#[async_trait]
impl ServerFetcher for FixtureServerService {
    async fn fetch_servers(&self) -> Result<(Vec<Server>, Option<ClientLocation>), Error> {
        self.client
            .get(&self.discovery_url)
            .send()
            .await
            .map_err(Error::NetworkError)?
            .error_for_status()
            .map_err(Error::NetworkError)?;
        Ok((vec![self.server.clone()], None))
    }
}

#[async_trait]
impl ServerPinger for FixtureServerService {
    async fn ping_server(&self, server: &Server) -> Result<(f64, f64, f64, Vec<f64>), Error> {
        netspeed_cli::servers::ping_test(&self.client, server).await
    }
}

impl ServerSelector for FixtureServerService {
    fn select_best(&self, servers: &[Server]) -> Result<Server, Error> {
        servers
            .first()
            .cloned()
            .ok_or_else(|| Error::ServerNotFound("fixture has no server".into()))
    }
}

impl ServerService for FixtureServerService {}

struct FixtureIpService;

#[async_trait]
impl IpDiscoverer for FixtureIpService {
    async fn discover_ip(&self) -> Result<String, Error> {
        Ok("192.0.2.10".into())
    }
}

struct FixtureServices {
    server: FixtureServerService,
    ip: FixtureIpService,
}

impl Services for FixtureServices {
    fn server_service(&self) -> &dyn ServerService {
        &self.server
    }

    fn ip_service(&self) -> &dyn IpDiscoverer {
        &self.ip
    }
}

#[tokio::test]
async fn pty_fixture_child() {
    let Ok(mode) = std::env::var("NETSPEED_PTY_CHILD") else {
        return;
    };
    let mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/servers"))
        .respond_with(
            ResponseTemplate::new(if mode == "failure" { 500 } else { 200 }).set_delay(
                if mode == "interrupt" {
                    Duration::from_secs(30)
                } else {
                    Duration::from_millis(600)
                },
            ),
        )
        .mount(&mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/latency.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock)
        .await;
    Mock::given(method("GET"))
        .and(wiremock::matchers::path_regex("/random.*\\.jpg$"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(vec![0; 2 * 1024 * 1024])
                .set_delay(Duration::from_millis(25)),
        )
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock)
        .await;

    let server = Server {
        id: "fixture-1".into(),
        url: format!("{}/upload.php", mock.uri()),
        name: "Local Fixture".into(),
        sponsor: "PTY Test Service".into(),
        country: "Local".into(),
        lat: 0.0,
        lon: 0.0,
        distance: 1.0,
    };
    let args = if mode == "interrupt" {
        <Args as clap::Parser>::parse_from([
            "netspeed-cli",
            "--no-download",
            "--no-upload",
            "--format",
            "dashboard",
            "--timeout",
            "30",
        ])
    } else {
        <Args as clap::Parser>::parse_from([
            "netspeed-cli",
            "--format",
            "dashboard",
            "--timeout",
            "30",
        ])
    };
    let orchestrator = Orchestrator::new(args, None)
        .expect("fixture configuration should be valid")
        .with_services(FixtureServices {
            server: FixtureServerService {
                client: reqwest::Client::new(),
                discovery_url: format!("{}/servers", mock.uri()),
                server,
            },
            ip: FixtureIpService,
        });

    if mode == "interrupt" {
        let run = orchestrator.run();
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .expect("register fixture interrupt handler");
        std::io::Write::write_all(&mut std::io::stderr().lock(), b"SIGNAL_READY\n")
            .expect("write signal readiness");
        let interrupt = async {
            signal.recv().await;
        };
        if netspeed_cli::interrupt::run_or_interrupt(run, interrupt)
            .await
            .is_err()
        {
            let _ = std::io::Write::write_all(&mut std::io::stderr().lock(), b"\nInterrupted.\n");
            let _ = std::io::Write::flush(&mut std::io::stderr().lock());
            std::process::exit(netspeed_cli::bin_errors::exit_codes::INTERRUPTED);
        }
    } else if mode == "failure" {
        assert!(orchestrator.run().await.is_err());
        std::io::Write::write_all(&mut std::io::stderr().lock(), b"FAILURE_CLEANED_UP\n")
            .expect("write cleanup marker");
    } else {
        orchestrator
            .run()
            .await
            .expect("fixture orchestration should complete");
    }
}

#[test]
#[serial]
fn pty_discovery_failure_restores_terminal_without_success_report() {
    let (mut child, mut master) = spawn_pty_fixture("failure", 80, 24);
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut output = Vec::new();
    while Instant::now() < deadline {
        read_ready(&mut master, &mut output, Duration::from_millis(50));
        if child.try_wait().expect("poll failure fixture").is_some() {
            read_ready(&mut master, &mut output, Duration::from_millis(50));
            break;
        }
    }
    assert!(child.try_wait().expect("poll failure exit").is_some());
    assert!(child.wait().expect("collect failure fixture").success());
    let captured = String::from_utf8_lossy(&output);
    assert!(captured.contains("Finding servers"), "{captured}");
    let (_, after) = captured
        .split_once("FAILURE_CLEANED_UP")
        .expect("cleanup marker");
    assert!(!after.contains('\u{1b}'));
    assert!(!captured.contains("CONNECTION HEALTH"));
    assert!(!captured.contains("\u{1b}[?25l"));
    assert!(!captured.contains("\u{1b}[?1049h"));
}

#[test]
#[serial]
fn pty_short_terminal_report_keeps_primary_results_and_context() {
    let (mut child, mut master) = spawn_pty_fixture("complete", 40, 8);
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut output = Vec::new();
    while Instant::now() < deadline {
        read_ready(&mut master, &mut output, Duration::from_millis(50));
        if child.try_wait().expect("poll child status").is_some() {
            break;
        }
    }
    assert!(child.try_wait().expect("poll final child status").is_some());
    let status = child.wait().expect("collect child status");
    let captured = String::from_utf8_lossy(&output);
    let text = console::strip_ansi_codes(&captured);
    assert!(status.success(), "short-terminal fixture failed: {text}");
    assert!(text.contains("NETSPEED"));
    assert!(text.contains("Server:"));
    assert!(text.contains("Download"));
    assert!(text.contains("Upload"));
    assert!(text.contains("Latency"));
}

fn spawn_pty_fixture(mode: &str, cols: u16, rows: u16) -> (Child, File) {
    let mut command = Command::new(std::env::current_exe().expect("test executable path"));
    command
        .args([
            "--exact",
            "pty_fixture_child",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("NETSPEED_PTY_CHILD", mode);
    spawn_pty_command(command, cols, rows, false)
}

fn spawn_pty_command(command: Command, cols: u16, rows: u16, pipe_stdout: bool) -> (Child, File) {
    spawn_pty_command_with_environment(command, cols, rows, pipe_stdout, None)
}

fn spawn_pty_command_with_environment(
    mut command: Command,
    cols: u16,
    rows: u16,
    pipe_stdout: bool,
    environment: Option<(&str, &str)>,
) -> (Child, File) {
    let mut master_fd = -1;
    let mut slave_fd = -1;
    let mut size = libc::winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    // SAFETY: openpty initializes two new descriptors and receives a valid winsize.
    let result = unsafe {
        libc::openpty(
            &mut master_fd,
            &mut slave_fd,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut size,
        )
    };
    assert_eq!(result, 0, "openpty should succeed");
    // SAFETY: successful openpty transfers ownership of both descriptors here.
    let master = unsafe { OwnedFd::from_raw_fd(master_fd) };
    // SAFETY: successful openpty transfers ownership of both descriptors here.
    let slave = unsafe { OwnedFd::from_raw_fd(slave_fd) };
    let slave_file = File::from(slave);
    command
        .env("TERM", "xterm-256color")
        .env("NO_COLOR", "1")
        .stdin(Stdio::from(
            slave_file.try_clone().expect("clone PTY slave"),
        ))
        .stdout(if pipe_stdout {
            Stdio::piped()
        } else {
            Stdio::from(slave_file.try_clone().expect("clone PTY slave"))
        })
        .stderr(Stdio::from(slave_file));
    if let Some((name, value)) = environment {
        command.env(name, value);
    }
    let child = command.spawn().expect("spawn PTY fixture child");
    let master_file = File::from(master);
    (child, master_file)
}

fn resize_pty(master: &File, cols: u16, rows: u16) {
    let size = libc::winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    // SAFETY: ioctl receives the PTY master descriptor and a valid winsize pointer.
    let result = unsafe { libc::ioctl(master.as_raw_fd(), libc::TIOCSWINSZ, &size) };
    assert_eq!(result, 0, "PTY resize should succeed");
}

fn read_ready(master: &mut File, output: &mut Vec<u8>, timeout: Duration) {
    let mut poll_fd = libc::pollfd {
        fd: master.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: poll receives one valid pollfd and a bounded timeout.
    let ready = unsafe { libc::poll(&mut poll_fd, 1, timeout.as_millis() as i32) };
    if ready <= 0 || poll_fd.revents & (libc::POLLIN | libc::POLLHUP) == 0 {
        return;
    }
    let mut chunk = [0u8; 2048];
    if let Ok(count) = master.read(&mut chunk) {
        output.extend_from_slice(&chunk[..count]);
    }
}

#[test]
#[serial]
fn actual_binary_honors_format_precedence_and_interrupts_with_stdout_piped() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let proxy = format!("http://{}", listener.local_addr().unwrap());
    let mut command = Command::new(env!("CARGO_BIN_EXE_netspeed-cli"));
    command
        .args([
            "--timeout",
            "30",
            "--format",
            "dashboard",
            "--quiet=false",
            "--minimal=false",
            "--json=true",
        ])
        .env("HTTPS_PROXY", &proxy)
        .env("HTTP_PROXY", &proxy)
        .env("NO_PROXY", "")
        .env_remove("CI")
        .env_remove("PREFER_REDUCED_MOTION");
    let (mut child, mut master) = spawn_pty_command(command, 80, 24, true);
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut output = Vec::new();
    while Instant::now() < deadline {
        read_ready(&mut master, &mut output, Duration::from_millis(50));
        if String::from_utf8_lossy(&output).contains("Finding servers") {
            break;
        }
        if child.try_wait().unwrap().is_some() {
            break;
        }
    }
    if !String::from_utf8_lossy(&output).contains("Finding servers") {
        let _ = child.kill();
        let _ = child.wait();
        panic!(
            "discovery did not render: {}",
            String::from_utf8_lossy(&output)
        );
    }
    // SAFETY: the child is running and its process identifier is valid.
    assert_eq!(
        unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGINT) },
        0
    );
    while Instant::now() < deadline {
        read_ready(&mut master, &mut output, Duration::from_millis(50));
        if child.try_wait().unwrap().is_some() {
            break;
        }
    }
    if child.try_wait().unwrap().is_none() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("interrupted binary did not exit");
    }
    for _ in 0..4 {
        read_ready(&mut master, &mut output, Duration::from_millis(20));
    }
    assert_eq!(child.wait().unwrap().code(), Some(130));
    let text = String::from_utf8_lossy(&output);
    assert!(text.contains("Interrupted."), "{text}");
    assert!(
        !text.contains("NETSPEED"),
        "partial results became a final report: {text}"
    );
    assert!(!text.contains("\u{1b}[?25l"));
    let mut stdout = Vec::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut stdout)
        .unwrap();
    assert!(stdout.is_empty(), "human output leaked to stdout");
}

#[test]
#[serial]
fn actual_binary_static_modes_emit_no_live_terminal_controls() {
    for (format, quiet, extra, environment) in [
        ("minimal", "false", None, None),
        ("dashboard", "false", Some("--no-live-ui"), None),
        ("dashboard", "true", None, None),
        ("json", "false", None, None),
        ("jsonl", "false", None, None),
        ("csv", "false", None, None),
        ("dashboard", "false", None, Some(("TERM", "dumb"))),
        ("dashboard", "false", None, Some(("CI", "1"))),
        (
            "dashboard",
            "false",
            None,
            Some(("PREFER_REDUCED_MOTION", "1")),
        ),
    ] {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let proxy = format!("http://{}", listener.local_addr().unwrap());
        let mut command = Command::new(env!("CARGO_BIN_EXE_netspeed-cli"));
        command
            .args(["--timeout", "30", "--format", format, "--minimal=false"])
            .arg(format!("--quiet={quiet}"))
            .env("HTTPS_PROXY", &proxy)
            .env("HTTP_PROXY", &proxy)
            .env("NO_PROXY", "")
            .env_remove("CI")
            .env_remove("PREFER_REDUCED_MOTION");
        if let Some(extra) = extra {
            command.arg(extra);
        }
        let (mut child, mut master) =
            spawn_pty_command_with_environment(command, 80, 24, true, environment);
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut output = Vec::new();
        let _connection = loop {
            read_ready(&mut master, &mut output, Duration::from_millis(10));
            if let Ok((connection, _)) = listener.accept() {
                break connection;
            }
            if Instant::now() >= deadline || child.try_wait().unwrap().is_some() {
                let _ = child.kill();
                let _ = child.wait();
                panic!(
                    "{format} did not reach discovery: {}",
                    String::from_utf8_lossy(&output)
                );
            }
        };
        // SAFETY: the child is still waiting for the local proxy response.
        assert_eq!(
            unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGINT) },
            0
        );
        while Instant::now() < deadline && child.try_wait().unwrap().is_none() {
            read_ready(&mut master, &mut output, Duration::from_millis(10));
        }
        if child.try_wait().unwrap().is_none() {
            let _ = child.kill();
        }
        assert_eq!(child.wait().unwrap().code(), Some(130));
        for _ in 0..4 {
            read_ready(&mut master, &mut output, Duration::from_millis(10));
        }
        assert!(
            !output.contains(&0x1b),
            "live controls in {format}: {output:?}"
        );
        let mut stdout = Vec::new();
        child
            .stdout
            .take()
            .unwrap()
            .read_to_end(&mut stdout)
            .unwrap();
        assert!(stdout.is_empty());
    }
}

#[test]
#[serial]
fn pty_completion_resizes_and_keeps_report_in_scrollback() {
    let (mut child, mut master) = spawn_pty_fixture("complete", 40, 8);
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut output = Vec::new();
    let mut saw_discovery = false;
    let mut resized = false;

    while Instant::now() < deadline {
        read_ready(&mut master, &mut output, Duration::from_millis(50));
        let text = String::from_utf8_lossy(&output);
        if text.contains("Finding servers") {
            saw_discovery = true;
        }
        if !resized && (text.contains("Download")) {
            resize_pty(&master, 120, 30);
            resized = true;
        }
        if child.try_wait().expect("poll child status").is_some() {
            break;
        }
    }
    if child.try_wait().expect("poll final child status").is_none() {
        let _ = child.kill();
        panic!("PTY fixture did not complete in time");
    }
    for _ in 0..4 {
        read_ready(&mut master, &mut output, Duration::from_millis(20));
    }

    let status = child.wait().expect("collect child status");
    let text = String::from_utf8_lossy(&output);
    assert!(status.success(), "PTY child failed: {text}");
    assert!(
        saw_discovery && resized,
        "expected discovery and active transfer before resize: {text}"
    );
    assert!(
        text.contains("PTY Test Service"),
        "server context missing: {text}"
    );
    assert!(text.contains("Latency"), "latency result missing: {text}");
    assert!(text.contains("Download"), "download phase missing: {text}");
    assert!(text.contains("Upload"), "upload phase missing: {text}");

    assert!(text.contains("NETSPEED"), "final report missing: {text}");
    assert!(text.contains("Profile"), "profile summary missing: {text}");
    assert!(
        !text.contains("\u{1b}[?1049h"),
        "alternate screen should not be used"
    );
    assert!(
        !text.contains("\u{1b}[?25l"),
        "cursor should remain visible"
    );
}

#[test]
#[serial]
fn pty_ctrl_c_cancels_and_returns_interrupt_status() {
    let (mut child, mut master) = spawn_pty_fixture("interrupt", 80, 24);
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut output = Vec::new();

    while Instant::now() < deadline {
        read_ready(&mut master, &mut output, Duration::from_millis(50));
        let captured = String::from_utf8_lossy(&output);
        if captured.contains("Finding servers") && captured.contains("SIGNAL_READY") {
            break;
        }
        if child.try_wait().expect("poll child status").is_some() {
            panic!(
                "PTY fixture exited before the interrupt: {}",
                String::from_utf8_lossy(&output)
            );
        }
    }
    assert!(
        String::from_utf8_lossy(&output).contains("Finding servers"),
        "server discovery spinner was not rendered before interrupt"
    );
    assert!(String::from_utf8_lossy(&output).contains("SIGNAL_READY"));

    // SAFETY: the child PID is valid while its process is running.
    let result = unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGINT) };
    assert_eq!(result, 0, "send SIGINT to PTY child");
    let status = child.wait().expect("collect interrupted child status");
    for _ in 0..4 {
        read_ready(&mut master, &mut output, Duration::from_millis(20));
    }
    let text = String::from_utf8_lossy(&output);
    assert_eq!(
        status.code(),
        Some(130),
        "unexpected interrupt status: {text}"
    );
    assert!(
        !text.contains("NETSPEED"),
        "partial test must not print final report"
    );
}
