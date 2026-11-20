//! Scoped inline dashboard for a CLI run.
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Output {
    #[cfg(unix)]
    file: std::fs::File,
    #[cfg(not(unix))]
    term: console::Term,
}
impl Output {
    fn open() -> std::io::Result<Self> {
        #[cfg(unix)]
        {
            Self::open_terminal(libc::STDERR_FILENO)
        }
        #[cfg(not(unix))]
        {
            Ok(Self {
                term: console::Term::stderr(),
            })
        }
    }
    #[cfg(unix)]
    fn open_terminal(descriptor: libc::c_int) -> std::io::Result<Self> {
        use std::os::unix::{ffi::OsStrExt, fs::OpenOptionsExt};
        let mut name = [0u8; 1024];
        // SAFETY: ttyname_r receives a valid writable buffer and stderr's descriptor.
        let error = unsafe { libc::ttyname_r(descriptor, name.as_mut_ptr().cast(), name.len()) };
        if error != 0 {
            return Err(std::io::Error::from_raw_os_error(error));
        }
        let end = name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name.len());
        let path = std::ffi::OsStr::from_bytes(&name[..end]);
        let file = std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_NOCTTY)
            .open(path)?;
        Ok(Self { file })
    }
    fn write(&mut self, text: &str) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            use std::io::Write;
            self.file.write_all(text.as_bytes())
        }
        #[cfg(not(unix))]
        {
            self.term.write_str(text)
        }
    }
    fn clear(&mut self, rows: usize) -> std::io::Result<()> {
        self.write(&"\x1b[1A\r\x1b[2K".repeat(rows))
    }
}

#[derive(Clone)]
pub(crate) struct Handle(Arc<Mutex<Model>>, Arc<AtomicU64>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Initializing,
    DiscoveringServers,
    MeasuringLatency,
    TestingDownload,
    TestingUpload,
    Finalizing,
}
impl Phase {
    fn label(self) -> &'static str {
        match self {
            Self::Initializing => "Initializing",
            Self::DiscoveringServers => "Finding servers",
            Self::MeasuringLatency => "Testing latency",
            Self::TestingDownload => "Download",
            Self::TestingUpload => "Upload",
            Self::Finalizing => "Finalizing",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RunState {
    Running,
    Completed,
    Failed,
    Interrupted,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TransferState {
    Pending,
    Running,
    Completed(f64),
    Skipped,
    Failed,
    Interrupted,
    Unavailable,
}
impl TransferState {
    fn text(self, bytes: bool) -> String {
        match self {
            Self::Pending => "pending".into(),
            Self::Running => "measuring".into(),
            Self::Completed(rate) => {
                crate::formatter::ratings::format_speed_plain(rate * 1_000_000.0, bytes)
            }
            Self::Skipped => "skipped".into(),
            Self::Failed => "failed".into(),
            Self::Interrupted => "interrupted".into(),
            Self::Unavailable => "unavailable".into(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Layout {
    Minimal,
    Compact,
    Standard,
    Expanded,
}
impl Layout {
    fn for_size(width: u16, height: u16) -> Self {
        if width < 50 || height < 5 {
            Self::Minimal
        } else if width < 75 || height < 7 {
            Self::Compact
        } else if width < 110 || height < 9 {
            Self::Standard
        } else {
            Self::Expanded
        }
    }
    fn chart_width(self) -> usize {
        match self {
            Self::Minimal => 0,
            Self::Compact => 16,
            Self::Standard => 32,
            Self::Expanded => 48,
        }
    }
}

#[derive(Clone)]
struct Model {
    phase: Phase,
    state: RunState,
    download: TransferState,
    upload: TransferState,
    rate: Option<f64>,
    samples: Vec<(u64, Instant, f64)>,
    bytes: u64,
    context: String,
    warnings: Vec<String>,
    omitted_warnings: usize,
}
impl Handle {
    pub(crate) fn new() -> Self {
        Handle(
            Arc::new(Mutex::new(Model {
                phase: Phase::Initializing,
                state: RunState::Running,
                download: TransferState::Pending,
                upload: TransferState::Pending,
                rate: None,
                samples: Vec::with_capacity(48),
                bytes: 0,
                context: String::new(),
                warnings: Vec::new(),
                omitted_warnings: 0,
            })),
            Arc::new(AtomicU64::new(0)),
        )
    }

    pub(crate) fn phase(&self, phase: Phase) {
        let mut model = self.0.lock().unwrap_or_else(|e| e.into_inner());
        model.phase = phase;
        match phase {
            Phase::TestingDownload => model.download = TransferState::Running,
            Phase::TestingUpload => model.upload = TransferState::Running,
            _ => {}
        }
        model.rate = None;
        model.samples.clear();
    }
    pub(crate) fn transfer_finished(&self, upload: bool, rate: Option<f64>) {
        let mut model = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let state = match rate {
            None => TransferState::Skipped,
            Some(rate) if rate.is_finite() && rate >= 0.0 => TransferState::Completed(rate),
            Some(_) => TransferState::Unavailable,
        };
        if upload {
            model.upload = state;
        } else {
            model.download = state;
        }
    }
    pub(crate) fn finish(&self, state: RunState) {
        let mut model = self.0.lock().unwrap_or_else(|e| e.into_inner());
        model.state = state;
        let interrupted = match state {
            RunState::Failed => TransferState::Failed,
            RunState::Interrupted => TransferState::Interrupted,
            _ => return,
        };
        if model.download == TransferState::Running {
            model.download = interrupted;
        }
        if model.upload == TransferState::Running {
            model.upload = interrupted;
        }
    }
    pub(crate) fn context(&self, text: String) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).context =
            crate::terminal::sanitize_display_text(&text, 120);
    }
    pub(crate) fn warning(&self, text: &str) {
        let mut model = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if model.warnings.len() < 32 {
            model
                .warnings
                .push(crate::terminal::sanitize_display_text(text, 240));
        } else {
            model.omitted_warnings += 1;
        }
    }
    pub(crate) fn sample(&self, rate: f64, bytes: u64) {
        let sequence = self.1.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut model) = self.0.try_lock() {
            model.rate = Some(rate);
            model.bytes = bytes;
            if model.samples.len() == 48 {
                model.samples.remove(0);
            }
            model.samples.push((sequence, Instant::now(), rate));
        }
    }
}
pub(crate) struct Session {
    handle: Handle,
    stop: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl Session {
    pub(crate) fn start(bytes: bool, theme: crate::theme::Theme) -> Option<Self> {
        let mut output = Output::open().ok()?;
        let handle = Handle::new();
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let model = handle.clone();
        let worker = std::thread::Builder::new()
            .name("netspeed-live".into())
            .spawn(move || {
                let term = console::Term::stderr();
                let start = Instant::now();
                let mut rows = 0;
                'frames: while !worker_stop.load(Ordering::Acquire) {
                    let (height, width) = term.size();
                    let snapshot = model.0.lock().unwrap_or_else(|e| e.into_inner()).clone();
                    let lines = render(&snapshot, width, height, start.elapsed(), bytes);
                    if output.clear(rows).is_err() {
                        model.warning("Live display unavailable. Testing continues with a static final report.");
                        break;
                    }
                    rows = 0;
                    for (index, line) in lines.into_iter().enumerate() {
                        let line =
                            console::truncate_str(&line, (width as usize).saturating_sub(1), "");
                        let line = if crate::terminal::no_color()
                            || theme == crate::theme::Theme::Monochrome
                        {
                            line.into_owned()
                        } else if index == 1 {
                            if matches!(snapshot.phase, Phase::TestingUpload) {
                                crate::theme::Colors::header(&line, theme)
                            } else {
                                crate::theme::Colors::info(&line, theme)
                            }
                        } else {
                            crate::theme::Colors::muted(&line, theme)
                        };
                        if output.write(&format!("{line}\n")).is_err() {
                            model.warning("Live display unavailable. Testing continues with a static final report.");
                            break 'frames;
                        }
                        rows += 1;
                    }
                    std::thread::park_timeout(Duration::from_millis(100));
                }
                let _ = output.clear(rows);
            })
            .ok()?;
        Some(Self {
            handle,
            stop,
            worker: Some(worker),
        })
    }
    pub(crate) fn handle(&self) -> Handle {
        self.handle.clone()
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let running = self
            .handle
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .state
            == RunState::Running;
        if running {
            self.handle.finish(RunState::Interrupted);
        }
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            let _ = worker.join();
        }
        let model = self.handle.0.lock().unwrap_or_else(|e| e.into_inner());
        for warning in &model.warnings {
            eprintln!("{warning}");
        }
        if model.omitted_warnings > 0 {
            eprintln!("{} additional warnings", model.omitted_warnings);
        }
    }
}

fn render(model: &Model, width: u16, height: u16, elapsed: Duration, bytes: bool) -> Vec<String> {
    let layout = Layout::for_size(width, height);
    let label = match model.state {
        RunState::Running => model.phase.label(),
        RunState::Completed => "Completed",
        RunState::Failed => "Failed",
        RunState::Interrupted => "Interrupted",
    };
    let mut lines = vec![format!("  {}  {:.1}s", label, elapsed.as_secs_f64())];
    if let Some(rate) = model.rate {
        let rate = if rate.is_finite() && rate >= 0.0 {
            crate::formatter::ratings::format_speed_plain(rate * 1_000_000.0, bytes)
        } else {
            "unavailable".into()
        };
        lines.push(format!("  {rate}"));
        if layout != Layout::Minimal {
            let max = model
                .samples
                .iter()
                .map(|(_, _, rate)| *rate)
                .filter(|v| v.is_finite())
                .fold(1.0, f64::max);
            let glyphs = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
            let mut previous = None;
            let mut chart = String::new();
            for (sequence, timestamp, value) in &model.samples {
                if previous.is_some_and(|(last, time)| {
                    *sequence > last + 1
                        || timestamp.saturating_duration_since(time) > Duration::from_millis(100)
                }) {
                    chart.push(' ');
                }
                chart.push(if value.is_finite() && *value >= 0.0 {
                    glyphs[((value / max).clamp(0.0, 1.0) * 7.0).round() as usize]
                } else {
                    ' '
                });
                previous = Some((*sequence, *timestamp));
            }
            let chart = chart
                .chars()
                .rev()
                .take(
                    (width as usize)
                        .saturating_sub(12)
                        .min(layout.chart_width()),
                )
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>();
            lines.push(format!("  {chart} samples"));
            if matches!(layout, Layout::Standard | Layout::Expanded) {
                lines.push(format!(
                    "  {} transferred",
                    crate::common::format_data_size(model.bytes)
                ));
            }
        }
    }
    if matches!(layout, Layout::Standard | Layout::Expanded) {
        lines.push(format!(
            "  Download {}   Upload {}",
            model.download.text(bytes),
            model.upload.text(bytes)
        ));
    }
    if layout != Layout::Minimal && !model.context.is_empty() {
        lines.push(format!("  {}", model.context));
    }
    lines.truncate((height as usize).saturating_sub(1).max(1));
    lines
        .into_iter()
        .map(|line| {
            console::truncate_str(&line, (width as usize).saturating_sub(1), "").into_owned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_modes_use_both_dimensions() {
        assert_eq!(Layout::for_size(40, 24), Layout::Minimal);
        assert_eq!(Layout::for_size(60, 24), Layout::Compact);
        assert_eq!(Layout::for_size(80, 24), Layout::Standard);
        assert_eq!(Layout::for_size(120, 24), Layout::Expanded);
        assert_eq!(Layout::for_size(120, 4), Layout::Minimal);
        assert_eq!(Layout::for_size(120, 6), Layout::Compact);
        assert_eq!(Layout::for_size(120, 8), Layout::Standard);
    }

    #[test]
    fn lifecycle_keeps_completed_data_and_distinguishes_skips_failure_and_interruption() {
        let handle = Handle::new();
        handle.phase(Phase::TestingDownload);
        handle.transfer_finished(false, Some(80.0));
        handle.phase(Phase::TestingUpload);
        handle.finish(RunState::Failed);
        {
            let model = handle.0.lock().unwrap();
            assert_eq!(model.download, TransferState::Completed(80.0));
            assert_eq!(model.upload, TransferState::Failed);
            assert_eq!(model.state, RunState::Failed);
        }
        let handle = Handle::new();
        handle.transfer_finished(false, None);
        handle.phase(Phase::TestingUpload);
        drop(Session {
            handle: handle.clone(),
            stop: Arc::new(AtomicBool::new(false)),
            worker: None,
        });
        let model = handle.0.lock().unwrap();
        assert_eq!(model.download, TransferState::Skipped);
        assert_eq!(model.upload, TransferState::Interrupted);
        assert_eq!(model.state, RunState::Interrupted);
    }

    #[test]
    fn completed_zero_and_unavailable_transfer_are_distinct() {
        let handle = Handle::new();
        handle.transfer_finished(false, Some(0.0));
        handle.transfer_finished(true, Some(f64::NAN));
        handle.finish(RunState::Completed);
        let model = handle.0.lock().unwrap();
        assert_eq!(model.download, TransferState::Completed(0.0));
        assert_eq!(model.upload, TransferState::Unavailable);
        assert_eq!(model.state, RunState::Completed);
    }
    #[cfg(unix)]
    #[test]
    fn full_terminal_buffer_returns_without_blocking_or_changing_stream_flags() {
        use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
        let mut master = -1;
        let mut slave = -1;
        // SAFETY: openpty receives valid pointers for its two output descriptors.
        assert_eq!(
            unsafe {
                libc::openpty(
                    &mut master,
                    &mut slave,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            },
            0
        );
        // SAFETY: successful openpty transfers ownership of both descriptors.
        let _master = unsafe { OwnedFd::from_raw_fd(master) };
        // SAFETY: successful openpty transfers ownership of the slave descriptor.
        let slave = unsafe { OwnedFd::from_raw_fd(slave) };
        // SAFETY: F_GETFL reads flags from an owned, valid descriptor.
        let original_flags = unsafe { libc::fcntl(slave.as_raw_fd(), libc::F_GETFL) };
        let mut output = Output::open_terminal(slave.as_raw_fd()).unwrap();
        let block = "x".repeat(4096);
        let start = Instant::now();
        let mut full = false;
        for _ in 0..1024 {
            if let Err(error) = output.write(&block) {
                assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);
                full = true;
                break;
            }
        }
        assert!(full, "expected the unread PTY buffer to fill");
        assert!(start.elapsed() < Duration::from_secs(2));
        // SAFETY: F_GETFL reads flags from the same owned descriptor.
        assert_eq!(
            unsafe { libc::fcntl(slave.as_raw_fd(), libc::F_GETFL) },
            original_flags
        );
    }
    fn fixture() -> Model {
        Model {
            phase: Phase::TestingDownload,
            state: RunState::Running,
            download: TransferState::Running,
            upload: TransferState::Pending,
            rate: Some(80.0),
            samples: vec![
                (1, Instant::now(), 80.0),
                (2, Instant::now(), 0.0),
                (4, Instant::now(), 40.0),
            ],
            bytes: 1_000_000,
            context: "東京 Test Server".into(),
            warnings: Vec::new(),
            omitted_warnings: 0,
        }
    }
    #[test]
    fn live_frames_fit_all_supported_sizes_and_byte_mode() {
        for width in [40, 60, 80, 120] {
            for height in [1, 4, 8, 24] {
                let frame = render(&fixture(), width, height, Duration::from_secs(1), true);
                assert!(frame.len() <= (height as usize).saturating_sub(1).max(1));
                assert!(
                    frame
                        .iter()
                        .all(|line| console::measure_text_width(line) < width as usize)
                );
                if height >= 4 {
                    assert!(frame.join("\n").contains("MB/s"));
                }
            }
        }
    }
    #[test]
    fn chart_keeps_zero_and_missing_sample_gap() {
        let frame = render(&fixture(), 80, 24, Duration::from_secs(1), false).join("\n");
        assert!(frame.contains("█▁ ▅"));
    }
    #[test]
    fn display_publication_is_bounded_and_nonblocking() {
        let handle = Handle(Arc::new(Mutex::new(fixture())), Arc::new(AtomicU64::new(0)));
        {
            let _guard = handle.0.lock().unwrap();
            handle.sample(1.0, 1);
        }
        for i in 0..1000 {
            handle.sample(i as f64, i);
        }
        assert_eq!(handle.0.lock().unwrap().samples.len(), 48);
    }

    #[test]
    #[ignore = "manual deterministic visual and performance evidence"]
    fn render_evidence() {
        let fixture = fixture();
        println!(
            "Minimal fixture\n{}",
            render(&fixture, 80, 4, Duration::from_secs(1), false).join("\n")
        );
        println!(
            "Diagnostic fixture\n{}",
            render(&fixture, 80, 24, Duration::from_secs(1), false).join("\n")
        );
        let start = Instant::now();
        for _ in 0..10_000 {
            std::hint::black_box(render(&fixture, 80, 24, Duration::from_secs(1), false));
        }
        println!("10000 frame builds took {:?}", start.elapsed());
        let handle = Handle(Arc::new(Mutex::new(fixture)), Arc::new(AtomicU64::new(0)));
        let start = Instant::now();
        for index in 0..100_000 {
            handle.sample(std::hint::black_box(100.0), index);
        }
        println!("100000 live publications took {:?}", start.elapsed());
    }
}
