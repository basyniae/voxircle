use crate::data_structures::Blocks;

// hard logic + geometric arguments
pub fn generate_alg_conservative(radius: f64, center_offset_x: f64, center_offset_y: f64) -> Blocks {
    let edge_length = ((2.0*radius).ceil() as usize) + 4; // the 4 is needed as a buffer.. 
    // i think we're able to get away with less but it doesn't matter. Buffer is required to make the interior work as expected
    let origin = [(edge_length / 2) as f64, (edge_length / 2) as f64]; 
    // in bitmatrix coordinates, where is the center of the grid?
    let mut output_vec = Vec::new();

    // The above part is the same for all algorithms (I think at this stage)


    for i in 0..edge_length.pow(2) { // loop over all coords
        // Step one: Check if any corner points are in the disk (same code as in `contained` but with || instead of &&)

        // Bottom right coordinate of the box in bitmatrix coordinates is [i % edge_length, i / edge_length]
        // We have that the box is contained in the disk <=> all corner of the box are in the disk
        // Want to get at the distance from the corners of a box to the origin + offset (we do this component-wise)
        let d_x_left = ((i % edge_length) as f64) - (origin[0] + center_offset_x);
        let d_x_right = ((i % edge_length) as f64 + 1.0) - (origin[0] + center_offset_x);
        let d_y_bottom = ((i / edge_length) as f64) - (origin[1] + center_offset_y);
        let d_y_top = ((i / edge_length) as f64 + 1.0) - (origin[1] + center_offset_y);
        // Easier to look at the squared distance.

        if (d_x_left.powi(2) + d_y_bottom.powi(2) <= radius.powi(2))
        || (d_x_right.powi(2) + d_y_bottom.powi(2) <= radius.powi(2))
        || (d_x_left.powi(2) + d_y_top.powi(2) <= radius.powi(2))
        || (d_x_right.powi(2) + d_y_top.powi(2) <= radius.powi(2)) {
            output_vec.push(true);
        } else if d_x_left <= 0.0 && d_x_right >= 0.0 { // Vertical strip
            // Step two: If none of the corner points are in the disk, then the circle must cross a single edge of the box twice.
            // so then the circle center must be in a strip a cardinal direction away from the box.
            // Case on horizontal or vertical strip
            // The origin is in the vertical strip iff d_x_left <= 0 and d_x_right >= 0. (be careful of signs!). Similar for vertical

            // Step three:
            // So the distance from the circle center to the line is the absolute value of the coordinate in which the strip is.
            // If this distance is less than the radius output true
            // This distance it the minimum of the absolute values of the d_xs and d_ys (we look for the closest point)
            if (d_y_bottom.abs() <= radius) || (d_y_top.abs() <= radius) {
                output_vec.push(true);
            } else {
                output_vec.push(false);
            };
        } else if d_y_bottom <= 0.0 && d_y_top >= 0.0 {
            if (d_x_left.abs() <= radius) || (d_x_right.abs() <= radius) {
                output_vec.push(true);
            } else {
                output_vec.push(false);
            };
        } else {
            output_vec.push(false);
        }
    }

    Blocks {
        blocks: output_vec,
        edge_length,
        origin,
    }
}