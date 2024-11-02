use eframe::egui::Color32;

// Colors based on Blender Minimal Dark scheme, 3D Viewport
pub const COLOR_BACKGROUND: Color32 = Color32::from_rgb(28, 28, 28);
// middle background color (dark gray)
pub const COLOR_WIRE: Color32 = Color32::from_rgb(33, 33, 33);
// "Wire" color (gray)
pub const COLOR_FACE: Color32 = Color32::from_rgb(161, 163, 164);
// Face color (light gray)
pub const COLOR_LIME: Color32 = Color32::from_rgb(0, 255, 47);
// "Active object" color (lime)
pub const COLOR_DARK_GREEN: Color32 = Color32::from_rgb(6, 137, 30);
// Slightly decreased HSL saturation, decreased saturation from Lime
pub const COLOR_DARK_BLUE: Color32 = Color32::from_rgb(23, 143, 176);
pub const COLOR_LIGHT_BLUE: Color32 = Color32::from_rgb(0, 217, 255);
// "Object selected" color (light blue)
pub const COLOR_ORANGE: Color32 = Color32::from_rgb(255, 133, 0);
// "Grease Pencil Vertex Select" color (orange)
pub const COLOR_DARK_ORANGE: Color32 = Color32::from_rgb(204, 106, 0);
// Darker shade of orange
pub const COLOR_MUTED_ORANGE: Color32 = Color32::from_rgb(212, 148, 78);
// Darker shade of orange
pub const COLOR_PURPLE: Color32 = Color32::from_rgb(179, 104, 186);
// Dark purple
pub const COLOR_YELLOW: Color32 = Color32::from_rgb(255, 242, 0);
// "Edge Angle Text" color (yellow)
pub const COLOR_X_AXIS: Color32 = Color32::from_rgb(123, 34, 34);
// x-axis color (red)
pub const COLOR_Y_AXIS: Color32 = Color32::from_rgb(44, 107, 44);
// y-axis color (green)
pub const COLOR_RED: Color32 = Color32::from_rgb(255, 0, 0); //todo: make proper color

/// convex combination in RGB
pub fn linear_gradient(color_a: Color32, color_b: Color32, t: f64) -> Color32 {
    if t < 0.0 {
        color_a
    } else if t > 1.0 {
        color_b
    } else {
        Color32::from_rgb(
            ((1.0 - t) * color_a.r() as f64 + t * color_b.r() as f64) as u8,
            ((1.0 - t) * color_a.g() as f64 + t * color_b.g() as f64) as u8,
            ((1.0 - t) * color_a.b() as f64 + t * color_b.b() as f64) as u8,
        )
    }
}

pub fn bilinear_gradient(color_a: Color32, color_b: Color32, color_c: Color32, t: f64) -> Color32 {
    if t < 0.5 {
        linear_gradient(color_a, color_b, 2.0 * t)
    } else {
        linear_gradient(color_b, color_c, 2.0 * t - 1.0)
    }
}
