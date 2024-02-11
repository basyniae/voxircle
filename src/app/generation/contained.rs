use crate::data_structures::Blocks;

pub fn generate_alg_contained(radius: f64, center_offset_x: f64, center_offset_y: f64) -> Blocks {
    let edge_length = ((2.0*radius).ceil() as usize) + 4; // the 4 is needed as a buffer.. 
    // i think we're able to get away with less but it doesn't matter. Buffer is required to make the interior work as expected
    let origin = [(edge_length / 2) as f64, (edge_length / 2) as f64]; 
    // in bitmatrix coordinates, where is the center of the grid?
    let mut output_vec = Vec::new();

    // The above part is the same for all algorithms (I think at this stage)


    for i in 0..edge_length.pow(2) { // loop over all coords
        // Bottom right coordinate of the box in bitmatrix coordinates is [i % edge_length, i / edge_length]
        // We have that the box is contained in the disk <=> all corner of the box are in the disk
        // Want to get at the distance from the corners of a box to the origin + offset (we do this component-wise)
        let d_x_left = ((i % edge_length) as f64) - (origin[0] + center_offset_x);
        let d_x_right = ((i % edge_length) as f64 + 1.0) - (origin[0] + center_offset_x);
        let d_y_bottom = ((i / edge_length) as f64) - (origin[1] + center_offset_y);
        let d_y_top = ((i / edge_length) as f64 + 1.0) - (origin[1] + center_offset_y);
        // Easier to look at the squared distance
        output_vec.push(
            (d_x_left.powi(2) + d_y_bottom.powi(2) <= radius.powi(2))
            && (d_x_right.powi(2) + d_y_bottom.powi(2) <= radius.powi(2))
            && (d_x_left.powi(2) + d_y_top.powi(2) <= radius.powi(2))
            && (d_x_right.powi(2) + d_y_top.powi(2) <= radius.powi(2))
        )
    }

    Blocks {
        blocks: output_vec,
        edge_length,
        origin,
    }
}