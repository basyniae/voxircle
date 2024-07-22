use crate::app::helpers::linear_algebra::{Mat2, Vec2};
use crate::app::helpers::optimization::minimize_maximum_straight_lines;

/// Return true if the closed line segments intersect, false otherwise.
/// (Is just a parameter wrapping of the intersect_lines function)
pub fn line_segments_intersect(line_one: [Vec2; 2], line_two: [Vec2; 2]) -> bool {
    let pure_intersect = intersect_lines(line_one, line_two);
    match pure_intersect {
        None => false,
        Some(params) => {
            let s = params[0];
            let t = params[1];
            (0.0 <= s) && (s <= 1.0) && (0.0 <= t) && (t <= 1.0)
        }
    }
}

/// Return true if the open ray from ray[0] through ray[1] intersects the closed line segment.
/// (Is just a parameter wrapping of the intersect_lines function)
pub fn ray_line_segment_intersect(ray: [Vec2; 2], line: [Vec2; 2]) -> bool {
    let pure_intersection = intersect_lines(ray, line);
    match pure_intersection {
        None => false,
        Some(params) => {
            let s = params[0];
            let t = params[1];
            (0.0 < s) && (0.0 <= t) && (t <= 1.0)
        }
    }
}

/// Return the pair of parameters for which the lines intersect if the lines are not parallel,
/// (as distance from the first to the second point)
/// None if the lines are parallel and have no intersection,
/// Any pair of parameters if the lines are parallel and have intersection, subject to the condition
/// that max {|t-1/2|, |s-1/2|} is minimal.
pub fn intersect_lines(line_one: [Vec2; 2], line_two: [Vec2; 2]) -> Option<[f64; 2]> {
    let p_1 = line_one[0];
    let d_1 = line_one[1] - line_one[0]; // End minus start for direction vector
    let p_2 = line_two[0];
    let d_2 = line_two[1] - line_two[0];

    // Consider the matrix with row vectors d_1, d_2, we want to invert this matrix
    // Compute the determinant
    let X = Mat2::from_columns(d_1, -1.0 * d_2);

    let det = X.det();

    if det == 0.0 {
        // i.e., the lines are parallel
        // Check if the lines coincide
        if d_1.y * (p_2.x - p_1.x) == d_1.x * (p_2.y - p_1.y) {
            // the lines coincide
            // Compute the values of s,t that define the other segment
            //TODO: what if d_2.x is zero?
            let s_start = (p_1.x - p_2.x) / d_2.x; // solves p_1 = p_2 + sd_2
            let s_end = (p_1.x + d_1.x - p_2.x) / d_2.x; // solves p_1 + d_1 = p_2 + sd_2
            let b = s_start;
            let a = s_end - s_start;
            // Now s = a*t + b

            // Pick the s,t if possible in the square [0,1]^2
            // I.e., minimize max {|t-1/2|, |s-1/2|} (maximum norm centered at 1/2)
            //  subject to s = a*t + b
            // Unfold:
            // minimize max {t-1/2, -t+1/2, at+b-1/2, -at-b+1/2} (t real)
            let [t, _] = minimize_maximum_straight_lines(vec![
                [1.0, -0.5],
                [-1.0, 0.5],
                [a, b - 0.5],
                [-a, -b + 0.5],
            ]);
            let s = a * t + b;

            Some([s, t])
        } else {
            None
        }
    } else {
        // Invert the matrix (by 2x2 matrix inverse formula)
        let Y = X.inverse().unwrap();
        let v = Y * (p_2 - p_1);

        Some([v.x, v.y])
    }
}
