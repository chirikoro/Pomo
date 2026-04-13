use eframe::egui::{self, RichText, Ui, vec2};

use crate::settings::PomoSettings;
use crate::theme::PomoTheme;

pub enum SettingsAction {
    Close,
    Save(PomoSettings),
}

pub fn draw_settings_panel(
    ui: &mut Ui,
    settings: &mut PomoSettings,
    theme: &PomoTheme,
) -> Option<SettingsAction> {
    let mut action = None;

    // Header
    ui.horizontal(|ui| {
        ui.add_space(16.0);
        ui.label(
            RichText::new("Settings")
                .size(18.0)
                .color(theme.text_primary)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(16.0);
            let close_btn = ui.add_sized(
                vec2(28.0, 28.0),
                egui::Button::new(
                    RichText::new("\u{2715}") // X mark
                        .size(16.0)
                        .color(theme.text_secondary),
                )
                .frame(false),
            );
            if close_btn.clicked() {
                action = Some(SettingsAction::Close);
            }
        });
    });

    ui.add_space(16.0);

    // Settings fields
    let label_width = 100.0;
    let slider_width = 140.0;

    ui.horizontal(|ui| {
        ui.add_space(24.0);
        ui.allocate_ui(vec2(label_width, 20.0), |ui| {
            ui.label(RichText::new("Work:").size(14.0).color(theme.text_primary));
        });
        let mut val = settings.work_minutes as f32;
        ui.add_sized(
            vec2(slider_width, 20.0),
            egui::Slider::new(&mut val, 1.0..=60.0)
                .suffix(" min")
                .integer(),
        );
        settings.work_minutes = val as u32;
    });

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.add_space(24.0);
        ui.allocate_ui(vec2(label_width, 20.0), |ui| {
            ui.label(RichText::new("Break:").size(14.0).color(theme.text_primary));
        });
        let mut val = settings.short_break_minutes as f32;
        ui.add_sized(
            vec2(slider_width, 20.0),
            egui::Slider::new(&mut val, 1.0..=30.0)
                .suffix(" min")
                .integer(),
        );
        settings.short_break_minutes = val as u32;
    });

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.add_space(24.0);
        ui.allocate_ui(vec2(label_width, 20.0), |ui| {
            ui.label(
                RichText::new("Long Break:")
                    .size(14.0)
                    .color(theme.text_primary),
            );
        });
        let mut val = settings.long_break_minutes as f32;
        ui.add_sized(
            vec2(slider_width, 20.0),
            egui::Slider::new(&mut val, 1.0..=60.0)
                .suffix(" min")
                .integer(),
        );
        settings.long_break_minutes = val as u32;
    });

    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.add_space(24.0);
        ui.allocate_ui(vec2(label_width, 20.0), |ui| {
            ui.label(RichText::new("Sets:").size(14.0).color(theme.text_primary));
        });
        let mut val = settings.num_sets as f32;
        ui.add_sized(
            vec2(slider_width, 20.0),
            egui::Slider::new(&mut val, 1.0..=10.0).integer(),
        );
        settings.num_sets = val as u32;
    });

    ui.add_space(20.0);

    // Save button
    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - 100.0) / 2.0);
        let save_btn = ui.add_sized(
            vec2(100.0, 36.0),
            egui::Button::new(
                RichText::new("Save")
                    .size(15.0)
                    .color(egui::Color32::WHITE),
            )
            .fill(egui::Color32::from_rgb(0xFF, 0x6B, 0x6B))
            .corner_radius(18.0),
        );
        if save_btn.clicked() {
            settings.clamp();
            action = Some(SettingsAction::Save(settings.clone()));
        }
    });

    action
}
