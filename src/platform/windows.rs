use super::{Action, Media, MediaBridge, MediaCommand};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use std::{sync::mpsc, thread, time::Duration};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};
use windows::{
    core::{w, Error, HSTRING},
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Win32::{
        Foundation::{CloseHandle, GetLastError, BOOL, ERROR_ALREADY_EXISTS, HANDLE, LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO},
        System::{
            Diagnostics::Debug::MessageBeep,
            Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS},
            Threading::CreateMutexW,
            WinRT::{RoInitialize, RO_INIT_MULTITHREADED},
        },
        UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK, MONITORINFOF_PRIMARY},
    },
};

fn monitors() -> Vec<MONITORINFO> {
    unsafe extern "system" fn collect(
        monitor: HMONITOR,
        _dc: HDC,
        _rect: *mut RECT,
        data: LPARAM,
    ) -> BOOL {
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        // The vector pointer lives until synchronous EnumDisplayMonitors returns.
        if GetMonitorInfoW(monitor, &mut info).as_bool() {
            (*(data.0 as *mut Vec<MONITORINFO>)).push(info);
        }
        BOOL(1)
    }
    let mut monitors = Vec::<MONITORINFO>::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(collect),
            LPARAM(&mut monitors as *mut _ as isize),
        );
    }
    monitors.sort_by_key(|m| {
        (
            m.dwFlags & MONITORINFOF_PRIMARY == 0,
            m.rcMonitor.left,
            m.rcMonitor.top,
        )
    });
    monitors
}

pub fn monitor_count() -> usize {
    monitors().len().max(1)
}

pub fn monitor_geometry(index: usize, ppp: f32) -> Option<(eframe::egui::Pos2, f32)> {
    let monitors = monitors();
    let info = monitors.get(index).or_else(|| monitors.first())?;
    Some((
        eframe::egui::pos2(
            info.rcMonitor.left as f32 / ppp,
            info.rcMonitor.top as f32 / ppp,
        ),
        (info.rcMonitor.right - info.rcMonitor.left) as f32 / ppp,
    ))
}

pub struct Instance(HANDLE);
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub fn single_instance() -> Result<Option<Instance>, String> {
    // A session-local mutex avoids duplicate overlays without blocking other Windows users.
    unsafe {
        let handle =
            CreateMutexW(None, false, w!("Local\\LumaNotch.App.01")).map_err(|e| e.to_string())?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            let _ = CloseHandle(handle);
            Ok(None)
        } else {
            Ok(Some(Instance(handle)))
        }
    }
}

pub fn battery() -> Option<crate::telemetry::Battery> {
    let mut status = SYSTEM_POWER_STATUS::default();
    unsafe {
        GetSystemPowerStatus(&mut status).ok()?;
    }
    if status.BatteryFlag == 255 || status.BatteryFlag & 128 != 0 || status.BatteryLifePercent > 100
    {
        return None;
    }
    Some(crate::telemetry::Battery {
        percent: status.BatteryLifePercent,
        plugged_in: status.ACLineStatus == 1,
    })
}

pub fn completion_sound() {
    unsafe {
        let _ = MessageBeep(MB_OK);
    }
}

pub fn startup_error(message: &str) {
    unsafe {
        let _ = MessageBoxW(
            None,
            &HSTRING::from(message),
            w!("Luma tidak dapat dibuka"),
            MB_OK | MB_ICONERROR,
        );
    }
}

fn read_media(session: &GlobalSystemMediaTransportControlsSession) -> windows::core::Result<Media> {
    let properties = session.TryGetMediaPropertiesAsync()?.get()?;
    let playback = session.GetPlaybackInfo()?;
    let controls = playback.Controls()?;
    let timeline = session.GetTimelineProperties()?;
    let start = timeline.StartTime()?.Duration as f64 / 10_000_000.0;
    let duration = (timeline.EndTime()?.Duration as f64 / 10_000_000.0 - start).max(0.0);
    let playing = playback.PlaybackStatus()?
        == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
    let updated = timeline.LastUpdatedTime()?.UniversalTime;
    let elapsed = if playing && updated > 0 {
        (chrono::Utc::now().timestamp_millis() as f64 / 1000.0 + 11_644_473_600.0
            - updated as f64 / 10_000_000.0)
            .max(0.0)
    } else {
        0.0
    };
    let position = (timeline.Position()?.Duration as f64 / 10_000_000.0 - start + elapsed).max(0.0);
    Ok(Media {
        title: properties.Title()?.to_string(),
        artist: properties.Artist()?.to_string(),
        source: session.SourceAppUserModelId()?.to_string(),
        playing,
        position: if duration > 0.0 {
            position.min(duration)
        } else {
            position
        },
        duration,
        can_toggle: controls.IsPlayPauseToggleEnabled()?,
        can_previous: controls.IsPreviousEnabled()?,
        can_next: controls.IsNextEnabled()?,
        error: None,
    })
}

fn run_command(
    session: &GlobalSystemMediaTransportControlsSession,
    command: MediaCommand,
) -> windows::core::Result<()> {
    let accepted = match command {
        MediaCommand::Toggle => session.TryTogglePlayPauseAsync()?.get()?,
        MediaCommand::Previous => session.TrySkipPreviousAsync()?.get()?,
        MediaCommand::Next => session.TrySkipNextAsync()?.get()?,
    };
    if accepted {
        Ok(())
    } else {
        Err(Error::new(
            windows::core::HRESULT(0x80004005u32 as i32),
            "Pemutar menolak perintah",
        ))
    }
}

pub fn media_bridge() -> MediaBridge {
    let (tx, updates) = mpsc::channel();
    let (commands, rx) = mpsc::channel();
    thread::Builder::new()
        .name("luma-media".into())
        .spawn(move || {
            // WinRT calls and async waits stay on a dedicated MTA thread, away from rendering.
            let setup = unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.and_then(|_| {
                GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.get()
            });
            let manager = match setup {
                Ok(manager) => manager,
                Err(e) => {
                    let _ = tx.send(Media {
                        error: Some(format!("Akses media Windows gagal: {e}")),
                        ..Media::default()
                    });
                    return;
                }
            };
            loop {
                let mut command_error = None;
                if let Ok(command) = rx.recv_timeout(Duration::from_millis(750)) {
                    match manager
                        .GetCurrentSession()
                        .and_then(|s| run_command(&s, command))
                    {
                        Ok(()) => (),
                        Err(e) => command_error = Some(format!("Kontrol media gagal: {e}")),
                    }
                }
                let mut media = match manager.GetCurrentSession() {
                    Ok(session) => read_media(&session).unwrap_or_else(|e| Media {
                        error: Some(format!("Tidak dapat membaca media: {e}")),
                        ..Media::default()
                    }),
                    Err(_) => Media::default(),
                };
                if command_error.is_some() {
                    media.error = command_error;
                }
                if tx.send(media).is_err() {
                    break;
                }
            }
        })
        .expect("cannot start media worker");
    MediaBridge { updates, commands }
}

pub struct DesktopIntegration {
    _tray: TrayIcon,
    _hotkeys: GlobalHotKeyManager,
    shortcut: u32,
    toggle: tray_icon::menu::MenuId,
    focus: tray_icon::menu::MenuId,
    settings: tray_icon::menu::MenuId,
    quit: tray_icon::menu::MenuId,
    warning: Option<String>,
}

impl DesktopIntegration {
    pub fn new() -> Result<Self, String> {
        let menu = Menu::new();
        let toggle = MenuItem::new("Buka / tutup    Ctrl+Alt+N", true, None);
        let focus = MenuItem::new("Timer fokus", true, None);
        let settings = MenuItem::new("Pengaturan", true, None);
        let quit = MenuItem::new("Keluar dari Luma", true, None);
        menu.append_items(&[&toggle, &focus, &settings, &quit])
            .map_err(|e| e.to_string())?;
        let mut rgba = vec![0u8; 32 * 32 * 4];
        for y in 5..27 {
            for x in 4..28 {
                let i = (y * 32 + x) * 4;
                let color = if (13..19).contains(&x) && (11..21).contains(&y) {
                    [169, 238, 192, 255]
                } else {
                    [24, 27, 26, 255]
                };
                rgba[i..i + 4].copy_from_slice(&color);
            }
        }
        let icon = Icon::from_rgba(rgba, 32, 32).map_err(|e| e.to_string())?;
        let tray = TrayIconBuilder::new()
            .with_tooltip("Luma Notch • Ctrl+Alt+N")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()
            .map_err(|e| e.to_string())?;
        let hotkeys = GlobalHotKeyManager::new().map_err(|e| e.to_string())?;
        let shortcut = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyN);
        // Shortcut conflicts must never remove the tray's recovery path.
        let warning = hotkeys
            .register(shortcut)
            .err()
            .map(|e| format!("Ctrl+Alt+N tidak tersedia: {e}. Gunakan menu tray atau klik notch."));
        Ok(Self {
            _tray: tray,
            _hotkeys: hotkeys,
            shortcut: shortcut.id(),
            toggle: toggle.id().clone(),
            focus: focus.id().clone(),
            settings: settings.id().clone(),
            quit: quit.id().clone(),
            warning,
        })
    }

    pub fn poll(&self) -> Option<Action> {
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id == self.shortcut && event.state == HotKeyState::Pressed {
                return Some(Action::Toggle);
            }
        }
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.toggle {
                return Some(Action::Toggle);
            }
            if event.id == self.focus {
                return Some(Action::Focus);
            }
            if event.id == self.settings {
                return Some(Action::Settings);
            }
            if event.id == self.quit {
                return Some(Action::Quit);
            }
        }
        None
    }

    pub fn warning(&self) -> Option<&str> {
        self.warning.as_deref()
    }
}
