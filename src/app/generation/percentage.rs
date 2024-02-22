use std::f64::consts::PI;

use crate::data_structures::Blocks;

// logic + geometry + integration
// Percentage is to be supplied as float between 0 and 1 (note that unexpected behaviour may occur if it 0.0 or 1.0 exactly due to numerical errors)
pub fn generate_alg_percentage(radius: f64, center_offset_x: f64, center_offset_y: f64, percentage: f64) -> Blocks {
    let edge_length = ((2.0*radius).ceil() as usize) + 4; // the 4 is needed as a buffer.. 
    // i think we're able to get away with less but it doesn't matter. Buffer is required to make the interior work as expected
    let origin = [(edge_length / 2) as f64, (edge_length / 2) as f64]; 
    // in bitmatrix coordinates, where is the center of the grid?
    let mut output_vec = Vec::new();

    // The above part is the same for all algorithms (I think at this stage)


    for i in 0..edge_length.pow(2) { // loop over all coords

        // Bottom right coordinate of the box in bitmatrix coordinates is [i % edge_length, i / edge_length]
        // Want to get at the distance from the corners of a box to the origin + offset (we do this component-wise)
        let d_x_left = ((i % edge_length) as f64) - (origin[0] + center_offset_x); // how far to the right of the left boundary of the box is the origin + offset?
        let d_x_right = ((i % edge_length) as f64 + 1.0) - (origin[0] + center_offset_x);
        let d_y_bottom = ((i / edge_length) as f64) - (origin[1] + center_offset_y);
        let d_y_top = ((i / edge_length) as f64 + 1.0) - (origin[1] + center_offset_y);

        
        output_vec.push(cell_disk_intersection_area(radius, d_x_left, d_x_right, d_y_bottom, d_y_top) >= percentage)

    }

    Blocks {
        blocks: output_vec,
        edge_length,
        origin,
    }
}

// With -radius <= x0 <= x1 <= radius, what is the area of the semicircle y=sqrt(R^2-x^2) from x0 to x1?
// Have an exact primitive for non-edge cases
fn area_of_semicircle_section(x0: f64, x1: f64, radius: f64) -> f64 {
    if !((-radius <= x0) && (x0 <= x1) && (x1 <= radius)) { // -radius <= x0 <= x1 <= radius
        panic!("ooo...")
    }

    // Primitive in case |x| != R
    fn primitive(x: f64, radius: f64) -> f64 {
        let sq = (radius.powi(2) - x.powi(2)).sqrt();
        0.5 * (x * sq + radius.powi(2) * (x/sq).atan())
    }

    let area_x0_to_0 : f64;
    if x0 <= -radius {
        area_x0_to_0 = -radius * PI/4.0;
    } else if x0 >= radius {
        area_x0_to_0 = radius * PI/4.0;
    } else {
        area_x0_to_0 = primitive(x0, radius);
    };
    
    let area_x1_to_0 : f64;
    if x1 <= -radius {
        area_x1_to_0 = -radius * PI/4.0;
    } else if x0 >= radius {
        area_x1_to_0 = radius * PI/4.0;
    } else {
        area_x1_to_0 = primitive(x1, radius);
    };

    area_x1_to_0 - area_x0_to_0 // fundamental theorem of calculus (finite additivity of integral)
}

// Given a circle of given radius centered at (0,0), and a y level, return the pair of intersections that result from intersecting the circle and line
// First the negative, then the positive
fn intersection_hline_circle(y: f64, radius: f64) -> [f64; 2] {
    let positive_intersection = (radius.powi(2) - y.powi(2)).sqrt();
    [-positive_intersection, positive_intersection]
}

pub fn cell_disk_intersection_area(radius: f64, d_x_left: f64, d_x_right: f64, d_y_bottom: f64, d_y_top: f64) -> f64 {
    

    let mut area = 0f64; // We compute this area and see if its less than the percentage parameter


    // Precompute which corner points are in the the disk (we condition on these a bunch)
    // write L for left, R for right, B for bottom, T for top.
    let LB = (d_x_left.powi(2) + d_y_bottom.powi(2) <= radius.powi(2));
    let RB = (d_x_right.powi(2) + d_y_bottom.powi(2) <= radius.powi(2));
    let LT = (d_x_left.powi(2) + d_y_top.powi(2) <= radius.powi(2));
    let RT = (d_x_right.powi(2) + d_y_top.powi(2) <= radius.powi(2));

    if LB && RB && LT && RT {
        // We have that the box is contained in the disk <=> all corner of the box are in the disk. Then the area is 1.0 
        // (aim to call arctan and sqrt as little as possible)
        area = 1.0;
    } else if (d_x_left.powi(2) + d_y_bottom.powi(2) > (radius+0.5).powi(2))
    && (d_x_right.powi(2) + d_y_bottom.powi(2) > (radius+0.5).powi(2))
    && (d_x_left.powi(2) + d_y_top.powi(2) > (radius+0.5).powi(2))
    && (d_x_right.powi(2) + d_y_top.powi(2) > (radius+0.5).powi(2)) {
        // If every corner point is more than radius + 0.5 away from the center, then the area of the intersection is zero
        // Worst case is for very small circle with center on the midpoint of an edge of the box.
        area = 0.0
        // These two statements should catch both the bulk of the interior and the bulk of the exterior
        // we can use more expensive functions for the remaining O(radius) cases (linear since the it scales as a boundary)

        // May assume (due to first if statement) that at least one corner point is not in the disk. Match on # corner points in the disk.
    } else {
        match LB as u8 + RB as u8 + LT as u8 + RT as u8 { // TODO: refactor to include the area=1.0 case of complete inclusion
            3 => {
                // Missing one point. case on which one. See last entry for explanation.
                if !LB { // FIXME: bug on integral radius
                    // missing bottom left
                    // pick the left entry
                    let d_x_intercept = intersection_hline_circle(d_y_bottom, radius)[0];

                    area = 1.0 * (d_x_right - d_x_intercept) + area_of_semicircle_section(d_x_left, d_x_intercept, radius) - (d_x_intercept - d_x_left) * (-d_y_top)
                    // make sure the signs are positive for each area: we have d_x_right > d_x_right
                    
                } else if !RB {
                    // missing bottom right
                    let d_x_intercept = intersection_hline_circle(d_y_bottom, radius)[1];

                    area = 1.0 * (d_x_intercept - d_x_left) + area_of_semicircle_section(d_x_intercept, d_x_right, radius) - (d_x_right - d_x_intercept) * (-d_y_top)
                    // The formulas are the same as top right, we just flip vertically so d_y_top and d_y_bottom are interchanged
                } else if !LT { // FIXME: bug on integral radius
                    // missing top left
                    // pick the left entry
                    let d_x_intercept = intersection_hline_circle(d_y_top, radius)[0];

                    area = 1.0 * (d_x_right - d_x_intercept) + area_of_semicircle_section(d_x_left, d_x_intercept, radius) - (d_x_intercept - d_x_left) * d_y_bottom
                    // make sure the signs are positive for each area

                } else if !RT {
                    // missing top right
                    // Find intersection of top edge with circle. intersection_hline_circle computes the distance from the origin
                    let d_x_intercept = intersection_hline_circle(d_y_top, radius)[1];
                    // we pick the rightmost intersection point, since the bottom LEFT point is missing (draw a picture)

                    // The area is
                    area = 1.0 * (d_x_intercept - d_x_left) + area_of_semicircle_section(d_x_intercept, d_x_right, radius) - (d_x_right - d_x_intercept) * d_y_bottom
                    // Namely divide the intersection of the box and disk into a rectangle and the region under the curve. The rectangle is the first term,
                    //  the region under the curve are the last two turns (first compute the are under the semicircle to the x-axis, then subtract the rectangle we shouldn't count)
                    // The signs work out since we're in the upper right quadrant (also checked this for the origin being to the right of the left boundary of the square)
                } else {
                    panic!("Has to miss one point (impossible to get here)")
                }
            },
            2 => {
                // Note that by the geometry, the two points have to be adjacent. So that leaves four possibilities: either both on left, right, bottom, or top.
                // Choose coordinate x to integrate over for {top, bottom}, choose coordinate y for {left, right}. Then we don't need to compute the intersection of the circle with an edge!
                // Choose nice absolute value so we can combine the formulas for the same coordinate to integrate over
                if LT && RT {
                    // integrate over x (the multiplication by 1.0 is to indicate that it is an area of 1.0 by d_y_top)
                    area = area_of_semicircle_section(d_x_left, d_x_right, radius) - 1.0 * (-d_y_top);
                } else if LB && RB {
                    // integrate over x
                    area = area_of_semicircle_section(d_x_left, d_x_right, radius) - 1.0 * d_y_bottom;
                } else if LT && LB {
                    // integrate over y (origin is to the left)
                    area = area_of_semicircle_section(d_y_bottom, d_y_top, radius) - 1.0 * d_x_left;
                } else if RT && RB {
                    // integrate over y
                    area = area_of_semicircle_section(d_y_bottom, d_y_top, radius) - 1.0 * (-d_x_right);
                }
            },
            1 => {
                // We want to pick the easiest possible integration coordinate. Case on which cardinal strip the origin is in, so we can avoid bend points
                // recall from `conservative.rs`: vertical strip is d_x_left <= 0.0 && d_x_right >= 0.0. Horizontal strip is d_y_bottom <= 0.0 && d_y_top >= 0.0
                if !(d_y_bottom <= 0.0 && d_y_top >= 0.0) {
                    // Not in the horizontal strip. So no bend point in the x direction, so it can be used as an integration coordinate.
                    // Case on which corner point
                    if LB {
                        let d_x_intercept = intersection_hline_circle(d_y_bottom, radius)[1];
                        area = area_of_semicircle_section(d_x_left, d_x_intercept, radius) - (d_x_intercept - d_x_left) * (d_y_bottom)
                    } else if RB {
                        let d_x_intercept = intersection_hline_circle(d_y_bottom, radius)[0];
                        area = area_of_semicircle_section(d_x_intercept, d_x_right, radius) - (d_x_right - d_x_intercept) * (d_y_bottom)
                    } else if LT {
                        let d_x_intercept = intersection_hline_circle(d_y_top, radius)[1];
                        area = area_of_semicircle_section(d_x_left, d_x_intercept, radius) - (d_x_intercept - d_x_left) * (-d_y_top)
                    } else if RT {
                        let d_x_intercept = intersection_hline_circle(d_y_top, radius)[0];
                        area = area_of_semicircle_section(d_x_intercept, d_x_right, radius) - (d_x_right - d_x_intercept) * (-d_y_top)
                    } else {
                        panic!("impossible")
                    }
                } else if !(d_x_left <= 0.0 && d_x_right >= 0.0) {
                    // Not in the vertical strip, so use y as integration coordinate FIXME: all are wrong. found a mistaking, testing..
                    if LB {
                        let d_y_intercept = intersection_hline_circle(d_x_left, radius)[1];
                        area = area_of_semicircle_section(d_y_bottom, d_y_intercept, radius) - (d_y_intercept - d_y_bottom) * (d_x_left)
                    } else if RB {
                        let d_y_intercept = intersection_hline_circle(d_x_right, radius)[1];
                        area = area_of_semicircle_section(d_y_bottom, d_y_intercept, radius) - (d_y_intercept - d_y_bottom) * (-d_x_right)
                    } else if LT  {
                        let d_y_intercept = intersection_hline_circle(d_x_left, radius)[0];
                        area = area_of_semicircle_section(d_y_intercept, d_y_top, radius) - (d_y_top - d_y_intercept) * (d_x_left)
                    } else if RT {
                        let d_y_intercept = intersection_hline_circle(d_x_right, radius)[0];
                        area = area_of_semicircle_section(d_y_intercept, d_y_top, radius) - (d_y_top - d_y_intercept) * (-d_x_right)
                    }
                } else {
                    // Origin both in horizontal and vertical strip, so the origin must be in the cell
                    // Split along the horizontal and vertical lines through the origin
                    if LB {
                        let d_x_intercept = intersection_hline_circle(d_y_bottom, radius)[1];
                        area = area_of_semicircle_section(d_x_left, 0.0, radius) + 2.0 * area_of_semicircle_section(0.0, d_x_intercept, radius) + (d_x_intercept - d_x_left) * (-d_y_bottom)
                    } else if RB {
                        let d_x_intercept = intersection_hline_circle(d_y_bottom, radius)[0];
                        area = area_of_semicircle_section(0.0, d_x_right, radius) + 2.0 * area_of_semicircle_section(d_x_intercept, 0.0, radius) + (d_x_right - d_x_intercept) * (-d_y_bottom)
                    } else if LT {
                        let d_x_intercept = intersection_hline_circle(d_y_top, radius)[1];
                        area = area_of_semicircle_section(d_x_left, 0.0, radius) + 2.0 * area_of_semicircle_section(0.0, d_x_intercept, radius) + (d_x_intercept - d_x_left) * (d_y_top)
                    } else if RT {
                        let d_x_intercept = intersection_hline_circle(d_y_top, radius)[0];
                        area = area_of_semicircle_section(0.0, d_x_right, radius) + 2.0 * area_of_semicircle_section(d_x_intercept, 0.0, radius) + (d_x_right - d_x_intercept) * (d_y_top)
                    }
                }
            }
            0 => {
                // For nonzero area we have to have that it's in a cardinal strip (as in `conservative.rs`)
                // Also use this condition for computation.
                if d_x_left <= 0.0 && d_x_right >= 0.0 {
                    // in vertical strip
                    // Case on to the top or inside or bottom
                    if d_y_top >= 0.0 {
                        // origin to the top
                        let d_x_intercepts = intersection_hline_circle(d_y_top, radius);
                        if d_x_intercepts[0].is_nan() && d_x_intercepts[1].is_nan() { // No intersection case
                            area = 0.0
                        } else {
                            area = area_of_semicircle_section(d_x_intercepts[0], d_x_intercepts[1], radius) - (d_x_intercepts[1] - d_x_intercepts[0]) * (-d_y_top)
                        }
                    } else if d_y_bottom <= 0.0 {
                        // origin to the bottom
                        let d_x_intercepts = intersection_hline_circle(d_y_bottom, radius);
                        if d_x_intercepts[0].is_nan() && d_x_intercepts[1].is_nan() { // No intersection case
                            area = 0.0
                        } else {
                            area = area_of_semicircle_section(d_x_intercepts[0], d_x_intercepts[1], radius) - (d_x_intercepts[1] - d_x_intercepts[0]) * (d_y_bottom)
                        }
                    } else {
                        // TODO: origin inside the cell, no corner hit. (very nasty case! Need to compute all intersections)
                        todo!()
                    }
                } else if d_y_bottom <= 0.0 && d_y_top >= 0.0 {
                    // in horizontal strip
                    // Case on left or right (note that inside is already covered by the above case)
                    if d_x_left <= 0.0 {
                        // origin to left
                        let d_y_intercepts = intersection_hline_circle(d_x_left, radius);
                        if d_y_intercepts[0].is_nan() && d_y_intercepts[1].is_nan() {
                            area = 0.0
                        } else {
                            area = area_of_semicircle_section(d_y_intercepts[0], d_y_intercepts[0], radius) - (d_y_intercepts[1] - d_y_intercepts[0]) * (-d_x_left)
                        }
                    } else if d_x_right >= 0.0 {
                        // origin to right                            
                        let d_y_intercepts = intersection_hline_circle(d_x_right, radius);
                        if d_y_intercepts[0].is_nan() && d_y_intercepts[1].is_nan() {
                            area = 0.0
                        } else {
                            area = area_of_semicircle_section(d_y_intercepts[0], d_y_intercepts[0], radius) - (d_y_intercepts[1] - d_y_intercepts[0]) * (d_x_right)
                        }
                    } else {
                        panic!("Impossible! but via a nontrivial argument")
                    }
                } else {
                    area = 0.0
                }
            },
            _ => {
                panic!("Nr. of points cannot be different from 3, 2, 1, or 0.")
            }
        }
    }

    area
}