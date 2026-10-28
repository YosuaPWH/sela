use super::{Action, Media, MediaBridge, MediaCommand};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{
    sync::{
        atomic::{AtomicIsize, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
};
use windows::{
    core::{w, Error, HSTRING},
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Win32::{
        Foundation::{
            CloseHandle, GetLastError, BOOL, COLORREF, ERROR_ALREADY_EXISTS, HANDLE, HWND, LPARAM,
            RECT,
        },
        Graphics::Gdi::{
            CreateRectRgn, DeleteObject, EnumDisplayMonitors, GetMonitorInfoW, SetWindowRgn, HDC,
            HMONITOR, MONITORINFO,
        },
        System::{
            Diagnostics::Debug::MessageBeep,
            Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS},
            Threading::{
                CreateEventW, CreateMutexW, OpenEventW, SetEvent, WaitForSingleObject,
                EVENT_MODIFY_STATE, INFINITE,
            },
            WinRT::{RoInitialize, RO_INIT_MULTITHREADED},
        },
        UI::WindowsAndMessaging::{
            GetWindowLongW, MessageBoxW, SetLayeredWindowAttributes, SetWindowLongW, GWL_EXSTYLE,
            LWA_ALPHA, MB_ICONERROR, MB_OK, MONITORINFOF_PRIMARY, WS_EX_LAYERED, WS_EX_TRANSPARENT,
        },
    },
};

/// Keep the backing window fixed; let Windows clip drawing and mouse input to
/// the animated silhouette's bounds instead of resizing/recentering the HWND.
#[derive(Default)]
pub struct WindowRegion {
    last: Option<[i32; 4]>,
}

impl WindowRegion {
    pub fn set(
        &mut self,
        frame: &eframe::Frame,
        rect: eframe::egui::Rect,
        ppp: f32,
    ) -> Result<(), String> {
        let bounds = [
            (rect.left() * ppp).floor() as i32,
            (rect.top() * ppp).floor() as i32,
            (rect.right() * ppp).ceil() as i32,
            (rect.bottom() * ppp).ceil() as i32,
        ];
        if self.last == Some(bounds) {
            return Ok(());
        }
        let hwnd = window(frame)?;
        unsafe {
            let region = CreateRectRgn(bounds[0], bounds[1], bounds[2], bounds[3]);
            if region.0.is_null() {
                let _ = SetWindowRgn(hwnd, None, false);
                return Err("Cannot create the notch region".into());
            }
            // On success Windows owns the HRGN; deleting it would corrupt the window.
            if SetWindowRgn(hwnd, region, false) == 0 {
                let _ = DeleteObject(region);
                let _ = SetWindowRgn(hwnd, None, false);
                return Err("Cannot set the notch region. Using compatibility mode.".into());
            }
        }
        self.last = Some(bounds);
        Ok(())
    }
}

fn window(frame: &eframe::Frame) -> Result<HWND, String> {
    let handle = frame.window_handle().map_err(|e| e.to_string())?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return Err("Windows window handle is unavailable".into());
    };
    Ok(HWND(handle.hwnd.get() as *mut _))
}

/// The overlay HWND, published so tray and shortcut handlers can restore mouse
/// input even while the frame loop is not running.
static OVERLAY: AtomicIsize = AtomicIsize::new(0);

/// Click-through is applied here instead of through winit: winit adds
/// WS_EX_LAYERED without layered attributes, and such a window is never painted
/// again, which also stops the frame loop that reacts to the tray and shortcut.
/// Returns false when the window handle is unavailable.
pub fn set_passthrough(frame: &eframe::Frame, enabled: bool) -> bool {
    let Ok(hwnd) = window(frame) else {
        return false;
    };
    OVERLAY.store(hwnd.0 as isize, Ordering::Relaxed);
    unsafe { apply_passthrough(hwnd, enabled) };
    true
}

unsafe fn apply_passthrough(hwnd: HWND, enabled: bool) {
    let flags = (WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0) as i32;
    let style = GetWindowLongW(hwnd, GWL_EXSTYLE);
    let wanted = if enabled {
        style | flags
    } else {
        style & !flags
    };
    if wanted == style {
        return;
    }
    SetWindowLongW(hwnd, GWL_EXSTYLE, wanted);
    if enabled {
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA);
    }
}

fn restore_input() {
    let hwnd = OVERLAY.load(Ordering::Relaxed);
    if hwnd != 0 {
        unsafe { apply_passthrough(HWND(hwnd as *mut _), false) };
    }
}

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

/// Signalled by a second launch so the running instance opens its panel.
const OPEN_EVENT: windows::core::PCWSTR = w!("Local\\Sela.App.Open");

/// Ask the running instance to open; a relaunch must never look like a no-op.
pub fn signal_existing() {
    unsafe {
        if let Ok(event) = OpenEventW(EVENT_MODIFY_STATE, false, OPEN_EVENT) {
            let _ = SetEvent(event);
            let _ = CloseHandle(event);
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
            w!("Sela could not start"),
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
        sampled_at: std::time::Instant::now(),
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
            "The player rejected the command",
        ))
    }
}

pub fn media_bridge() -> MediaBridge {
    let (tx, updates) = mpsc::channel();
    let (commands, rx) = mpsc::channel();
    thread::Builder::new()
        .name("sela-media".into())
        .spawn(move || {
            // WinRT calls and async waits stay on a dedicated MTA thread, away from rendering.
            let setup = unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.and_then(|_| {
                GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.get()
            });
            let manager = match setup {
                Ok(manager) => manager,
                Err(e) => {
                    let _ = tx.send(Media {
                        error: Some(format!("Windows media access failed: {e}")),
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
                        Err(e) => command_error = Some(format!("Media control failed: {e}")),
                    }
                }
                let mut media = match manager.GetCurrentSession() {
                    Ok(session) => read_media(&session).unwrap_or_else(|e| Media {
                        error: Some(format!("Cannot read media: {e}")),
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
    actions: mpsc::Receiver<Action>,
    warning: Option<String>,
}

impl DesktopIntegration {
    pub fn new(ctx: &eframe::egui::Context) -> Result<Self, String> {
        let menu = Menu::new();
        let toggle = MenuItem::new("Open / close    Ctrl+Alt+N", true, None);
        let focus = MenuItem::new("Focus timer", true, None);
        let settings = MenuItem::new("Settings", true, None);
        let quit = MenuItem::new("Quit Sela", true, None);
        menu.append_items(&[&toggle, &focus, &settings, &quit])
            .map_err(|e| e.to_string())?;
        let sheet = image::load_from_memory(include_bytes!("../../assets/pijar-sheet.png"))
            .map_err(|e| e.to_string())?;
        let pijar = sheet
            .crop_imm(0, 0, (sheet.width() as f32 * 0.30) as u32, sheet.height())
            .resize(32, 32, image::imageops::FilterType::Lanczos3)
            .to_rgba8();
        // Fit the full character into a square tray icon, preserving transparent edges.
        let mut canvas = image::RgbaImage::new(32, 32);
        image::imageops::overlay(&mut canvas, &pijar, ((32 - pijar.width()) / 2).into(), 0);
        let icon = Icon::from_rgba(canvas.into_raw(), 32, 32).map_err(|e| e.to_string())?;
        let tray = TrayIconBuilder::new()
            .with_tooltip("Sela • Ctrl+Alt+N")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            // Left click toggles the panel; right click opens the menu.
            .with_menu_on_left_click(false)
            .build()
            .map_err(|e| e.to_string())?;
        let hotkeys = GlobalHotKeyManager::new().map_err(|e| e.to_string())?;
        let shortcut = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyN);
        // Shortcut conflicts must never remove the tray's recovery path.
        let warning = hotkeys.register(shortcut).err().map(|e| {
            format!("Ctrl+Alt+N is unavailable: {e}. Use the tray menu or click the notch.")
        });
        let (tx, actions) = mpsc::channel();
        let ctx = ctx.clone();
        // Handlers run inside the Windows message loop, so they still fire when
        // the overlay is click-through or not repainting.
        let dispatch = move |action: Action| {
            if action == Action::Quit {
                // Last resort if the frame loop never gets to close the window.
                thread::spawn(|| {
                    thread::sleep(Duration::from_secs(2));
                    std::process::exit(0);
                });
            } else {
                restore_input();
            }
            let _ = tx.send(action);
            ctx.request_repaint();
        };
        let menu_dispatch = dispatch.clone();
        let tray_dispatch = dispatch.clone();
        let relaunch_dispatch = dispatch.clone();
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                tray_dispatch(Action::Toggle);
            }
        }));
        // Auto-reset event; HANDLE is not Send, so the thread receives its value.
        if let Ok(event) = unsafe { CreateEventW(None, false, false, OPEN_EVENT) } {
            let event = event.0 as isize;
            thread::Builder::new()
                .name("sela-relaunch".into())
                .spawn(move || loop {
                    let wait = unsafe { WaitForSingleObject(HANDLE(event as *mut _), INFINITE) };
                    if wait.0 != 0 {
                        break;
                    }
                    relaunch_dispatch(Action::Open);
                })
                .map_err(|e| e.to_string())?;
        }
        let (toggle, focus, settings, quit) = (
            toggle.id().clone(),
            focus.id().clone(),
            settings.id().clone(),
            quit.id().clone(),
        );
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let action = if event.id == toggle {
                Action::Toggle
            } else if event.id == focus {
                Action::Focus
            } else if event.id == settings {
                Action::Settings
            } else if event.id == quit {
                Action::Quit
            } else {
                return;
            };
            menu_dispatch(action);
        }));
        let shortcut = shortcut.id();
        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.id == shortcut && event.state == HotKeyState::Pressed {
                dispatch(Action::Toggle);
            }
        }));
        Ok(Self {
            _tray: tray,
            _hotkeys: hotkeys,
            actions,
            warning,
        })
    }

    pub fn poll(&self) -> Option<Action> {
        self.actions.try_recv().ok()
    }

    pub fn warning(&self) -> Option<&str> {
        self.warning.as_deref()
    }
}
