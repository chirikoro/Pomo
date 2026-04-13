use tray_icon::Icon;

/// Load the icon from the embedded .ico file.
/// Falls back to a programmatically generated icon if decoding fails.
pub fn create_tray_icon() -> Icon {
    load_icon_from_file().unwrap_or_else(|| generate_fallback_icon())
}

/// Load icon as egui IconData for the window titlebar icon.
pub fn create_window_icon() -> eframe::egui::IconData {
    let ico_bytes = include_bytes!("../assets/icon.ico");
    if let Ok(img) = image::load_from_memory(ico_bytes) {
        let img = img.into_rgba8();
        let (w, h) = img.dimensions();
        eframe::egui::IconData {
            rgba: img.into_raw(),
            width: w,
            height: h,
        }
    } else {
        // Fallback: 32x32 coral circle
        let (rgba, size) = generate_fallback_rgba();
        eframe::egui::IconData {
            rgba,
            width: size,
            height: size,
        }
    }
}

fn load_icon_from_file() -> Option<Icon> {
    let ico_bytes = include_bytes!("../assets/icon.ico");
    let img = image::load_from_memory(ico_bytes).ok()?;
    let img = img.into_rgba8();
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.into_raw(), w, h).ok()
}

fn generate_fallback_icon() -> Icon {
    let (rgba, size) = generate_fallback_rgba();
    Icon::from_rgba(rgba, size, size).expect("Failed to create fallback tray icon")
}

fn generate_fallback_rgba() -> (Vec<u8>, u32) {
    let size: u32 = 32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0;
    let radius = center - 1.0;
    let (r, g, b) = (0xFF, 0x6B, 0x6B); // Coral

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center + 0.5;
            let dy = y as f32 - center + 0.5;
            let dist = (dx * dx + dy * dy).sqrt();

            let alpha = if dist <= radius - 0.5 {
                255
            } else if dist <= radius + 0.5 {
                ((radius + 0.5 - dist) * 255.0) as u8
            } else {
                0
            };

            let idx = ((y * size + x) * 4) as usize;
            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = alpha;
        }
    }

    (rgba, size)
}
