use eframe::egui::{pos2, vec2, Color32, Painter, Rect, Shape, Stroke};

#[derive(Clone, Copy)]
pub enum Icon {
    Play,
    Pause,
    Previous,
    Next,
    Focus,
    System,
    Settings,
    Pin,
    Chevron,
    Close,
    Reset,
    Music,
    Battery,
}

pub fn draw(p: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let s = rect.width().min(rect.height());
    let origin = rect.center() - vec2(s, s) * 0.5;
    let pt = |x: f32, y: f32| origin + vec2(x, y) * s;
    let stroke = Stroke::new(1.65_f32, color);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([pt(a.0, a.1), pt(b.0, b.1)], stroke);
    };
    match icon {
        Icon::Play => {
            p.add(Shape::convex_polygon(
                vec![pt(0.32, 0.2), pt(0.8, 0.5), pt(0.32, 0.8)],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Pause => {
            for x in [0.3, 0.59] {
                p.rect_filled(
                    Rect::from_min_max(pt(x, 0.22), pt(x + 0.12, 0.78)),
                    1,
                    color,
                );
            }
        }
        Icon::Previous | Icon::Next => {
            let flip = matches!(icon, Icon::Previous);
            let x = |v: f32| if flip { 1.0 - v } else { v };
            p.add(Shape::convex_polygon(
                vec![pt(x(0.23), 0.23), pt(x(0.7), 0.5), pt(x(0.23), 0.77)],
                color,
                Stroke::NONE,
            ));
            line((x(0.78), 0.23), (x(0.78), 0.77));
        }
        Icon::Focus => {
            p.circle_stroke(pt(0.5, 0.55), s * 0.32, stroke);
            line((0.5, 0.55), (0.5, 0.34));
            line((0.4, 0.08), (0.6, 0.08));
            line((0.5, 0.08), (0.5, 0.18));
        }
        Icon::System => {
            p.rect_stroke(
                Rect::from_min_max(pt(0.25, 0.25), pt(0.75, 0.75)),
                2,
                stroke,
                eframe::egui::StrokeKind::Inside,
            );
            for v in [0.35, 0.65] {
                line((v, 0.08), (v, 0.25));
                line((v, 0.75), (v, 0.92));
                line((0.08, v), (0.25, v));
                line((0.75, v), (0.92, v));
            }
        }
        Icon::Settings => {
            for (y, knob) in [(0.26, 0.64), (0.5, 0.34), (0.74, 0.6)] {
                line((0.15, y), (0.85, y));
                p.circle_filled(pt(knob, y), s * 0.08, color);
            }
        }
        Icon::Pin => {
            line((0.35, 0.18), (0.65, 0.18));
            line((0.38, 0.18), (0.38, 0.46));
            line((0.62, 0.18), (0.62, 0.46));
            line((0.38, 0.46), (0.23, 0.65));
            line((0.62, 0.46), (0.77, 0.65));
            line((0.23, 0.65), (0.77, 0.65));
            line((0.5, 0.65), (0.5, 0.9));
        }
        Icon::Chevron => {
            line((0.25, 0.6), (0.5, 0.35));
            line((0.5, 0.35), (0.75, 0.6));
        }
        Icon::Close => {
            line((0.28, 0.28), (0.72, 0.72));
            line((0.72, 0.28), (0.28, 0.72));
        }
        Icon::Reset => {
            let points = (0..=24)
                .map(|i| {
                    let angle = -1.4 + i as f32 / 24.0 * 5.3;
                    pt(0.5 + angle.cos() * 0.31, 0.52 + angle.sin() * 0.31)
                })
                .collect();
            p.add(Shape::line(points, stroke));
            line((0.26, 0.19), (0.26, 0.4));
            line((0.26, 0.4), (0.46, 0.4));
        }
        Icon::Music => {
            line((0.4, 0.7), (0.4, 0.25));
            line((0.4, 0.25), (0.8, 0.17));
            line((0.8, 0.17), (0.8, 0.61));
            p.circle_filled(pt(0.29, 0.73), s * 0.115, color);
            p.circle_filled(pt(0.69, 0.64), s * 0.115, color);
        }
        Icon::Battery => {
            p.rect_stroke(
                Rect::from_min_max(pt(0.1, 0.3), pt(0.8, 0.7)),
                2,
                stroke,
                eframe::egui::StrokeKind::Inside,
            );
            line((0.9, 0.43), (0.9, 0.57));
            p.rect_filled(Rect::from_min_max(pt(0.2, 0.4), pt(0.62, 0.6)), 0, color);
        }
    }
}

pub fn logo(p: &Painter, center: eframe::egui::Pos2, color: Color32) {
    for (x, h) in [(-7.0, 6.0), (0.0, 14.0), (7.0, 9.0)] {
        p.line_segment(
            [
                pos2(center.x + x, center.y - h * 0.5),
                pos2(center.x + x, center.y + h * 0.5),
            ],
            Stroke::new(3.0_f32, color),
        );
    }
}
