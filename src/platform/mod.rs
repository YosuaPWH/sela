use std::sync::mpsc;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[derive(Debug, Clone, Default)]
pub struct Media {
    pub title: String,
    pub artist: String,
    pub source: String,
    pub playing: bool,
    pub position: f64,
    pub duration: f64,
    pub can_toggle: bool,
    pub can_previous: bool,
    pub can_next: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum MediaCommand {
    Toggle,
    Previous,
    Next,
}

pub struct MediaBridge {
    pub updates: mpsc::Receiver<Media>,
    pub commands: mpsc::Sender<MediaCommand>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Toggle,
    Focus,
    Settings,
    Quit,
}

#[cfg(not(windows))]
pub fn battery() -> Option<crate::telemetry::Battery> {
    None
}

#[cfg(not(windows))]
pub fn media_bridge() -> MediaBridge {
    let (_tx, updates) = mpsc::channel();
    let (commands, _rx) = mpsc::channel();
    MediaBridge { updates, commands }
}

#[cfg(not(windows))]
pub struct DesktopIntegration;

#[cfg(not(windows))]
impl DesktopIntegration {
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
    pub fn poll(&self) -> Option<Action> {
        None
    }
    pub fn warning(&self) -> Option<&str> {
        None
    }
}

#[cfg(not(windows))]
pub fn completion_sound() {}

#[cfg(not(windows))]
pub fn startup_error(message: &str) {
    eprintln!("Luma tidak dapat dibuka: {message}");
}

#[cfg(not(windows))]
pub struct Instance;

#[cfg(not(windows))]
pub fn single_instance() -> Result<Option<Instance>, String> {
    Ok(Some(Instance))
}

#[cfg(not(windows))]
pub fn monitor_geometry(_index: usize, _ppp: f32) -> Option<(eframe::egui::Pos2, f32)> {
    None
}

#[cfg(not(windows))]
pub fn monitor_count() -> usize {
    1
}
