use std::sync::mpsc;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[derive(Debug, Clone)]
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
    pub sampled_at: std::time::Instant,
}

impl Default for Media {
    fn default() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            source: String::new(),
            playing: false,
            position: 0.0,
            duration: 0.0,
            can_toggle: false,
            can_previous: false,
            can_next: false,
            error: None,
            sampled_at: std::time::Instant::now(),
        }
    }
}
impl Media {
    pub fn position_at(&self, now: std::time::Instant) -> f64 {
        let position = self.position
            + if self.playing {
                now.saturating_duration_since(self.sampled_at).as_secs_f64()
            } else {
                0.0
            };
        let position = if position.is_finite() {
            position.max(0.0)
        } else {
            0.0
        };
        if self.duration.is_finite() && self.duration > 0.0 {
            position.min(self.duration)
        } else {
            position
        }
    }
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
    pub fn new(_ctx: &eframe::egui::Context) -> Result<Self, String> {
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
pub fn set_passthrough(_frame: &eframe::Frame, _enabled: bool) -> bool {
    false
}

#[cfg(not(windows))]
pub fn completion_sound() {}

#[cfg(not(windows))]
pub fn startup_error(message: &str) {
    eprintln!("Sela could not start: {message}");
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

#[cfg(not(windows))]
#[derive(Default)]
pub struct WindowRegion;

#[cfg(not(windows))]
impl WindowRegion {
    pub fn set(
        &mut self,
        _frame: &eframe::Frame,
        _rect: eframe::egui::Rect,
        _ppp: f32,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    #[test]
    fn media_clock_holds_on_pause_clamps_and_accepts_backward_seek() {
        let now = Instant::now();
        let mut media = Media {
            playing: true,
            position: 10.0,
            duration: 20.0,
            sampled_at: now,
            ..Default::default()
        };
        assert_eq!(media.position_at(now + Duration::from_secs(3)), 13.0);
        assert_eq!(media.position_at(now + Duration::from_secs(30)), 20.0);
        media.playing = false;
        assert_eq!(media.position_at(now + Duration::from_secs(30)), 10.0);
        media.position = 2.0;
        media.sampled_at = now + Duration::from_secs(30);
        media.playing = true;
        assert_eq!(media.position_at(now + Duration::from_secs(31)), 3.0);
        media.position = f64::NAN;
        assert_eq!(media.position_at(now), 0.0);
    }
}
