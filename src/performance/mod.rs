//! Local performance capture. Presented-frame cadence, not a display/generated-frame FPS claim.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, VecDeque},
    io::{BufRead, BufReader, Read},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[cfg(windows)]
mod windows;
#[cfg(not(windows))]
mod fallback {
    use super::*;
    #[derive(Default)]
    pub struct Hardware;
    impl Hardware {
        pub fn sample(&mut self) -> HardwareReading {
            HardwareReading::default()
        }
    }
    pub fn applications() -> Vec<Application> {
        vec![]
    }
    pub fn foreground() -> Option<Application> {
        None
    }
    pub fn foreground_window() -> Option<Foreground> {
        None
    }
    pub fn prepare_command(_: &mut Command) {}
    pub struct ChildGuard;
    pub fn guard_child(_: &std::process::Child) -> Result<ChildGuard, String> {
        Ok(ChildGuard)
    }
    pub fn open_folder(_: &std::path::Path) -> Result<(), String> {
        Err("Open the export folder in your file manager.".into())
    }
}
#[cfg(not(windows))]
use fallback as native;
#[cfg(windows)]
use windows as native;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LineStyle {
    #[default]
    TextOnly,
    Background,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Configuration {
    pub enabled: bool,
    pub target: String,
    pub demo: bool,
    /// Lowercase executable names that always count as games.
    pub games: Vec<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Application {
    pub pid: u32,
    pub name: String,
}
/// What owns the foreground window.
pub enum Foreground {
    Own,
    App { app: Application, fullscreen: bool },
}

/// Fullscreen applications that render continuously but are not games.
const NOT_GAMES: &[&str] = &[
    "chrome.exe",
    "msedge.exe",
    "firefox.exe",
    "opera.exe",
    "brave.exe",
    "vivaldi.exe",
    "vlc.exe",
    "mpc-hc64.exe",
    "mpc-be64.exe",
    "potplayermini64.exe",
    "wmplayer.exe",
    "microsoft.media.player.exe",
    "video.ui.exe",
    "powerpnt.exe",
];

fn game_in_foreground(
    foreground: Option<Foreground>,
    target: Option<&Application>,
    presenting: bool,
    was_gaming: bool,
    games: &[String],
) -> bool {
    presenting
        && match foreground {
            // Sela takes the foreground while its panel is open over a game.
            Some(Foreground::Own) => was_gaming,
            Some(Foreground::App { app, fullscreen }) => {
                target.is_some_and(|t| t.pid == app.pid) && is_game(&app.name, fullscreen, games)
            }
            None => false,
        }
}

/// Brief alt-tabs and loading screens must not make the gaming line flicker.
const GAME_GRACE: Duration = Duration::from_secs(2);

/// A presenting foreground application is a game when the player marked it as
/// one, or when it covers its monitor and is not a known video or browser app.
pub fn is_game(name: &str, fullscreen: bool, games: &[String]) -> bool {
    let name = name.to_ascii_lowercase();
    games.contains(&name) || fullscreen && !NOT_GAMES.contains(&name.as_str())
}

#[derive(Debug, Clone, Default)]
pub struct HardwareReading {
    pub gpu: Option<f64>,
    pub vram_gb: Option<f64>,
    pub gpu_temp: Option<f64>,
    pub cpu_temp: Option<f64>,
    pub power: Option<f64>,
    pub source: String,
}
#[derive(Debug, Clone, Default)]
pub struct Reading {
    pub frames: Option<FrameStats>,
    pub cpu: Option<f64>,
    pub ram_gb: Option<f64>,
    pub hardware: HardwareReading,
    pub target: Option<Application>,
    pub applications: Vec<Application>,
    pub status: String,
    pub history: Vec<f64>,
    pub recording_seconds: Option<f64>,
    pub completed: Option<Arc<Session>>,
    pub demo: bool,
    /// A game is in the foreground and presenting frames.
    pub gaming: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameStats {
    pub fps: f64,
    pub low: f64,
    pub frame_ms: f64,
    pub worst_ms: f64,
    pub frames: usize,
}
impl FrameStats {
    pub fn calculate(values: &[f64]) -> Option<Self> {
        if values.is_empty() || values.iter().any(|v| !v.is_finite() || *v <= 0.0) {
            return None;
        }
        let sum: f64 = values.iter().sum();
        if !sum.is_finite() {
            return None;
        }
        let mut slow = values.to_vec();
        slow.sort_by(|a, b| b.total_cmp(a));
        let n = values.len().div_ceil(100);
        Some(Self {
            fps: 1000.0 * values.len() as f64 / sum,
            low: 1000.0 / (slow[..n].iter().sum::<f64>() / n as f64),
            frame_ms: sum / values.len() as f64,
            worst_ms: slow[0],
            frames: values.len(),
        })
    }
}
#[derive(Debug, Clone)]
pub struct Frame {
    pub pid: u32,
    pub chain: String,
    pub ms: f64,
}
/// Read header names, not fixed positions; application names may contain quoted commas.
#[derive(Default)]
pub struct CsvFrames {
    columns: Option<(usize, usize, usize)>,
}
impl CsvFrames {
    pub fn parse(&mut self, line: &str) -> Option<Frame> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(line.trim_start_matches('\u{feff}').as_bytes());
        let row = reader.records().next()?.ok()?;
        if let Some(pid) = row.iter().position(|v| v.eq_ignore_ascii_case("ProcessID")) {
            self.columns = Some((
                pid,
                row.iter()
                    .position(|v| v.eq_ignore_ascii_case("SwapChainAddress"))?,
                row.iter()
                    .position(|v| v.eq_ignore_ascii_case("MsBetweenPresents"))?,
            ));
            return None;
        }
        let (pid, chain, ms) = self.columns?;
        let ms: f64 = row.get(ms)?.parse().ok()?;
        if !ms.is_finite() || ms <= 0.0 || ms > 10_000.0 {
            return None;
        }
        Some(Frame {
            pid: row.get(pid)?.parse().ok()?,
            chain: row.get(chain)?.to_owned(),
            ms,
        })
    }
}
#[derive(Default)]
pub struct FrameWindow {
    chains: BTreeMap<String, VecDeque<(Instant, f64)>>,
    pid: u32,
}
impl FrameWindow {
    pub fn reset(&mut self, pid: u32) {
        self.chains.clear();
        self.pid = pid;
    }
    pub fn push(&mut self, frame: &Frame, now: Instant) {
        if frame.pid != self.pid {
            return;
        }
        if self.chains.len() >= 16 && !self.chains.contains_key(&frame.chain) {
            return;
        }
        let chain = self.chains.entry(frame.chain.clone()).or_default();
        chain.push_back((now, frame.ms));
        if chain.len() > 8192 {
            chain.pop_front();
        }
    }
    pub fn current(&mut self, now: Instant) -> Option<(String, FrameStats, Vec<f64>)> {
        for chain in self.chains.values_mut() {
            while chain
                .front()
                .is_some_and(|(t, _)| now.saturating_duration_since(*t) > Duration::from_secs(5))
            {
                chain.pop_front();
            }
        }
        let (key, chain) = self
            .chains
            .iter()
            .filter(|(_, c)| {
                c.back().is_some_and(|(t, _)| {
                    now.saturating_duration_since(*t) < Duration::from_secs(2)
                })
            })
            .max_by_key(|(_, c)| c.len())?;
        let values: Vec<f64> = chain.iter().map(|(_, v)| *v).collect();
        let stats = FrameStats::calculate(&values)?;
        let history = values
            .iter()
            .rev()
            .take(120)
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        Some((key.clone(), stats, history))
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: u128,
    pub application: String,
    pub pid: u32,
    pub swap_chain: String,
    pub demo: bool,
    pub frame_times_ms: Vec<f64>,
    pub stats: FrameStats,
    pub captured_seconds: f64,
}
impl Session {
    pub fn export(&self) -> Result<PathBuf, String> {
        let folder = directories::ProjectDirs::from("dev", "Sela", "Sela")
            .ok_or("Data folder unavailable")?
            .data_local_dir()
            .join("benchmarks");
        std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
        let base = folder.join(format!("Sela-{}", self.id));
        let json = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(base.with_extension("json"), json).map_err(|e| e.to_string())?;
        let mut csv =
            csv::Writer::from_path(base.with_extension("csv")).map_err(|e| e.to_string())?;
        csv.write_record([
            "source",
            "application",
            "pid",
            "swap_chain",
            "frame",
            "elapsed_ms",
            "frame_time_ms",
        ])
        .map_err(|e| e.to_string())?;
        let application = if self.application.starts_with(['=', '+', '-', '@']) {
            format!("'{}", self.application)
        } else {
            self.application.clone()
        };
        let mut elapsed = 0.0;
        for (i, ms) in self.frame_times_ms.iter().enumerate() {
            elapsed += ms;
            csv.write_record([
                if self.demo { "demo" } else { "PresentMon" }.to_owned(),
                application.clone(),
                self.pid.to_string(),
                self.swap_chain.clone(),
                (i + 1).to_string(),
                format!("{elapsed:.4}"),
                format!("{ms:.4}"),
            ])
            .map_err(|e| e.to_string())?;
        }
        csv.flush().map_err(|e| e.to_string())?;
        Ok(folder)
    }
}
struct Recording {
    target: Application,
    chain: String,
    frames: Vec<f64>,
    /// Running total of `frames`, in milliseconds.
    captured_ms: f64,
    duration: f64,
    demo: bool,
}
impl Recording {
    fn push(&mut self, frame: &Frame) {
        if frame.pid == self.target.pid && frame.chain == self.chain {
            self.frames.push(frame.ms);
            self.captured_ms += frame.ms;
        }
    }
    fn finish(self) -> Option<Arc<Session>> {
        let stats = FrameStats::calculate(&self.frames)?;
        Some(Arc::new(Session {
            id: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()?
                .as_micros(),
            application: self.target.name,
            pid: self.target.pid,
            swap_chain: self.chain,
            demo: self.demo,
            captured_seconds: self.captured_ms / 1000.0,
            frame_times_ms: self.frames,
            stats,
        }))
    }
}
enum Control {
    Configure(Configuration),
    Start(u32),
    Finish,
    Retry,
}
pub struct Monitor {
    pub updates: mpsc::Receiver<Reading>,
    commands: mpsc::Sender<Control>,
    configuration: Configuration,
}
impl Monitor {
    pub fn configure(&mut self, c: Configuration) {
        if c != self.configuration {
            self.configuration = c.clone();
            let _ = self.commands.send(Control::Configure(c));
        }
    }
    pub fn start(&self, seconds: u32) {
        let _ = self.commands.send(Control::Start(seconds.clamp(1, 600)));
    }
    pub fn finish(&self) {
        let _ = self.commands.send(Control::Finish);
    }
    pub fn retry(&self) {
        let _ = self.commands.send(Control::Retry);
    }
}
struct Capture {
    _guard: native::ChildGuard,
    provider: PathBuf,
    session: String,
    child: Child,
    frames: mpsc::Receiver<Frame>,
    diagnostics: Arc<Mutex<VecDeque<String>>>,
}
impl Drop for Capture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let mut cleanup = Command::new(&self.provider);
        cleanup
            .args([
                "--session_name",
                &self.session,
                "--terminate_existing_session",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        native::prepare_command(&mut cleanup);
        let _ = cleanup.status();
    }
}
fn remember_diagnostic(log: &Mutex<VecDeque<String>>, line: &str) {
    if let Ok(mut log) = log.lock() {
        if log.len() >= 32 {
            log.pop_front();
        }
        log.push_back(line.trim().chars().take(512).collect());
    }
}
fn capture_diagnostics(capture: &Capture) -> String {
    capture
        .diagnostics
        .lock()
        .map(|lines| lines.iter().cloned().collect::<Vec<_>>().join(" | "))
        .unwrap_or_default()
}
fn provider() -> Result<PathBuf, String> {
    const BYTES: &[u8] = include_bytes!(env!("SELA_PRESENTMON_BUNDLE"));
    const HASH: &str = "b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af";
    if BYTES.is_empty() {
        return Err("PresentMon is not bundled in this development build. Use the Windows installer or portable release.".into());
    }
    use sha2::{Digest, Sha256};
    if format!("{:x}", Sha256::digest(BYTES)) != HASH {
        return Err("Bundled PresentMon checksum mismatch.".into());
    }
    let folder = directories::ProjectDirs::from("dev", "Sela", "Sela")
        .ok_or("Cache folder unavailable")?
        .cache_dir()
        .join("providers/2.6.0");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let path = folder.join("PresentMon.exe");
    if std::fs::read(&path)
        .ok()
        .is_none_or(|b| format!("{:x}", Sha256::digest(&b)) != HASH)
    {
        let temporary = path.with_extension("tmp");
        std::fs::write(&temporary, BYTES).map_err(|e| e.to_string())?;
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        std::fs::rename(temporary, &path).map_err(|e| e.to_string())?;
    }
    Ok(path)
}
fn capture(app: &Application) -> Result<Capture, String> {
    let provider = provider()?;
    use sha2::{Digest, Sha256};
    let user_key = format!(
        "{:x}",
        Sha256::digest(provider.to_string_lossy().as_bytes())
    );
    let session = format!("Sela-{}", &user_key[..12]);
    let mut command = Command::new(&provider);
    command.args([
        "--process_id",
        &app.pid.to_string(),
        "--output_stdout",
        "--no_console_stats",
        "--v1_metrics",
        "--no_track_display",
        "--no_track_input",
        "--no_track_gpu",
        "--terminate_on_proc_exit",
        "--session_name",
        &session,
        "--stop_existing_session",
    ]);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    native::prepare_command(&mut command);
    let mut child = command
        .spawn()
        .map_err(|e| format!("Cannot start PresentMon: {e}"))?;
    let guard = match native::guard_child(&child) {
        Ok(g) => g,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
    };
    let stdout = child.stdout.take().ok_or("PresentMon stdout unavailable")?;
    let diagnostics = Arc::new(Mutex::new(VecDeque::new()));
    if let Some(stderr) = child.stderr.take() {
        let log = diagnostics.clone();
        thread::spawn(move || {
            let mut input = BufReader::new(stderr);
            let mut line = String::new();
            loop {
                line.clear();
                match input.by_ref().take(64 * 1024).read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        remember_diagnostic(&log, &line);
                        if line.len() >= 64 * 1024 {
                            break;
                        }
                    }
                }
            }
        });
    }
    let output_log = diagnostics.clone();
    let (tx, frames) = mpsc::sync_channel(4096);
    thread::spawn(move || {
        let mut parser = CsvFrames::default();
        let mut input = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            line.clear();
            match input.by_ref().take(64 * 1024).read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if line.len() >= 64 * 1024 {
                        break;
                    }
                    if let Some(frame) = parser.parse(&line) {
                        if tx.send(frame).is_err() {
                            break;
                        }
                    } else {
                        remember_diagnostic(&output_log, &line);
                    }
                }
            }
        }
    });
    Ok(Capture {
        child,
        frames,
        _guard: guard,
        provider,
        session,
        diagnostics,
    })
}

pub fn spawn() -> Monitor {
    let (commands, rx) = mpsc::channel();
    let (tx, updates) = mpsc::channel();
    thread::Builder::new()
        .name("sela-performance".into())
        .spawn(move || run(rx, tx))
        .expect("cannot start performance worker");
    Monitor {
        updates,
        commands,
        configuration: Configuration::default(),
    }
}
fn run(rx: mpsc::Receiver<Control>, tx: mpsc::Sender<Reading>) {
    let mut config = Configuration::default();
    let mut reading = Reading::default();
    let mut stream: Option<Capture> = None;
    let mut target: Option<Application> = None;
    let mut window = FrameWindow::default();
    let mut hardware: Option<native::Hardware> = None;
    let mut system = sysinfo::System::new();
    let mut recording: Option<Recording> = None;
    let mut cpu_ready = false;
    let mut blocked = false;
    let mut last_apps = Instant::now() - Duration::from_secs(3);
    let mut last_hardware = last_apps;
    let mut last_game: Option<Instant> = None;
    loop {
        match rx.recv_timeout(if config.enabled {
            Duration::from_millis(250)
        } else {
            Duration::from_secs(30)
        }) {
            Ok(Control::Configure(next)) => {
                // The games list only affects detection; keep the running capture.
                if (next.enabled, &next.target, next.demo)
                    != (config.enabled, &config.target, config.demo)
                {
                    if let Some(r) = recording.take() {
                        reading.completed = r.finish();
                    }
                    stream = None;
                    target = None;
                    window.reset(0);
                    blocked = false;
                    reading.frames = None;
                    reading.history.clear();
                    reading.status = String::new();
                }
                config = next;
            }
            Ok(Control::Start(seconds)) => {
                if config.enabled {
                    if let (Some(app), Some((chain, _, _))) =
                        (target.clone(), window.current(Instant::now()))
                    {
                        recording = Some(Recording {
                            target: app,
                            chain,
                            frames: vec![],
                            captured_ms: 0.0,
                            duration: seconds as f64,
                            demo: config.demo,
                        });
                    }
                }
            }
            Ok(Control::Finish) => {
                if let Some(r) = recording.take() {
                    reading.completed = r.finish();
                }
            }
            Ok(Control::Retry) => {
                stream = None;
                blocked = false;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        let now = Instant::now();
        reading.demo = config.demo;
        if !config.enabled {
            cpu_ready = false;
            stream = None;
            hardware = None;
            reading.frames = None;
            reading.cpu = None;
            reading.ram_gb = None;
            reading.hardware = Default::default();
            reading.target = None;
            reading.history.clear();
            reading.recording_seconds = None;
            reading.gaming = false;
            last_game = None;
            reading.status = "Performance monitoring is off.".into();
            if tx.send(reading.clone()).is_err() {
                break;
            }
            continue;
        }
        if now.duration_since(last_apps) >= Duration::from_secs(2) {
            reading.applications = if config.demo {
                vec![Application {
                    pid: 1,
                    name: "Sela demo.exe".into(),
                }]
            } else {
                native::applications()
            };
            last_apps = now;
        }
        let requested = if config.demo {
            Some(Application {
                pid: 1,
                name: "Sela demo.exe".into(),
            })
        } else if config.target.is_empty() {
            native::foreground().or_else(|| {
                target
                    .clone()
                    .filter(|t| reading.applications.iter().any(|a| a.pid == t.pid))
            })
        } else {
            reading
                .applications
                .iter()
                .find(|p| p.name.eq_ignore_ascii_case(&config.target))
                .cloned()
        };
        if requested != target {
            if let Some(r) = recording.take() {
                reading.completed = r.finish();
            }
            stream = None;
            target = requested;
            window.reset(target.as_ref().map_or(0, |p| p.pid));
            blocked = false;
            reading.frames = None;
            reading.history.clear();
            reading.status.clear();
        }
        reading.target = target.clone();
        if !config.demo && stream.is_none() && !blocked {
            if let Some(app) = &target {
                match capture(app) {
                    Ok(c) => {
                        stream = Some(c);
                        reading.status = "Waiting for PresentMon frames…".into();
                    }
                    Err(e) => {
                        reading.status = e;
                        blocked = true;
                    }
                }
            } else {
                reading.status =
                    "Waiting for a foreground application. Choose a game below.".into();
            }
        }
        let mut frames = Vec::new();
        if config.demo {
            for _ in 0..35 {
                frames.push(Frame {
                    pid: 1,
                    chain: "demo".into(),
                    ms: 7.0,
                });
            }
        }
        if let Some(c) = &mut stream {
            frames.extend(c.frames.try_iter().take(8192));
            if let Ok(Some(code)) = c.child.try_wait() {
                reading.status=format!("PresentMon stopped ({code}). Check capture permissions; try running Sela as administrator, then Retry. {}", capture_diagnostics(c));
                blocked = true;
                stream = None;
            }
        }
        for frame in frames {
            window.push(&frame, now);
            if let Some(r) = &mut recording {
                r.push(&frame);
            }
        }
        let current = window.current(now);
        reading.frames = current.as_ref().map(|(_, stats, _)| stats.clone());
        reading.history = current.map(|(_, _, h)| h).unwrap_or_default();
        if reading.frames.is_some() {
            reading.status = if config.demo {
                "DEMO · simulated frames and hardware · no game capture"
            } else {
                "PresentMon · presented-frame cadence · rolling 5 seconds"
            }
            .into();
        } else if stream.is_some() {
            reading.status="No recent frames. Keep the game running; choose its process or check capture permissions.".into();
        }
        // Without capture permission there are no frames to confirm rendering;
        // fall back to the fullscreen check alone rather than never showing the line.
        let presenting = reading.frames.is_some() || blocked && target.is_some();
        let detected = config.demo
            || game_in_foreground(
                native::foreground_window(),
                target.as_ref(),
                presenting,
                reading.gaming,
                &config.games,
            );
        if detected {
            last_game = Some(now);
        }
        reading.gaming = last_game.is_some_and(|t| now.duration_since(t) < GAME_GRACE);
        if now.duration_since(last_hardware) >= Duration::from_secs(1) {
            if config.demo {
                reading.cpu = Some(35.0);
                reading.ram_gb = Some(10.8);
                reading.hardware = HardwareReading {
                    gpu: Some(64.0),
                    vram_gb: Some(5.6),
                    gpu_temp: Some(64.0),
                    cpu_temp: Some(59.0),
                    power: Some(142.0),
                    source: "DEMO · sample hardware".into(),
                };
            } else {
                system.refresh_cpu_usage();
                system.refresh_memory();
                reading.cpu = cpu_ready.then(|| system.global_cpu_usage() as f64);
                cpu_ready = true;
                reading.ram_gb = Some(system.used_memory() as f64 / 1_073_741_824.0);
                reading.hardware = hardware.get_or_insert_with(Default::default).sample();
            }
            last_hardware = now;
        }
        reading.recording_seconds = recording.as_ref().map(|r| r.captured_ms / 1000.0);
        if recording
            .as_ref()
            .is_some_and(|r| r.frames.len() >= 1_000_000 || r.captured_ms / 1000.0 >= r.duration)
        {
            reading.completed = recording.take().and_then(Recording::finish);
            reading.recording_seconds = None;
        }
        if tx.send(reading.clone()).is_err() {
            break;
        }
    }
}

pub fn open_folder(path: &std::path::Path) -> Result<(), String> {
    native::open_folder(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_handles_reordered_headers_quoted_names_and_invalid_samples() {
        let mut parser = CsvFrames::default();
        assert!(parser
            .parse("\u{feff}Application,MsBetweenPresents,SwapChainAddress,ProcessID")
            .is_none());
        let f = parser.parse("\"Game, Title.exe\",16.5,0x42,123").unwrap();
        assert_eq!((f.pid, f.chain, f.ms), (123, "0x42".into(), 16.5));
        for row in [
            "game,NaN,0,123",
            "game,0,0,123",
            "game,-1,0,123",
            "game,10001,0,123",
            "bad",
        ] {
            assert!(parser.parse(row).is_none());
        }
    }
    #[test]
    fn pinned_presentmon_v1_header_uses_lowercase_ms() {
        // PresentMon 2.6.0 CsvOutput.cpp: WriteCsvHeader<FrameMetrics1>.
        let mut parser = CsvFrames::default();
        assert!(parser.parse("Application,ProcessID,SwapChainAddress,Runtime,SyncInterval,PresentFlags,Dropped,TimeInSeconds,msInPresentAPI,msBetweenPresents").is_none());
        let frame = parser
            .parse("game.exe,7,0x42,DXGI,0,0,0,1.5,0.1,16.6667")
            .unwrap();
        assert_eq!(frame.pid, 7);
        assert_eq!(frame.chain, "0x42");
        assert_eq!(frame.ms, 16.6667);
    }
    #[test]
    fn games_are_fullscreen_non_media_apps_or_marked() {
        let marked = vec!["chrome.exe".to_owned()];
        assert!(is_game("Game.exe", true, &[]));
        assert!(!is_game("Game.exe", false, &[]));
        assert!(!is_game("Chrome.exe", true, &[]));
        assert!(is_game("Chrome.exe", false, &marked));
    }

    #[test]
    fn detection_follows_the_presenting_foreground_target() {
        let game = Application {
            pid: 7,
            name: "game.exe".into(),
        };
        let front = |pid, fullscreen| {
            Some(Foreground::App {
                app: Application {
                    pid,
                    name: "game.exe".into(),
                },
                fullscreen,
            })
        };
        assert!(game_in_foreground(
            front(7, true),
            Some(&game),
            true,
            false,
            &[]
        ));
        assert!(!game_in_foreground(
            front(8, true),
            Some(&game),
            true,
            false,
            &[]
        ));
        assert!(!game_in_foreground(
            front(7, false),
            Some(&game),
            true,
            false,
            &[]
        ));
        // Sela in front keeps the state only while the game still presents.
        assert!(game_in_foreground(
            Some(Foreground::Own),
            Some(&game),
            true,
            true,
            &[]
        ));
        assert!(!game_in_foreground(
            Some(Foreground::Own),
            None,
            false,
            true,
            &[]
        ));
        assert!(!game_in_foreground(None, Some(&game), true, true, &[]));
    }

    #[test]
    fn fps_and_low_use_captured_frame_times() {
        let stats = FrameStats::calculate(&[10.0, 10.0, 20.0]).unwrap();
        assert_eq!(stats.fps, 75.0);
        assert_eq!(stats.low, 50.0);
        assert_eq!(stats.frame_ms, 40.0 / 3.0);
        let mut values = vec![10.0; 99];
        values.push(100.0);
        assert_eq!(FrameStats::calculate(&values).unwrap().low, 10.0);
        assert!(FrameStats::calculate(&[]).is_none());
        assert!(FrameStats::calculate(&[f64::NAN]).is_none());
    }
    #[test]
    fn swap_chains_are_not_summed_and_stale_pids_cannot_replace_target() {
        let now = Instant::now();
        let mut window = FrameWindow::default();
        window.reset(7);
        for _ in 0..10 {
            window.push(
                &Frame {
                    pid: 7,
                    chain: "main".into(),
                    ms: 10.0,
                },
                now,
            );
        }
        for _ in 0..5 {
            window.push(
                &Frame {
                    pid: 7,
                    chain: "overlay".into(),
                    ms: 20.0,
                },
                now,
            );
        }
        for _ in 0..100 {
            window.push(
                &Frame {
                    pid: 8,
                    chain: "wrong".into(),
                    ms: 1.0,
                },
                now,
            );
        }
        let (chain, stats, _) = window.current(now).unwrap();
        assert_eq!(chain, "main");
        assert_eq!(stats.fps, 100.0);
        assert_eq!(stats.frames, 10);
        assert!(window.current(now + Duration::from_secs(3)).is_none());
        window.reset(8);
        assert!(window.current(now).is_none());
    }
    #[test]
    fn benchmark_does_not_contain_other_processes_or_chains() {
        let mut r = Recording {
            target: Application {
                pid: 3,
                name: "game.exe".into(),
            },
            chain: "a".into(),
            frames: vec![],
            captured_ms: 0.0,
            duration: 30.0,
            demo: false,
        };
        r.push(&Frame {
            pid: 99,
            chain: "a".into(),
            ms: 1.0,
        });
        r.push(&Frame {
            pid: 3,
            chain: "wrong".into(),
            ms: 1.0,
        });
        r.push(&Frame {
            pid: 3,
            chain: "a".into(),
            ms: 10.0,
        });
        r.push(&Frame {
            pid: 3,
            chain: "a".into(),
            ms: 20.0,
        });
        let session = r.finish().unwrap();
        assert_eq!(session.stats.frames, 2);
        assert_eq!(session.pid, 3);
        assert_eq!(session.swap_chain, "a");
        assert_eq!(session.captured_seconds, 0.03);
    }
    #[test]
    fn demo_worker_is_explicit_and_stops_when_disabled() {
        let mut monitor = spawn();
        monitor.configure(Configuration {
            enabled: true,
            demo: true,
            target: String::new(),
            games: vec![],
        });
        loop {
            let r = monitor
                .updates
                .recv_timeout(Duration::from_secs(3))
                .unwrap();
            if r.frames.is_some() {
                assert!(r.gaming);
                assert!(r.demo);
                break;
            }
        }
        monitor.start(1);
        let session = loop {
            let r = monitor
                .updates
                .recv_timeout(Duration::from_secs(3))
                .unwrap();
            if let Some(s) = r.completed {
                break s;
            }
        };
        assert!(session.demo);
        assert!(session.captured_seconds >= 1.0);
        monitor.configure(Configuration::default());
        loop {
            let r = monitor
                .updates
                .recv_timeout(Duration::from_secs(3))
                .unwrap();
            if r.status == "Performance monitoring is off." {
                assert!(r.frames.is_none());
                break;
            }
        }
    }
}
