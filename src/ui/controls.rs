use eframe::egui::{self, vec2, RichText, Ui};

use crate::audio::BgmMode;
use crate::theme::PomoTheme;
use crate::timer::{PomodoroTimer, TimerState};

pub enum ControlAction {
    StartPause,
    Reset,
    Skip,
    SetBgm(BgmMode),
    ToggleSettings,
}

pub fn draw_controls(
    ui: &mut Ui,
    timer: &PomodoroTimer,
    theme: &PomoTheme,
    current_bgm: BgmMode,
) -> Option<ControlAction> {
    let mut action = None;

    // Main control buttons
    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - 220.0) / 2.0);

        // Start / Pause button
        let (label, color) = match timer.state {
            TimerState::Running { .. } => (
                "  Pause  ",
                theme.phase_colors(&timer.phase).primary,
            ),
            TimerState::Finished => (
                "  Next  ",
                theme.phase_colors(&timer.phase).primary,
            ),
            _ => ("  Start  ", theme.phase_colors(&timer.phase).primary),
        };

        let btn = ui.add_sized(
            vec2(110.0, 40.0),
            egui::Button::new(
                RichText::new(label)
                    .size(16.0)
                    .color(egui::Color32::WHITE),
            )
            .fill(color)
            .corner_radius(20.0),
        );
        if btn.clicked() {
            action = Some(ControlAction::StartPause);
        }

        ui.add_space(8.0);

        // Reset button
        let reset_btn = ui.add_sized(
            vec2(90.0, 40.0),
            egui::Button::new(
                RichText::new("  Reset  ")
                    .size(14.0)
                    .color(theme.text_primary),
            )
            .corner_radius(20.0),
        );
        if reset_btn.clicked() {
            action = Some(ControlAction::Reset);
        }
    });

    ui.add_space(6.0);

    // Skip button (centered, smaller)
    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - 80.0) / 2.0);
        let skip_btn = ui.add_sized(
            vec2(80.0, 30.0),
            egui::Button::new(
                RichText::new("Skip")
                    .size(12.0)
                    .color(theme.text_secondary),
            )
            .corner_radius(15.0),
        );
        if skip_btn.clicked() {
            action = Some(ControlAction::Skip);
        }
    });

    ui.add_space(16.0);

    // BGM selector
    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - 200.0) / 2.0);
        ui.label(RichText::new("BGM:").size(12.0).color(theme.text_secondary));
        ui.add_space(4.0);

        for mode in BgmMode::all() {
            let is_selected = current_bgm == *mode;
            let label = mode.label();
            let text_color = if is_selected {
                egui::Color32::WHITE
            } else {
                theme.text_secondary
            };
            let bg = if is_selected {
                theme.phase_colors(&timer.phase).primary.linear_multiply(0.7)
            } else {
                theme.surface
            };

            let btn = ui.add_sized(
                vec2(56.0, 24.0),
                egui::Button::new(RichText::new(label).size(11.0).color(text_color))
                    .fill(bg)
                    .corner_radius(12.0),
            );
            if btn.clicked() {
                action = Some(ControlAction::SetBgm(*mode));
            }
        }
    });

    action
}
