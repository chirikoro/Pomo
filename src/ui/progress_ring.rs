use eframe::egui::{Color32, Painter, Pos2, Stroke};
use std::f32::consts::{PI, TAU};

pub fn draw_progress_ring(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    thickness: f32,
    progress: f32,
    track_color: Color32,
    fill_color: Color32,
) {
    // Draw background track (full circle)
    painter.circle_stroke(center, radius, Stroke::new(thickness, track_color));

    if progress <= 0.0 {
        return;
    }

    // Draw progress arc
    let progress = progress.clamp(0.0, 1.0);
    let start_angle = -PI / 2.0; // 12 o'clock
    let sweep = TAU * progress;
    let num_points = ((progress * 120.0) as usize).max(2);

    let points: Vec<Pos2> = (0..=num_points)
        .map(|i| {
            let angle = start_angle + sweep * (i as f32 / num_points as f32);
            Pos2::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            )
        })
        .collect();

    // Draw the arc as a thick path
    let stroke = Stroke::new(thickness, fill_color);
    for i in 0..points.len().saturating_sub(1) {
        painter.line_segment([points[i], points[i + 1]], stroke);
    }

    // Draw rounded end caps
    if num_points > 0 {
        let cap_radius = thickness / 2.0;
        // Start cap
        painter.circle_filled(points[0], cap_radius, fill_color);
        // End cap
        if let Some(last) = points.last() {
            painter.circle_filled(*last, cap_radius, fill_color);

            // Glow effect at the leading edge
            let glow_color = Color32::from_rgba_unmultiplied(
                fill_color.r(),
                fill_color.g(),
                fill_color.b(),
                60,
            );
            painter.circle_filled(*last, cap_radius * 2.0, glow_color);
        }
    }
}
