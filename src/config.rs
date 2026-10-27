use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub expand_on_hover: bool,
    pub reduced_motion: bool,
    pub sound: bool,
    pub focus_minutes: u32,
    pub top_offset: f32,
    pub monitor: usize,
    pub lyrics_enabled: bool,
    pub show_mascot: bool,
    pub performance_enabled: bool,
    /// Show the gaming line only while a game is detected.
    pub performance_auto: bool,
    pub performance_games: Vec<String>,
    pub performance_style: crate::performance::LineStyle,
    pub performance_target: String,
    pub performance_profiles: std::collections::BTreeMap<String, crate::performance::LineStyle>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            expand_on_hover: true,
            reduced_motion: false,
            sound: true,
            focus_minutes: 25,
            top_offset: 0.0,
            monitor: 0,
            lyrics_enabled: true,
            show_mascot: true,
            performance_enabled: false,
            performance_auto: true,
            performance_games: vec![],
            performance_style: Default::default(),
            performance_target: String::new(),
            performance_profiles: Default::default(),
        }
    }
}

impl Settings {
    pub fn sanitize(&mut self) {
        self.focus_minutes = self.focus_minutes.clamp(1, 120);
        self.top_offset = if self.top_offset.is_finite() {
            self.top_offset.clamp(0.0, 80.0)
        } else {
            0.0
        };
    }

    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        match fs::read(path) {
            Ok(bytes) => {
                let mut settings: Self = serde_json::from_slice(&bytes).map_err(|e| {
                    format!(
                        "Invalid settings: {e}. Using defaults; the original file is preserved."
                    )
                })?;
                settings.sanitize();
                Ok(settings)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("Cannot read settings: {e}")),
        }
    }

    pub fn save(&self, path: &std::path::Path) -> Result<(), String> {
        let parent = path.parent().ok_or("Invalid settings location")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let temporary = path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
        fs::rename(&temporary, path).map_err(|e| format!("Cannot save settings: {e}"))
    }
}

pub fn settings_path() -> Result<PathBuf, String> {
    directories::ProjectDirs::from("dev", "Sela", "Sela")
        .map(|p| p.config_dir().join("settings.json"))
        .ok_or_else(|| "User settings folder is unavailable".into())
}

pub fn legacy_settings_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("dev", "Luma", "LumaNotch")
        .map(|p| p.config_dir().join("settings.json"))
}
pub fn lyrics_cache_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("dev", "Sela", "Sela").map(|p| p.cache_dir().join("lyrics"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_settings_get_new_defaults_and_invalid_values_are_clamped() {
        let mut s: Settings =
            serde_json::from_str(r#"{"focus_minutes":0,"top_offset":900}"#).unwrap();
        s.sanitize();
        assert_eq!(s.focus_minutes, 1);
        assert_eq!(s.top_offset, 80.0);
        assert!(s.expand_on_hover);
        assert!(s.lyrics_enabled && s.show_mascot);
        assert!(!s.performance_enabled);
        assert!(s.performance_auto);
        assert_eq!(s.performance_style, crate::performance::LineStyle::TextOnly);
        let with_profile: Settings = serde_json::from_str(
            r#"{"performance_style":"background","performance_profiles":{"game.exe":"text_only"}}"#,
        )
        .unwrap();
        assert_eq!(
            with_profile.performance_style,
            crate::performance::LineStyle::Background
        );
        assert_eq!(
            with_profile.performance_profiles["game.exe"],
            crate::performance::LineStyle::TextOnly
        );
    }

    #[test]
    fn malformed_settings_are_not_overwritten_on_load() {
        let dir = std::env::temp_dir().join(format!("luma-config-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        fs::write(&path, "broken").unwrap();
        assert!(Settings::load(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "broken");
        Settings::default().save(&path).unwrap();
        let mut replacement = Settings::load(&path).unwrap();
        replacement.focus_minutes = 50;
        replacement.save(&path).unwrap();
        assert_eq!(Settings::load(&path).unwrap().focus_minutes, 50);
        fs::remove_dir_all(dir).unwrap();
    }
}
