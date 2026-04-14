mod settings;
mod timer;

use std::sync::Mutex;

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconEvent,
};

use settings::PomoSettings;
use timer::{Phase, PomodoroTimer, TimerState};

struct AppState {
    timer: Mutex<PomodoroTimer>,
    settings: Mutex<PomoSettings>,
}

#[derive(Clone, Serialize)]
struct TimerStatus {
    phase: String,
    state: String,
    remaining_secs: u64,
    progress: f32,
    cycle: u32,
    num_sets: u32,
    is_running: bool,
    is_idle: bool,
    phase_finished: bool,
}

fn get_timer_status(timer: &mut PomodoroTimer) -> TimerStatus {
    let finished = timer.tick();
    TimerStatus {
        phase: match timer.phase {
            Phase::Work => "work".into(),
            Phase::ShortBreak => "short_break".into(),
            Phase::LongBreak => "long_break".into(),
        },
        state: match timer.state {
            TimerState::Idle => "idle".into(),
            TimerState::Running { .. } => "running".into(),
            TimerState::Paused { .. } => "paused".into(),
            TimerState::Finished => "finished".into(),
        },
        remaining_secs: timer.remaining().as_secs(),
        progress: timer.progress(),
        cycle: timer.cycle,
        num_sets: timer.num_sets,
        is_running: timer.is_running(),
        is_idle: timer.is_idle(),
        phase_finished: finished,
    }
}

#[tauri::command]
fn tick(state: tauri::State<'_, AppState>) -> TimerStatus {
    let mut timer = state.timer.lock().unwrap();
    get_timer_status(&mut timer)
}

#[tauri::command]
fn start(state: tauri::State<'_, AppState>) -> TimerStatus {
    let mut timer = state.timer.lock().unwrap();
    timer.start();
    get_timer_status(&mut timer)
}

#[tauri::command]
fn pause(state: tauri::State<'_, AppState>) -> TimerStatus {
    let mut timer = state.timer.lock().unwrap();
    timer.pause();
    get_timer_status(&mut timer)
}

#[tauri::command]
fn reset(state: tauri::State<'_, AppState>) -> TimerStatus {
    let mut timer = state.timer.lock().unwrap();
    timer.reset();
    get_timer_status(&mut timer)
}

#[tauri::command]
fn skip(state: tauri::State<'_, AppState>) -> TimerStatus {
    let mut timer = state.timer.lock().unwrap();
    timer.skip();
    get_timer_status(&mut timer)
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> PomoSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn save_settings(
    new_settings: PomoSettings,
    state: tauri::State<'_, AppState>,
) -> TimerStatus {
    let mut settings = state.settings.lock().unwrap();
    *settings = new_settings;
    settings.clamp();

    let mut timer = state.timer.lock().unwrap();
    if timer.is_idle() {
        timer.apply_settings(&settings);
        timer.full_reset();
    }
    get_timer_status(&mut timer)
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub fn run() {
    let settings = PomoSettings::default();
    let timer = PomodoroTimer::new(&settings);

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(AppState {
            timer: Mutex::new(timer),
            settings: Mutex::new(settings),
        })
        .setup(|app| {
            // Build tray menu
            let show_hide = MenuItemBuilder::with_id("show_hide", "Show/Hide").build(app)?;
            let start_pause = MenuItemBuilder::with_id("start_pause", "Start").build(app)?;
            let reset_item = MenuItemBuilder::with_id("reset", "Reset").build(app)?;
            let skip_item = MenuItemBuilder::with_id("skip", "Skip").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_hide)
                .separator()
                .item(&start_pause)
                .item(&reset_item)
                .item(&skip_item)
                .separator()
                .item(&quit)
                .build()?;

            if let Some(tray) = app.tray_by_id("pomo-tray") {
                tray.set_menu(Some(menu))?;
                tray.on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "show_hide" => {
                            toggle_window(app);
                        }
                        "start_pause" => {
                            let state = app.state::<AppState>();
                            let mut timer = state.timer.lock().unwrap();
                            if timer.is_running() {
                                timer.pause();
                            } else {
                                timer.start();
                            }
                            let status = get_timer_status(&mut timer);
                            let _ = app.emit("timer-update", &status);
                        }
                        "reset" => {
                            let state = app.state::<AppState>();
                            let mut timer = state.timer.lock().unwrap();
                            timer.reset();
                            let status = get_timer_status(&mut timer);
                            let _ = app.emit("timer-update", &status);
                        }
                        "skip" => {
                            let state = app.state::<AppState>();
                            let mut timer = state.timer.lock().unwrap();
                            timer.skip();
                            let status = get_timer_status(&mut timer);
                            let _ = app.emit("timer-update", &status);
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                });
                tray.on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::DoubleClick { .. } = event {
                        toggle_window(tray.app_handle());
                    }
                });
            }

            // Hide to tray on close instead of quitting
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tick, start, pause, reset, skip, get_settings, save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
