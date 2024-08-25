use egui_plot::{Line, PlotBounds, PlotPoints, Polygon};
use std::f64::consts::PI;

pub fn square_at_coords(coord: [f64; 2]) -> Polygon {
    // Specifiying the bottom left coordinates of the square.
    let x = coord[0];
    let y = coord[1];

    let square_pts = PlotPoints::new(vec![
        [x + 0.0, y + 0.0],
        [x + 0.0, y + 1.0],
        [x + 1.0, y + 1.0],
        [x + 1.0, y + 0.0],
    ]);

    Polygon::new(square_pts).name("square".to_owned())
}

pub fn superellipse_at_coords(
    center_x: f64,
    center_y: f64,
    radius_a: f64,
    radius_b: f64,
    tilt: f64,
    squircle_parameter: f64,
) -> Line {
    let circlepts: PlotPoints = (0..=1005)
        // Near the square (squircle_parameter = Infinity) we get weird holes (the parameterization
        //  is not equally spaced), so need a few more points for it to make sense
        .map(|i| {
            let t = ((i as f64) * (2.0 * PI)) / 1000.0;
            let notilt = [
                radius_a * t.cos().abs().powf(2.0 / squircle_parameter) * t.cos().signum(),
                radius_b * t.sin().abs().powf(2.0 / squircle_parameter) * t.sin().signum(),
            ]; // the power is for squircles
            [
                center_x + notilt[0] * tilt.cos() + notilt[1] * tilt.sin(),
                center_y + notilt[0] * tilt.sin() - notilt[1] * tilt.cos(),
            ]
        })
        .collect();

    Line::new(circlepts)
}

/// Draw a tilted line through the origin in the given bounds
pub fn tilted_line_in_bounds(bnds: PlotBounds, tilt: f64, offset_x: f64, offset_y: f64) -> Line {
    let [min_x, min_y] = bnds.min();
    let [max_x, max_y] = bnds.max();

    let intersection_pts = vec![
        [min_x, (min_x - offset_x) * tilt.tan() + offset_y],
        [max_x, (max_x - offset_x) * tilt.tan() + offset_y],
        [(min_y - offset_y) / tilt.tan() + offset_x, min_y],
        [(max_y - offset_y) / tilt.tan() + offset_x, max_y],
    ];

    let intersection_pts_in_box = intersection_pts
        .into_iter()
        .filter(|point| {
            point[0] >= min_x && point[0] <= max_x && point[1] >= min_y && point[1] <= max_y
        })
        .collect();

    Line::new(PlotPoints::new(intersection_pts_in_box))
}
