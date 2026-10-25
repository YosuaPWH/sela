#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui;
use sela_notch::{app::NotchApp, platform};

fn main() -> eframe::Result {
    let _instance = match platform::single_instance() {
        Ok(Some(instance)) => instance,
        Ok(None) => return Ok(()),
        Err(e) => {
            platform::startup_error(&format!("Cannot check for an existing instance: {e}"));
            return Ok(());
        }
    };
    let expanded = std::env::args().any(|a| a == "--expanded");
    let demo = std::env::args().any(|a| a == "--demo");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Sela")
            .with_inner_size([840.0, 440.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_taskbar(false)
            .with_resizable(false),
        centered: true,
        renderer: eframe::Renderer::Glow,
        vsync: true,
        ..Default::default()
    };
    eframe::run_native(
        "Sela",
        options,
        Box::new(move |cc| Ok(Box::new(NotchApp::new(cc, expanded, demo)))),
    )
    .inspect_err(|e| platform::startup_error(&e.to_string()))
}
