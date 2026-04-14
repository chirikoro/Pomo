use std::sync::Arc;
use std::time::Duration;

use eframe::egui::{self, vec2, CentralPanel, ViewportCommand};

use crate::audio::{AudioManager, BgmMode};
use crate::notification::notify_phase_complete;
use crate::settings::PomoSettings;
use crate::theme::PomoTheme;
use crate::timer::PomodoroTimer;
use crate::tray::{self, TrayHandle};
use crate::ui::controls::{self, ControlAction};
use crate::ui::progress_ring;
use crate::ui::settings_panel::{self, SettingsAction};
use crate::ui::status_bar;

pub struct PomoApp {
    timer: PomodoroTimer,
    theme: PomoTheme,
    settings: PomoSettings,
    edit_settings: PomoSettings,
    show_settings: bool,
    audio: Option<AudioManager>,
    tray_handle: Option<TrayHandle>,
    window_visible: bool,
}

impl PomoApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = PomoTheme::new();
        theme.apply_to_context(&cc.egui_ctx);

        // Setup custom fonts
        setup_fonts(&cc.egui_ctx);

        let settings = PomoSettings::default();
        let timer = PomodoroTimer::new(&settings);

        // Setup tray icon
        let tray_handle = Some(tray::setup_tray());

        // Setup audio
        let audio = AudioManager::new();

        Self {
            timer,
            theme,
            settings: settings.clone(),
            edit_settings: settings,
            show_settings: false,
            audio,
            tray_handle,
            window_visible: true,
        }
    }

    fn handle_control_action(&mut self, action: ControlAction) {
        match action {
            ControlAction::StartPause => {
                if self.timer.is_running() {
                    self.timer.pause();
                    if let Some(audio) = &mut self.audio {
                        audio.on_timer_pause();
                    }
                } else {
                    self.timer.start();
                    if let Some(audio) = &mut self.audio {
                        audio.on_timer_start(self.timer.phase);
                    }
                }
                self.update_tray_label();
            }
            ControlAction::Reset => {
                self.timer.reset();
                if let Some(audio) = &mut self.audio {
                    audio.on_timer_reset();
                }
                self.update_tray_label();
            }
            ControlAction::Skip => {
                self.timer.skip();
                if let Some(audio) = &mut self.audio {
                    audio.on_phase_change(self.timer.phase);
                }
                self.update_tray_label();
            }
            ControlAction::SetBgm(mode) => {
                if let Some(audio) = &mut self.audio {
                    audio.set_mode(mode);
                }
            }
            ControlAction::ToggleSettings => {
                self.show_settings = !self.show_settings;
                if self.show_settings {
                    self.edit_settings = self.settings.clone();
                }
            }
        }
    }

    fn update_tray_label(&self) {
        if let Some(handle) = &self.tray_handle {
            let label = if self.timer.is_running() {
                "Pause"
            } else {
                "Start"
            };
            handle.menu_ids.start_pause.set_text(label);
        }
    }

    fn poll_tray_events(&mut self, ctx: &egui::Context) {
        while let Some(event) = tray::poll_tray_events() {
            if let Some(handle) = &self.tray_handle {
                if event.id() == handle.menu_ids.start_pause.id() {
                    self.handle_control_action(ControlAction::StartPause);
                } else if event.id() == handle.menu_ids.reset.id() {
                    self.handle_control_action(ControlAction::Reset);
                } else if event.id() == handle.menu_ids.skip.id() {
                    self.handle_control_action(ControlAction::Skip);
                } else if event.id() == handle.menu_ids.show_hide.id() {
                    self.window_visible = !self.window_visible;
                    if self.window_visible {
                        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                        ctx.send_viewport_cmd(ViewportCommand::Focus);
                    } else {
                        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                    }
                } else if event.id() == handle.menu_ids.quit.id() {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            }
        }
    }
}

impl eframe::App for PomoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle close request -> hide to tray instead of quitting
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
            self.window_visible = false;
        }

        // Poll tray events
        self.poll_tray_events(ctx);

        // Tick timer
        let phase_before = self.timer.phase;
        if self.timer.tick() {
            // Phase just finished
            notify_phase_complete(&phase_before);
            self.timer.skip(); // Auto-advance to next phase
            // Switch BGM tracks for the new phase
            if let Some(audio) = &mut self.audio {
                audio.on_phase_change(self.timer.phase);
            }
            self.update_tray_label();
        }

        // Request repaint at appropriate rate
        if self.timer.is_running() {
            ctx.request_repaint_after(Duration::from_millis(100));
        } else {
            ctx.request_repaint_after(Duration::from_secs(1));
        }

        // Draw UI
        CentralPanel::default()
            .frame(egui::Frame::new().fill(self.theme.background))
            .show(ctx, |ui| {
                ui.add_space(16.0);

                if self.show_settings {
                    self.draw_settings_view(ui);
                } else {
                    self.draw_timer_view(ui);
                }
            });
    }
}

impl PomoApp {
    fn draw_timer_view(&mut self, ui: &mut egui::Ui) {
        // Header with phase label and settings gear
        if let Some(action) = status_bar::draw_header(ui, &self.timer, &self.theme) {
            self.handle_control_action(action);
        }

        ui.add_space(16.0);

        // Progress ring
        let ring_size = 200.0;
        let available_width = ui.available_width();
        ui.horizontal(|ui| {
            ui.add_space((available_width - ring_size) / 2.0);
            let (response, painter) = ui.allocate_painter(
                vec2(ring_size, ring_size),
                egui::Sense::hover(),
            );
            let center = response.rect.center();
            let radius = ring_size / 2.0 - 16.0;
            let thickness = 10.0;
            let colors = self.theme.phase_colors(&self.timer.phase);

            progress_ring::draw_progress_ring(
                &painter,
                center,
                radius,
                thickness,
                self.timer.progress(),
                colors.track,
                colors.primary,
            );

            // Countdown text in center
            let remaining = self.timer.remaining();
            let mins = remaining.as_secs() / 60;
            let secs = remaining.as_secs() % 60;
            let time_text = format!("{:02}:{:02}", mins, secs);

            painter.text(
                center + vec2(0.0, -8.0),
                egui::Align2::CENTER_CENTER,
                &time_text,
                egui::FontId::new(42.0, egui::FontFamily::Proportional),
                self.theme.text_primary,
            );

            // Session info
            let session_text = format!("Session {}/{}", self.timer.cycle, self.timer.num_sets);
            painter.text(
                center + vec2(0.0, 22.0),
                egui::Align2::CENTER_CENTER,
                &session_text,
                egui::FontId::new(13.0, egui::FontFamily::Proportional),
                self.theme.text_secondary,
            );
        });

        ui.add_space(20.0);

        // Control buttons
        let current_bgm = self
            .audio
            .as_ref()
            .map(|a| a.current_mode())
            .unwrap_or(BgmMode::Off);
        if let Some(action) = controls::draw_controls(ui, &self.timer, &self.theme, current_bgm) {
            self.handle_control_action(action);
        }

        ui.add_space(12.0);

        // Cycle dots
        status_bar::draw_cycle_dots(ui, &self.timer, &self.theme);
    }

    fn draw_settings_view(&mut self, ui: &mut egui::Ui) {
        match settings_panel::draw_settings_panel(
            ui,
            &mut self.edit_settings,
            &self.theme,
        ) {
            Some(SettingsAction::Close) => {
                self.show_settings = false;
            }
            Some(SettingsAction::Save(new_settings)) => {
                self.settings = new_settings;
                if self.timer.is_idle() {
                    self.timer.apply_settings(&self.settings);
                    self.timer.full_reset();
                }
                self.show_settings = false;
            }
            None => {}
        }
    }
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "NotoSans-Bold".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/NotoSans-Bold.ttf"
        ))),
    );

    // Use NotoSans-Bold as the primary proportional font
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "NotoSans-Bold".into());

    ctx.set_fonts(fonts);
}
