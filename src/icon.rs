use tray_icon::Icon;

pub fn create_tray_icon() -> Icon {
    let size: u32 = 32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0;
    let radius = center - 1.0;

    // Coral color (#FF6B6B)
    let (r, g, b) = (0xFF, 0x6B, 0x6B);

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

    Icon::from_rgba(rgba, size, size).expect("Failed to create tray icon")
}
