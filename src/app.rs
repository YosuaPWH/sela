use crate::{
    config::{self, Settings},
    focus::{FocusTimer, Phase},
    icons::{self, Icon},
    lyrics::{self, Data as LyricData, State as LyricState},
    motion::{Dimension, Expansion},
    navigation::HeaderWheel,
    performance::{self, LineStyle},
    platform::{self, Action, Media, MediaBridge, MediaCommand},
    telemetry::{self, Snapshot},
};
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Rect, RichText,
    Sense, Stroke, StrokeKind, Vec2,
};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

const BG: Color32 = Color32::from_rgb(14, 17, 16);
const SURFACE: Color32 = Color32::from_rgb(36, 43, 39);
const TEXT: Color32 = Color32::from_rgb(243, 242, 236);
const MUTED: Color32 = Color32::from_rgb(166, 178, 165);
const ACCENT: Color32 = Color32::from_rgb(184, 216, 186);
const AMBER: Color32 = Color32::from_rgb(242, 200, 121);
const ERROR: Color32 = Color32::from_rgb(255, 180, 171);
const COMPACT: Vec2 = vec2(324.0, 48.0);
const EXPANDED: Vec2 = vec2(464.0, 370.0);
const MUSIC_COMPACT: Vec2 = vec2(580.0, 54.0);
const CANVAS: Vec2 = vec2(600.0, 370.0);
const PERFORMANCE_PANEL: Vec2 = vec2(600.0, 440.0);
/// How long the pointer rests on the notch before it opens, and how long after
/// it leaves before a hover-opened panel closes.
const HOVER_OPEN: Duration = Duration::from_millis(150);
const HOVER_CLOSE: Duration = Duration::from_millis(300);
/// Settings are written this long after the last change, so dragging a slider
/// does not rewrite the file every frame.
const SAVE_DELAY: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Media,
    Focus,
    System,
    Settings,
}

impl Tab {
    fn step(self, forward: bool) -> Self {
        let tabs = [Self::Media, Self::Focus, Self::System, Self::Settings];
        let current = tabs.iter().position(|tab| *tab == self).unwrap_or(0);
        tabs[(current + if forward { 1 } else { tabs.len() - 1 }) % tabs.len()]
    }
}

pub struct NotchApp {
    settings: Settings,
    settings_path: Option<PathBuf>,
    save_due: Option<Instant>,
    error: Option<String>,
    expanded: bool,
    pinned: bool,
    hover_opened: bool,
    pointer_left: Option<Instant>,
    hover_started: Option<Instant>,
    suppress_hover: bool,
    tab: Tab,
    timer: FocusTimer,
    last_tick: Instant,
    snapshot: Snapshot,
    monitor: telemetry::Monitor,
    header_wheel: HeaderWheel,
    bridge: MediaBridge,
    media: Media,
    integration: Option<platform::DesktopIntegration>,
    last_geometry: Option<(Vec2, f32, f32, egui::Pos2, f32)>,
    motion: Expansion,
    width: Dimension,
    height: Dimension,
    mascot: Option<egui::TextureHandle>,
    lyrics_worker: lyrics::Worker,
    lyrics: lyrics::Selection,
    active_line: String,
    line_changed: Instant,
    demo: bool,
    demo_track: usize,
    window_region: platform::WindowRegion,
    fixed_surface: bool,
    performance: performance::Monitor,
    performance_reading: performance::Reading,
    performance_page: bool,
    performance_demo: bool,
    performance_sessions: Vec<Arc<performance::Session>>,
    benchmark_seconds: u32,
    export_folder: Option<PathBuf>,
    monitor_width: f32,
    last_passthrough: bool,
}

impl NotchApp {
    pub fn new(cc: &eframe::CreationContext<'_>, expanded: bool, demo: bool) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "dm-sans".into(),
            egui::FontData::from_static(include_bytes!("../assets/dm-sans-400.ttf")).into(),
        );
        fonts.font_data.insert(
            "dm-semibold".into(),
            egui::FontData::from_static(include_bytes!("../assets/dm-sans-600.ttf")).into(),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "dm-sans".into());
        fonts.families.insert(
            egui::FontFamily::Name("semibold".into()),
            vec!["dm-semibold".into(), "dm-sans".into()],
        );
        cc.egui_ctx.set_fonts(fonts);
        let mascot = image::load_from_memory(include_bytes!("../assets/pijar-sheet.png"))
            .ok()
            .map(|image| {
                // Upload only Pijar; the reference sheet also contains archived concepts.
                // Its original width exceeds the GL 2.0 minimum texture limit (2048).
                let rgba = image
                    .crop_imm(0, 0, (image.width() as f32 * 0.30) as u32, image.height())
                    .to_rgba8();
                cc.egui_ctx.load_texture(
                    "Pijar",
                    egui::ColorImage::from_rgba_unmultiplied(
                        [rgba.width() as usize, rgba.height() as usize],
                        rgba.as_raw(),
                    ),
                    egui::TextureOptions::LINEAR,
                )
            });
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.override_text_color = Some(TEXT);
        visuals.selection.bg_fill = ACCENT.gamma_multiply(0.3);
        visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
        visuals.widgets.inactive.bg_fill = SURFACE;
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 58, 51);
        visuals.widgets.active.bg_fill = ACCENT.gamma_multiply(0.3);
        visuals.widgets.active.fg_stroke = Stroke::new(1.5_f32, ACCENT);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, TEXT);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, SURFACE);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
        visuals.widgets.active.corner_radius = CornerRadius::same(8);
        cc.egui_ctx.set_visuals(visuals);
        cc.egui_ctx.style_mut(|s| {
            s.spacing.item_spacing = vec2(10.0, 10.0);
            s.spacing.button_padding = vec2(14.0, 8.0);
            s.animation_time = 0.2;
        });
        let (settings_path, mut error) = match config::settings_path() {
            Ok(path) => (Some(path), None),
            Err(e) => (None, Some(e)),
        };
        let mut settings = settings_path
            .as_ref()
            .map(|p| {
                let legacy =
                    config::legacy_settings_path().filter(|old| !p.exists() && old.exists());
                Settings::load(legacy.as_ref().unwrap_or(p)).unwrap_or_else(|e| {
                    error = Some(e);
                    Settings::default()
                })
            })
            .unwrap_or_default();
        let performance_demo = std::env::args().any(|a| a == "--performance-demo");
        if performance_demo {
            settings.performance_enabled = true;
        }
        let performance_page = settings.performance_enabled;
        // Outside a game the game-gated line shows the normal notch, so start there.
        let start_on_performance =
            performance_page && (!settings.performance_auto || performance_demo);
        let integration = match platform::DesktopIntegration::new(&cc.egui_ctx) {
            Ok(integration) => {
                if let Some(warning) = integration.warning() {
                    error = Some(warning.to_owned());
                }
                Some(integration)
            }
            Err(e) => {
                error = Some(format!("Tray unavailable: {e}. Open Settings to quit."));
                None
            }
        };
        let media = if demo {
            demo_media(0)
        } else {
            Media::default()
        };
        let lyrics = if demo {
            lyrics::Selection {
                state: LyricState::Ready(demo_lyrics(0)),
                key: lyrics::TrackKey::from_media(&media),
                ..Default::default()
            }
        } else {
            lyrics::Selection::default()
        };
        let initial_size =
            if settings.performance_enabled && !settings.performance_auto && !expanded {
                vec2(816.0, 48.0)
            } else if expanded {
                CANVAS
            } else if demo && settings.lyrics_enabled {
                MUSIC_COMPACT
            } else {
                COMPACT
            };
        Self {
            timer: FocusTimer::new(settings.focus_minutes),
            settings,
            settings_path,
            save_due: None,
            error,
            expanded,
            pinned: false,
            hover_opened: false,
            pointer_left: None,
            hover_started: None,
            suppress_hover: false,
            tab: if start_on_performance {
                Tab::System
            } else {
                Tab::Media
            },
            last_tick: Instant::now(),
            snapshot: Snapshot::default(),
            monitor: telemetry::spawn(),
            header_wheel: HeaderWheel::default(),
            bridge: platform::media_bridge(),
            media,
            integration,
            last_geometry: None,
            motion: Expansion::new(expanded),
            width: Dimension::new(initial_size.x),
            height: Dimension::new(initial_size.y),
            mascot,
            lyrics_worker: lyrics::Worker::spawn(config::lyrics_cache_path()),
            lyrics,
            active_line: String::new(),
            line_changed: Instant::now(),
            demo,
            demo_track: 0,
            window_region: Default::default(),
            fixed_surface: true,
            performance: performance::spawn(),
            performance_reading: performance::Reading {
                status: "Performance monitoring is off.".into(),
                ..Default::default()
            },
            performance_page,
            performance_demo,
            performance_sessions: vec![],
            benchmark_seconds: 30,
            export_folder: None,
            monitor_width: cc
                .egui_ctx
                .input(|i| i.viewport().monitor_size)
                .map_or(1920.0, |v| v.x),
            last_passthrough: false,
        }
    }

    fn open(&mut self, ctx: &egui::Context) {
        self.expanded = true;
        self.hover_opened = false;
        self.hover_started = None;
        self.pointer_left = None;
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    /// Open from the keyboard, tray or a relaunch: over a game, show its controls.
    fn open_shell(&mut self, ctx: &egui::Context) {
        if self.gaming_line() {
            self.tab = Tab::System;
            self.performance_page = true;
        }
        self.open(ctx);
    }

    fn collapse(&mut self) {
        self.expanded = false;
        self.pinned = false;
        self.suppress_hover = true;
        self.hover_started = None;
        self.pointer_left = None;
    }

    fn persist(&mut self) {
        self.save_due = Some(Instant::now() + SAVE_DELAY);
    }

    fn save_now(&mut self) {
        self.save_due = None;
        if let Some(path) = &self.settings_path {
            if let Err(e) = self.settings.save(path) {
                self.error = Some(e);
            }
        }
    }

    fn geometry(&mut self, ctx: &egui::Context, size: Vec2) {
        let ppp = ctx.pixels_per_point();
        let screen = ctx
            .input(|i| i.viewport().monitor_size)
            .unwrap_or(vec2(1920.0, 1080.0));
        let (origin, width) = platform::monitor_geometry(self.settings.monitor, ppp)
            .unwrap_or((egui::Pos2::ZERO, screen.x));
        let key = (size, ppp, self.settings.top_offset, origin, width);
        if self.last_geometry == Some(key) {
            return;
        }
        self.last_geometry = Some(key);
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos2(
            origin.x + (width - size.x) * 0.5,
            origin.y + self.settings.top_offset,
        )));
    }

    /// Lowercase executable name of the captured application.
    fn target_name(&self) -> Option<String> {
        self.performance_reading
            .target
            .as_ref()
            .map(|a| a.name.to_ascii_lowercase())
    }

    /// The style saved for the captured application, else the default style.
    fn line_style(&self) -> LineStyle {
        self.target_name()
            .and_then(|name| self.settings.performance_profiles.get(&name).copied())
            .unwrap_or(self.settings.performance_style)
    }

    /// Whether the closed notch is the click-through gaming line right now.
    fn gaming_line(&self) -> bool {
        self.settings.performance_enabled
            && (!self.settings.performance_auto || self.performance_reading.gaming)
    }

    fn compact_size(&self) -> Vec2 {
        if self.gaming_line() {
            return vec2(816.0_f32.min((self.monitor_width - 24.0).max(320.0)), 48.0);
        }
        if self.settings.lyrics_enabled
            && !self.media.title.is_empty()
            && !matches!(self.timer.phase, Phase::Running | Phase::Paused)
        {
            MUSIC_COMPACT
        } else {
            COMPACT
        }
    }

    fn expanded_size(&self) -> Vec2 {
        if self.tab == Tab::System && self.performance_page {
            PERFORMANCE_PANEL
        } else if self.tab == Tab::System {
            vec2(EXPANDED.x, 440.0)
        } else if self.tab == Tab::Media {
            CANVAS
        } else {
            EXPANDED
        }
    }
    fn target_size(&self) -> Vec2 {
        if self.expanded {
            self.expanded_size()
        } else {
            self.compact_size()
        }
    }

    fn paint_mascot(&self, painter: &egui::Painter, rect: Rect) {
        if self.settings.show_mascot {
            if let Some(texture) = &self.mascot {
                painter.image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
                return;
            }
        }
        icons::logo(painter, rect.center(), ACCENT);
    }

    fn sync_lyrics(&mut self) {
        if !self.settings.lyrics_enabled {
            if !matches!(self.lyrics.state, LyricState::Disabled) {
                self.lyrics.id = self.lyrics_worker.cancel();
                self.lyrics.key = None;
                self.lyrics.state = LyricState::Disabled;
            }
        } else if self.demo {
            if !matches!(self.lyrics.state, LyricState::Ready(_)) {
                self.lyrics.state = LyricState::Ready(demo_lyrics(self.demo_track));
            }
        } else {
            let key = lyrics::TrackKey::from_media(&self.media);
            if key != self.lyrics.key || matches!(self.lyrics.state, LyricState::Disabled) {
                self.lyrics.id = self.lyrics_worker.cancel();
                self.lyrics.key = key.clone();
                self.lyrics.state = if let Some(key) = key {
                    self.lyrics.id = self.lyrics_worker.request(key, false);
                    LyricState::Loading
                } else {
                    LyricState::NoTrack
                };
            }
        }
        while let Ok(update) = self.lyrics_worker.updates.try_recv() {
            self.lyrics.accept(update);
        }
        let line = self
            .lyrics
            .state
            .compact_line(self.media.position_at(Instant::now()));
        if line != self.active_line {
            self.active_line = line.to_owned();
            self.line_changed = Instant::now();
        }
    }

    fn retry_lyrics(&mut self) {
        if self.demo {
            return;
        }
        if let Some(key) = &self.lyrics.key {
            self.lyrics.id = self.lyrics_worker.request(key.clone(), true);
            self.lyrics.state = LyricState::Loading;
        }
    }

    fn compact(&mut self, ui: &mut egui::Ui, shape: Rect) {
        if self.gaming_line() {
            self.performance_line(ui, shape);
            return;
        }
        let origin = shape.min;
        let center_y = shape.top() + self.compact_size().y * 0.5;
        self.paint_mascot(
            ui.painter(),
            Rect::from_center_size(pos2(origin.x + 30.0, center_y), vec2(36.0, 40.0)),
        );
        let timer_active = matches!(self.timer.phase, Phase::Running | Phase::Paused);
        let music = self.compact_size() == MUSIC_COMPACT;
        let label = if timer_active {
            format!("Focus  {}", self.timer.label())
        } else if !self.media.title.is_empty() {
            self.media.title.clone()
        } else {
            format!("Sela   {}", chrono::Local::now().format("%H:%M"))
        };
        let title_width = if music {
            (shape.width() * 0.27).min(150.0)
        } else {
            shape.width() - 126.0
        };
        let text_rect = Rect::from_min_size(
            pos2(origin.x + 58.0, center_y - 10.0),
            vec2(title_width, 22.0),
        );
        paint_line(ui.painter(), text_rect, &label, 13.0, TEXT, Align::Min);
        if music {
            let divider = text_rect.right() + 15.0;
            ui.painter().line_segment(
                [pos2(divider, center_y - 9.0), pos2(divider, center_y + 9.0)],
                Stroke::new(1.0_f32, SURFACE),
            );
            let fade = if self.settings.reduced_motion {
                1.0
            } else {
                (self.line_changed.elapsed().as_secs_f32() / 0.24).min(1.0)
            };
            let rect = Rect::from_min_max(
                pos2(divider + 16.0, center_y - 10.0 + (1.0 - fade) * 2.0),
                pos2(shape.right() - 24.0, center_y + 14.0),
            );
            paint_line(
                ui.painter(),
                rect,
                &self.active_line,
                13.0,
                AMBER.gamma_multiply(0.4 + 0.6 * fade),
                Align::Center,
            );
        } else {
            let trailing = self
                .snapshot
                .battery
                .as_ref()
                .map(|b| format!("{}%", b.percent))
                .unwrap_or_else(|| "•••".into());
            ui.painter().text(
                pos2(shape.right() - 24.0, center_y),
                Align2::RIGHT_CENTER,
                trailing,
                FontId::proportional(11.0),
                MUTED,
            );
        }
        let response = ui.interact(
            Rect::from_min_size(origin, vec2(shape.width(), self.compact_size().y)),
            Id::new("compact"),
            Sense::click(),
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                true,
                format!(
                    "Open Sela. {label}. {}",
                    if music { &self.active_line } else { "" }
                ),
            )
        });
        if response.clicked() {
            self.open(ui.ctx());
        }
        response.on_hover_text(if music {
            format!("{label}\n{}\nOpen Sela · Ctrl+Alt+N", self.active_line)
        } else {
            "Open Sela · Ctrl+Alt+N".into()
        });
    }

    fn expanded_ui(&mut self, ui: &mut egui::Ui) {
        let header = ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(22.0, 24.0), Sense::hover());
            self.paint_mascot(ui.painter(), r.expand2(vec2(2.0, 5.0)));
            ui.label(RichText::new("Sela").size(17.0).font(semibold(17.0)))
                .on_hover_text("Scroll over the header to switch pages");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_button(ui, Icon::Chevron, "Close panel · Esc", false, true).clicked() {
                    self.collapse();
                }
                if icon_button(
                    ui,
                    Icon::Pin,
                    if self.pinned {
                        "Unpin panel"
                    } else {
                        "Keep panel open"
                    },
                    self.pinned,
                    true,
                )
                .clicked()
                {
                    self.pinned = !self.pinned;
                }
            });
        });
        let over_header = ui.input(|input| {
            input
                .pointer
                .hover_pos()
                .is_some_and(|p| header.response.rect.contains(p))
        });
        if self.expanded && over_header {
            let delta = ui.input(|input| input.raw_scroll_delta.y);
            if let Some(forward) = self.header_wheel.input(delta, Instant::now()) {
                self.tab = self.tab.step(forward);
            }
        } else {
            self.header_wheel.leave();
        }
        ui.add_space(12.0);
        let tab_width = (ui.available_width() - 12.0) / 4.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for (tab, icon, text) in [
                (Tab::Media, Icon::Music, "Media"),
                (Tab::Focus, Icon::Focus, "Focus"),
                (Tab::System, Icon::System, "System"),
                (Tab::Settings, Icon::Settings, "Settings"),
            ] {
                let active = self.tab == tab;
                let (r, response) = ui.allocate_exact_size(vec2(tab_width, 34.0), Sense::click());
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text)
                });
                if active || response.hovered() || response.has_focus() {
                    ui.painter().rect_filled(r, 8, SURFACE);
                }
                if response.has_focus() {
                    ui.painter().rect_stroke(
                        r,
                        8,
                        Stroke::new(1.0_f32, ACCENT),
                        StrokeKind::Inside,
                    );
                }
                let color = if active { ACCENT } else { MUTED };
                icons::draw(
                    ui.painter(),
                    Rect::from_center_size(r.left_center() + vec2(20.0, 0.0), vec2(17.0, 17.0)),
                    icon,
                    color,
                );
                ui.painter().text(
                    r.left_center() + vec2(35.0, 0.0),
                    Align2::LEFT_CENTER,
                    text,
                    FontId::proportional(13.0),
                    color,
                );
                if response.clicked() {
                    self.tab = tab;
                }
                response.on_hover_text(format!("Open {text}"));
            }
        });
        ui.add_space(16.0);
        match self.tab {
            Tab::Media => self.media_ui(ui),
            Tab::Focus => self.focus_ui(ui),
            Tab::System => self.system_ui(ui),
            Tab::Settings => self.settings_ui(ui),
        }
    }

    fn media_ui(&mut self, ui: &mut egui::Ui) {
        if let Some(error) = &self.media.error {
            let error = error.clone();
            ui.add_space(16.0);
            ui.label(
                RichText::new("Media is unavailable.")
                    .size(20.0)
                    .color(ERROR),
            );
            ui.label("Make sure your player is open, then try again.");
            ui.add(egui::Label::new(RichText::new(&error).size(12.0).color(MUTED)).truncate())
                .on_hover_text(&error);
            if ui.button("Try again").clicked() {
                self.bridge = platform::media_bridge();
                self.media = Media::default();
            }
            return;
        }
        if self.media.title.is_empty() {
            ui.add_space(11.0);
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::hover());
            icons::draw(
                ui.painter(),
                Rect::from_center_size(r.center(), vec2(32.0, 32.0)),
                Icon::Music,
                ACCENT,
            );
            ui.add_space(12.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Room for your music.").size(21.0).strong());
                ui.add_space(6.0);
                ui.label(RichText::new("Play music in Spotify, your browser, or a player\nthat supports Windows media controls.").size(13.0).color(MUTED));
                if !cfg!(windows) {
                    ui.add_space(10.0);
                    ui.colored_label(MUTED, "Linux preview · media integration is available on Windows");
                }
            });
            return;
        }
        let body = ui.available_rect_before_wrap();
        let left = Rect::from_min_size(body.min, vec2(226.0, 208.0));
        let right = Rect::from_min_max(
            body.min + vec2(254.0, 0.0),
            pos2(body.right(), body.top() + 208.0),
        );
        ui.painter().line_segment(
            [
                pos2(body.left() + 240.0, body.top() + 2.0),
                pos2(body.left() + 240.0, body.top() + 188.0),
            ],
            Stroke::new(1.0_f32, SURFACE),
        );
        let mut player = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("player")
                .max_rect(left)
                .layout(Layout::top_down(Align::Min)),
        );
        player.horizontal(|ui| {
            let (cover, _) = ui.allocate_exact_size(vec2(52.0, 52.0), Sense::hover());
            ui.painter().rect_filled(cover, 12, ACCENT);
            icons::draw(ui.painter(), cover.shrink(15.0), Icon::Music, BG);
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.add(
                    egui::Label::new(RichText::new(&self.media.title).font(semibold(18.0)))
                        .truncate(),
                )
                .on_hover_text(&self.media.title);
                ui.add(
                    egui::Label::new(
                        RichText::new(if self.media.artist.is_empty() {
                            "Artist unavailable"
                        } else {
                            &self.media.artist
                        })
                        .size(12.0)
                        .color(MUTED),
                    )
                    .truncate(),
                )
                .on_hover_text(&self.media.artist);
            });
        });
        player.add_space(3.0);
        player.label(
            RichText::new(if self.demo {
                "DEMO · SAMPLE DATA"
            } else if self.media.playing {
                "NOW PLAYING"
            } else {
                "PAUSED"
            })
            .size(10.0)
            .color(ACCENT),
        );
        player.add_space(20.0);
        let position = self.media.position_at(Instant::now());
        progress(
            &mut player,
            if self.media.duration > 0.0 {
                (position / self.media.duration).clamp(0.0, 1.0) as f32
            } else {
                0.0
            },
            ACCENT,
        );
        player.horizontal(|ui| {
            ui.label(RichText::new(media_time(position)).size(11.0).color(MUTED));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(media_time(self.media.duration))
                        .size(11.0)
                        .color(MUTED),
                );
            });
        });
        player.add_space(8.0);
        player.horizontal(|ui| {
            ui.add_space(49.0);
            for (icon, label, command, enabled) in [
                (
                    Icon::Previous,
                    "Previous track",
                    MediaCommand::Previous,
                    self.media.can_previous,
                ),
                (
                    if self.media.playing {
                        Icon::Pause
                    } else {
                        Icon::Play
                    },
                    if self.media.playing { "Pause" } else { "Play" },
                    MediaCommand::Toggle,
                    self.media.can_toggle,
                ),
                (
                    Icon::Next,
                    "Next track",
                    MediaCommand::Next,
                    self.media.can_next,
                ),
            ] {
                let response = if matches!(command, MediaCommand::Toggle) {
                    play_button(ui, icon, label, enabled)
                } else {
                    icon_button(ui, icon, label, false, enabled)
                };
                if response.clicked() {
                    if self.demo {
                        self.media.position = self.media.position_at(Instant::now());
                        self.media.sampled_at = Instant::now();
                        match command {
                            MediaCommand::Toggle => self.media.playing = !self.media.playing,
                            MediaCommand::Next | MediaCommand::Previous => {
                                self.demo_track = (self.demo_track
                                    + if matches!(command, MediaCommand::Next) {
                                        1
                                    } else {
                                        4
                                    })
                                    % 5;
                                let playing = self.media.playing;
                                self.media = demo_media(self.demo_track);
                                self.media.playing = playing;
                                self.lyrics.state = LyricState::Ready(demo_lyrics(self.demo_track));
                                self.lyrics.key = lyrics::TrackKey::from_media(&self.media);
                            }
                        }
                    } else if self.bridge.commands.send(command).is_err() {
                        self.error = Some("The media service stopped. Restart Sela.".into());
                    }
                }
            }
        });
        let mut lyric_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("lyrics")
                .max_rect(right)
                .layout(Layout::top_down(Align::Min)),
        );
        self.lyrics_ui(&mut lyric_ui, position);
        ui.advance_cursor_after_rect(Rect::from_min_max(
            body.min,
            pos2(body.right(), body.top() + 208.0),
        ));
    }

    fn lyrics_ui(&mut self, ui: &mut egui::Ui, position: f64) {
        let state = self.lyrics.state.clone();
        let synced = matches!(state, LyricState::Ready(LyricData::Synced(_)));
        ui.label(
            RichText::new(if synced {
                "SYNCED LYRICS"
            } else {
                "LYRICS STATUS"
            })
            .size(10.0)
            .color(MUTED),
        );
        ui.add_space(18.0);
        match &state {
            LyricState::Ready(data @ LyricData::Synced(lines)) => {
                let active = data.active_index(position);
                let index = active.unwrap_or(0);
                let previous = index
                    .checked_sub(1)
                    .and_then(|i| lines.get(i))
                    .map(|l| l.text.as_str())
                    .unwrap_or("");
                lyric_label(ui, previous, 13.0, MUTED, false);
                ui.add_space(7.0);
                let fade = if self.settings.reduced_motion {
                    1.0
                } else {
                    (self.line_changed.elapsed().as_secs_f32() / 0.24).min(1.0)
                };
                lyric_label(
                    ui,
                    &self.active_line,
                    18.0,
                    AMBER.gamma_multiply(0.4 + 0.6 * fade),
                    true,
                );
                ui.add_space(7.0);
                let next = lines
                    .get(if active.is_some() { index + 1 } else { 0 })
                    .map(|l| l.text.as_str())
                    .unwrap_or("");
                lyric_label(ui, next, 13.0, MUTED, false);
                ui.add_space(22.0);
                ui.label(
                    RichText::new(if self.demo {
                        "Original Sela demo lyrics · no audio"
                    } else if self.media.playing {
                        "Following your player · LRCLIB"
                    } else {
                        "Paused · holding this line · LRCLIB"
                    })
                    .size(10.0)
                    .color(MUTED),
                );
            }
            LyricState::Ready(LyricData::Plain(text)) => {
                ui.label(
                    RichText::new("Lyrics without timestamps")
                        .size(12.0)
                        .color(AMBER),
                );
                egui::ScrollArea::vertical()
                    .id_salt("plain-lyrics")
                    .max_height(122.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new(text).size(13.0));
                    });
                ui.label(
                    RichText::new(if self.demo {
                        "Original Sela demo lyrics · no audio"
                    } else {
                        "Source: LRCLIB"
                    })
                    .size(10.0)
                    .color(MUTED),
                );
            }
            _ => {
                let heading = if matches!(state, LyricState::Error(_)) {
                    "Lyrics couldn't load"
                } else {
                    state.compact_line(position)
                };
                lyric_label(
                    ui,
                    heading,
                    17.0,
                    if matches!(state, LyricState::Error(_)) {
                        ERROR
                    } else {
                        TEXT
                    },
                    true,
                );
                ui.add_space(8.0);
                let explanation = match &state {
                    LyricState::Disabled => "Enable lyrics in Settings to show them.",
                    LyricState::Loading => {
                        "Looking for a matching title, artist, and duration on LRCLIB."
                    }
                    LyricState::NoTrack => "Your player needs to provide a title and artist.",
                    LyricState::Error(error) => error.as_str(),
                    LyricState::Ready(LyricData::Instrumental) => {
                        if self.demo {
                            "A sample instrumental track with no lyrics."
                        } else {
                            "This track is marked as instrumental on LRCLIB."
                        }
                    }
                    _ => {
                        if self.demo {
                            "A sample unavailable-lyrics state. Music can still play."
                        } else {
                            "No matching lyrics on LRCLIB yet. Music can still play."
                        }
                    }
                };
                ui.label(RichText::new(explanation).size(12.0).color(MUTED));
                if matches!(
                    state,
                    LyricState::Error(_) | LyricState::Ready(LyricData::NotFound)
                ) && !self.demo
                    && ui.small_button("Try again").clicked()
                {
                    self.retry_lyrics();
                }
            }
        }
    }

    fn focus_ui(&mut self, ui: &mut egui::Ui) {
        let (caption, color) = match self.timer.phase {
            Phase::Ready => ("One thing. Your full attention.", MUTED),
            Phase::Running => ("Time to focus. You've got this.", ACCENT),
            Phase::Paused => ("Paused. Resume when you're ready.", MUTED),
            Phase::Complete => ("Session complete. Take a short break.", ACCENT),
        };
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(caption).size(13.0).color(color));
            ui.add_space(1.0);
            ui.label(
                RichText::new(self.timer.label())
                    .size(57.0)
                    .monospace()
                    .color(TEXT),
            );
        });
        progress(ui, self.timer.progress(), ACCENT);
        ui.add_space(9.0);
        ui.horizontal(|ui| {
            ui.add_space(80.0);
            for minutes in [15, 25, 50] {
                let selected = self.settings.focus_minutes == minutes;
                let enabled = matches!(self.timer.phase, Phase::Ready | Phase::Complete);
                if ui
                    .add_enabled(
                        enabled,
                        egui::Button::new(
                            RichText::new(format!("{minutes} min")).color(if selected {
                                ACCENT
                            } else {
                                MUTED
                            }),
                        )
                        .fill(if selected { SURFACE } else { BG }),
                    )
                    .on_hover_text("Choose the duration of your next session")
                    .clicked()
                {
                    self.settings.focus_minutes = minutes;
                    self.timer.reset(minutes);
                    self.persist();
                }
            }
        });
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.add_space(93.0);
            let title = match self.timer.phase {
                Phase::Running => "Pause focus",
                Phase::Paused => "Resume",
                _ => "Start focus",
            };
            if ui
                .add_sized(
                    [160.0, 35.0],
                    egui::Button::new(RichText::new(title).color(BG).strong()).fill(ACCENT),
                )
                .clicked()
            {
                self.timer.toggle();
                self.last_tick = Instant::now();
            }
            if icon_button(ui, Icon::Reset, "Reset timer", false, true).clicked() {
                self.timer.reset(self.settings.focus_minutes);
            }
        });
    }

    fn performance_line(&self, ui: &mut egui::Ui, shape: Rect) {
        let r = &self.performance_reading;
        let h = &r.hardware;
        let scale = shape.width() / 816.0;
        let outlined = self.line_style() == LineStyle::TextOnly;
        let number = |n: Option<f64>, decimals: usize| {
            n.filter(|v| v.is_finite())
                .map_or_else(|| "—".to_owned(), |v| format!("{v:.decimals$}"))
        };
        let cells = [
            (
                "FPS",
                number(r.frames.as_ref().map(|s| s.fps), 0),
                74.0,
                ACCENT,
            ),
            ("RAM", format!("{} GB", number(r.ram_gb, 1)), 104.0, TEXT),
            ("VRAM", format!("{} GB", number(h.vram_gb, 1)), 110.0, TEXT),
            (
                "GPU",
                format!("{}% {}°C", number(h.gpu, 0), number(h.gpu_temp, 0)),
                126.0,
                if h.gpu_temp.is_some_and(|v| v > 85.0) {
                    AMBER
                } else {
                    TEXT
                },
            ),
            (
                "CPU",
                format!("{}% {}°C", number(r.cpu, 0), number(h.cpu_temp, 0)),
                126.0,
                TEXT,
            ),
            (
                "FT",
                format!("{} ms", number(r.frames.as_ref().map(|s| s.frame_ms), 1)),
                92.0,
                TEXT,
            ),
            (
                "1% LOW",
                number(r.frames.as_ref().map(|s| s.low), 0),
                104.0,
                ACCENT,
            ),
        ];
        let y = shape.top() + 24.0;
        let label_color = if outlined { TEXT } else { MUTED };
        outlined_text(
            ui.painter(),
            pos2(shape.left() + 14.0 * scale, y),
            Align2::LEFT_CENTER,
            if self.performance_demo {
                "DEMO"
            } else {
                "Sela"
            },
            FontId::proportional(11.0 * scale),
            if self.performance_demo {
                AMBER
            } else {
                label_color
            },
            outlined,
        );
        let mut x = shape.left() + 66.0 * scale;
        let mut accessible = String::from("Performance. ");
        for (label, value, w, color) in cells {
            outlined_text(
                ui.painter(),
                pos2(x, y),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(10.0 * scale),
                label_color,
                outlined,
            );
            outlined_text(
                ui.painter(),
                pos2(x + (w - 14.0) * scale, y),
                Align2::RIGHT_CENTER,
                &value,
                FontId::monospace(11.5 * scale),
                color,
                outlined,
            );
            accessible.push_str(&format!("{label}: {value}. "));
            x += w * scale;
        }
        if let Some(seconds) = r.recording_seconds {
            outlined_text(
                ui.painter(),
                pos2(shape.right() - 20.0, shape.top() + 42.0),
                Align2::RIGHT_CENTER,
                &format!("REC {seconds:.1}s"),
                FontId::monospace(9.0 * scale),
                AMBER,
                outlined,
            );
        }
        let response = ui.interact(shape, Id::new("performance-line"), Sense::hover());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Label, true, accessible.clone())
        });
        response.on_hover_text(
            "Gaming line is click-through. Ctrl+Alt+N or the tray opens its controls.",
        );
    }

    fn performance_ui(&mut self, ui: &mut egui::Ui) {
        let mut changed = false;
        egui::ScrollArea::vertical().id_salt("performance-scroll").max_height(278.0).show(ui,|ui|{
            changed|=ui.checkbox(&mut self.settings.performance_enabled,"Show gaming performance line").changed();
            changed|=ui.checkbox(&mut self.settings.performance_auto,"Only while a game is playing").changed();
            ui.label(RichText::new("A game is a fullscreen application that is rendering frames. Browsers and video players are ignored unless marked as a game.").size(11.0).color(MUTED));
            let mut style=self.line_style();
            ui.horizontal(|ui|{
                ui.label("FPS line style");
                ui.radio_value(&mut style,LineStyle::TextOnly,"Text only");
                ui.radio_value(&mut style,LineStyle::Background,"With background");
            });
            if style!=self.line_style(){
                // Edit the captured app's saved style when it has one, else the default.
                match self.target_name().filter(|name|self.settings.performance_profiles.contains_key(name)){
                    Some(name)=>{self.settings.performance_profiles.insert(name,style);}
                    None=>self.settings.performance_style=style,
                }
                changed=true;
            }
            ui.label(RichText::new("All readings stay on one line. When closed, clicks pass through to your game; use Ctrl+Alt+N or the tray to open Sela.").size(11.0).color(MUTED));
            let applications=self.performance_reading.applications.clone();
            ui.horizontal(|ui|{
                ui.label("Capture");
                egui::ComboBox::from_id_salt("performance-process").width(240.0).selected_text(if self.settings.performance_target.is_empty(){"Foreground application"}else{&self.settings.performance_target}).show_ui(ui,|ui|{
                    changed|=ui.selectable_value(&mut self.settings.performance_target,String::new(),"Foreground application").changed();
                    for app in &applications {changed|=ui.selectable_value(&mut self.settings.performance_target,app.name.clone(),format!("{} · PID {}",app.name,app.pid)).changed();}
                });
                if ui.small_button("Retry").clicked(){self.performance.retry();}
            });
            if let Some(app)=&self.performance_reading.target {
                ui.horizontal(|ui|{
                    ui.label(RichText::new(format!("{} · PID {}",app.name,app.pid)).size(11.0).color(MUTED));
                    let name=app.name.to_ascii_lowercase();
                    if self.settings.performance_profiles.contains_key(&name){
                        if ui.small_button("Use default style").clicked(){self.settings.performance_profiles.remove(&name);changed=true;}
                    } else if ui.small_button("Save style for this app").clicked(){self.settings.performance_profiles.insert(name.clone(),self.settings.performance_style);changed=true;}
                    let marked=self.settings.performance_games.contains(&name);
                    if ui.small_button(if marked {"Unmark as game"} else {"Mark as game"}).clicked(){
                        if marked {self.settings.performance_games.retain(|g|*g!=name);} else {self.settings.performance_games.push(name);}
                        changed=true;
                    }
                });
            }
            // Windowed games are never captured automatically, so they are marked from this list.
            ui.horizontal_wrapped(|ui|{
                ui.label("Games");
                egui::ComboBox::from_id_salt("mark-game").width(200.0).selected_text("Mark a running app…").show_ui(ui,|ui|{
                    for app in &applications {
                        let name=app.name.to_ascii_lowercase();
                        if !self.settings.performance_games.contains(&name)&&ui.selectable_label(false,&app.name).clicked(){self.settings.performance_games.push(name);changed=true;}
                    }
                });
                let mut removed=None;
                for game in &self.settings.performance_games {
                    if ui.small_button(format!("{game} ×")).on_hover_text("Unmark as game").clicked(){removed=Some(game.clone());}
                }
                if let Some(game)=removed{self.settings.performance_games.retain(|g|*g!=game);changed=true;}
            });
            ui.label(RichText::new(&self.performance_reading.status).size(11.0).color(if self.performance_demo {AMBER}else{MUTED}));
            let number=|n:Option<f64>,decimals:usize|n.filter(|v|v.is_finite()).map_or_else(||"Unavailable".to_owned(),|v|format!("{v:.decimals$}"));
            let r=&self.performance_reading;
            ui.columns(3,|columns|{
                for(ui,(name,value))in columns.iter_mut().zip([
                    ("FPS",number(r.frames.as_ref().map(|s|s.fps),0)),
                    ("1% low",number(r.frames.as_ref().map(|s|s.low),0)),
                    ("Frame time",r.frames.as_ref().map_or_else(||"Unavailable".to_owned(),|s|format!("{:.2} ms",s.frame_ms)))
                ]) {ui.label(RichText::new(name).size(11.0).color(MUTED));ui.label(RichText::new(value).monospace().size(21.0).color(ACCENT));}
            });
            let (chart,_)=ui.allocate_exact_size(vec2(ui.available_width(),58.0),Sense::hover());
            ui.painter().rect_filled(chart,8,SURFACE);
            let max=self.performance_reading.history.iter().copied().fold(25.0,f64::max)*1.1;
            let values=&self.performance_reading.history;
            if values.len()>1 {
                let points=values.iter().enumerate().map(|(i,v)|pos2(chart.left()+i as f32/(values.len()-1)as f32*chart.width(),chart.bottom()-(*v/max)as f32*chart.height())).collect::<Vec<_>>();
                ui.painter().add(egui::Shape::line(points,Stroke::new(1.5_f32,ACCENT)));
            } else {ui.painter().text(chart.center(),Align2::CENTER_CENTER,"Waiting for frames",FontId::proportional(11.0),MUTED);}
            ui.label(RichText::new("Frame time · latest 120 presents · lower is better").size(10.0).color(MUTED));
            egui::Grid::new("performance-hardware").num_columns(4).spacing(vec2(18.0,7.0)).show(ui,|ui|{
                for(a,av,b,bv)in [
                    ("CPU",format!("{}%",number(r.cpu,0)),"RAM",format!("{} GB",number(r.ram_gb,1))),
                    ("GPU",format!("{}%",number(r.hardware.gpu,0)),"VRAM",format!("{} GB",number(r.hardware.vram_gb,1))),
                    ("GPU temp",format!("{} °C",number(r.hardware.gpu_temp,0)),"GPU power",format!("{} W",number(r.hardware.power,0))),
                ] {ui.label(a);ui.monospace(av);ui.label(b);ui.monospace(bv);ui.end_row();}
            });
            ui.label(RichText::new(&r.hardware.source).size(10.0).color(MUTED));
            ui.label(RichText::new("Frame capture may require administrator or Performance Log Users permissions. If FPS is unavailable, relaunch with the required permission and Retry.").size(10.0).color(MUTED));
            ui.collapsing("PresentMon license",|ui|{ui.label(include_str!("../assets/licenses/PresentMon.txt"));});
            ui.label(RichText::new("— means unavailable. Fullscreen and anti-cheat compatibility depend on the game; borderless mode is recommended.").size(10.0).color(MUTED));
            ui.separator();
            ui.horizontal(|ui|{
                ui.label("Benchmark");
                egui::ComboBox::from_id_salt("benchmark-length").selected_text(format!("{} s",self.benchmark_seconds)).show_ui(ui,|ui|{for n in [15,30,60]{ui.selectable_value(&mut self.benchmark_seconds,n,format!("{n} seconds"));}});
                if let Some(seconds)=self.performance_reading.recording_seconds {
                    ui.label(format!("REC {seconds:.1}s captured"));
                    if ui.button("Finish").clicked(){self.performance.finish();}
                } else if ui.add_enabled(self.performance_reading.frames.is_some(),egui::Button::new("Start recording")).clicked(){self.performance.start(self.benchmark_seconds);}
            });
            ui.label(RichText::new("Capture excludes gaps without frames. Recording finishes if its process changes or monitoring stops.").size(10.0).color(MUTED));
            if let Some(session)=self.performance_sessions.last().cloned(){
                ui.collapsing("Latest benchmark and comparison",|ui|{
                    ui.label(format!("{} · {:.1} seconds · {} frames",session.application,session.captured_seconds,session.stats.frames));
                    let previous=self.performance_sessions.iter().rev().nth(1);
                    egui::Grid::new("benchmark-comparison").show(ui,|ui|{
                        ui.label("Measurement");ui.label("Latest");ui.label("Previous");ui.end_row();
                        for(name,current,old)in [("Average FPS",session.stats.fps,previous.map(|s|s.stats.fps)),("1% low",session.stats.low,previous.map(|s|s.stats.low)),("Frame time (ms)",session.stats.frame_ms,previous.map(|s|s.stats.frame_ms)),("Worst frame (ms)",session.stats.worst_ms,previous.map(|s|s.stats.worst_ms))]{ui.label(name);ui.monospace(format!("{current:.2}"));ui.monospace(number(old,2));ui.end_row();}
                    });
                    ui.label(RichText::new("1% low = 1000 / mean of the slowest 1% of captured frame times. Previous sessions may use different applications.").size(10.0).color(MUTED));
                    if ui.button("Export CSV + JSON").clicked(){match session.export(){Ok(folder)=>self.export_folder=Some(folder),Err(e)=>self.error=Some(e)}}
                });
            }else{ui.label(RichText::new("No recorded sessions yet.").size(11.0).color(MUTED));}
            if let Some(folder)=&self.export_folder{ui.horizontal(|ui|{ui.label("Report saved.");if ui.button("Open export folder").clicked(){if let Err(e)=performance::open_folder(folder){self.error=Some(e);}}});ui.label(RichText::new(folder.display().to_string()).size(10.0).color(MUTED));}
        });
        if changed {
            self.persist();
        }
    }

    fn system_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.performance_page, false, "System overview");
            ui.selectable_value(&mut self.performance_page, true, "Performance");
        });
        ui.add_space(10.0);
        if self.performance_page {
            self.performance_ui(ui);
            return;
        }
        if !self.snapshot.metrics_ready {
            ui.add_space(36.0);
            ui.label(RichText::new("Preparing readings…").size(18.0));
            ui.label(
                RichText::new("CPU and memory are sampled while System is open.").color(MUTED),
            );
            return;
        }
        metric(
            ui,
            "Processor",
            &format!("{:.0}%", self.snapshot.cpu),
            self.snapshot.cpu / 100.0,
        );
        ui.add_space(15.0);
        let gb = |bytes: u64| bytes as f64 / 1_073_741_824.0;
        let ram = if self.snapshot.memory_total == 0 {
            0.0
        } else {
            self.snapshot.memory_used as f32 / self.snapshot.memory_total as f32
        };
        metric(
            ui,
            "Memory",
            &format!(
                "{:.1} / {:.1} GB",
                gb(self.snapshot.memory_used),
                gb(self.snapshot.memory_total)
            ),
            ram,
        );
        ui.add_space(15.0);
        if let Some(battery) = &self.snapshot.battery {
            metric(
                ui,
                if battery.plugged_in {
                    "Battery · plugged in"
                } else {
                    "Battery"
                },
                &format!("{}%", battery.percent),
                f32::from(battery.percent) / 100.0,
            );
        } else {
            ui.horizontal(|ui| {
                ui.colored_label(MUTED, "Battery");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.colored_label(MUTED, "Unavailable");
                });
            });
        }
        ui.add_space(18.0);
        ui.label(
            RichText::new("Updates every 2 seconds while this page is open")
                .size(11.0)
                .color(MUTED),
        );
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        let mut changed = false;
        egui::ScrollArea::vertical()
            .max_height(207.0)
            .show(ui, |ui| {
                changed |= ui
                    .checkbox(
                        &mut self.settings.expand_on_hover,
                        "Open when the pointer hovers over the notch",
                    )
                    .changed();
                changed |= ui
                    .checkbox(&mut self.settings.reduced_motion, "Reduce motion")
                    .changed();
                changed |= ui.checkbox(&mut self.settings.show_mascot, "Show Pijar, Sela's mascot").changed();
                changed |= ui.checkbox(&mut self.settings.performance_enabled, "Show gaming performance line").changed();
                ui.label(RichText::new("System → Performance configures capture, style, and game detection. The line is click-through; Ctrl+Alt+N or the tray opens Sela.").size(11.0).color(MUTED));
                changed |= ui.checkbox(&mut self.settings.lyrics_enabled, "Show music lyrics").changed();
                ui.label(RichText::new("When enabled, the title, artist, and duration are sent to LRCLIB. Lyrics are cached locally.").size(11.0).color(MUTED));
                changed |= ui
                    .checkbox(&mut self.settings.sound, "Play a sound when a focus session ends")
                    .changed();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Top offset");
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut self.settings.top_offset, 0.0..=80.0)
                                .suffix(" px"),
                        )
                        .changed();
                });
                let count = platform::monitor_count();
                self.settings.monitor = self.settings.monitor.min(count - 1);
                if count > 1 {
                    ui.horizontal(|ui| {
                        ui.label("Monitor");
                        egui::ComboBox::from_id_salt("monitor")
                            .selected_text(format!("Monitor {}", self.settings.monitor + 1))
                            .show_ui(ui, |ui| {
                                for index in 0..count {
                                    changed |= ui
                                        .selectable_value(
                                            &mut self.settings.monitor,
                                            index,
                                            format!("Monitor {}", index + 1),
                                        )
                                        .changed();
                                }
                            });
                    });
                }
                ui.add_space(5.0);
                ui.label(
                    RichText::new("Ctrl+Alt+N  open / close  ·  Esc  close")
                        .size(11.0)
                        .color(MUTED),
                );
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(concat!("Sela ", env!("CARGO_PKG_VERSION")))
                            .size(11.0)
                            .color(MUTED),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("Quit Sela").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
                if let Some(error) = &self.error {
                    ui.colored_label(ERROR, error);
                }
            });
        if changed {
            self.persist();
        }
    }
}

impl eframe::App for NotchApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.save_due.is_some() {
            self.save_now();
        }
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;
        if let Some(due) = self.save_due {
            if now >= due {
                self.save_now();
            } else {
                ctx.request_repaint_after(due - now);
            }
        }
        if self.timer.advance(elapsed) {
            if !self.gaming_line() {
                self.tab = Tab::Focus;
                self.expanded = true;
            }
            self.hover_opened = false;
            if self.settings.sound {
                platform::completion_sound();
            }
        }
        while let Ok(snapshot) = self.monitor.updates.try_recv() {
            self.snapshot = snapshot;
        }
        while let Ok(media) = self.bridge.updates.try_recv() {
            if !self.demo {
                self.media = media;
            }
        }
        if self.demo && self.media.playing && self.media.position_at(now) >= self.media.duration {
            self.media.position = self.media.duration;
            self.media.playing = false;
        }
        self.sync_lyrics();
        if let Some(integration) = &self.integration {
            if let Some(action) = integration.poll() {
                match action {
                    Action::Toggle if self.expanded => self.collapse(),
                    Action::Toggle | Action::Open => {
                        if !self.expanded {
                            self.open_shell(ctx);
                        }
                    }
                    Action::Focus => {
                        self.tab = Tab::Focus;
                        self.open(ctx);
                    }
                    Action::Settings => {
                        self.tab = Tab::Settings;
                        self.open(ctx);
                    }
                    Action::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                }
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.collapse();
        }
        if !self.expanded
            && ctx.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space))
        {
            self.open_shell(ctx);
        }
        while let Ok(reading) = self.performance.updates.try_recv() {
            if let Some(session) = &reading.completed {
                if !self.performance_sessions.iter().any(|s| s.id == session.id) {
                    self.performance_sessions.push(session.clone());
                    if self.performance_sessions.len() > 20 {
                        self.performance_sessions.remove(0);
                    }
                }
            }
            self.performance_reading = reading;
        }
        let ppp = ctx.pixels_per_point();
        self.monitor_width = platform::monitor_geometry(self.settings.monitor, ppp).map_or_else(
            || {
                ctx.input(|i| i.viewport().monitor_size)
                    .map_or(1920.0, |s| s.x)
            },
            |(_, w)| w,
        );
        let passthrough = self.gaming_line() && !self.expanded;
        if !platform::set_passthrough(frame, passthrough) && passthrough != self.last_passthrough {
            ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(passthrough));
            self.last_passthrough = passthrough;
        }
        let previous_size = vec2(self.width.value(), self.height.value());
        let previous_shape = Rect::from_min_size(
            pos2((ctx.screen_rect().width() - previous_size.x) * 0.5, 0.0),
            previous_size,
        );
        let pointer_inside = ctx.input(|i| {
            i.pointer
                .hover_pos()
                .is_some_and(|p| previous_shape.contains(p))
        });
        if pointer_inside {
            self.pointer_left = None;
            if self.expanded {
                self.hover_started = None;
            }
            if !self.expanded
                && !self.gaming_line()
                && self.settings.expand_on_hover
                && !self.suppress_hover
            {
                let since = self.hover_started.get_or_insert(now);
                if now.duration_since(*since) >= HOVER_OPEN {
                    self.expanded = true;
                    self.hover_opened = true;
                    self.hover_started = None;
                }
            }
        } else {
            self.hover_started = None;
            self.suppress_hover = false;
            if !self.expanded || !self.hover_opened || self.pinned {
                self.pointer_left = None;
            }
            if self.expanded && self.hover_opened && !self.pinned {
                let since = self.pointer_left.get_or_insert(now);
                if now.duration_since(*since) >= HOVER_CLOSE {
                    self.expanded = false;
                    self.pointer_left = None;
                }
            }
        }
        self.motion
            .advance(self.expanded, elapsed, self.settings.reduced_motion);
        let target = self.target_size();
        self.width
            .advance(target.x, elapsed, self.settings.reduced_motion);
        self.height
            .advance(target.y, elapsed, self.settings.reduced_motion);
        let ppp = ctx.pixels_per_point();
        let size = vec2(self.width.value(), self.height.value());
        // Windows keeps one fixed backing surface and clips its native input area.
        // Linux uses the same canvas for visual previews; native input clipping is Windows-only.
        let canvas = if self.fixed_surface || self.expanded || self.motion.is_active() {
            vec2(840.0_f32.min(self.monitor_width.max(600.0)), 440.0)
        } else {
            size
        };
        self.geometry(ctx, canvas);
        let canvas_width = ctx.screen_rect().width();
        let left = (((canvas_width - size.x) * 0.5) * ppp).round() / ppp;
        let shape = Rect::from_min_max(
            pos2(left, 0.0),
            pos2(
                ((left + size.x) * ppp).round() / ppp,
                (size.y * ppp).round() / ppp,
            ),
        );
        if self.fixed_surface {
            if let Err(error) = self.window_region.set(frame, shape, ppp) {
                self.fixed_surface = false;
                self.error = Some(error);
            }
        }
        let compact_alpha = self.motion.compact_opacity();
        let content_alpha = self.motion.content_opacity();
        let widget_duration = if self.settings.reduced_motion {
            0.0
        } else {
            0.2
        };
        ctx.style_mut(|style| style.animation_time = widget_duration);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let radius = CornerRadius {
                    nw: if self.settings.top_offset > 0.0 {
                        16
                    } else {
                        0
                    },
                    ne: if self.settings.top_offset > 0.0 {
                        16
                    } else {
                        0
                    },
                    sw: 24,
                    se: 24,
                };
                let text_only = self.gaming_line()
                    && !self.expanded
                    && self.line_style() == LineStyle::TextOnly;
                if !text_only {
                    ui.painter().rect_filled(shape, radius, BG);
                }
                if compact_alpha > 0.001 {
                    let mut compact = ui.new_child(
                        egui::UiBuilder::new()
                            .id_salt("compact-layer")
                            .max_rect(shape),
                    );
                    compact.set_opacity(compact_alpha);
                    self.compact(&mut compact, shape);
                }
                if content_alpha > 0.001 {
                    // Lay out at the final width throughout the reveal: text never
                    // rewraps as the silhouette grows, and the mask does the reveal.
                    let expanded_size = self.expanded_size();
                    let expanded_left = (canvas_width - expanded_size.x) * 0.5;
                    let content = Rect::from_min_size(
                        pos2(expanded_left, (1.0 - content_alpha) * 10.0),
                        expanded_size,
                    )
                    .shrink2(vec2(24.0, 18.0));
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .id_salt("expanded-layer")
                            .max_rect(content)
                            .layout(Layout::top_down(Align::Min)),
                    );
                    child.set_clip_rect(shape.shrink2(vec2(12.0, 3.0)).intersect(ui.clip_rect()));
                    child.set_opacity(content_alpha);
                    self.expanded_ui(&mut child);
                    let footer = if self.error.is_some() {
                        "Needs attention · see Settings".to_owned()
                    } else if self.timer.completed > 0 {
                        format!("Focus sessions completed: {}", self.timer.completed)
                    } else {
                        "Sela · a little space for you.".into()
                    };
                    let footer_painter =
                        ui.painter().with_clip_rect(shape.intersect(ui.clip_rect()));
                    footer_painter.text(
                        pos2(content.left(), content.bottom()),
                        Align2::LEFT_CENTER,
                        footer,
                        FontId::proportional(10.0),
                        (if self.error.is_some() { ERROR } else { MUTED })
                            .gamma_multiply(content_alpha),
                    );
                    footer_painter.circle_filled(
                        pos2(content.right() - 2.0, content.bottom()),
                        2.5,
                        ACCENT.gamma_multiply(content_alpha),
                    );
                }
            });
        self.motion.retarget(self.expanded);
        let target = self.target_size();
        self.width.retarget(target.x);
        self.height.retarget(target.y);
        self.performance.configure(performance::Configuration {
            enabled: self.settings.performance_enabled,
            target: self.settings.performance_target.clone(),
            demo: self.performance_demo,
            auto: self.settings.performance_auto,
            games: self.settings.performance_games.clone(),
        });
        if self
            .monitor
            .set_active(self.expanded && self.tab == Tab::System && !self.performance_page)
            && self.expanded
            && self.tab == Tab::System
        {
            self.snapshot.metrics_ready = false;
            ctx.request_repaint();
        }
        if !self.expanded && self.motion.is_active() {
            ctx.memory_mut(|memory| {
                if let Some(focused) = memory.focused() {
                    memory.surrender_focus(focused);
                }
            });
        }
        if self.motion.is_active()
            || self.width.is_active()
            || self.height.is_active()
            || !self.settings.reduced_motion
                && self.line_changed.elapsed() < Duration::from_millis(240)
        {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(Duration::from_millis(if self.media.playing {
                100
            } else {
                250
            }));
        }
        if let Some(started) = self.hover_started {
            ctx.request_repaint_after(HOVER_OPEN.saturating_sub(now.duration_since(started)));
        }
        if let Some(left) = self.pointer_left {
            ctx.request_repaint_after(HOVER_CLOSE.saturating_sub(now.duration_since(left)));
        }
    }
}

fn icon_button(
    ui: &mut egui::Ui,
    icon: Icon,
    label: &str,
    active: bool,
    enabled: bool,
) -> egui::Response {
    ui.add_enabled_ui(enabled, |ui| {
        let (r, response) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
        if active || response.hovered() {
            ui.painter().rect_filled(r, 8, SURFACE);
        }
        if response.has_focus() {
            ui.painter()
                .rect_stroke(r, 8, Stroke::new(1.0_f32, ACCENT), StrokeKind::Inside);
        }
        let color = if !enabled {
            MUTED.gamma_multiply(0.35)
        } else if active {
            ACCENT
        } else {
            MUTED
        };
        icons::draw(ui.painter(), r.shrink(6.0), icon, color);
        response.on_hover_text(label)
    })
    .inner
}

fn progress(ui: &mut egui::Ui, fraction: f32, color: Color32) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 3.0), Sense::hover());
    ui.painter().rect_filled(r, 2, SURFACE);
    if fraction > 0.0 {
        ui.painter().rect_filled(
            Rect::from_min_size(r.min, vec2(r.width() * fraction.clamp(0.0, 1.0), 3.0)),
            2,
            color,
        );
    }
}

fn metric(ui: &mut egui::Ui, label: &str, value: &str, fraction: f32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(13.0).color(MUTED));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).size(16.0).monospace());
        });
    });
    ui.add_space(3.0);
    progress(ui, fraction, ACCENT);
}

fn media_time(seconds: f64) -> String {
    if !seconds.is_finite() || seconds <= 0.0 {
        return "0:00".into();
    }
    let seconds = seconds as u64;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn semibold(size: f32) -> FontId {
    FontId::new(size, egui::FontFamily::Name("semibold".into()))
}

fn paint_line(
    painter: &egui::Painter,
    rect: Rect,
    text: &str,
    size: f32,
    color: Color32,
    align: Align,
) {
    let mut job = egui::text::LayoutJob::simple(
        text.replace(['\n', '\r'], " "),
        FontId::proportional(size),
        color,
        rect.width().max(1.0),
    );
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    let galley = painter.layout_job(job);
    let x = if align == Align::Center {
        rect.left() + ((rect.width() - galley.size().x) * 0.5).max(0.0)
    } else {
        rect.left()
    };
    painter
        .with_clip_rect(rect.intersect(painter.clip_rect()))
        .galley(pos2(x, rect.top()), galley, color);
}

fn lyric_label(ui: &mut egui::Ui, text: &str, size: f32, color: Color32, strong: bool) {
    let font = if strong {
        semibold(size)
    } else {
        FontId::proportional(size)
    };
    // Allocate a full-width row, then anchor its galley at the shared left edge.
    // add_sized creates a centered child layout even for a left-aligned Label.
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), size + 8.0), Sense::hover());
    let mut job = egui::text::LayoutJob::simple(
        text.replace(['\n', '\r'], " "),
        font,
        color,
        rect.width().max(1.0),
    );
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    let galley = ui.painter().layout_job(job);
    let origin = pos2(rect.left(), rect.center().y - galley.size().y * 0.5);
    ui.painter()
        .with_clip_rect(rect.intersect(ui.clip_rect()))
        .galley(origin, galley, color);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, text));
    response.on_hover_text(text);
}

// Original sample text. Enabled only by the explicit --demo flag; no audio or LRCLIB requests.
const DEMO_LYRICS: &str = "[00:00]Take it slow, the morning is still young\n[00:06]A little light between your days\n[00:12]One small step, and then another\n[00:18]Let the world wait for a moment\n[00:24]We have a little room to breathe\n[00:30]Even little things can glow\n[00:36]Keep this warmth until tomorrow\n[00:42]Find your way back to the quiet";
fn demo_media(index: usize) -> Media {
    Media {
        title: [
            "Morning Light",
            "A little room for the days that move slowly — Sela Sessions",
            "Quiet Hours",
            "Soft Rain",
            "Little Lantern",
        ][index % 5]
            .into(),
        artist: "Sela Studio".into(),
        source: "Demo".into(),
        playing: true,
        position: 6.0,
        duration: 48.0,
        can_toggle: true,
        can_previous: true,
        can_next: true,
        ..Default::default()
    }
}

fn demo_lyrics(index: usize) -> LyricData {
    match index % 5 {
        1 => LyricData::Synced(lyrics::parse_lrc("[00:00]Between the longer steps we take\n[00:06]We have a little room to take a breath and keep a gentle light alive in the middle of this busy day\n[00:12]Carry on when you are ready")),
        2 => LyricData::NotFound,
        3 => LyricData::Plain("Soft rain against the window\nWe take shelter for a while\nThere is quiet in the spaces\nBetween the things still left to do".into()),
        4 => LyricData::Instrumental,
        _ => LyricData::Synced(lyrics::parse_lrc(DEMO_LYRICS)),
    }
}

fn play_button(ui: &mut egui::Ui, icon: Icon, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled_ui(enabled, |ui| {
        let (r, response) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::click());
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
        ui.painter().rect_filled(
            r,
            12,
            if response.hovered() {
                ACCENT.gamma_multiply(0.86)
            } else {
                ACCENT
            },
        );
        if response.has_focus() {
            ui.painter().rect_stroke(
                r.expand(2.0),
                14,
                Stroke::new(1.0_f32, TEXT),
                StrokeKind::Outside,
            );
        }
        icons::draw(ui.painter(), r.shrink(12.0), icon, BG);
        response.on_hover_text(label)
    })
    .inner
}

fn outlined_text(
    painter: &egui::Painter,
    position: egui::Pos2,
    align: Align2,
    text: &str,
    font: FontId,
    color: Color32,
    outline: bool,
) {
    if outline {
        for (x, y) in [
            (-1.0, -1.0),
            (0.0, -1.0),
            (1.0, -1.0),
            (-1.0, 0.0),
            (1.0, 0.0),
            (-1.0, 1.0),
            (0.0, 1.0),
            (1.0, 1.0),
        ] {
            painter.text(position + vec2(x, y), align, text, font.clone(), BG);
        }
    }
    painter.text(position, align, text, font, color);
}
