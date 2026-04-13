use eframe::egui::{self, Color32, CornerRadius, Stroke, Style, Visuals};

use crate::timer::Phase;

pub struct PhaseColors {
    pub primary: Color32,
    pub track: Color32,
}

pub struct PomoTheme {
    pub background: Color32,
    pub surface: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub button_idle: Color32,
    pub button_hover: Color32,
    pub button_text: Color32,
}

impl PomoTheme {
    pub fn new() -> Self {
        Self {
            background: Color32::from_rgb(0xFF, 0xF8, 0xF0),  // Warm White
            surface: Color32::from_rgb(0xFF, 0xF1, 0xE6),      // Soft Cream
            text_primary: Color32::from_rgb(0x2D, 0x34, 0x36),  // Charcoal
            text_secondary: Color32::from_rgb(0x88, 0x88, 0x88), // Warm Gray
            button_idle: Color32::from_rgb(0xE0, 0xD8, 0xD0),   // Soft Gray
            button_hover: Color32::from_rgb(0xFF, 0xD4, 0xB8),  // Warm Peach
            button_text: Color32::from_rgb(0x2D, 0x34, 0x36),   // Charcoal
        }
    }

    pub fn phase_colors(&self, phase: &Phase) -> PhaseColors {
        match phase {
            Phase::Work => PhaseColors {
                primary: Color32::from_rgb(0xFF, 0x6B, 0x6B), // Coral
                track: Color32::from_rgb(0xFF, 0xE0, 0xE0),   // Light Coral
            },
            Phase::ShortBreak => PhaseColors {
                primary: Color32::from_rgb(0x51, 0xD8, 0x8A), // Mint
                track: Color32::from_rgb(0xD4, 0xF5, 0xE0),   // Light Mint
            },
            Phase::LongBreak => PhaseColors {
                primary: Color32::from_rgb(0x9B, 0x8E, 0xC4), // Lavender
                track: Color32::from_rgb(0xE8, 0xE0, 0xF0),   // Light Lavender
            },
        }
    }

    pub fn apply_to_context(&self, ctx: &egui::Context) {
        let mut style = Style::default();
        let visuals = &mut style.visuals;

        *visuals = Visuals::light();
        visuals.panel_fill = self.background;
        visuals.window_fill = self.background;
        visuals.extreme_bg_color = self.surface;

        visuals.widgets.inactive.bg_fill = self.button_idle;
        visuals.widgets.inactive.weak_bg_fill = self.button_idle;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.button_text);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(20);

        visuals.widgets.hovered.bg_fill = self.button_hover;
        visuals.widgets.hovered.weak_bg_fill = self.button_hover;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.button_text);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(20);

        visuals.widgets.active.bg_fill = self.button_hover;
        visuals.widgets.active.weak_bg_fill = self.button_hover;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.button_text);
        visuals.widgets.active.corner_radius = CornerRadius::same(20);

        visuals.window_corner_radius = CornerRadius::same(12);

        ctx.set_style(style);
    }
}
