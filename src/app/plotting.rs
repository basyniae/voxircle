use std::f64::consts::PI;

use crate::app::data_structures::squircle_params::SquircleParams;
use egui_plot::{Line, PlotBounds, PlotPoints, Polygon};

/// Specifiying the bottom left coordinates of the square.
pub fn square_at_coords(coord: [f64; 2]) -> Polygon<'static> {
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

pub fn superellipse_at_coords(slice_parameters: &SquircleParams) -> Line {
    let radius_a = slice_parameters.radius_a;
    let radius_b = slice_parameters.radius_b;
    let tilt = slice_parameters.tilt;
    let center_x = slice_parameters.center_offset_x;
    let center_y = slice_parameters.center_offset_y;
    let squircle_parameter = slice_parameters.squircle_parameter;

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
pub fn tilted_line_in_bounds(
    bnds: PlotBounds,
    tilt: f64,
    offset_x: f64,
    offset_y: f64,
) -> Line<'static> {
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

/// Plot a square given minimal and maximal points. Input: diagonally opposing corners of the square
pub fn bounds_from_square(bounds: [[f64; 2]; 2]) -> Line<'static> {
    let [[x_1, y_1], [x_2, y_2]] = bounds;

    Line::new(PlotPoints::new(vec![
        [x_1, y_1],
        [x_1, y_2],
        [x_2, y_2],
        [x_2, y_1],
        [x_1, y_1],
    ]))
}
