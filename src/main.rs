mod app;
mod audio;
mod icon;
mod notification;
mod settings;
mod theme;
mod timer;
mod tray;
mod ui;

use eframe::egui::ViewportBuilder;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([300.0, 460.0])
            .with_resizable(false)
            .with_title("Pomo"),
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "Pomo",
        options,
        Box::new(|cc| Ok(Box::new(app::PomoApp::new(cc)))),
    )
}
