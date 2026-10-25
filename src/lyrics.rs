//! LRCLIB lookup is confined to a cancellable worker. The UI owns selection and timing.
use serde::{Deserialize, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    io::Read,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackKey {
    pub title: String,
    pub artist: String,
    pub duration: u32,
}
impl TrackKey {
    pub fn from_media(media: &crate::platform::Media) -> Option<Self> {
        let title = media.title.trim();
        let artist = media.artist.trim();
        if title.is_empty() || artist.is_empty() {
            return None;
        }
        Some(Self {
            title: title.into(),
            artist: artist.into(),
            duration: if media.duration.is_finite() {
                media.duration.max(0.0).round() as u32
            } else {
                0
            },
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Line {
    pub seconds: f64,
    pub text: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Data {
    Synced(Vec<Line>),
    Plain(String),
    Instrumental,
    NotFound,
}
impl Data {
    pub fn active_index(&self, position: f64) -> Option<usize> {
        let Self::Synced(lines) = self else {
            return None;
        };
        if !position.is_finite() {
            return None;
        }
        lines
            .partition_point(|line| line.seconds <= position)
            .checked_sub(1)
    }
}
#[derive(Debug, Clone)]
pub enum State {
    NoTrack,
    Disabled,
    Loading,
    Ready(Data),
    Error(String),
}
impl State {
    pub fn compact_line(&self, position: f64) -> &str {
        match self {
            Self::NoTrack => "Artist unavailable",
            Self::Disabled => "Lyrics are off",
            Self::Loading => "Finding lyrics…",
            Self::Error(_) => "Lyrics couldn't load · open panel",
            Self::Ready(data) => match data {
                Data::Synced(lines) => data
                    .active_index(position)
                    .map(|i| lines[i].text.as_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("Waiting for the next line…"),
                Data::Plain(_) => "Untimed lyrics · open panel",
                Data::Instrumental => "Instrumental",
                Data::NotFound => "Lyrics unavailable",
            },
        }
    }
}

/// Accept minute:second fractions and repeated timestamps. Metadata is not lyric text.
/// LRC positive offset makes lines appear earlier; negative offset makes them later.
pub fn parse_lrc(input: &str) -> Vec<Line> {
    let mut offset = 0.0;
    for raw in input.lines() {
        if let Some(value) = raw
            .trim()
            .strip_prefix("[offset:")
            .and_then(|s| s.strip_suffix(']'))
        {
            if let Ok(ms) = value.trim().parse::<i64>() {
                offset = ms as f64 / 1000.0;
            }
        }
    }
    let mut lines = Vec::new();
    for raw in input.lines() {
        let mut rest = raw.trim().trim_start_matches('\u{feff}');
        let mut times = Vec::new();
        while let Some(tag) = rest.strip_prefix('[').and_then(|s| s.split_once(']')) {
            let Some((minutes, seconds)) = tag.0.split_once(':') else {
                break;
            };
            let (Ok(minutes), Ok(seconds)) = (minutes.parse::<u32>(), seconds.parse::<f64>())
            else {
                break;
            };
            if !seconds.is_finite() || !(0.0..60.0).contains(&seconds) {
                break;
            }
            times.push((minutes as f64 * 60.0 + seconds - offset).max(0.0));
            rest = tag.1;
        }
        if rest.starts_with('[') {
            continue;
        }
        for seconds in times {
            lines.push(Line {
                seconds,
                text: rest.trim().to_owned(),
            });
        }
    }
    lines.sort_by(|a, b| a.seconds.total_cmp(&b.seconds));
    lines
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Record {
    track_name: String,
    artist_name: String,
    duration: f64,
    #[serde(default)]
    instrumental: bool,
    synced_lyrics: Option<String>,
    plain_lyrics: Option<String>,
}
fn normalize(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
fn matches(record: &Record, key: &TrackKey) -> bool {
    normalize(&record.track_name) == normalize(&key.title)
        && normalize(&record.artist_name) == normalize(&key.artist)
        && (key.duration == 0
            || record.duration.is_finite() && (record.duration - key.duration as f64).abs() <= 3.0)
}
fn data(record: Record) -> Data {
    if record.instrumental {
        return Data::Instrumental;
    }
    if let Some(s) = record.synced_lyrics {
        let lines = parse_lrc(&s);
        if !lines.is_empty() {
            return Data::Synced(lines);
        }
    }
    record
        .plain_lyrics
        .filter(|s| !s.trim().is_empty())
        .map(Data::Plain)
        .unwrap_or(Data::NotFound)
}

pub struct Client {
    http: reqwest::blocking::Client,
    base: String,
}
impl Client {
    pub fn new() -> Result<Self, String> {
        let http = reqwest::blocking::Client::builder()
            .https_only(true)
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(10))
            .user_agent(concat!(
                "Sela/",
                env!("CARGO_PKG_VERSION"),
                " (Windows notch; LRCLIB client)"
            ))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            http,
            base: "https://lrclib.net".into(),
        })
    }
    fn json(&self, path: &str, key: &TrackKey) -> Result<Option<Vec<u8>>, String> {
        let duration = key.duration.to_string();
        let mut params = vec![
            ("track_name", key.title.as_str()),
            ("artist_name", key.artist.as_str()),
        ];
        if path == "/api/get" && key.duration > 0 {
            params.push(("duration", duration.as_str()));
        }
        let response = self
            .http
            .get(format!("{}{path}", self.base))
            .query(&params)
            .send()
            .map_err(|_| {
                "Cannot reach LRCLIB. Check your connection, then try again.".to_string()
            })?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err("LRCLIB is limiting requests. Wait a moment, then try again.".into());
        }
        let response = response
            .error_for_status()
            .map_err(|_| "LRCLIB is unavailable. Try again later.".to_string())?;
        let mut bytes = Vec::new();
        response
            .take(1_048_577)
            .read_to_end(&mut bytes)
            .map_err(|_| "The lyric response was interrupted.".to_string())?;
        if bytes.len() > 1_048_576 {
            return Err("The lyric response is too large.".into());
        }
        Ok(Some(bytes))
    }
    pub fn lookup(&self, key: &TrackKey) -> Result<Data, String> {
        self.lookup_checked(key, || true)
    }
    fn lookup_checked(&self, key: &TrackKey, current: impl Fn() -> bool) -> Result<Data, String> {
        if !current() {
            return Err("Request canceled".into());
        }
        if key.duration > 0 {
            if let Some(bytes) = self.json("/api/get", key)? {
                let record: Record = serde_json::from_slice(&bytes)
                    .map_err(|_| "Invalid LRCLIB response format.".to_string())?;
                if matches(&record, key) {
                    return Ok(data(record));
                }
            }
        }
        if !current() {
            return Err("Request canceled".into());
        }
        let Some(bytes) = self.json("/api/search", key)? else {
            return Ok(Data::NotFound);
        };
        let records: Vec<Record> = serde_json::from_slice(&bytes)
            .map_err(|_| "Invalid LRCLIB search response.".to_string())?;
        let record = records
            .into_iter()
            .filter(|r| matches(r, key))
            .min_by(|a, b| {
                (a.duration - key.duration as f64)
                    .abs()
                    .total_cmp(&(b.duration - key.duration as f64).abs())
            });
        Ok(record.map(data).unwrap_or(Data::NotFound))
    }
}
#[derive(Serialize, Deserialize)]
struct Cached {
    key: TrackKey,
    fetched: u64,
    data: Data,
}
fn epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn cache_file(dir: &Path, key: &TrackKey) -> PathBuf {
    let mut hash = DefaultHasher::new();
    key.hash(&mut hash);
    dir.join(format!("{:016x}.json", hash.finish()))
}
fn read_cache(dir: &Path, key: &TrackKey) -> Option<Data> {
    let path = cache_file(dir, key);
    if fs::metadata(&path).ok()?.len() > 1_048_576 {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    if bytes.len() > 1_048_576 {
        return None;
    }
    let cached: Cached = serde_json::from_slice(&bytes).ok()?;
    let ttl = if cached.data == Data::NotFound {
        86_400
    } else {
        30 * 86_400
    };
    if cached.key != *key || epoch().saturating_sub(cached.fetched) > ttl {
        return None;
    }
    Some(cached.data)
}
fn write_cache(dir: &Path, key: &TrackKey, data: &Data) {
    if fs::create_dir_all(dir).is_err() {
        return;
    }
    let path = cache_file(dir, key);
    let tmp = path.with_extension("tmp");
    if let Ok(bytes) = serde_json::to_vec(&Cached {
        key: key.clone(),
        fetched: epoch(),
        data: data.clone(),
    }) {
        if fs::write(&tmp, bytes).is_ok() {
            let _ = fs::rename(tmp, path);
        }
    }
}
struct Request {
    id: u64,
    key: TrackKey,
    force: bool,
}
pub struct Update {
    pub id: u64,
    pub result: Result<Data, String>,
}
pub struct Worker {
    tx: mpsc::Sender<Request>,
    pub updates: mpsc::Receiver<Update>,
    generation: Arc<AtomicU64>,
}
impl Worker {
    pub fn spawn(cache: Option<PathBuf>) -> Self {
        let (tx, rx) = mpsc::channel::<Request>();
        let (out, updates) = mpsc::channel();
        let generation = Arc::new(AtomicU64::new(0));
        let current = generation.clone();
        thread::Builder::new()
            .name("sela-lyrics".into())
            .spawn(move || {
                let client = Client::new();
                while let Ok(first) = rx.recv() {
                    let request = rx.try_iter().last().unwrap_or(first);
                    if current.load(Ordering::Acquire) != request.id {
                        continue;
                    }
                    let cached = if request.force {
                        None
                    } else {
                        cache
                            .as_deref()
                            .and_then(|dir| read_cache(dir, &request.key))
                    };
                    let result = if let Some(data) = cached {
                        Ok(data)
                    } else {
                        match &client {
                            Ok(client) => client.lookup_checked(&request.key, || {
                                current.load(Ordering::Acquire) == request.id
                            }),
                            Err(e) => Err(e.clone()),
                        }
                    };
                    if current.load(Ordering::Acquire) != request.id {
                        continue;
                    }
                    if let (Ok(data), Some(dir)) = (&result, cache.as_deref()) {
                        write_cache(dir, &request.key, data);
                    }
                    if out
                        .send(Update {
                            id: request.id,
                            result,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })
            .expect("cannot start lyrics worker");
        Self {
            tx,
            updates,
            generation,
        }
    }
    pub fn request(&self, key: TrackKey, force: bool) -> u64 {
        let id = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let _ = self.tx.send(Request { id, key, force });
        id
    }
    pub fn cancel(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::AcqRel) + 1
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// Tracks only semantic lyric changes, not every media poll; stale worker results are rejected.
pub struct Selection {
    pub state: State,
    pub key: Option<TrackKey>,
    pub id: u64,
    pub changed: Instant,
}
impl Default for Selection {
    fn default() -> Self {
        Self {
            state: State::NoTrack,
            key: None,
            id: 0,
            changed: Instant::now(),
        }
    }
}
impl Selection {
    pub fn accept(&mut self, update: Update) -> bool {
        if update.id != self.id {
            return false;
        }
        self.state = match update.result {
            Ok(data) => State::Ready(data),
            Err(e) => State::Error(e),
        };
        self.changed = Instant::now();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lrc_handles_fraction_repeated_timestamps_metadata_and_offset() {
        let lines=parse_lrc("[ar:Demo]\n[offset:250]\n[00:10.50][01:02.125] chorus\n[00:02] first\n[00:99] broken\n[00:12.00]\n[bad] ignored");
        assert_eq!(lines.len(), 4);
        assert_eq!(
            lines[0],
            Line {
                seconds: 1.75,
                text: "first".into()
            }
        );
        assert_eq!(lines[1].seconds, 10.25);
        assert_eq!(lines[3].seconds, 61.875);
        let d = Data::Synced(lines);
        assert_eq!(d.active_index(0.0), None);
        assert_eq!(d.active_index(10.25), Some(1));
        assert_eq!(d.active_index(5.0), Some(0));
        assert_eq!(d.active_index(f64::NAN), None);
    }
    #[test]
    fn no_false_sync_for_plain_or_instrumental() {
        let r:Record=serde_json::from_str(r#"{"trackName":"A","artistName":"B","duration":20,"syncedLyrics":"[ti:A]","plainLyrics":"words"}"#).unwrap();
        assert_eq!(data(r), Data::Plain("words".into()));
        assert_eq!(
            State::Ready(Data::Plain("words".into())).compact_line(3.0),
            "Untimed lyrics · open panel"
        );
    }
    #[test]
    fn stale_results_cannot_replace_the_selected_track() {
        let mut selection = Selection {
            id: 3,
            ..Default::default()
        };
        assert!(!selection.accept(Update {
            id: 2,
            result: Ok(Data::Instrumental)
        }));
        assert!(matches!(selection.state, State::NoTrack));
        assert!(selection.accept(Update {
            id: 3,
            result: Ok(Data::NotFound)
        }));
    }
    #[test]
    fn cache_validates_identity_and_handles_corruption() {
        let dir = std::env::temp_dir().join(format!("sela-lyrics-cache-{}", std::process::id()));
        let key = TrackKey {
            title: "A / ..".into(),
            artist: "B".into(),
            duration: 40,
        };
        write_cache(&dir, &key, &Data::Instrumental);
        assert_eq!(read_cache(&dir, &key), Some(Data::Instrumental));
        let expired = Cached {
            key: key.clone(),
            fetched: epoch() - 86_401,
            data: Data::NotFound,
        };
        fs::write(
            cache_file(&dir, &key),
            serde_json::to_vec(&expired).unwrap(),
        )
        .unwrap();
        assert_eq!(read_cache(&dir, &key), None);
        let wrong = Cached {
            key: TrackKey {
                title: "Another".into(),
                ..key.clone()
            },
            fetched: epoch(),
            data: Data::Instrumental,
        };
        fs::write(cache_file(&dir, &key), serde_json::to_vec(&wrong).unwrap()).unwrap();
        assert_eq!(read_cache(&dir, &key), None);
        fs::write(cache_file(&dir, &key), "bad").unwrap();
        assert_eq!(read_cache(&dir, &key), None);
        let _ = fs::remove_dir_all(dir);
    }
    #[test]
    fn provider_http_encodes_metadata_and_parses_response() {
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut buf = [0; 8192];
            let n = socket.read(&mut buf).unwrap();
            let request = String::from_utf8_lossy(&buf[..n]);
            assert!(request.contains("track_name=A+%26+B"));
            let body = r#"{"trackName":"A & B","artistName":"Demo","duration":30,"syncedLyrics":"[00:01]hello","plainLyrics":null}"#;
            use std::io::Write;
            write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",body.len(),body).unwrap();
        });
        let client = Client {
            http: reqwest::blocking::Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap(),
            base: format!("http://{address}"),
        };
        let result = client
            .lookup(&TrackKey {
                title: "A & B".into(),
                artist: "Demo".into(),
                duration: 30,
            })
            .unwrap();
        assert!(matches!(result, Data::Synced(_)));
        server.join().unwrap();
    }
    #[test]
    fn track_matching_rejects_wrong_version_and_artist() {
        let key = TrackKey {
            title: "Song!".into(),
            artist: "Singer".into(),
            duration: 120,
        };
        let mut r = Record {
            track_name: "song".into(),
            artist_name: "SINGER".into(),
            duration: 121.0,
            instrumental: false,
            synced_lyrics: None,
            plain_lyrics: None,
        };
        assert!(matches(&r, &key));
        r.duration = 140.0;
        assert!(!matches(&r, &key));
        r.duration = 120.0;
        r.artist_name = "Another".into();
        assert!(!matches(&r, &key));
    }

    #[test]
    fn exact_miss_searches_but_rejects_unrelated_results() {
        use std::{io::Write, net::TcpListener};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for (path, status, body) in [
                ("/api/get?", "404 Not Found", "{}"),
                (
                    "/api/search?",
                    "200 OK",
                    r#"[{"trackName":"Song","artistName":"Wrong","duration":30,"plainLyrics":"wrong"},{"trackName":"Song","artistName":"Singer","duration":31,"plainLyrics":"right"}]"#,
                ),
            ] {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut buffer = [0; 8192];
                let n = socket.read(&mut buffer).unwrap();
                assert!(String::from_utf8_lossy(&buffer[..n]).contains(path));
                write!(
                    socket,
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            }
        });
        let client = Client {
            http: reqwest::blocking::Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap(),
            base: format!("http://{address}"),
        };
        assert_eq!(
            client
                .lookup(&TrackKey {
                    title: "Song".into(),
                    artist: "Singer".into(),
                    duration: 30
                })
                .unwrap(),
            Data::Plain("right".into())
        );
        server.join().unwrap();
    }

    #[test]
    fn cancellation_stops_network_before_the_next_request() {
        let client = Client::new().unwrap();
        let result = client.lookup_checked(
            &TrackKey {
                title: "Song".into(),
                artist: "Singer".into(),
                duration: 30,
            },
            || false,
        );
        assert_eq!(result.unwrap_err(), "Request canceled");
    }
}
