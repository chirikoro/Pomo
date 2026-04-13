use eframe::egui::{self, RichText, Ui, vec2};

use crate::theme::PomoTheme;
use crate::timer::PomodoroTimer;
use crate::ui::controls::ControlAction;

pub fn draw_header(
    ui: &mut Ui,
    timer: &PomodoroTimer,
    theme: &PomoTheme,
) -> Option<ControlAction> {
    let mut action = None;
    let phase_color = theme.phase_colors(&timer.phase).primary;

    ui.horizontal(|ui| {
        ui.add_space(16.0);
        ui.label(
            RichText::new(timer.phase.label())
                .size(18.0)
                .color(phase_color)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(16.0);
            let settings_btn = ui.add_sized(
                vec2(28.0, 28.0),
                egui::Button::new(
                    RichText::new("\u{2699}") // gear icon
                        .size(18.0)
                        .color(theme.text_secondary),
                )
                .frame(false),
            );
            if settings_btn.clicked() {
                action = Some(ControlAction::ToggleSettings);
            }
        });
    });

    action
}

pub fn draw_cycle_dots(
    ui: &mut Ui,
    timer: &PomodoroTimer,
    theme: &PomoTheme,
) {
    let num_sets = timer.num_sets;
    let current_cycle = timer.cycle;
    let phase_color = theme.phase_colors(&timer.phase).primary;
    let dot_size = 6.0;
    let spacing = 4.0;
    let total_width = num_sets as f32 * (dot_size * 2.0 + spacing) - spacing;

    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - total_width) / 2.0);

        for i in 1..=num_sets {
            let (response, painter) = ui.allocate_painter(
                vec2(dot_size * 2.0, dot_size * 2.0),
                egui::Sense::hover(),
            );
            let center = response.rect.center();

            if i <= current_cycle {
                painter.circle_filled(center, dot_size, phase_color);
            } else {
                painter.circle_stroke(
                    center,
                    dot_size,
                    egui::Stroke::new(1.5, theme.text_secondary),
                );
            }

            if i < num_sets {
                ui.add_space(spacing);
            }
        }
    });
}
