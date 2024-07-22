use crate::app::helpers::blocks::Blocks;
use crate::app::helpers::circle_geometry::{area_of_semicircle_section, intersection_hline_circle};
use crate::app::helpers::linear_algebra::Vec2;
use std::f64::consts::PI;

// logic + geometry + integration
// Percentage is to be supplied as float between 0 and 1 (note that unexpected behaviour may occur if it 0.0 or 1.0 exactly due to numerical errors)
pub fn generate_alg_percentage(radius: f64, center_offset: Vec2, percentage: f64) -> Blocks {
    let edge_length = ((2.0 * radius).ceil() as usize) + 4; // the 4 is needed as a buffer..
                                                            // i think we're able to get away with less but it doesn't matter. Buffer is required to make the interior work as expected
    let origin = Vec2::from([(edge_length / 2) as f64, (edge_length / 2) as f64]);
    // in bitmatrix coordinates, where is the center of the grid?
    let mut output_vec = Vec::new();

    for i in 0..edge_length.pow(2) {
        // loop over all coords

        // Bottom right coordinate of the box in bitmatrix coordinates is [i % edge_length, i / edge_length], so to get the center we add 0.5.
        let mut x_center = ((i % edge_length) as f64) + 0.5 - (origin.x + center_offset.x); // Relative to the circle center, what is the x-position of the center of the box?
        let mut y_center = ((i / edge_length) as f64) + 0.5 - (origin.y + center_offset.y); // "" "" what is the y-position of the center of the box

        // Symmetrize. We may assume that the origin is to the bottom left of the box center, and under the bottom-left to top-right diagonal
        //  by dihedral symmetry of the square.
        // Formulaically: x_center >= 0, y_center >= 0, y_center >= x_center
        // Compute this via an if tree casing on signs
        if x_center < 0.0 {
            x_center = -x_center;
        }
        if y_center < 0.0 {
            y_center = -y_center;
        }
        if x_center > y_center {
            (y_center, x_center) = (x_center, y_center);
        }

        // Then compute the area under the assumptions on the location above and compare to the desired percentage
        output_vec.push(cell_disk_intersection_area(radius, x_center, y_center) >= percentage);
    }

    Blocks {
        blocks: output_vec,
        edge_length,
        origin,
    }
}

pub fn cell_disk_intersection_area(radius: f64, x_center: f64, y_center: f64) -> f64 {
    let area: f64; // We compute this area and see if its less than the percentage parameter

    // Get the coordinates of the sides of the box
    let x_left = x_center - 0.5; // what is the x-coordinate of the left boundary of the box?
    let x_right = x_center + 0.5; // what is the x-coordinate of the right boundary of the box?
    let y_bottom = y_center - 0.5; // etc.
    let y_top = y_center + 0.5;

    // Symmetrization makes it so that
    // x_left <= 0.5, x_right <= -0.5,  y_bottom <= 0.5, y_top <= -0.5,
    //  as well as the origin-under-diagonal condition (which is also some sort of linear inequality)
    //  x_left <= y_bottom, x_right <= y_top

    // Precompute which corner points are in the the disk (we condition on these a bunch)
    // write L for left, R for right, B for bottom, T for top.
    let rt = x_right.powi(2) + y_top.powi(2) <= radius.powi(2);
    let lt = x_left.powi(2) + y_top.powi(2) <= radius.powi(2);
    let rb = x_right.powi(2) + y_bottom.powi(2) <= radius.powi(2);
    let lb = x_left.powi(2) + y_bottom.powi(2) <= radius.powi(2);

    // Due to symmetrization we get LB => RB => LT => RT (logical implies)
    // Example: if we know LT is in the disk, then automatically also RB and LB are in the disk

    if x_left.powi(2) + y_bottom.powi(2) > (radius + 0.5).powi(2)
        && x_right.powi(2) + y_bottom.powi(2) > (radius + 0.5).powi(2)
        && x_left.powi(2) + y_top.powi(2) > (radius + 0.5).powi(2)
        && x_right.powi(2) + y_top.powi(2) > (radius + 0.5).powi(2)
    {
        // If every corner point is more than radius + 0.5 away from the center, then the area of the intersection is zero
        // Worst case is for very small circle with center on the midpoint of an edge of the box.
        area = 0.0;
        // These two statements should catch both the bulk of the interior and the bulk of the exterior
        // we can use more expensive functions for the remaining O(radius) cases (linear since the it scales as a boundary)
    } else {
        match (lb as u8) + (rb as u8) + (lt as u8) + (rt as u8) {
            4 => {
                // Recall from contained.rs that if all cornerpoints are contained in the disk, then
                //  then the entire box is in the disk. So the overlap area is 1 (the entire box).
                area = 1.0;
            }
            3 => {
                // Missing one point. By symmetry, it must be the top right
                // Find intersection of top edge with the circle.
                let x_intercept = intersection_hline_circle(y_top, radius)[1];
                // We pick the rightmost intersection point, since the top right point is missing (draw a picture)
                // Note that x_left < x_intercept < x_right.

                // The area is
                area = (x_intercept - x_left) * 1.0
                    + area_of_semicircle_section(x_intercept, x_right, radius)
                    - (x_right - x_intercept) * y_bottom;
                // Namely divide the intersection of the box and disk into a rectangle and the region under the curve. The rectangle is the first term,
                //  the region under the curve are the last two turns (first compute the are under the semicircle to the x-axis, then subtract the rectangle we shouldn't count)
                // The signs work out since we're in the upper right quadrant (also checked this for the origin being to the right of the left boundary of the square)
                //  (note that every term in the expression for the area should be positive)
            }
            2 => {
                // Missing two points. By symmetry, it must be top left and top right.
                // Two cases: either the the circle pokes through the top edge or it doesn't.
                // Note that since exactly two points are missing, if the circle pokes through the top edge it does so twice.
                // Meaning that the vertical strip must contain the origin: x_left < 0 < x_right, and also that radius > y_top
                //  (by symmetry, x_right > 0 always)
                if x_left < 0.0 && radius > y_top.abs() {
                    let x_intercept_left = intersection_hline_circle(y_top, radius)[0];
                    let x_intercept_right = intersection_hline_circle(y_top, radius)[1];
                    // Have x_left < x_intercept_left < x_intercept_right < x_right

                    area = area_of_semicircle_section(x_left, x_intercept_left, radius)
                        - (x_intercept_left - x_left) * y_bottom
                        + (x_intercept_right - x_intercept_left) * 1.0
                        + (area_of_semicircle_section(x_intercept_right, x_right, radius)
                            - (x_right - x_intercept_right) * y_bottom);
                } else {
                    // In case the circle does not poke through the the top edge, the area is
                    area = area_of_semicircle_section(x_left, x_right, radius) - 1.0 * y_bottom;
                    // namely it is the area of the semicircle section.
                }
            }
            1 => {
                // Missing three points. By symmetry, only the RB point is in the disk.
                // Two cases: either the origin is above or below the bottom edge of the box.
                if y_bottom > 0.0 {
                    // Origin below the bottom edge
                    // No turning point, we can integrate as normal
                    let x_intercept = intersection_hline_circle(y_bottom, radius)[1];
                    // Have x_left < x_intercept < x_right

                    area = area_of_semicircle_section(x_left, x_intercept, radius)
                        - (x_intercept - x_left) * y_bottom;
                } else {
                    // Origin above the bottom edge
                    // Turning point in the x-direction. So need to split up the area into sections
                    let x_intercept = intersection_hline_circle(y_bottom, radius)[1];
                    // Have x_left < x_intercept < x_right

                    area = area_of_semicircle_section(x_left, x_intercept, radius)
                        - (x_intercept - x_left) * y_bottom
                        + 2.0 * area_of_semicircle_section(x_intercept, radius, radius);
                    // First two terms compute the area under the curve from x_left to x_intercept
                    // Third terms is the area of the remaining slice (the turning point is at x1 = radius,
                    //  our area function is sufficiently robust to handle that)
                }
            }
            0 => {
                // No cornerpoints are in the disk
                // Three cases: either the origin is outside of the box and the disk peeks through the bottom edge,
                //  or the origin is inside the box, or neither of those things happens (in which case the area is zero).
                if y_bottom > 0.0 && x_left < 0.0 && radius > y_bottom.abs() {
                    // y_bottom > 0.0 <=> origin is outside of the box
                    // x_left < 0.0 <=> origin is in vertical strip, radius > y_bottom then means the disk peeks through the bottom edge
                    let x_intercept_left = intersection_hline_circle(y_bottom, radius)[0];
                    let x_intercept_right = intersection_hline_circle(y_bottom, radius)[1];
                    // Get x_left < x_intercept_left < x_intercept_right < x_right
                    area = area_of_semicircle_section(x_intercept_left, x_intercept_right, radius)
                        - (x_intercept_right - x_intercept_left) * y_bottom;
                    // Compute the area as usual
                } else if y_bottom < 0.0 {
                    // Origin inside the circle.
                    // (running time note: this is a very rare case. only happens once and only for very small radii)
                    // Kind of nasty. The disk may peek through all edges (take say the box centered at (0,0), a radius strictly between 1 and sqrt 2)
                    // Idea: Compute area of complete disk, subtract peek-through regions
                    let mut running_area = PI * radius.powi(2);

                    // Casing on peek-through is just casing on radius being greater than the edge coordinates.
                    //  They are all essentially independent (we don't use dihedral symmetry for this computation)
                    if radius > y_top.abs() {
                        // Peek through top edge
                        let x_intercept_left = intersection_hline_circle(y_top, radius)[0];
                        let x_intercept_right = intersection_hline_circle(y_top, radius)[1];
                        running_area -=
                            area_of_semicircle_section(x_intercept_left, x_intercept_right, radius)
                                - (x_intercept_right - x_intercept_left) * y_top.abs();
                    }
                    if radius > y_bottom.abs() {
                        // Peek through bottom edge
                        let x_intercept_left = intersection_hline_circle(y_bottom, radius)[0];
                        let x_intercept_right = intersection_hline_circle(y_bottom, radius)[1];
                        running_area -=
                            area_of_semicircle_section(x_intercept_left, x_intercept_right, radius)
                                - (x_intercept_right - x_intercept_left) * y_bottom.abs();
                    }
                    if radius > x_right.abs() {
                        // Peek through right edge
                        let y_intercept_bottom = intersection_hline_circle(x_right, radius)[0];
                        let y_intercept_top = intersection_hline_circle(x_right, radius)[1];
                        running_area -=
                            area_of_semicircle_section(y_intercept_bottom, y_intercept_top, radius)
                                - (y_intercept_top - y_intercept_bottom) * x_right.abs();
                    }
                    if radius > x_left.abs() {
                        // Peek through top edge
                        let y_intercept_bottom = intersection_hline_circle(x_left, radius)[0];
                        let y_intercept_top = intersection_hline_circle(x_left, radius)[1];
                        running_area -=
                            area_of_semicircle_section(y_intercept_bottom, y_intercept_top, radius)
                                - (y_intercept_top - y_intercept_bottom) * y_top.abs();
                    }

                    area = running_area;
                } else {
                    // No overlap case
                    area = 0.0;
                }
            }
            _ => {
                panic!("Nr. of points cannot be different from 4, 3, 2, 1, or 0.");
            }
        }
    }
    area
}
