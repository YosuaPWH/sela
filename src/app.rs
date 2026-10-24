use crate::{
    config::{self, Settings},
    focus::{FocusTimer, Phase},
    icons::{self, Icon},
    platform::{self, Action, Media, MediaBridge, MediaCommand},
    telemetry::{self, Snapshot},
};
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Rect, RichText,
    Sense, Stroke, StrokeKind, Vec2,
};
use std::{
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant},
};

const BG: Color32 = Color32::from_rgb(9, 11, 10);
const SURFACE: Color32 = Color32::from_rgb(32, 38, 34);
const TEXT: Color32 = Color32::from_rgb(243, 246, 244);
const MUTED: Color32 = Color32::from_rgb(169, 180, 173);
const ACCENT: Color32 = Color32::from_rgb(169, 238, 192);
const ERROR: Color32 = Color32::from_rgb(255, 180, 171);
const COMPACT: Vec2 = vec2(324.0, 48.0);
const EXPANDED: Vec2 = vec2(464.0, 370.0);

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Media,
    Focus,
    System,
    Settings,
}

pub struct NotchApp {
    settings: Settings,
    settings_path: Option<PathBuf>,
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
    system_rx: mpsc::Receiver<Snapshot>,
    bridge: MediaBridge,
    media: Media,
    integration: Option<platform::DesktopIntegration>,
    last_geometry: Option<(Vec2, f32, f32, egui::Pos2, f32)>,
}

impl NotchApp {
    pub fn new(cc: &eframe::CreationContext<'_>, expanded: bool) -> Self {
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
        let settings = settings_path
            .as_ref()
            .map(|p| {
                Settings::load(p).unwrap_or_else(|e| {
                    error = Some(e);
                    Settings::default()
                })
            })
            .unwrap_or_default();
        let integration = match platform::DesktopIntegration::new() {
            Ok(integration) => {
                if let Some(warning) = integration.warning() {
                    error = Some(warning.to_owned());
                }
                Some(integration)
            }
            Err(e) => {
                error = Some(format!("Tray tidak tersedia: {e}. Buka Atur untuk keluar."));
                None
            }
        };
        Self {
            timer: FocusTimer::new(settings.focus_minutes),
            settings,
            settings_path,
            error,
            expanded,
            pinned: false,
            hover_opened: false,
            pointer_left: None,
            hover_started: None,
            suppress_hover: false,
            tab: Tab::Media,
            last_tick: Instant::now(),
            snapshot: Snapshot::default(),
            system_rx: telemetry::spawn(),
            bridge: platform::media_bridge(),
            media: Media::default(),
            integration,
            last_geometry: None,
        }
    }

    fn open(&mut self, ctx: &egui::Context) {
        self.expanded = true;
        self.hover_opened = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    fn collapse(&mut self) {
        self.expanded = false;
        self.pinned = false;
        self.suppress_hover = true;
        self.hover_started = None;
    }

    fn persist(&mut self) {
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

    fn compact(&mut self, ui: &mut egui::Ui, width: f32) {
        let row = Rect::from_min_size(pos2(20.0, 0.0), vec2(width - 40.0, 48.0));
        icons::logo(ui.painter(), pos2(29.0, 24.0), ACCENT);
        let timer_active = self.timer.phase == Phase::Running || self.timer.phase == Phase::Paused;
        let label = if timer_active {
            format!("Fokus  {}", self.timer.label())
        } else if !self.media.title.is_empty() {
            self.media.title.clone()
        } else {
            format!("Luma   {}", chrono::Local::now().format("%H:%M"))
        };
        let text_rect = Rect::from_min_max(pos2(52.0, 12.0), pos2(width - 72.0, 37.0));
        let galley =
            ui.painter()
                .layout(label, FontId::proportional(13.0), TEXT, text_rect.width());
        ui.painter()
            .with_clip_rect(text_rect)
            .galley(text_rect.min + vec2(0.0, 4.0), galley, TEXT);
        let trailing = self
            .snapshot
            .battery
            .as_ref()
            .map(|b| format!("{}%", b.percent))
            .unwrap_or_else(|| "•••".into());
        ui.painter().text(
            pos2(row.right() - 4.0, 24.0),
            Align2::RIGHT_CENTER,
            trailing,
            FontId::proportional(11.0),
            MUTED,
        );
        let response = ui.interact(
            Rect::from_min_size(egui::Pos2::ZERO, vec2(width, 48.0)),
            Id::new("compact"),
            Sense::click(),
        );
        response
            .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Buka Luma"));
        if response.clicked() {
            self.open(ui.ctx());
        }
        response.on_hover_text("Buka Luma · Ctrl+Alt+N");
    }

    fn expanded_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(22.0, 24.0), Sense::hover());
            icons::logo(ui.painter(), r.center(), ACCENT);
            ui.label(RichText::new("Luma").size(17.0).strong());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_button(ui, Icon::Chevron, "Tutup panel · Esc", false, true).clicked() {
                    self.collapse();
                }
                if icon_button(
                    ui,
                    Icon::Pin,
                    if self.pinned {
                        "Lepas pin panel"
                    } else {
                        "Biarkan panel terbuka"
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
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for (tab, icon, text) in [
                (Tab::Media, Icon::Music, "Media"),
                (Tab::Focus, Icon::Focus, "Fokus"),
                (Tab::System, Icon::System, "Sistem"),
                (Tab::Settings, Icon::Settings, "Atur"),
            ] {
                let active = self.tab == tab;
                let (r, response) = ui.allocate_exact_size(vec2(99.0, 34.0), Sense::click());
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
                    self.hover_opened = false;
                }
                response.on_hover_text(format!("Buka {text}"));
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
                RichText::new("Media belum dapat diakses.")
                    .size(20.0)
                    .color(ERROR),
            );
            ui.label("Pastikan pemutar aktif, lalu coba lagi.");
            ui.add(egui::Label::new(RichText::new(&error).size(12.0).color(MUTED)).truncate())
                .on_hover_text(&error);
            if ui.button("Coba lagi").clicked() {
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
                ui.label(RichText::new("Ruang untuk musikmu.").size(21.0).strong());
                ui.add_space(6.0);
                ui.label(RichText::new("Putar musik di Spotify, browser, atau pemutar\nyang mendukung kontrol media Windows.").size(13.0).color(MUTED));
                if !cfg!(windows) {
                    ui.add_space(10.0);
                    ui.colored_label(MUTED, "Pratinjau Linux · integrasi media tersedia di Windows");
                }
            });
            return;
        }
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(58.0, 58.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 28.0, SURFACE);
            ui.painter().circle_stroke(
                r.center(),
                19.0,
                Stroke::new(1.0_f32, MUTED.gamma_multiply(0.3)),
            );
            ui.painter().circle_filled(r.center(), 6.0, ACCENT);
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.add(
                    egui::Label::new(RichText::new(&self.media.title).size(19.0).strong())
                        .truncate(),
                );
                ui.add(
                    egui::Label::new(
                        RichText::new(if self.media.artist.is_empty() {
                            "Artis tidak tersedia"
                        } else {
                            &self.media.artist
                        })
                        .size(13.0)
                        .color(MUTED),
                    )
                    .truncate(),
                );
                ui.label(
                    RichText::new(if self.media.playing {
                        "Sedang diputar"
                    } else {
                        "Dijeda"
                    })
                    .size(11.0)
                    .color(ACCENT),
                );
            });
        });
        ui.add_space(22.0);
        let fraction = if self.media.duration > 0.0 {
            (self.media.position / self.media.duration).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };
        progress(ui, fraction, ACCENT);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(media_time(self.media.position))
                    .size(11.0)
                    .color(MUTED)
                    .monospace(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(media_time(self.media.duration))
                        .size(11.0)
                        .color(MUTED)
                        .monospace(),
                );
            });
        });
        ui.add_space(7.0);
        ui.horizontal(|ui| {
            ui.add_space(126.0);
            for (icon, label, command, enabled) in [
                (
                    Icon::Previous,
                    "Lagu sebelumnya",
                    MediaCommand::Previous,
                    self.media.can_previous,
                ),
                (
                    if self.media.playing {
                        Icon::Pause
                    } else {
                        Icon::Play
                    },
                    if self.media.playing { "Jeda" } else { "Putar" },
                    MediaCommand::Toggle,
                    self.media.can_toggle,
                ),
                (
                    Icon::Next,
                    "Lagu berikutnya",
                    MediaCommand::Next,
                    self.media.can_next,
                ),
            ] {
                if icon_button(
                    ui,
                    icon,
                    label,
                    matches!(command, MediaCommand::Toggle),
                    enabled,
                )
                .clicked()
                    && self.bridge.commands.send(command).is_err()
                {
                    self.error = Some("Layanan media berhenti. Mulai ulang Luma.".into());
                }
            }
        });
    }

    fn focus_ui(&mut self, ui: &mut egui::Ui) {
        let (caption, color) = match self.timer.phase {
            Phase::Ready => ("Satu hal. Sepenuh perhatian.", MUTED),
            Phase::Running => ("Waktunya fokus. Kamu bisa.", ACCENT),
            Phase::Paused => ("Dijeda. Lanjutkan saat siap.", MUTED),
            Phase::Complete => ("Sesi selesai. Ambil jeda sejenak.", ACCENT),
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
                            RichText::new(format!("{minutes} m")).color(if selected {
                                ACCENT
                            } else {
                                MUTED
                            }),
                        )
                        .fill(if selected { SURFACE } else { BG }),
                    )
                    .on_hover_text("Pilih durasi sesi berikutnya")
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
                Phase::Running => "Jeda fokus",
                Phase::Paused => "Lanjutkan",
                _ => "Mulai fokus",
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
                self.hover_opened = false;
            }
            if icon_button(ui, Icon::Reset, "Reset timer", false, true).clicked() {
                self.timer.reset(self.settings.focus_minutes);
            }
        });
    }

    fn system_ui(&mut self, ui: &mut egui::Ui) {
        metric(
            ui,
            "Prosesor",
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
            "Memori",
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
                    "Baterai · tersambung listrik"
                } else {
                    "Baterai"
                },
                &format!("{}%", battery.percent),
                f32::from(battery.percent) / 100.0,
            );
        } else {
            ui.horizontal(|ui| {
                ui.colored_label(MUTED, "Baterai");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.colored_label(MUTED, "Tidak tersedia");
                });
            });
        }
        ui.add_space(18.0);
        ui.label(
            RichText::new("Diperbarui setiap 2 detik · data perangkat ini")
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
                        "Buka saat kursor menyentuh notch",
                    )
                    .changed();
                changed |= ui
                    .checkbox(&mut self.settings.reduced_motion, "Kurangi animasi")
                    .changed();
                changed |= ui
                    .checkbox(&mut self.settings.sound, "Bunyi saat sesi fokus selesai")
                    .changed();
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Jarak dari atas");
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
                    RichText::new("Ctrl+Alt+N  buka / tutup  ·  Esc  tutup")
                        .size(11.0)
                        .color(MUTED),
                );
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Luma Notch 0.1.0").size(11.0).color(MUTED));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("Keluar dari Luma").clicked() {
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

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;
        if self.timer.advance(elapsed) {
            self.tab = Tab::Focus;
            self.expanded = true;
            self.hover_opened = false;
            if self.settings.sound {
                platform::completion_sound();
            }
        }
        while let Ok(snapshot) = self.system_rx.try_recv() {
            self.snapshot = snapshot;
        }
        while let Ok(media) = self.bridge.updates.try_recv() {
            self.media = media;
        }
        if let Some(integration) = &self.integration {
            if let Some(action) = integration.poll() {
                match action {
                    Action::Toggle => {
                        if self.expanded {
                            self.collapse();
                        } else {
                            self.open(ctx);
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
            self.open(ctx);
        }
        let pointer_inside = ctx.input(|i| {
            i.pointer
                .hover_pos()
                .is_some_and(|p| i.screen_rect().contains(p))
        });
        if pointer_inside {
            self.pointer_left = None;
            if !self.expanded && self.settings.expand_on_hover && !self.suppress_hover {
                let since = self.hover_started.get_or_insert(now);
                if now.duration_since(*since) >= Duration::from_millis(240) {
                    self.expanded = true;
                    self.hover_opened = true;
                }
            }
        } else {
            self.hover_started = None;
            self.suppress_hover = false;
            if self.expanded && self.hover_opened && !self.pinned {
                let since = self.pointer_left.get_or_insert(now);
                if now.duration_since(*since) >= Duration::from_millis(650) {
                    self.expanded = false;
                }
            }
        }
        let t = if self.settings.reduced_motion {
            if self.expanded {
                1.0
            } else {
                0.0
            }
        } else {
            ctx.animate_bool_with_time(Id::new("expansion"), self.expanded, 0.24)
        };
        let eased = 1.0 - (1.0 - t).powi(3);
        let size = COMPACT + (EXPANDED - COMPACT) * eased;
        let size = vec2(size.x.round(), size.y.round());
        self.geometry(ctx, size);
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let shape = Rect::from_min_size(egui::Pos2::ZERO, size);
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
                ui.painter().rect_filled(shape, radius, BG);
                if t < 0.98 {
                    self.compact(ui, size.x);
                } else {
                    let content = shape.shrink2(vec2(24.0, 18.0));
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(content)
                            .layout(Layout::top_down(Align::Min)),
                    );
                    self.expanded_ui(&mut child);
                    let footer = if self.error.is_some() {
                        "Perlu perhatian · lihat Atur".to_owned()
                    } else if self.timer.completed > 0 {
                        format!("{} sesi fokus selesai", self.timer.completed)
                    } else {
                        "Sedikit ruang. Lebih banyak fokus.".into()
                    };
                    ui.painter().text(
                        pos2(24.0, size.y - 18.0),
                        Align2::LEFT_CENTER,
                        footer,
                        FontId::proportional(10.0),
                        if self.error.is_some() { ERROR } else { MUTED },
                    );
                    ui.painter()
                        .circle_filled(pos2(size.x - 26.0, size.y - 18.0), 2.5, ACCENT);
                }
            });
        ctx.request_repaint_after(Duration::from_millis(
            if self.timer.phase == Phase::Running
                || self.hover_started.is_some()
                || self.pointer_left.is_some()
            {
                80
            } else {
                250
            },
        ));
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
