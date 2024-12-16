use eframe::egui::Color32;

// colors for options should be here also

/// Viewport
pub const COLOR_VIEWPORT_BACKGROUND: Color32 = Color32::from_rgb(25, 25, 25);
pub const COLOR_WIRE: Color32 = Color32::from_rgb(33, 33, 33);

// match target shape
pub const COLOR_CENTER_DOT: Color32 = Color32::from_rgb(255, 255, 255);
// red for x, green for y, slight blue hint for tilted variants
pub const COLOR_X_AXIS: Color32 = Color32::from_rgb(186, 31, 31);
pub const COLOR_Y_AXIS: Color32 = Color32::from_rgb(32, 117, 32);
pub const COLOR_TILTED_X_AXIS: Color32 = Color32::from_rgb(189, 17, 77);
pub const COLOR_TILTED_Y_AXIS: Color32 = Color32::from_rgb(0, 162, 67);

// orange
pub const COLOR_CONV_HULL: Color32 = Color32::from_rgb(162, 67, 10);
pub const COLOR_OUTER_CORNERS: Color32 = Color32::from_rgb(162, 67, 10);
// yellow
pub const COLOR_BOUNDS: Color32 = Color32::from_rgb(111, 101, 15);
pub const COLOR_MIRRORS: Color32 = Color32::from_rgb(205, 169, 43);

pub const COLOR_TARGET_SHAPE: Color32 = Color32::from_rgb(255, 255, 255);
pub const COLOR_SAMPLE_A: Color32 = Color32::from_rgb(200, 200, 200);
pub const COLOR_SAMPLE_B: Color32 = Color32::from_rgb(200, 200, 200);

/// Blocks (in viewport)
/// These should all have the same lightness as COLOR_BLOCKS (which is 73.3 in Gimp)
pub const COLOR_BLOCKS: Color32 = Color32::from_rgb(170, 170, 170);
// purple for boundary, blue for interior. hotter for 3d
pub const COLOR_BOUNDARY_2D: Color32 = Color32::from_rgb(220, 122, 246);
pub const COLOR_BOUNDARY_3D: Color32 = Color32::from_rgb(231, 122, 208);
pub const COLOR_INTERIOR_2D: Color32 = Color32::from_rgb(135, 156, 230);
pub const COLOR_INTERIOR_3D: Color32 = Color32::from_rgb(155, 139, 241);
// dark cyan
pub const COLOR_COMPLEMENT_2D: Color32 = Color32::from_rgb(27, 73, 72);
// yellow to match the bounds and mirrors
pub const COLOR_CENTER_BLOCKS: Color32 = Color32::from_rgb(204, 177, 82);

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
