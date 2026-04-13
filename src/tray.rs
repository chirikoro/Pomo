use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::icon::create_tray_icon;

pub struct TrayMenuIds {
    pub start_pause: MenuItem,
    pub reset: MenuItem,
    pub skip: MenuItem,
    pub show_hide: MenuItem,
    pub quit: MenuItem,
}

pub struct TrayHandle {
    pub menu_ids: TrayMenuIds,
    pub _tray_icon: TrayIcon,
}

pub fn setup_tray() -> TrayHandle {
    let icon = create_tray_icon();

    let start_pause = MenuItem::new("Start", true, None);
    let reset = MenuItem::new("Reset", true, None);
    let skip = MenuItem::new("Skip", true, None);
    let show_hide = MenuItem::new("Show/Hide", true, None);
    let quit = MenuItem::new("Quit", true, None);

    let menu = Menu::new();
    let _ = menu.append(&start_pause);
    let _ = menu.append(&reset);
    let _ = menu.append(&skip);
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&show_hide);
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&quit);

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Pomo - Pomodoro Timer")
        .with_icon(icon)
        .build()
        .expect("Failed to create tray icon");

    let menu_ids = TrayMenuIds {
        start_pause,
        reset,
        skip,
        show_hide,
        quit,
    };

    TrayHandle {
        menu_ids,
        _tray_icon: tray_icon,
    }
}

pub fn poll_tray_events() -> Option<MenuEvent> {
    MenuEvent::receiver().try_recv().ok()
}
